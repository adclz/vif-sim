use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::plc::primitives::family_traits::GetRawPointerPrimitive;
use crate::plc::primitives::family_traits::{AsMutPrimitive, Primitive};
use crate::plc::primitives::primitive_traits::{PrimitiveTrait, RawMut};
use crate::plc::primitives::string::_char::_Char;
use crate::plc::primitives::string::_string::_String;
use crate::plc::primitives::string::wchar::{wchar, WChar};
use crate::plc::primitives::string::wstring::{wstr256, WString};
use crate::{create_family, error, impl_primitive_traits, key_reader};
use camelpaste::paste;
use fixedstr::str256;
use serde::Serializer;
use serde_json::{Map, Value};
use std::str::FromStr;

create_family!(
    #[enum_dispatch(MetaData, SetMetaData, ToggleMonitor)]
    PlcString(_Char, _String, WChar, WString)
);

impl_primitive_traits!(PlcString, {
    bool, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    char, [self.is__char], [get_mut as_mut__char], [get as__char],
    wchar, [self.is_w_char], [get_mut as_mut_w_char], [get as_w_char],
    str256, [self.is__string], [get_mut as_mut__string], [get as__string],
    wstr256, [self.is_w_string], [get_mut as_mut_w_string], [get as_w_string],
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

impl TryFrom<(&Map<String, Value>, &str)> for PlcString {
    type Error = Stop;

    fn try_from(src: (&Map<String, Value>, &str)) -> Result<Self, Self::Error> {
        let _src = src.0;
        let ty = src.1;
        key_reader!(
           format!("Parse PlcString {}", ty),
           _src {
                value?,
            }
        );

        match value {
            None => match ty {
                "Char" => Ok(Self::_Char(_Char::default())),
                "String" => Ok(Self::_String(_String::default())),
                "WChar" => Ok(Self::WChar(WChar::default())),
                "WString" => Ok(Self::WString(WString::default())),
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
                                Ok(Self::_Char(_Char::new(&char::from_str(v).unwrap())?))
                            }
                        }
                        "String" => Ok(Self::_String(_String::new(&v.into())?)),
                        "WChar" => {
                            if v.len() > 1 {
                                Err(error!(
                                    format!("Invalid Char value: {}", v),
                                    "Parse PlcString".to_string()
                                ))
                            } else {
                                Ok(Self::WChar(WChar::new(&char::from_str(v).unwrap())?))
                            }
                        }
                        "WString" => Ok(Self::WString(WString::new(&v.into())?)),
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
