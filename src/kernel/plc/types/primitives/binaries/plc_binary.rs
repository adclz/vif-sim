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
use crate::{create_family, error, impl_primitive_traits, key_reader};
use camelpaste::paste;
use fixedstr::str256;

use serde::Serializer;
use serde_json::{Map, Value};
use crate::kernel::plc::types::primitives::string::_string::plcstr;
use crate::kernel::plc::types::primitives::string::wstring::plcwstr;

create_family!(
    #[enum_dispatch(Crement, MetaData, SetMetaData, ToggleMonitor)]
    PlcBinary(Byte, Word, DWord, LWord)
);

impl_primitive_traits!(PlcBinary, {
    bool, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    char, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wchar, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    plcstr, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    plcwstr, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
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

impl TryFrom<&Map<String, Value>> for PlcBinary {
    type Error = Stop;

    fn try_from(data: &Map<String, Value>) -> Result<Self, Self::Error> {
        key_reader!(
           format!("Parse PlcBinary"),
           data {
                ty => as_str,
                src => {
                    value?,
                    id => as_u64,
                }
            }
        );
        let id = id as u32;
        match value {
            None => match ty {
                "Byte" => Ok(PlcBinary::Byte(Byte::new_default(id))),
                "Word" => Ok(PlcBinary::Word(Word::new_default(id))),
                "DWord" => Ok(PlcBinary::DWord(DWord::new_default(id))),
                "LWord" => Ok(PlcBinary::LWord(LWord::new_default(id))),
                _ => Err(error!(
                    format!("Invalid PlcBinary type: {}", ty),
                    format!("Parse PlcBinary")
                )),
            },
            Some(value) => {
                if let Some(v) = value.as_u64() {
                    match ty {
                        "Byte" => Ok(PlcBinary::Byte(Byte::new(&(v as u8), id)?)),
                        "Word" => Ok(PlcBinary::Word(Word::new(&(v as u16), id)?)),
                        "DWord" => Ok(PlcBinary::DWord(DWord::new(&(v as u32), id)?)),
                        "LWord" => Ok(PlcBinary::LWord(LWord::new(&(v), id)?)),
                        _ => Err(error!(
                            format!("Invalid PlcBinary type: {}", ty),
                            format!("Parse PlcBinary")
                        )),
                    }
                    /*} else if let Some(v) = value.as_str() {
                        match ty {
                            "LWord" => Ok(PlcBinary::try_from(v, id)?),
                            _ => Err(error!(
                                format!("Invalid PlcBinary type: {}", ty),
                                "Parse PlcBinary".to_string()
                            )),
                        }*/
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
