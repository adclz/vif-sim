#[macro_export]
macro_rules! required_key {
    ($json: expr, $key: expr, $_as: ident, $err_msg: expr) => {
        match $json.contains_key($key) {
            true => match $json[$key].$_as() {
                Some(v) => Ok(v),
                None => Err($err_msg),
            },
            false => Err($err_msg),
        }
    };
    ($json: expr, $key: expr, $err_msg: expr) => {
        match $json.contains_key($key) {
            true => Ok(&$json[$key]),
            false => Err($err_msg),
        }
    };
}

#[macro_export]
macro_rules! key_reader {
    // Recursive
    (@key $location: expr, $json: ident { $($key:tt)* } $field: ident => { $($rest:tt)* }) => {
      let field_str = stringify!($field);
        let $field = match $json.contains_key(field_str) {
            true => match $json[field_str].as_object() {
                Some(v) => Ok(v),
                None => Err($crate::error!(
                    format!("Invalid JSON: {} is not a sub-object", field_str), $location)),
            },
            false => Err($crate::error!(
                format!("Invalid JSON: key '{}'  is missing in {:?}", field_str, $json), $location)),
        }?;

        // recurse into the nested field
        key_reader!(
            @key $location,
            $field
            { $($key)* }
            $($rest)*
        );
    };
    // Required
    (@key $location: expr, $json: ident { $($key:tt)* } $field: ident => $_as: ident, $($rest:tt)*) => {
        key_reader!(@key $location, $json
            {
                $($key)*
                let field_str = stringify!($field);
                let $field = match $json.contains_key(field_str) {
                    true => match $json[field_str].$_as() {
                        Some(v) => Ok(v),
                        None => Err($crate::error!(format!("Invalid JSON: {} is not of type {}", stringify!($field), stringify!($_as)), $location)),
                    },
                    false => Err($crate::error!(format!("Invalid JSON: key '{}' is missing in {:?}", field_str, $json), $location)),
                }?;
            }
            $($rest)*);
    };
    (@key $location: expr, $json: ident { $($key:tt)* } $field: ident, $($rest:tt)*) => {
        key_reader!(@key $location, $json
            {
                $($key)*
                let field_str = stringify!($field);
                let $field = match $json.contains_key(field_str) {
                    true => Ok(&$json[field_str]),
                    false => Err($crate::error!(format!("Invalid JSON: key '{}' is missing in {:?}", field_str, $json), $location)),
                }?;
            }
            $($rest)*);
    };
    // Optional
    (@key $location: expr, $json: ident { $($key:tt)* } $field: ident? => $_as: ident, $($rest:tt)*) => {
        key_reader!(@key $location, $json
            {
                $($key)*
                let field_str = stringify!($field);
                let $field = match $json.contains_key(field_str) {
                    true => match $json[field_str].$_as() {
                        Some(v) => Some(v),
                        None => None,
                    },
                    false => None,
                };
            }
            $($rest)*);
    };
    (@key $location: expr, $json: ident { $($key:tt)* } $field: ident?, $($rest:tt)*) => {
        key_reader!(@key $location, $json
            {
                $($key)*
                let field_str = stringify!($field);
                let $field = match $json.contains_key(field_str) {
                    true => Some(&$json[field_str]),
                    false => None,
                };
            }
            $($rest)*);
    };
    (@key $location: expr, $json: ident { $($key:tt)* }) => {
        $($key)*
    };
    ($location: expr, $json: ident { $($key:tt)* }) => {
        key_reader!(@key $location, $json {} $($key)*);
    };
}  