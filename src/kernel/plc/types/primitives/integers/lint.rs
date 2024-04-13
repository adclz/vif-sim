use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::*;
use crate::kernel::plc::types::primitives::traits::primitive_traits::*;
use crate::kernel::plc::types::primitives::traits::meta_data::*;
use crate::kernel::plc::types::primitives::traits::crement::Crement;
use crate::{error, impl_primitive_all, impl_primitive_crement};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::Value;
use smart_default::SmartDefault;
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use std::borrow::Cow;
use crate::kernel::registry::Kernel;
use crate::kernel::registry::get_full_path;

#[derive(Clone, SmartDefault)]
pub struct LInt {
    default: i64,
    value: i64,
    #[default(_code = "get_id()")]
    id: usize,
    monitor: bool,
    read_only: bool,
    alias: Option<usize>,
    path: Vec<usize>
}

impl_primitive_crement!(LInt);
impl_primitive_all!(LInt, i64);

impl TryFrom<&Value> for LInt {
    type Error = Stop;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        if let Some(a) = value.as_str() {
            let h = a
                .parse::<i64>()
                .map_err(|_| error!(format!("Failed to parse LInt from Bigint {}", a)))?;
            Ok(LInt::new(&h)?)
        } else if let Some(a) = value.as_i64() {
            Ok(LInt::new(&a)?)
        } else {
            Err(error!(format!("Invalid value {} for LInt", value)))
        }
    }
}
