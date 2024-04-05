#![allow(non_snake_case)]
#![warn(unused_imports)]

use std::borrow::Cow;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::plc::complex::array::PlcArray;
use crate::plc::complex::instance::fb_instance::FbInstance;
use crate::plc::complex::r#struct::PlcStruct;
use crate::plc::primitives::binaries::plc_binary::PlcBinary;
use crate::plc::primitives::boolean::plc_bool::PlcBool;

use crate::plc::primitives::floats::plc_float::PlcFloat;
use crate::plc::primitives::integers::plc_integer::PlcInteger;
use crate::plc::primitives::primitive_traits::RawMut;
use crate::plc::primitives::string::plc_string::PlcString;
use crate::plc::primitives::string::wchar::wchar;
use crate::plc::primitives::string::wstring::wstr256;
use crate::plc::primitives::timers::plc_time::PlcTime;
use crate::plc::primitives::tod::plc_tod::PlcTod;
use camelpaste::paste;
use fixedstr::str256;

use std::fmt::Display;

use crate::plc::primitives::boolean::bool::Bool;
use crate::plc::primitives::boolean::bit_access::BitAccess;

use crate::plc::primitives::binaries::byte::Byte;
use crate::plc::primitives::binaries::dword::DWord;
use crate::plc::primitives::binaries::lword::LWord;
use crate::plc::primitives::binaries::word::Word;

use crate::plc::primitives::integers::dint::DInt;
use crate::plc::primitives::integers::int::Int;
use crate::plc::primitives::integers::lint::LInt;
use crate::plc::primitives::integers::sint::SInt;
use crate::plc::primitives::integers::udint::UDInt;
use crate::plc::primitives::integers::uint::UInt;
use crate::plc::primitives::integers::ulint::ULInt;
use crate::plc::primitives::integers::usint::USInt;

use crate::plc::primitives::floats::lreal::LReal;
use crate::plc::primitives::floats::real::Real;

use crate::plc::primitives::string::_char::_Char;
use crate::plc::primitives::string::_string::_String;
use crate::plc::primitives::string::wchar::WChar;
use crate::plc::primitives::string::wstring::WString;

use crate::plc::primitives::timers::lTime::LTime;
use crate::plc::primitives::timers::time::Time;

use crate::plc::primitives::tod::ltod::LTod;
use crate::plc::primitives::tod::tod::Tod;
use crate::registry::registry::Kernel;

pub trait GetRawPointerPrimitive {
    fn get_raw_pointer(&mut self) -> *mut dyn RawMut;
}

macro_rules! create_families_traits {
    ($($family: ident),+) => {
        paste! {
            #[enum_dispatch::enum_dispatch]
            pub trait IsFamily {
                $(fn [<is_$family:snake>](&self) -> bool;)+
                fn is_complex(&self) -> bool;
                fn is_primitive(&self) -> bool {
                    !self.is_complex()
                }
            }

            #[enum_dispatch::enum_dispatch]
            pub trait WithRefFamily {
                $(fn [<with_$family:snake>]<R>(&self, channel: &Broadcast, f: impl Fn(&$family) -> R) -> Result<R, Stop>;)+
            }

            #[enum_dispatch::enum_dispatch]
            pub trait WithTypeFamily {
                $(fn [<with_type_$family:snake>]<R>(&self, f: impl Fn(&$family) -> R) -> Result<R, Stop>;)+
            }

            #[enum_dispatch::enum_dispatch]
            pub trait WithMutFamily {
                 $(fn [<with_mut_$family:snake>]<R>(&self, channel: &Broadcast, f: &mut impl FnMut(&mut $family) -> R) -> Result<R, Stop>;)+
            }
        }
    };
}

macro_rules! primitive_traits {
    ($($primitive: ident),+) => {
        paste! {
            #[enum_dispatch::enum_dispatch]
            pub trait Primitive {
                $(
                    fn [<is_$primitive>](&self) -> bool;
                    fn [<as_$primitive>](&self, channel: &Broadcast) -> Result<$primitive, Stop>;
                )+
            }
            #[enum_dispatch::enum_dispatch]
            pub trait AsMutPrimitive {
                $(
                     fn [<set_$primitive>](&mut self, other: $primitive, channel: &Broadcast) -> Result<(), Stop>;
                     fn [<set_default_$primitive>](&mut self, other: $primitive) -> Result<(), Stop>;
                )+
            }
            #[enum_dispatch::enum_dispatch]
            pub trait WithMut {
                 $(fn [<with_mut_$primitive>]<R>(&self, channel: &Broadcast, f: &mut impl FnMut(&mut $primitive) -> R) -> Result<R, Stop>;)+
            }
        }
    };
}

primitive_traits!(
    bool, u8, i8, u16, i16, u32, i32, u64, i64, f32, f64, str256, char, wstr256, wchar
);

create_families_traits!(
    PlcBool, PlcInteger, PlcFloat, PlcBinary, PlcTime, PlcTod, PlcString, PlcArray,
    PlcStruct, FbInstance
);

///
/// Metadata
///
/// name: Family type (Int, Bool ...) or Complex (Array, Struct ...)
///
/// alias: Udt Name, Instance of, or primitive alias
///
#[enum_dispatch::enum_dispatch]
pub trait MetaData {
    fn name(&self) -> &'static str;
    fn get_alias_str<'a>(&'a self, kernel: &'a Kernel) -> Option<&'a String>;
    fn get_alias_id(&self, kernel: & Kernel) -> Option<usize>;
    fn is_read_only(&self) -> bool;
}

#[enum_dispatch::enum_dispatch]
pub trait SetMetaData : MetaData {
    fn set_alias(&mut self, alias: &str, kernel: &Kernel);
    fn set_read_only(&mut self, value: bool);
}

#[enum_dispatch::enum_dispatch]
pub trait ToggleMonitor {
    fn set_monitor(&mut self, activate: bool);
}

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

        impl std::fmt::Display for $family {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $($family::$primitive(a) => write!(f, "[{}: {}]", stringify!($primitive), &a),)+
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
