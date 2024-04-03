use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::error;
use crate::parser::local_type::constant_type::{parse_constant_type};
use crate::plc::complex::instance::public::PublicInstanceTrait;
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::interface::traits::InterfaceAccessors;
use crate::plc::internal::template_impl::TemplateMemory;
use crate::plc::operations::operations::{BuildJsonOperation, JsonOperation, RunTimeOperation};
use crate::plc::primitives::binaries::byte::Byte;
use crate::plc::primitives::binaries::dword::DWord;
use crate::plc::primitives::binaries::lword::LWord;
use crate::plc::primitives::binaries::plc_binary::PlcBinary;
use crate::plc::primitives::binaries::word::Word;
use crate::plc::primitives::boolean::bit_access::BitAccess;
use crate::plc::primitives::boolean::bool::Bool;
use crate::plc::primitives::boolean::plc_bool::PlcBool;
use crate::plc::primitives::floats::lreal::LReal;
use crate::plc::primitives::floats::plc_float::PlcFloat;
use crate::plc::primitives::floats::real::Real;
use crate::plc::primitives::integers::dint::DInt;
use crate::plc::primitives::integers::int::Int;
use crate::plc::primitives::integers::lint::LInt;
use crate::plc::primitives::integers::plc_integer::PlcInteger;
use crate::plc::primitives::integers::sint::SInt;
use crate::plc::primitives::integers::udint::UDInt;
use crate::plc::primitives::integers::uint::UInt;
use crate::plc::primitives::integers::ulint::ULInt;
use crate::plc::primitives::integers::usint::USInt;
use crate::plc::primitives::primitive_traits::PrimitiveTrait;
use crate::plc::primitives::string::_char::_Char;
use crate::plc::primitives::string::_string::_String;
use crate::plc::primitives::string::plc_string::PlcString;
use crate::plc::primitives::string::wchar::WChar;
use crate::plc::primitives::string::wstring::WString;
use crate::plc::primitives::timers::lTime::LTime;
use crate::plc::primitives::timers::plc_time::PlcTime;
use crate::plc::primitives::timers::time::Time;
use crate::plc::primitives::tod::ltod::LTod;
use crate::plc::primitives::tod::plc_tod::PlcTod;
use crate::plc::primitives::tod::tod::Tod;
use crate::registry::any::any_type::AnyRefType;
use crate::registry::constant::r#type::ConstantType;
use crate::registry::global::pointer::GlobalPointer;
use crate::registry::local::pointer::LocalPointer;
use crate::registry::local::r#type::LocalType;
use crate::registry::registry::{GlobalOrLocal, Kernel};
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum JsonTarget {
    Local(Vec<String>),
    // Local reference
    LocalOut(Vec<String>),
    // Local reference outside
    Global(Vec<String>),
    // Global pointer
    Constant(Map<String, Value>),
    // Constant value
    Inner(Vec<String>),
    // Template reference
    Operation(Box<JsonOperation>),
    // Any operation
    Access(Map<String, Value>), // Access
}

impl Display for JsonTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonTarget::Local(a) => writeln!(f, "Local -> {:?}", a),
            JsonTarget::LocalOut(a) => writeln!(f, "Local Out -> {:?}", a),
            JsonTarget::Global(a) => writeln!(f, "Global -> {:?}", a),
            JsonTarget::Constant(a) => writeln!(f, "Constant -> {:?}", a),
            JsonTarget::Inner(a) => writeln!(f, "Inner -> {:?}", a),
            JsonTarget::Operation(_a) => writeln!(f, "Operation"),
            JsonTarget::Access(a) => writeln!(f, "Access {:?}", a),
        }
    }
}

impl JsonTarget {
    pub fn is_constant(&self) -> bool {
        matches!(self, Self::Constant(_))
    }
    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local(_))
    }
    pub fn is_inner(&self) -> bool {
        matches!(self, Self::Inner(_))
    }
    pub fn is_local_out(&self) -> bool {
        matches!(self, Self::LocalOut(_))
    }
    pub fn is_global(&self) -> bool {
        matches!(self, Self::Global(_))
    }
    pub fn is_operation(&self) -> bool {
        matches!(self, Self::Operation(_))
    }
    pub fn is_access(&self) -> bool { matches!(self, Self::Access(_)) }

    pub fn solve_as_local_pointer(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast,
    ) -> Option<LocalPointer> {
        match self {
            Self::LocalOut(global) => match registry.get_and_find_nested(global) {
                Some(GlobalOrLocal::Local(a)) => Some(a.clone()),
                _ => None,
            },
            Self::Local(local) => interface.try_get_nested(local),
            Self::Inner(inner) => template.and_then(|x| x.try_get_nested(inner)),
            Self::Access(access) => {
                Some(LocalPointer::from(LocalType::PlcBool(PlcBool::BitAccess(
                    BitAccess::new_(access, interface, template, registry, channel).unwrap(),
                ))))
                // todo eventually add other slice access (Byte, Word ...)
            }
            _ => None,
        }
    }

    pub fn solve_as_global_pointer(&self, registry: &Kernel) -> Option<GlobalPointer> {
        match self {
            Self::Global(global) => registry.get(&global[0]).clone(),
            _ => None,
        }
    }

    pub fn solve_as_operation(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast,
    ) -> Result<RunTimeOperation, Stop> {
        match self {
            Self::Operation(op) => op.build(interface, template, registry, channel),
            _ => Err(error!(format!("Expected operation, found {}", self))),
        }
    }

    pub fn get_raw_constant(&self) -> Option<&Map<String, Value>> {
        match self {
            Self::Constant(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn solve_to_ref(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        force_constant_type: Option<LocalType>,
        registry: &Kernel,
        channel: &Broadcast,
    ) -> Result<AnyRefType, Stop> {
        match self {
            Self::Local(a) | Self::LocalOut(a) | Self::Inner(a) => Ok(AnyRefType::Local(
                self.solve_as_local_pointer(interface, template, registry, channel)
                    .ok_or(error!(format!(
                        "Could not solve json target as local reference {:?}",
                        a
                    )))?,
            )),
            Self::Access(a) => Ok(AnyRefType::Local(
                self.solve_as_local_pointer(interface, template, registry, channel)
                    .ok_or(error!(format!(
                        "Could not solve json target as local reference {:?}",
                        a
                    )))?,
            )),
            Self::Constant(..) => Ok(AnyRefType::Constant(
                self.solve_as_constant(&registry, force_constant_type)?,
            )),
            Self::Operation(..) => Ok(AnyRefType::Operation(
                self.solve_as_operation(interface, template, registry, channel)?,
            )),
            _ => Err(error!(format!(
                "A global reference can not be used in this context {}",
                self
            ))),
        }
    }

    pub fn solve_as_constant(
        &self,
        registry: &Kernel,
        force_constant_type: Option<LocalType>,
    ) -> Result<ConstantType, Stop> {
        match self {
            Self::Constant(inner) => {
                let result = parse_constant_type(inner, registry, force_constant_type)?;

                // Checks if type is allowed
                registry.check_excluded_type(&result)?;

                Ok(result)
            }
            _ => Err(error!("Invalid constant type".to_string())),
        }
    }
}
