#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use serde_json::Value;
    use crate::kernel::registry::{get_or_insert_global_string, get_string, Kernel};
    use crate::container::broadcast::broadcast::Broadcast;
    use crate::parser::main::program::parse_program;
    use uuid::Uuid;
    use crate::kernel::plc::interface::traits::InterfaceAccessors;
    use crate::kernel::plc::types::primitives::traits::family_traits::{IsFamily, WithRefFamily};
    use crate::kernel::plc::types::primitives::traits::primitive_traits::PrimitiveTrait;
    use crate::container::error::error::Stop;
    use crate::kernel::plc::interface::section::Section;
    use crate::key_reader;
    use crate::parser::main::exclude::{parse_exclude_sections, parse_exclude_types, parse_filter_operations, parse_return_operations};

    #[test]
    fn test_parse_exclude_types() {
        let data = r#"
        {
            "exclude_types": [
                "Bool", "Int"
            ]
        }"#;

        let json: HashMap<String, Value> = serde_json::from_str(data).unwrap();

        key_reader!(
            format!("_"),
            json {
                exclude_types? => as_array,
            }
        );

        let uuid = Uuid::default();
        let mut kernel = Kernel::default();
        let channel = Broadcast::new(&uuid);

        parse_exclude_types(exclude_types, &mut kernel).unwrap();

        assert!(kernel.get_mut_excluded_types().contains(&"Bool".to_string()));
        assert!(kernel.get_mut_excluded_types().contains(&"Int".to_string()));
    }

    #[test]
    fn test_filter_operations() {
        let data = r#"
        {
            "filter_operations": {
                "cmp": {
                    "Int": ["Time"]
                }
            }
        }"#;

        let json: HashMap<String, Value> = serde_json::from_str(data).unwrap();

        key_reader!(
            format!("_"),
            json {
                filter_operations? => as_object,
            }
        );

        let uuid = Uuid::default();
        let mut kernel = Kernel::default();
        let channel = Broadcast::new(&uuid);

        parse_filter_operations(filter_operations, &mut kernel).unwrap();

        let cmp_exclude = kernel.get_mut_excluded_operation(&"cmp".to_string());
        assert!(cmp_exclude.contains_key(&"Int".to_string()));
        assert!(cmp_exclude.get(&"Int".to_string()).unwrap().contains(&"Time".to_string()));
    }

    #[test]
    fn test_parse_exclude_sections() {
        let data = r#"
        {
            "exclude_sections": {
                "constant": ["Instance", "Udt", "Struct", "Array"]
            }
        }"#;

        let json: HashMap<String, Value> = serde_json::from_str(data).unwrap();

        key_reader!(
            format!("_"),
            json {
                exclude_sections? => as_object,
            }
        );

        let uuid = Uuid::default();
        let mut kernel = Kernel::default();
        let channel = Broadcast::new(&uuid);

        parse_exclude_sections(exclude_sections, &mut kernel).unwrap();

        assert!(kernel.get_mut_excluded_types_in_section(&Section::Constant).contains(&"Instance".to_string()));
        assert!(kernel.get_mut_excluded_types_in_section(&Section::Constant).contains(&"Udt".to_string()));
        assert!(kernel.get_mut_excluded_types_in_section(&Section::Constant).contains(&"Struct".to_string()));
        assert!(kernel.get_mut_excluded_types_in_section(&Section::Constant).contains(&"Array".to_string()));
    }

    #[test]
    fn test_override_return() {
        let data = r#"
        {
            "override_return": {
               "sub": {
                    "Time": [["Tod", "Tod"]]
               }
            }
        }"#;

        let json: HashMap<String, Value> = serde_json::from_str(data).unwrap();

        key_reader!(
            format!("_"),
            json {
                override_return? => as_object,
            }
        );

        let uuid = Uuid::default();
        let mut kernel = Kernel::default();
        let channel = Broadcast::new(&uuid);

        parse_return_operations(override_return, &mut kernel).unwrap();

        let cmp_exclude = kernel.get_mut_return_operation(&"sub".to_string());
        assert!(cmp_exclude.contains_key(&"Tod".to_string()));
        assert!(cmp_exclude.get(&"Tod".to_string()).unwrap().contains_key(&"Tod".to_string()));
        assert!(cmp_exclude.get(&"Tod".to_string()).unwrap().get(&"Tod".to_string()).unwrap().is_plc_time());
    }
}
