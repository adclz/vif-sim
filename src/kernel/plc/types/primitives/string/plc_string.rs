use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::GetRawPointerPrimitive;
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive, PrimitiveTrait, RawMut};
use crate::kernel::plc::types::primitives::string::_char::_Char;
use crate::kernel::plc::types::primitives::string::_string::{_String, plcstr};
use crate::kernel::plc::types::primitives::string::wchar::{wchar, WChar};
use crate::kernel::plc::types::primitives::string::wstring::{plcwstr, WString};
use crate::{create_family, error, impl_primitive_traits, key_reader};
use camelpaste::paste;
use fixedstr::str256;
use serde::Serializer;
use serde_json::{Map, Value};
use core::str::FromStr;

create_family!(
    #[enum_dispatch(MetaData, SetMetaData, ToggleMonitor)]
    PlcString(_Char, _String, WChar, WString)
);

impl_primitive_traits!(PlcString, {
    bool, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    char, [self.is__char], [get_mut as_mut__char], [get as__char],
    wchar, [self.is_w_char], [get_mut as_mut_w_char], [get as_w_char],
    plcstr, [self.is__string], [get_mut as_mut__string], [get as__string],
    plcwstr, [self.is_w_string], [get_mut as_mut_w_string], [get as_w_string],
    f32, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    f64, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u8, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u16, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u32, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u64, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i8, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i16, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i32, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i64, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))]
});

impl TryFrom<&Map<String, Value>> for PlcString {
    type Error = Stop;

    fn try_from(data: &Map<String, Value>) -> Result<Self, Self::Error> {
        key_reader!(
           format!("Parse PlcString"),
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
                "Char" => Ok(Self::_Char(_Char::new_default(id))),
                "String" => Ok(Self::_String(_String::new_default(id))),
                "WChar" => Ok(Self::WChar(WChar::new_default(id))),
                "WString" => Ok(Self::WString(WString::new_default(id))),
                _ => Err(error!(
                    format!("Invalid PlcString type: {}", ty),
                    format!("Parse PlcString")
                )),
            },
            Some(value) => {
                if let Some(v) = value.as_str() {
                    match ty {
                        "Char" => {
                            if v.len() > 1 {
                                Err(error!(
                                    format!("Invalid Char value: {}", v),
                                    "Parse PlcString".to_string()
                                ))
                            } else {
                                Ok(Self::_Char(_Char::new(&char::from_str(v).unwrap(), id)?))
                            }
                        }
                        "String" => Ok(Self::_String(_String::new(&plcstr(v.into()), id)?)),
                        "WChar" => {
                            if v.len() > 1 {
                                Err(error!(
                                    format!("Invalid Char value: {}", v),
                                    "Parse PlcString".to_string()
                                ))
                            } else {
                                Ok(Self::WChar(WChar::new(&char::from_str(v).unwrap(), id)?))
                            }
                        }
                        "WString" => Ok(Self::WString(WString::new(&plcwstr(v.into()), id)?)),
                        _ => Err(error!(
                            format!("Invalid PlcString type: {}", ty),
                            "Parse PlcString".to_string()
                        )),
                    }
                } else {
                    Err(error!(
                        format!("Invalid PlcString value: {}", value),
                        "Parse PlcString".to_string()
                    ))
                }
            }
        }
    }
}
