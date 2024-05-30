#![allow(non_snake_case)]

use core::cell::RefCell;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::complex::array::PlcArray;
use crate::kernel::plc::types::complex::instance::fb_instance::FbInstance;
use crate::kernel::plc::types::complex::r#struct::PlcStruct;
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

use core::fmt::{Display, Formatter};
use std::rc::Rc;

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
use crate::kernel::registry::Kernel;
use crate::kernel::arch::local::r#type::LocalType;
use crate::kernel::arch::any::any_type::AnyRefType;
use crate::kernel::arch::constant::r#type::ConstantType;

pub trait GetRawPointerPrimitive {
    fn get_raw_pointer(&mut self) -> *mut dyn RawMut;
}

macro_rules! create_families_traits {
    ($($family: ident),+) => {
        paste! {
            #[enum_dispatch::enum_dispatch]
            pub trait IsFamily {
                $(fn [<is_$family:snake>](&self) -> bool;)+
                fn is_complex(&self) -> bool;
                fn is_primitive(&self) -> bool {
                    !self.is_complex()
                }
            }

            #[enum_dispatch::enum_dispatch]
            pub trait WithRefFamily {
                $(fn [<with_$family:snake>]<R>(&self, channel: &Broadcast, f: impl Fn(&$family) -> R) -> Result<R, Stop>;)+
            }

            #[enum_dispatch::enum_dispatch]
            pub trait WithTypeFamily {
                $(fn [<with_type_$family:snake>]<R>(&self, f: impl Fn(&$family) -> R) -> Result<R, Stop>;)+
            }

            #[enum_dispatch::enum_dispatch]
            pub trait WithMutFamily {
                 $(fn [<with_mut_$family:snake>]<R>(&self, channel: &Broadcast, f: &mut impl FnMut(&mut $family) -> R) -> Result<R, Stop>;)+
            }
        }
    };
}

create_families_traits!(
    PlcBool, PlcInteger, PlcFloat, PlcBinary, PlcTime, PlcTod, PlcString, PlcArray,
    PlcStruct, FbInstance
);
