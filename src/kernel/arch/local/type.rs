#![allow(non_snake_case)]
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::complex::array::PlcArray;
use crate::kernel::plc::types::complex::instance::fb_instance::FbInstance;
use crate::kernel::plc::types::complex::r#struct::PlcStruct;
use crate::kernel::plc::types::primitives::binaries::plc_binary::PlcBinary;
use crate::kernel::plc::types::primitives::boolean::plc_bool::PlcBool;

use crate::kernel::plc::types::primitives::floats::plc_float::PlcFloat;
use crate::kernel::plc::types::primitives::integers::plc_integer::PlcInteger;
use crate::kernel::plc::types::primitives::string::plc_string::PlcString;
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::kernel::plc::types::primitives::timers::plc_time::PlcTime;
use crate::kernel::plc::types::primitives::tod::plc_tod::PlcTod;
use crate::kernel::arch::constant::r#type::ConstantType;
use camelpaste::paste;
use crate::kernel::plc::types::primitives::string::_string::plcstr;
use crate::kernel::plc::types::primitives::string::wstring::plcwstr;
use std::borrow::Cow;
use crate::kernel::registry::Kernel;
use crate::kernel::plc::types::primitives::traits::family_traits::{IsFamily, WithRefFamily, WithMutFamily, WithTypeFamily};
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive};
use crate::kernel::plc::types::primitives::traits::meta_data::{MetaData, SetMetaData};
use crate::kernel::plc::types::primitives::traits::primitive_traits::ToggleMonitor;
use crate::error;

impl From<ConstantType> for LocalType {
    fn from(value: ConstantType) -> Self {
        match value {
            ConstantType::PlcBool(a) => Self::PlcBool(a),
            ConstantType::PlcInteger(a) => Self::PlcInteger(a),
            ConstantType::PlcFloat(a) => Self::PlcFloat(a),
            ConstantType::PlcBinary(a) => Self::PlcBinary(a),
            ConstantType::PlcTime(a) => Self::PlcTime(a),
            ConstantType::PlcString(a) => Self::PlcString(a),
            ConstantType::PlcTod(a) => Self::PlcTod(a),
        }
    }
}

#[enum_dispatch::enum_dispatch]
pub trait IntoLocalType {
    fn transform(&self) -> Result<LocalType, Stop>;
}

impl IntoLocalType for LocalType {
    fn transform(&self) -> Result<LocalType, Stop> {
        Ok(self.clone())
    }
}

macro_rules! impl_local_types {
    ($($simple_family: ident),+ + $($complex_family: ident),+) => {

        #[enum_dispatch::enum_dispatch(Primitive, AsMutPrimitive, MetaData, SetMetaData, ToggleMonitor)]
        pub enum LocalType {
            $($simple_family($simple_family)),+,
            $($complex_family($complex_family)),+
        }

        paste! {
            impl IsFamily for LocalType {
                $(
                    fn [<is_$simple_family:snake>](&self) -> bool {
                        match self {
                            LocalType::$simple_family(..) => true,
                            _ => false
                        }
                    }
                )+
                $(
                    fn [<is_$complex_family:snake>](&self) -> bool {
                        match self {
                            LocalType::$complex_family(..) => true,
                            _ => false
                        }
                    }
                )+

                fn is_complex(&self) -> bool {
                    match self {
                        $(LocalType::$complex_family(..) => true,)+
                        _ => false
                    }
                }
            }

            impl WithRefFamily for LocalType {
                $(
                    fn [<with_$simple_family:snake>]<R>(&self, _channel: &Broadcast, f: impl Fn(&$simple_family) -> R) -> Result<R, Stop> {
                        match self {
                            LocalType::$simple_family(a) => Ok(f(&*a)),
                            _ => Err(error!(format!("Could not borrow LocalPointer, expected {}, got {}", stringify!($simple_family), self)))
                        }
                    }
                )+
                $(
                    fn [<with_$complex_family:snake>]<R>(&self, _channel: &Broadcast, f: impl Fn(&$complex_family) -> R) -> Result<R, Stop> {
                        match self {
                            LocalType::$complex_family(a) => Ok(f(&*a)),
                            _ => Err(error!(format!("Could not borrow LocalPointer, expected {}, got {}", stringify!($complex_family), self)))
                        }
                    }
                )+
            }

            impl WithTypeFamily for LocalType {
                $(
                    fn [<with_type_$simple_family:snake>]<R>(&self, f: impl Fn(&$simple_family) -> R) -> Result<R, Stop> {
                        match self {
                            LocalType::$simple_family(a) => Ok(f(&*a)),
                            _ => Err(error!(format!("Could not borrow LocalPointer, expected {}, got {}", stringify!($simple_family), self)))
                        }
                    }
                )+
                $(
                    fn [<with_type_$complex_family:snake>]<R>(&self, f: impl Fn(&$complex_family) -> R) -> Result<R, Stop> {
                        match self {
                            LocalType::$complex_family(a) => Ok(f(&*a)),
                            _ => Err(error!(format!("Could not borrow LocalPointer, expected {}, got {}", stringify!($complex_family), self)))
                        }
                    }
                )+
            }
            impl Clone for LocalType {
                fn clone(&self) -> Self {
                    match self {
                        $(Self::$simple_family($simple_family) => Self::$simple_family($simple_family.clone()),)+
                        $(Self::$complex_family($complex_family) => Self::$complex_family($complex_family.clone()),)+
                    }
                }
            }

            impl core::fmt::Display for LocalType {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    match self {
                        $(Self::$simple_family($simple_family) => write!(f, "{}", &$simple_family),)+
                        $(Self::$complex_family($complex_family) => write!(f, "{}", &$complex_family),)+
                    }
                }
            }
        }
    };
}

impl_local_types!(
    PlcBool,
    PlcInteger,
    PlcFloat,
    PlcBinary,
    PlcTime,
    PlcString,
    PlcTod +

    PlcStruct,
    PlcArray,
    FbInstance
);
