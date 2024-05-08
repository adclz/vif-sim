use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::meta_data::{MetaData, SetMetaData};
use crate::kernel::plc::types::primitives::traits::primitive_traits::*;
use crate::{error, impl_primitive_base, impl_primitive_raw_mut, impl_primitive_serialize, impl_primitive_type_name};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::Value;
use smart_default::SmartDefault;
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use crate::kernel::plc::types::primitives::floats::checked_float::TryIntoCheck;
use std::borrow::Cow;
use crate::kernel::plc::types::primitives::binaries::byte::Byte;
use crate::kernel::registry::Kernel;
use crate::kernel::registry::get_string;

#[derive(Clone, SmartDefault)]
pub struct Real {
    default: f32,
    value: f32,
    #[default(_code = "get_id()")]
    id: usize,
    monitor: bool,
    read_only: bool,
    alias: Option<usize>,
    path: usize
}

impl TryFrom<&Value> for Real {
    type Error = Stop;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_f64() {
            None => Err(error!(format!("Invalid value {} for Real", value))),
            Some(a) => Ok(Real::new(&TryIntoCheck::try_into(a)?)?),
        }
    }
}

impl_primitive_serialize!(Real, f32);
impl_primitive_type_name!(Real, f32);
impl_primitive_raw_mut!(Real, f32);
impl_primitive_base!(Real, f32);


pub fn scale(f: f32) -> usize {
    match f as i64 {
        0 => 7,
        1..=10 => 6,
        11..=100 => 5,
        101..=1000 => 4,
        1001..=10000 => 3,
        10001..=100000 => 2,
        _ => 1
    }
}

impl Display for Real {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(Real: {float:.scale$})", self.get_path(), float = self.value, scale = scale(self.value))
    }
}


impl RawDisplay for Real {
    fn raw_display<'a>(&'a self) -> impl Display +'a {
        struct Raw<'a>(&'a Real);
        impl<'a> Display for Raw<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{float:.scale$}", float = self.0.value, scale = scale(self.0.value))
            }
        }
        Raw(self)
    }
}
