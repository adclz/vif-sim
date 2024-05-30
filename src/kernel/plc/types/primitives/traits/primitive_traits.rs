use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use core::any::TypeId;

use crate::kernel::plc::types::primitives::binaries::plc_binary::PlcBinary;
use crate::kernel::plc::types::primitives::boolean::plc_bool::PlcBool;

use crate::kernel::plc::types::primitives::floats::plc_float::PlcFloat;
use crate::kernel::plc::types::primitives::integers::plc_integer::PlcInteger;
use crate::kernel::plc::types::primitives::string::plc_string::PlcString;
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::kernel::plc::types::primitives::timers::plc_time::PlcTime;
use crate::kernel::plc::types::primitives::tod::plc_tod::PlcTod;
use camelpaste::paste;

use core::fmt::{Display, Formatter};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

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
use crate::kernel::plc::types::primitives::string::_string::{_String, plcstr};
use crate::kernel::plc::types::primitives::string::wchar::WChar;
use crate::kernel::plc::types::primitives::string::wstring::{plcwstr, WString};

use crate::kernel::plc::types::primitives::timers::lTime::LTime;
use crate::kernel::plc::types::primitives::timers::time::Time;

use crate::kernel::plc::types::primitives::tod::ltod::LTod;
use crate::kernel::plc::types::primitives::tod::tod::Tod;
use crate::kernel::registry::Kernel;

pub trait RawMut {
    fn reset_ptr(&mut self, channel: &Broadcast);
}

#[enum_dispatch::enum_dispatch]
pub trait ToggleMonitor {
    fn set_monitor(&self, kernel: &Kernel);
}

pub trait SerializeValue {
    fn get_value(&self) -> JsValue;
}

pub trait PrimitiveTrait {
    type Native;
    type PlcPrimitive;
    /// Creates a new PlcPrimitive from a native type.
    ///
    /// The provided value is also the default value.
    fn new(value: &Self::Native, id: u32) -> Result<Self::PlcPrimitive, Stop>;
    
    fn new_default(id: u32) -> Self::PlcPrimitive;

    /// Borrows the value field.
    fn get(&self, channel: &Broadcast) -> Result<Self::Native, Stop>;

    /// Sets the value from native.
    ///
    /// If monitor is set, this will trigger a monitor event.
    fn set(&mut self, value: Self::Native, channel: &Broadcast) -> Result<(), Stop>;

    fn set_default(&mut self, value: Self::Native) -> Result<(), Stop>;

    /// Resets the native value.
    ///
    /// Basically the value field is a copy of the default field.
    fn reset(&mut self, channel: &Broadcast);

    /// Returns the id of this type.
    ///
    /// Since ids are defined with an AtomicUsize, they are all unique
    fn get_id(&self) -> u32;

    fn get_type_id(&self) -> TypeId;
}

macro_rules! primitive_traits {
    ($($primitive: ident),+) => {
        paste! {
            #[enum_dispatch::enum_dispatch]
            pub trait Primitive {
                $(
                    fn [<is_$primitive>](&self) -> bool;
                    fn [<as_$primitive>](&self, channel: &Broadcast) -> Result<$primitive, Stop>;
                )+
            }
            #[enum_dispatch::enum_dispatch]
            pub trait AsMutPrimitive {
                $(
                     fn [<set_$primitive>](&mut self, other: $primitive, channel: &Broadcast) -> Result<(), Stop>;
                     fn [<set_default_$primitive>](&mut self, other: $primitive) -> Result<(), Stop>;
                )+
            }
            #[enum_dispatch::enum_dispatch]
            pub trait WithMut {
                 $(fn [<with_mut_$primitive>]<R>(&self, channel: &Broadcast, f: &mut impl FnMut(&mut $primitive) -> R) -> Result<R, Stop>;)+
            }
        }
    };
}

primitive_traits!(
    bool, u8, i8, u16, i16, u32, i32, u64, i64, f32, f64, plcstr, char, plcwstr, wchar
);

pub trait RawDisplay {
    fn raw_display<'a>(&'a self) -> impl Display + 'a;
}