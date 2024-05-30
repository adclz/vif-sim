#![allow(non_snake_case)]
use core::fmt::{Display, Formatter};
use camelpaste::paste;
use crate::{error};
use crate::kernel::plc::types::primitives::binaries::plc_binary::PlcBinary;
use crate::kernel::plc::types::primitives::boolean::plc_bool::PlcBool;
use crate::kernel::plc::types::primitives::integers::plc_integer::PlcInteger;
use crate::kernel::plc::types::primitives::floats::plc_float::PlcFloat;
use crate::kernel::plc::types::primitives::string::plc_string::PlcString;
use crate::kernel::plc::types::primitives::timers::plc_time::PlcTime;
use crate::kernel::plc::types::primitives::string::_string::plcstr;
use crate::kernel::plc::types::primitives::string::wstring::plcwstr;
use crate::kernel::plc::types::primitives::tod::plc_tod::PlcTod;
use crate::kernel::plc::types::complex::array::PlcArray;
use crate::kernel::plc::types::complex::r#struct::PlcStruct;
use crate::kernel::plc::types::complex::instance::fb_instance::FbInstance;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::meta_data::{MetaData, SetMetaData, HeapOrStatic, MaybeHeapOrStatic};
use crate::container::broadcast::broadcast::Broadcast;
use crate::kernel::arch::local::r#type::{IntoLocalType, LocalType};
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use std::borrow::Cow;
use crate::kernel::registry::Kernel;
use crate::kernel::plc::types::primitives::traits::primitive_traits::Primitive;
use crate::kernel::plc::types::primitives::traits::family_traits::{WithRefFamily, WithTypeFamily, IsFamily};
macro_rules! impl_constant_type {
    ({$($family: ident),+}, [$($forbid_family: ident),+]) => {

        #[enum_dispatch::enum_dispatch(Primitive, MetaData, SetMetaData)]
        pub enum ConstantType {
            $($family($family)),+
        }

        paste! {
            impl IsFamily for ConstantType {
                $(
                    fn [<is_$family:snake>](&self) -> bool {
                        match self {
                            Self::$family(..) => true,
                            _ => false
                        }
                    }
                )+
                $(
                    fn [<is_$forbid_family:snake>](&self) -> bool {
                        false
                    }
                )+

                fn is_complex(&self) -> bool {
                    false
                }
            }

            impl WithRefFamily for ConstantType {
                $(
                    fn [<with_$family:snake>]<R>(&self, _channel: &Broadcast, f: impl Fn(&$family) -> R) -> Result<R, Stop> {
                        match self {
                            Self::$family(a) => Ok(f(&*a)),
                            _ => Err(error!(format!("Could not borrow Constant, expected {}, got {}", stringify!($family), self)))
                        }
                    }
                )+
                $(
                    fn [<with_$forbid_family:snake>]<R>(&self, _channel: &Broadcast, _f: impl Fn(&$forbid_family) -> R) -> Result<R, Stop> {
                       Err(error!(format!("Could not borrow Constant, expected {}, got {}", stringify!($forbid_family), self)))
                    }
                )+
            }

            impl WithTypeFamily for ConstantType {
                $(
                    fn [<with_type_$family:snake>]<R>(&self, f: impl Fn(&$family) -> R) -> Result<R, Stop> {
                        match self {
                            Self::$family(a) => Ok(f(&*a)),
                            _ => Err(error!(format!("Could not borrow Constant, expected {}, got {}", stringify!($family), self)))
                        }
                    }
                )+
                $(
                    fn [<with_type_$forbid_family:snake>]<R>(&self, _f: impl Fn(&$forbid_family) -> R) -> Result<R, Stop> {
                       Err(error!(format!("Could not borrow Constant, expected {}, got {}", stringify!($forbid_family), self)))
                    }
                )+
            }

            impl IntoLocalType for ConstantType {
                fn transform(&self) -> Result<LocalType, Stop> {
                    match self {
                        $(Self::$family(a) => Ok(LocalType::$family(a.clone()))),+
                    }
                }
            }
            
            impl Clone for ConstantType {
                fn clone(&self) -> Self {
                    match self {
                        $(Self::$family($family) => Self::$family($family.clone()),)+
                    }
                }
            }
            
            impl Display for ConstantType {
                fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                    match self {
                        $(Self::$family(local) => write!(f, "{}", local),)+
                    }
                }
            }
        }
    };
}

impl_constant_type!(
    {
        PlcBool,
        PlcInteger,
        PlcFloat,
        PlcBinary,
        PlcTime,
        PlcString,
        PlcTod
    },
    // forbid
    [PlcStruct, PlcArray, FbInstance]
);