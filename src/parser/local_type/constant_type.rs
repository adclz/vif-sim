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
            key_reader!(
                format!("Parse constant value"),
                src {
                    value,
                }
            );
            match force_constant_type {
                None => Err(error!("Implicit constant but vif could not determine the context. Maybe wrap your value in a primitive ?".to_string())),
                Some(a) => {
                    match a {
                        LocalType::PlcBool(_) => Ok(ConstantType::PlcBool(PlcBool::Bool(Bool::try_from(value)?))),
                        LocalType::PlcInteger(b) => {
                            match b {
                                PlcInteger::USInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::USInt(USInt::try_from(value)?))),
                                PlcInteger::SInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::SInt(SInt::try_from(value)?))),
                                PlcInteger::UInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::UInt(UInt::try_from(value)?))),
                                PlcInteger::Int(_) => Ok(ConstantType::PlcInteger(PlcInteger::Int(Int::try_from(value)?))),
                                PlcInteger::UDInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::UDInt(UDInt::try_from(value)?))),
                                PlcInteger::DInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::DInt(DInt::try_from(value)?))),
                                PlcInteger::ULInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::ULInt(ULInt::try_from(value)?))),
                                PlcInteger::LInt(_) => Ok(ConstantType::PlcInteger(PlcInteger::LInt(LInt::try_from(value)?))),
                            }
                        }
                        LocalType::PlcFloat(b) => {
                            match b {
                                PlcFloat::Real(_) => Ok(ConstantType::PlcFloat(PlcFloat::Real(Real::try_from(value)?))),
                                PlcFloat::LReal(_) => Ok(ConstantType::PlcFloat(PlcFloat::LReal(LReal::try_from(value)?))),
                            }
                        }
                        LocalType::PlcBinary(b) => {
                            match b {
                                PlcBinary::Byte(_) => Ok(ConstantType::PlcBinary(PlcBinary::Byte(Byte::try_from(value)?))),
                                PlcBinary::Word(_) => Ok(ConstantType::PlcBinary(PlcBinary::Word(Word::try_from(value)?))),
                                PlcBinary::DWord(_) => Ok(ConstantType::PlcBinary(PlcBinary::DWord(DWord::try_from(value)?))),
                                PlcBinary::LWord(_) => Ok(ConstantType::PlcBinary(PlcBinary::LWord(LWord::try_from(value)?))),
                            }
                        }
                        LocalType::PlcTime(b) => {
                            match b {
                                PlcTime::Time(_) => Ok(ConstantType::PlcTime(PlcTime::Time(Time::try_from(value)?))),
                                PlcTime::LTime(_) => Ok(ConstantType::PlcTime(PlcTime::LTime(LTime::try_from(value)?))),
                            }
                        }
                        LocalType::PlcTod(b) => {
                            match b {
                                PlcTod::Tod(_) => Ok(ConstantType::PlcTod(PlcTod::Tod(Tod::try_from(value)?))),
                                PlcTod::LTod(_) => Ok(ConstantType::PlcTod(PlcTod::LTod(LTod::try_from(value)?))),
                            }
                        }
                        LocalType::PlcString(b) => {
                            match b {
                                PlcString::_Char(_) => Ok(ConstantType::PlcString(PlcString::_Char(_Char::try_from(value)?))),
                                PlcString::_String(_) => Ok(ConstantType::PlcString(PlcString::_String(_String::try_from(value)?))),
                                PlcString::WChar(_) => Ok(ConstantType::PlcString(PlcString::WChar(WChar::try_from(value)?))),
                                PlcString::WString(_) => Ok(ConstantType::PlcString(PlcString::WString(WString::try_from(value)?))),
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
        "Bool" => Ok(ConstantType::PlcBool(PlcBool::try_from((src, ty))?)),

        // Binary
        "Byte" | "Word" | "DWord" | "LWord" => Ok(ConstantType::PlcBinary(PlcBinary::try_from((src, ty))?)),

        // Integers
        "SInt" | "Int" | "DInt" | "LInt" | "USInt" | "UInt" | "UDInt" | "ULInt" =>
            Ok(ConstantType::PlcInteger(PlcInteger::try_from((src, ty))?)),

        // Floats
        "Real" | "LReal" => Ok(ConstantType::PlcFloat(PlcFloat::try_from((src, ty))?)),

        // Time
        "Time" | "LTime" => Ok(ConstantType::PlcTime(PlcTime::try_from((src, ty))?)),

        // TOD
        "Tod" | "LTod" => Ok(ConstantType::PlcTod(PlcTod::try_from((src, ty))?)),

        //String
        "String" | "Char" | "WString" | "WChar" => Ok(ConstantType::PlcString(PlcString::try_from((src, ty))?)),
        _ => Err(error!(format!("Unknown constant type: {}", ty))),
    }
}

pub fn create_default_constant_from_str(identifier: &str) -> Result<ConstantType, Stop> {
    match identifier {
        "Bool" => Ok(ConstantType::PlcBool(PlcBool::Bool(Bool::default()))),
        "Byte" => Ok(ConstantType::PlcBinary(PlcBinary::Byte(Byte::default()))),
        "Word" => Ok(ConstantType::PlcBinary(PlcBinary::Word(Word::default()))),
        "DWord" => Ok(ConstantType::PlcBinary(PlcBinary::DWord(DWord::default()))),
        "LWord" => Ok(ConstantType::PlcBinary(PlcBinary::LWord(LWord::default()))),
        "SInt" => Ok(ConstantType::PlcInteger(PlcInteger::SInt(SInt::default()))),
        "Int" => Ok(ConstantType::PlcInteger(PlcInteger::Int(Int::default()))),
        "DInt" => Ok(ConstantType::PlcInteger(PlcInteger::DInt(DInt::default()))),
        "LInt" => Ok(ConstantType::PlcInteger(PlcInteger::LInt(LInt::default()))),
        "USInt" => Ok(ConstantType::PlcInteger(PlcInteger::USInt(USInt::default()))),
        "UInt" => Ok(ConstantType::PlcInteger(PlcInteger::UInt(UInt::default()))),
        "UDInt" => Ok(ConstantType::PlcInteger(PlcInteger::UDInt(UDInt::default()))),
        "ULInt" => Ok(ConstantType::PlcInteger(PlcInteger::ULInt(ULInt::default()))),
        "Real" => Ok(ConstantType::PlcFloat(PlcFloat::Real(Real::default()))),
        "LReal" => Ok(ConstantType::PlcFloat(PlcFloat::LReal(LReal::default()))),
        "Time" => Ok(ConstantType::PlcTime(PlcTime::Time(Time::default()))),
        "LTime" => Ok(ConstantType::PlcTime(PlcTime::LTime(LTime::default()))),
        "Tod" => Ok(ConstantType::PlcTod(PlcTod::Tod(Tod::default()))),
        "LTod" => Ok(ConstantType::PlcTod(PlcTod::LTod(LTod::default()))),
        _ => Err(error!(format!("Invalid type {}", identifier)))
    }
}