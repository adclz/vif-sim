use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::meta_data::{MetaData, SetMetaData};
use crate::kernel::plc::types::primitives::traits::primitive_traits::*;
use crate::{error, impl_primitive_base, impl_primitive_raw_mut, impl_primitive_serialize, impl_primitive_type_name, key_reader};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use core::any::{Any, TypeId};
use core::fmt::{Display, Formatter};
use std::borrow::Cow;
use crate::kernel::plc::types::primitives::floats::real::Real;
use crate::kernel::registry::Kernel;
use crate::kernel::registry::get_string;

#[derive(Clone)]
pub struct LReal {
    default: f64,
    value: f64,
    id: u32,
    read_only: bool,
    alias: Option<usize>,
    path: usize
}

impl_primitive_serialize!(LReal, f64);
impl_primitive_type_name!(LReal, f64);
impl_primitive_raw_mut!(LReal, f64);
impl_primitive_base!(LReal, f64);


impl TryFrom<&Map<String, Value>> for LReal {
    type Error = Stop;

    fn try_from(data: &Map<String, Value>) -> Result<Self, Self::Error> {
        key_reader!(
            format!("Parse LReal"),
            data {
                value,
                id => as_u64,
            }
        );
        let id = id as u32;
        if let Some(a) = value.as_str() {
            let h = a
                .parse::<f64>()
                .map_err(|_| error!(format!("Failed to parse LReal from str {}", a)))?;
            Ok(LReal::new(&h, id)?)
        } else if let Some(a) = value.as_f64() {
            Ok(LReal::new(&a, id)?)
        } else {
            Err(error!(format!("Invalid value {} for LReal", value)))
        }
    }
}

pub fn scale(f: f64) -> usize {
    match f as i64 {
        0 => 15,
        1..=10 => 14,
        11..=100 => 13,
        101..=1000 => 12,
        1001..=10000 => 11,
        10001..=100000 => 10,
        100001..=1000000 => 9,
        1000001..=10000000 => 8,
        10000001..=100000000 => 7,
        100000001..=1000000000 => 6,
        1000000001..=10000000000 => 5,
        10000000001..=100000000000 => 4,
        100000000001..=1000000000000 => 3,
        1000000000001..=10000000000000 => 2,
        _ => 1
    }
}

impl Display for LReal {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}(LReal: {float:.scale$})", self.get_path(), float = self.value, scale = scale(self.value))
    }
}

impl RawDisplay for LReal {
    fn raw_display<'a>(&'a self) -> impl Display +'a {
        struct Raw<'a>(&'a LReal);
        impl<'a> Display for Raw<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                write!(f, "{float:.scale$}", float = self.0.value, scale = scale(self.0.value))
            }
        }
        Raw(self)
    }
}