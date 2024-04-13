use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::GetRawPointerPrimitive;
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive, PrimitiveTrait, RawMut};
use crate::kernel::plc::types::primitives::integers::dint::DInt;
use crate::kernel::plc::types::primitives::integers::int::Int;
use crate::kernel::plc::types::primitives::integers::lint::LInt;
use crate::kernel::plc::types::primitives::integers::sint::SInt;
use crate::kernel::plc::types::primitives::integers::udint::UDInt;
use crate::kernel::plc::types::primitives::integers::uint::UInt;
use crate::kernel::plc::types::primitives::integers::ulint::ULInt;
use crate::kernel::plc::types::primitives::integers::usint::USInt;
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::kernel::plc::types::primitives::string::wstring::wstr256;
use crate::kernel::plc::types::primitives::traits::crement::Crement;
use crate::{create_family, error, impl_primitive_traits, key_reader};
use camelpaste::paste;
use fixedstr::str256;

use serde::Serializer;
use serde_json::{Map, Value};

create_family!(
    #[enum_dispatch(Crement, MetaData, SetMetaData, ToggleMonitor)]
    PlcInteger(USInt, SInt, UInt, Int, UDInt, DInt, ULInt, LInt)
);

impl_primitive_traits!(PlcInteger, {
    bool, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    char, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wchar, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    str256, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wstr256, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    f32, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    f64, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u8, [self.is_u_s_int], [get_mut as_mut_u_s_int], [get as_u_s_int],
    u16, [self.is_u_int], [get_mut as_mut_u_int], [get as_u_int],
    u32, [self.is_u_d_int], [get_mut as_mut_u_d_int], [get as_u_d_int],
    u64, [self.is_u_l_int], [get_mut as_mut_u_l_int], [get as_u_l_int],
    i8, [self.is_s_int], [get_mut as_mut_s_int], [get as_s_int],
    i16, [self.is_int], [get_mut as_mut_int], [get as_int],
    i32, [self.is_d_int], [get_mut as_mut_d_int], [get as_d_int],
    i64, [self.is_l_int], [get_mut as_mut_l_int], [get as_l_int]
});

impl From<u64> for PlcInteger {
    fn from(value: u64) -> Self {
        match value {
            0..=255 => Self::USInt(USInt::new(&(value as u8)).unwrap()),
            0..=65_536 => Self::UInt(UInt::new(&(value as u16)).unwrap()),
            0..=4_294_967_295 => Self::UDInt(UDInt::new(&(value as u32)).unwrap()),
            0..=18_446_744_073_709_551_615 => Self::ULInt(ULInt::new(&value).unwrap()),
        }
    }
}

impl From<i64> for PlcInteger {
    fn from(value: i64) -> Self {
        match value {
            -128..=127 => Self::SInt(SInt::new(&(value as i8)).unwrap()),
            -32_768..=32_767 => Self::Int(Int::new(&(value as i16)).unwrap()),
            -2_147_483_648..=2_147_483_647 => Self::DInt(DInt::new(&(value as i32)).unwrap()),
            -9_223_372_036_854_775_808..=9_223_372_036_854_775_807 => {
                Self::LInt(LInt::new(&value).unwrap())
            }
        }
    }
}

impl TryFrom<&str> for PlcInteger {
    type Error = Stop;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.parse::<i64>() {
            Ok(a) => Ok(Self::LInt(LInt::new(&a).unwrap())),
            Err(_) => match value.parse::<u64>() {
                Ok(a) => Ok(Self::ULInt(ULInt::new(&a).unwrap())),
                Err(_) => Err(error!(
                    format!("Number is an invalid BigInt"),
                    format!("Parse BigInt to PlcInteger")
                )),
            },
        }
    }
}

impl TryFrom<(&Map<String, Value>, &str)> for PlcInteger {
    type Error = Stop;

    fn try_from(src: (&Map<String, Value>, &str)) -> Result<Self, Self::Error> {
        let _src = src.0;
        let ty = src.1;
        key_reader!(
            format!("Parse PlcInteger {}", ty),
            _src {
                value?,
            }
        );

        match value {
            None => match ty {
                "SInt" => Ok(PlcInteger::SInt(SInt::default())),
                "Int" => Ok(PlcInteger::Int(Int::default())),
                "DInt" => Ok(PlcInteger::DInt(DInt::default())),
                "LInt" => Ok(PlcInteger::LInt(LInt::default())),

                "USInt" => Ok(PlcInteger::USInt(USInt::default())),
                "UInt" => Ok(PlcInteger::UInt(UInt::default())),
                "UDInt" => Ok(PlcInteger::UDInt(UDInt::default())),
                "ULInt" => Ok(PlcInteger::ULInt(ULInt::default())),
                _ => Err(error!(
                    format!("Invalid PlcInteger type: {}", ty),
                    "Parse PlcInteger".to_string()
                )),
            },
            Some(value) => {
                if let Some(v) = value.as_i64() {
                    match ty {
                        "SInt" => Ok(PlcInteger::SInt(SInt::new(&(v as i8))?)),
                        "Int" => Ok(PlcInteger::Int(Int::new(&(v as i16))?)),
                        "DInt" => Ok(PlcInteger::DInt(DInt::new(&(v as i32))?)),
                        "LInt" => Ok(PlcInteger::LInt(LInt::new(&(v))?)),
                        "USInt" => Ok(PlcInteger::USInt(USInt::new(&(v as u8))?)),
                        "UInt" => Ok(PlcInteger::UInt(UInt::new(&(v as u16))?)),
                        "UDInt" => Ok(PlcInteger::UDInt(UDInt::new(&(v as u32))?)),
                        "ULInt" => Ok(PlcInteger::ULInt(ULInt::new(&(v as u64))?)),
                        _ => Err(error!(
                            format!("Invalid PlcInteger type: {}", ty),
                            "Parse PlcInteger".to_string()
                        )),
                    }
                } else if let Some(v) = value.as_u64() {
                    match ty {
                        "SInt" => Ok(PlcInteger::SInt(SInt::new(&(v as i8))?)),
                        "Int" => Ok(PlcInteger::Int(Int::new(&(v as i16))?)),
                        "DInt" => Ok(PlcInteger::DInt(DInt::new(&(v as i32))?)),
                        "LInt" => Ok(PlcInteger::LInt(LInt::new(&(v as i64))?)),
                        "USInt" => Ok(PlcInteger::USInt(USInt::new(&(v as u8))?)),
                        "UInt" => Ok(PlcInteger::UInt(UInt::new(&(v as u16))?)),
                        "UDInt" => Ok(PlcInteger::UDInt(UDInt::new(&(v as u32))?)),
                        "ULInt" => Ok(PlcInteger::ULInt(ULInt::new(&(v))?)),
                        _ => Err(error!(
                            format!("Invalid PlcInteger type: {}", ty),
                            "Parse PlcInteger".to_string()
                        )),
                    }
                } else if let Some(v) = value.as_str() {
                    match ty {
                        "LInt" => Ok(PlcInteger::try_from(v)?),
                        "ULInt" => Ok(PlcInteger::try_from(v)?),
                        _ => Err(error!(
                            format!("Invalid PlcInteger type: {}", ty),
                            "Parse PlcInteger".to_string()
                        )),
                    }
                } else {
                    Err(error!(
                        format!("Invalid PlcInteger value: {}", value),
                        "Parse PlcInteger".to_string()
                    ))
                }
            }
        }
    }
}
