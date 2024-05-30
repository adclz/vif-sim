use core::cell::RefCell;
use core::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::kernel::registry::Kernel;

use crate::kernel::plc::types::primitives::binaries::plc_binary::PlcBinary;
use crate::kernel::plc::types::primitives::boolean::plc_bool::PlcBool;

use crate::kernel::plc::types::primitives::floats::plc_float::PlcFloat;
use crate::kernel::plc::types::primitives::integers::plc_integer::PlcInteger;
use crate::kernel::plc::types::primitives::traits::primitive_traits::RawMut;
use crate::kernel::plc::types::primitives::string::plc_string::PlcString;
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::kernel::plc::types::primitives::timers::plc_time::PlcTime;
use crate::kernel::plc::types::primitives::tod::plc_tod::PlcTod;
use camelpaste::paste;

use crate::kernel::plc::types::primitives::boolean::bool::Bool;
use crate::kernel::plc::types::primitives::boolean::bit_access::BitAccess;

use crate::kernel::plc::types::primitives::binaries::byte::Byte;
use crate::kernel::plc::types::primitives::binaries::dword::DWord;
use crate::kernel::plc::types::primitives::binaries::lword::LWord;
use crate::kernel::plc::types::primitives::binaries::word::Word;

use crate::kernel::plc::types::primitives::integers::dint::DInt;
use crate::kernel::plc::types::primitives::integers::int::Int;
use crate::kernel::plc::types::primitives::integers::lint::LInt;
use crate::kernel::plc::types::primitives::integers::sint::SInt;
use crate::kernel::plc::types::primitives::integers::udint::UDInt;
use crate::kernel::plc::types::primitives::integers::uint::UInt;
use crate::kernel::plc::types::primitives::integers::ulint::ULInt;
use crate::kernel::plc::types::primitives::integers::usint::USInt;

use crate::kernel::plc::types::primitives::floats::lreal::LReal;
use crate::kernel::plc::types::primitives::floats::real::Real;

use crate::kernel::plc::types::primitives::string::_char::_Char;
use crate::kernel::plc::types::primitives::string::_string::_String;
use crate::kernel::plc::types::primitives::string::wchar::WChar;
use crate::kernel::plc::types::primitives::string::wstring::WString;

use crate::kernel::plc::types::primitives::timers::lTime::LTime;
use crate::kernel::plc::types::primitives::timers::time::Time;

use crate::kernel::plc::types::primitives::tod::ltod::LTod;
use crate::kernel::plc::types::primitives::tod::tod::Tod;

///
/// Metadata
///
/// name: Family type (Int, Bool ...) or Complex (Array, Struct ...)
///
/// alias: Udt Name, Instance of, or primitive alias
///
#[enum_dispatch::enum_dispatch]
pub trait MetaData {
    fn name(&self) -> &'static str;
    fn get_alias_str<'a>(&'a self, kernel: &'a Kernel) -> Option<&'a String>;
    fn get_alias_id(&self, kernel: &Kernel) -> Option<usize>;
    fn is_read_only(&self) -> bool;
    fn get_path(&self) -> String;
}

#[derive(Clone)]
pub enum HeapOrStatic {
    Static(&'static str),
    Heap(String),
    Closure(Rc<RefCell<dyn Fn() -> String>>)
}

impl Display for HeapOrStatic {
    fn fmt(&self, f: &mut Formatter<'_>, ) -> core::fmt::Result {
        match self {
            HeapOrStatic::Static(a) => write!(f, "{}", a),
            HeapOrStatic::Heap(a) => write!(f, "{}", a),
            HeapOrStatic::Closure(a) => write!(f, "{}", a.borrow()())
        }
    }
}

#[derive(Clone)]
pub struct MaybeHeapOrStatic(pub Option<HeapOrStatic>);

impl Display for MaybeHeapOrStatic {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            None => write!(f, "[internal]"),
            Some(a) => write!(f, "{}", a)
        }
    }
}

#[enum_dispatch::enum_dispatch]
pub trait SetMetaData : MetaData {
    fn set_alias(&mut self, alias: &str, kernel: &Kernel);
    fn set_read_only(&mut self, value: bool);
    fn set_name(&mut self, path: usize);
}