use serde_json::{Map, Value};
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use crate::kernel::plc::types::primitives::binaries::byte::Byte;
use crate::kernel::plc::types::primitives::binaries::dword::DWord;
use crate::kernel::plc::types::primitives::binaries::lword::LWord;
use crate::kernel::plc::types::primitives::binaries::plc_binary::PlcBinary;
use crate::kernel::plc::types::primitives::binaries::word::Word;
use crate::kernel::plc::types::primitives::boolean::bool::Bool;
use crate::kernel::plc::types::primitives::boolean::plc_bool::PlcBool;
use crate::kernel::plc::types::primitives::traits::meta_data::{MetaData};
use crate::kernel::plc::types::primitives::floats::lreal::LReal;
use crate::kernel::plc::types::primitives::floats::plc_float::PlcFloat;
use crate::kernel::plc::types::primitives::floats::real::Real;
use crate::kernel::plc::types::primitives::integers::dint::DInt;
use crate::kernel::plc::types::primitives::integers::int::Int;
use crate::kernel::plc::types::primitives::integers::lint::LInt;
use crate::kernel::plc::types::primitives::integers::plc_integer::PlcInteger;
use crate::kernel::plc::types::primitives::integers::sint::SInt;
use crate::kernel::plc::types::primitives::integers::udint::UDInt;
use crate::kernel::plc::types::primitives::integers::uint::UInt;
use crate::kernel::plc::types::primitives::integers::ulint::ULInt;
use crate::kernel::plc::types::primitives::integers::usint::USInt;
use crate::kernel::plc::types::primitives::string::_char::_Char;
use crate::kernel::plc::types::primitives::string::_string::_String;
use crate::kernel::plc::types::primitives::string::plc_string::PlcString;
use crate::kernel::plc::types::primitives::string::wchar::WChar;
use crate::kernel::plc::types::primitives::string::wstring::WString;
use crate::kernel::plc::types::primitives::timers::lTime::LTime;
use crate::kernel::plc::types::primitives::timers::plc_time::PlcTime;
use crate::kernel::plc::types::primitives::timers::time::Time;
use crate::kernel::plc::types::primitives::tod::ltod::LTod;
use crate::kernel::plc::types::primitives::tod::plc_tod::PlcTod;
use crate::kernel::plc::types::primitives::tod::tod::Tod;
use crate::kernel::arch::constant::r#type::ConstantType;
use crate::kernel::arch::local::r#type::{IntoLocalType, LocalType};
use crate::kernel::plc::types::primitives::traits::primitive_traits::PrimitiveTrait;
use crate::kernel::registry::Kernel;

pub fn parse_constant_type(
    json: & Map<String, Value>,
    registry: &Kernel,
    force_constant_type: Option<LocalType>,
) -> Result<ConstantType, Stop> {
    key_reader!(
            format!("Parse constant"),
            json {
                ty => as_str,
                src => as_object,
            }
        );

    match ty {
        "Implicit" => {
            match force_constant_type {
                None => Err(error!("Implicit constant but vif could not determine the context. Maybe wrap your value in a primitive ?".to_string())),
                Some(a) => {
                    match a {
                        LocalType::PlcBool(_) => Ok(ConstantType::PlcBool(PlcBool::Bool(Bool::try_from(src)?))),
                        LocalType::PlcInteger(b) => {
                            match b {
                                PlcInteger::USInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::USInt(USInt::try_from(src)?))),
                                PlcInteger::SInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::SInt(SInt::try_from(src)?))),
                                PlcInteger::UInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::UInt(UInt::try_from(src)?))),
                                PlcInteger::Int(_) => Ok(ConstantType::PlcInteger(PlcInteger::Int(Int::try_from(src)?))),
                                PlcInteger::UDInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::UDInt(UDInt::try_from(src)?))),
                                PlcInteger::DInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::DInt(DInt::try_from(src)?))),
                                PlcInteger::ULInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::ULInt(ULInt::try_from(src)?))),
                                PlcInteger::LInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::LInt(LInt::try_from(src)?))),
                            }
                        }
                        LocalType::PlcFloat(b) => {
                            match b {
                                PlcFloat::Real(_) => Ok(ConstantType::PlcFloat(PlcFloat::Real(Real::try_from(src)?))),
                                PlcFloat::LReal(_) => Ok(ConstantType::PlcFloat(PlcFloat::LReal(LReal::try_from(src)?))),
                            }
                        }
                        LocalType::PlcBinary(b) => {
                            match b {
                                PlcBinary::Byte(_) => Ok(ConstantType::PlcBinary(PlcBinary::Byte(Byte::try_from(src)?))),
                                PlcBinary::Word(_) => Ok(ConstantType::PlcBinary(PlcBinary::Word(Word::try_from(src)?))),
                                PlcBinary::DWord(_) => Ok(ConstantType::PlcBinary(PlcBinary::DWord(DWord::try_from(src)?))),
                                PlcBinary::LWord(_) => Ok(ConstantType::PlcBinary(PlcBinary::LWord(LWord::try_from(src)?))),
                            }
                        }
                        LocalType::PlcTime(b) => {
                            match b {
                                PlcTime::Time(_) => Ok(ConstantType::PlcTime(PlcTime::Time(Time::try_from(src)?))),
                                PlcTime::LTime(_) => Ok(ConstantType::PlcTime(PlcTime::LTime(LTime::try_from(src)?))),
                            }
                        }
                        LocalType::PlcTod(b) => {
                            match b {
                                PlcTod::Tod(_) => Ok(ConstantType::PlcTod(PlcTod::Tod(Tod::try_from(src)?))),
                                PlcTod::LTod(_) => Ok(ConstantType::PlcTod(PlcTod::LTod(LTod::try_from(src)?))),
                            }
                        }
                        LocalType::PlcString(b) => {
                            match b {
                                PlcString::_Char(_) => Ok(ConstantType::PlcString(PlcString::_Char(_Char::try_from(src)?))),
                                PlcString::_String(_) => Ok(ConstantType::PlcString(PlcString::_String(_String::try_from(src)?))),
                                PlcString::WChar(_) => Ok(ConstantType::PlcString(PlcString::WChar(WChar::try_from(src)?))),
                                PlcString::WString(_) => Ok(ConstantType::PlcString(PlcString::WString(WString::try_from(src)?))),
                            }
                        }
                        _ => Err(error!("Unknown constant type".to_string()))
                    }
                }
            }
        },

        // Alias
        "alias" => {
            key_reader!(
                format!("Parse type alias"),
                src {
                    name => as_str,
                    data => as_object,
                }
            );
            // Checks if alias exists
            match registry.get_type_alias_as_constant_type(name) {
                None => Err(error!(format!("Type alias {} is not registered", name))),
                Some(as_constant) => {
                    match parse_constant_type(data, registry, Some(as_constant.transform()?)) {
                        Ok(b) => Ok(b),
                        Err(e) => Err(error!(format!("Type alias do not match the registered type alias, expected {}, got {}", as_constant, name)))
                    }
                }
            }
        },

        // Bool
        "Bool" => Ok(ConstantType::PlcBool(PlcBool::try_from(json)?)),

        // Binary
        "Byte" | "Word" | "DWord" | "LWord" => Ok(ConstantType::PlcBinary(PlcBinary::try_from(json)?)),

        // Integers
        "SInt" | "Int" | "DInt" | "LInt" | "USInt" | "UInt" | "UDInt" | "ULInt" =>
            Ok(ConstantType::PlcInteger(PlcInteger::try_from((json))?)),

        // Floats
        "Real" | "LReal" => Ok(ConstantType::PlcFloat(PlcFloat::try_from(json)?)),

        // Time
        "Time" | "LTime" => Ok(ConstantType::PlcTime(PlcTime::try_from(json)?)),

        // TOD
        "Tod" | "LTod" => Ok(ConstantType::PlcTod(PlcTod::try_from(json)?)),

        //String
        "String" | "Char" | "WString" | "WChar" => Ok(ConstantType::PlcString(PlcString::try_from(json)?)),
        _ => Err(error!(format!("Unknown constant type: {}", ty))),
    }
}

pub fn create_default_constant_from_str(identifier: &str) -> Result<ConstantType, Stop> {
    match identifier {
        "Bool" => Ok(ConstantType::PlcBool(PlcBool::Bool(Bool::new_default(0)))),
        "Byte" => Ok(ConstantType::PlcBinary(PlcBinary::Byte(Byte::new_default(0)))),
        "Word" => Ok(ConstantType::PlcBinary(PlcBinary::Word(Word::new_default(0)))),
        "DWord" => Ok(ConstantType::PlcBinary(PlcBinary::DWord(DWord::new_default(0)))),
        "LWord" => Ok(ConstantType::PlcBinary(PlcBinary::LWord(LWord::new_default(0)))),
        "SInt" => Ok(ConstantType::PlcInteger(PlcInteger::SInt(SInt::new_default(0)))),
        "Int" => Ok(ConstantType::PlcInteger(PlcInteger::Int(Int::new_default(0)))),
        "DInt" => Ok(ConstantType::PlcInteger(PlcInteger::DInt(DInt::new_default(0)))),
        "LInt" => Ok(ConstantType::PlcInteger(PlcInteger::LInt(LInt::new_default(0)))),
        "USInt" => Ok(ConstantType::PlcInteger(PlcInteger::USInt(USInt::new_default(0)))),
        "UInt" => Ok(ConstantType::PlcInteger(PlcInteger::UInt(UInt::new_default(0)))),
        "UDInt" => Ok(ConstantType::PlcInteger(PlcInteger::UDInt(UDInt::new_default(0)))),
        "ULInt" => Ok(ConstantType::PlcInteger(PlcInteger::ULInt(ULInt::new_default(0)))),
        "Real" => Ok(ConstantType::PlcFloat(PlcFloat::Real(Real::new_default(0)))),
        "LReal" => Ok(ConstantType::PlcFloat(PlcFloat::LReal(LReal::new_default(0)))),
        "Time" => Ok(ConstantType::PlcTime(PlcTime::Time(Time::new_default(0)))),
        "LTime" => Ok(ConstantType::PlcTime(PlcTime::LTime(LTime::new_default(0)))),
        "Tod" => Ok(ConstantType::PlcTod(PlcTod::Tod(Tod::new_default(0)))),
        "LTod" => Ok(ConstantType::PlcTod(PlcTod::LTod(LTod::new_default(0)))),
        _ => Err(error!(format!("Invalid type {}", identifier)))
    }
}