#![allow(unused_imports)]
use crate::error;
use crate::kernel::plc::operations::operations::{JsonOperation, Operation, RunTimeOperation};
use crate::kernel::arch::constant::r#type::ConstantType;
use crate::kernel::arch::global::pointer::GlobalPointer;
use crate::kernel::arch::global::r#type::GlobalType;
use crate::kernel::arch::local::r#type::{LocalType, IntoLocalType};
use crate::kernel::arch::local::pointer::LocalPointer;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use camelpaste::paste;
use core::fmt::{Display, Formatter};

use crate::kernel::plc::types::complex::array::PlcArray;
use crate::kernel::plc::types::complex::instance::fb_instance::FbInstance;
use crate::kernel::plc::types::complex::r#struct::PlcStruct;
use crate::kernel::plc::types::primitives::binaries::plc_binary::PlcBinary;
use crate::kernel::plc::types::primitives::floats::plc_float::PlcFloat;
use crate::kernel::plc::types::primitives::boolean::plc_bool::PlcBool;
use crate::kernel::plc::types::primitives::integers::plc_integer::PlcInteger;
use crate::kernel::plc::types::primitives::string::plc_string::PlcString;
use crate::kernel::plc::types::primitives::timers::plc_time::PlcTime;
use crate::kernel::plc::types::primitives::tod::plc_tod::PlcTod;
use crate::kernel::plc::types::primitives::traits::family_traits::{IsFamily, WithTypeFamily, WithRefFamily, WithMutFamily};
use crate::kernel::plc::types::primitives::traits::primitive_traits::{Primitive};
use crate::kernel::plc::types::primitives::traits::meta_data::{MetaData, HeapOrStatic, MaybeHeapOrStatic};
use core::ops::Deref;
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use std::borrow::Cow;
use crate::kernel::registry::Kernel;
use crate::kernel::plc::types::primitives::string::_string::plcstr;
use crate::kernel::plc::types::primitives::string::wstring::plcwstr;
#[enum_dispatch::enum_dispatch(IsFamily, WithTypeFamily, WithRefFamily, Primitive, IntoLocalType, MetaData)]
#[derive(Clone)]
pub enum AnyRefType {
    Local(LocalPointer),
    Constant(ConstantType),
    Operation(RunTimeOperation),
}

impl Display for AnyRefType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            AnyRefType::Local(a) => write!(f, "{}", a),
            AnyRefType::Constant(a) => write!(f, "{}", a),
            AnyRefType::Operation(a) => write!(f, "{}", a),
        }
    }
}