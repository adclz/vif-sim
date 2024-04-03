#![allow(unused_imports)]
use crate::error;
use crate::plc::operations::operations::{JsonOperation, Operation, RunTimeOperation};
use crate::registry::constant::r#type::ConstantType;
use crate::registry::global::pointer::GlobalPointer;
use crate::registry::global::r#type::GlobalType;
use crate::registry::local::r#type::{LocalType, IntoLocalType};
use crate::registry::local::pointer::LocalPointer;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use camelpaste::paste;
use std::fmt::{Display, Formatter};

use crate::plc::complex::array::PlcArray;
use crate::plc::complex::instance::fb_instance::FbInstance;
use crate::plc::complex::r#struct::PlcStruct;
use crate::plc::primitives::binaries::plc_binary::PlcBinary;
use crate::plc::primitives::floats::plc_float::PlcFloat;
use crate::plc::primitives::boolean::plc_bool::PlcBool;
use crate::plc::primitives::integers::plc_integer::PlcInteger;
use crate::plc::primitives::string::plc_string::PlcString;
use crate::plc::primitives::timers::plc_time::PlcTime;
use crate::plc::primitives::tod::plc_tod::PlcTod;
use crate::plc::primitives::family_traits::{IsFamily, WithTypeFamily, WithRefFamily, WithMutFamily, Primitive, MetaData};
use std::ops::Deref;
use fixedstr::str256;
use crate::plc::primitives::string::wstring::wstr256;
use crate::plc::primitives::string::wchar::wchar;
use std::borrow::Cow;
use crate::registry::registry::Kernel;

#[enum_dispatch::enum_dispatch(IsFamily, WithTypeFamily, WithRefFamily, Primitive, IntoLocalType, MetaData)]
#[derive(Clone)]
pub enum AnyRefType {
    Local(LocalPointer),
    Constant(ConstantType),
    Operation(RunTimeOperation),
}

impl Display for AnyRefType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyRefType::Local(a) => write!(f, "{}", a),
            AnyRefType::Constant(a) => write!(f, "{}", a),
            AnyRefType::Operation(a) => write!(f, "{}", a),
        }
    }
}
