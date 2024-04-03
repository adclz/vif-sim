use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::plc::primitives::family_traits::*;
use crate::plc::primitives::primitive_traits::*;
use crate::plc::primitives::traits::crement::Crement;
use crate::{error, impl_primitive_all, impl_primitive_crement};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::Value;
use smart_default::SmartDefault;
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use std::borrow::Cow;
use crate::registry::registry::Kernel;

#[derive(Clone, Copy, SmartDefault)]
pub struct DWord {
    default: u32,
    value: u32,
    #[default(_code = "get_id()")]
    id: usize,
    monitor: bool,
    read_only: bool,
    alias: Option<usize>
}

impl_primitive_crement!(DWord);
impl_primitive_all!(DWord, u32);

impl TryFrom<&Value> for DWord {
    type Error = Stop;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_u64() {
            None => Err(error!(format!("Invalid value {} for DWord", value))),
            Some(a) => Ok(DWord::new(
                &a.try_into().map_err(|e| error!(format!("{}", e)))?,
            )?),
        }
    }
}
