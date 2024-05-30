
#[macro_export]
macro_rules! create_family {
    ($(#[enum_dispatch($($trait: ident),+)])? $family: ident ($($primitive: ident),+)) => {

        $(#[enum_dispatch::enum_dispatch($($trait),+)])?
        #[derive(Clone)]
        pub enum $family {
            $($primitive($primitive)),+
        }

        impl GetRawPointerPrimitive for $family {
            fn get_raw_pointer(&mut self) -> *mut dyn RawMut {
                match self {
                    $(Self::$primitive(a) => a as *mut dyn RawMut),+
                }
            }
        }

        camelpaste::paste! {
            impl $family {
                $(
                    pub fn [<is_$primitive:snake>](&self) -> bool {
                        match self {
                            Self::$primitive(_) => true,
                            _ => false
                        }
                    }

                    pub fn [<as_$primitive:snake>](&self) -> Result<&$primitive, Stop> {
                        match self {
                            Self::$primitive(ref a) => Ok(a),
                            _ => Err($crate::error!(format!("Expected {}, got {}", stringify!($primitive), self)))
                        }
                    }

                    pub fn [<as_mut_$primitive:snake>](&mut self) -> Result<&mut $primitive, Stop> {
                        match self {
                            Self::$primitive(ref mut a) => Ok(a),
                            _ => Err($crate::error!(format!("Expected {}, got {}", stringify!($primitive), self)))
                        }
                    }
                )+
            }
        }

        impl core::fmt::Display for $family {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                match self {
                    $($family::$primitive(a) => write!(f, "{}", &a),)+
                }
            }
        }

        impl serde::Serialize for $family {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
                match self {
                    $($family::$primitive(a) => a.serialize(serializer),)+
                }
            }
        }
    };
}
