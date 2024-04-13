use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::error;
use crate::parser::local_type::constant_type::{parse_constant_type};
use crate::kernel::plc::types::complex::instance::public::PublicInstanceTrait;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::interface::traits::InterfaceAccessors;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::kernel::plc::operations::operations::{BuildJsonOperation, JsonOperation, RunTimeOperation};
use crate::kernel::plc::types::primitives::binaries::byte::Byte;
use crate::kernel::plc::types::primitives::binaries::dword::DWord;
use crate::kernel::plc::types::primitives::binaries::lword::LWord;
use crate::kernel::plc::types::primitives::binaries::plc_binary::PlcBinary;
use crate::kernel::plc::types::primitives::binaries::word::Word;
use crate::kernel::plc::types::primitives::boolean::bit_access::BitAccess;
use crate::kernel::plc::types::primitives::boolean::bool::Bool;
use crate::kernel::plc::types::primitives::boolean::plc_bool::PlcBool;
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
use crate::kernel::plc::types::primitives::traits::primitive_traits::PrimitiveTrait;
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
use crate::kernel::arch::any::any_type::AnyRefType;
use crate::kernel::arch::constant::r#type::ConstantType;
use crate::kernel::arch::global::pointer::GlobalPointer;
use crate::kernel::arch::local::pointer::LocalPointer;
use crate::kernel::arch::local::r#type::LocalType;
use crate::kernel::registry::{convert_string_path_to_usize, get_or_insert_global_string, GlobalOrLocal, Kernel};
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
            Self::LocalOut(global) => match registry.get_and_find_nested(&convert_string_path_to_usize(global)) {
                Some(GlobalOrLocal::Local(a)) => Some(a.clone()),
                _ => None,
            },
            Self::Local(local) => interface.try_get_nested(&convert_string_path_to_usize(local)),
            Self::Inner(inner) => template.and_then(|x| x.try_get_nested(&convert_string_path_to_usize(inner))),
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
            Self::Global(global) => registry.get(&get_or_insert_global_string(&global[0])).clone(),
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
                    .ok_or_else(move || error!(format!(
                        "Could not solve json target as local reference {:?}",
                        a
                    )))?,
            )),
            Self::Access(a) => Ok(AnyRefType::Local(
                self.solve_as_local_pointer(interface, template, registry, channel)
                    .ok_or_else(move || error!(format!(
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
