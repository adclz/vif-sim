#![allow(non_snake_case)]
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::binaries::byte::Byte;
use crate::kernel::plc::types::primitives::binaries::dword::DWord;
use crate::kernel::plc::types::primitives::binaries::lword::LWord;
use crate::kernel::plc::types::primitives::binaries::word::Word;
use crate::kernel::plc::types::primitives::traits::family_traits::GetRawPointerPrimitive;
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive, PrimitiveTrait, RawMut};
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::kernel::plc::types::primitives::string::wstring::wstr256;
use crate::{create_family, error, impl_primitive_traits, key_reader};
use camelpaste::paste;
use fixedstr::str256;

use serde::Serializer;
use serde_json::{Map, Value};

create_family!(
    #[enum_dispatch(Crement, MetaData, SetMetaData, ToggleMonitor)]
    PlcBinary(Byte, Word, DWord, LWord)
);

impl_primitive_traits!(PlcBinary, {
    bool, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    char, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wchar, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    str256, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wstr256, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    f32, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    f64, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u8, [self.is_byte], [get_mut as_mut_byte], [get as_byte],
    u16, [self.is_word], [get_mut as_mut_word], [get as_word],
    u32, [self.is_d_word], [get_mut as_mut_d_word], [get as_d_word],
    u64, [self.is_l_word], [get_mut as_mut_l_word], [get as_l_word],
    i8, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i16, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i32, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i64, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))]
});

impl From<u64> for PlcBinary {
    fn from(value: u64) -> Self {
        match value {
            0..=255 => Self::Byte(Byte::new(&(value as u8)).unwrap()),
            0..=65_536 => Self::Word(Word::new(&(value as u16)).unwrap()),
            0..=4_294_967_295 => Self::DWord(DWord::new(&(value as u32)).unwrap()),
            0..=18_446_744_073_709_551_615 => Self::LWord(LWord::new(&value).unwrap()),
        }
    }
}

impl From<i64> for PlcBinary {
    fn from(value: i64) -> Self {
        match value {
            -128..=127 => Self::Byte(Byte::new(&(value as u8)).unwrap()),
            -32_768..=32_767 => Self::Word(Word::new(&(value as u16)).unwrap()),
            -2_147_483_648..=2_147_483_647 => Self::DWord(DWord::new(&(value as u32)).unwrap()),
            -9_223_372_036_854_775_808..=9_223_372_036_854_775_807 => {
                Self::LWord(LWord::new(&(value as u64)).unwrap())
            }
        }
    }
}

impl TryFrom<&str> for PlcBinary {
    type Error = Stop;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.parse::<i64>() {
            Ok(a) => Ok(Self::LWord(LWord::new(&(a as u64)).unwrap())),
            Err(_) => match value.parse::<u64>() {
                Ok(a) => Ok(Self::LWord(LWord::new(&a).unwrap())),
                Err(_) => Err(error!(
                    format!("Number is an invalid BigInt"),
                    format!("Parse BigInt to PlcBinary")
                )),
            },
        }
    }
}

impl TryFrom<(&Map<String, Value>, &str)> for PlcBinary {
    type Error = Stop;

    fn try_from(src: (&Map<String, Value>, &str)) -> Result<Self, Self::Error> {
        let _src = src.0;
        let ty = src.1;
        key_reader!(
            format!("Parse PlcBinary {}", ty),
            _src {
                value?,
            }
        );

        match value {
            None => match ty {
                "Byte" => Ok(PlcBinary::Byte(Byte::default())),
                "Word" => Ok(PlcBinary::Word(Word::default())),
                "DWord" => Ok(PlcBinary::DWord(DWord::default())),
                "LWord" => Ok(PlcBinary::LWord(LWord::default())),
                _ => Err(error!(
                    format!("Invalid PlcBinary type: {}", ty),
                    format!("Parse PlcBinary")
                )),
            },
            Some(value) => {
                if let Some(v) = value.as_u64() {
                    match ty {
                        "Byte" => Ok(PlcBinary::Byte(Byte::new(&(v as u8))?)),
                        "Word" => Ok(PlcBinary::Word(Word::new(&(v as u16))?)),
                        "DWord" => Ok(PlcBinary::DWord(DWord::new(&(v as u32))?)),
                        "LWord" => Ok(PlcBinary::LWord(LWord::new(&(v))?)),
                        _ => Err(error!(
                            format!("Invalid PlcBinary type: {}", ty),
                            format!("Parse PlcBinary")
                        )),
                    }
                } else if let Some(v) = value.as_str() {
                    match ty {
                        "LWord" => Ok(PlcBinary::try_from(v)?),
                        _ => Err(error!(
                            format!("Invalid PlcBinary type: {}", ty),
                            "Parse PlcBinary".to_string()
                        )),
                    }
                } else {
                    Err(error!(
                        format!("Invalid PlcBinary value: {}", value),
                        "Parse PlcBinary".to_string()
                    ))
                }
            }
        }
    }
}
