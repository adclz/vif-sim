use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::*;
use crate::kernel::plc::types::primitives::traits::primitive_traits::*;
use crate::kernel::plc::types::primitives::traits::meta_data::*;
use crate::{error, impl_primitive_all};
use fixedstr::str256;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::Value;
use smart_default::SmartDefault;
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::borrow::Cow;
use crate::kernel::registry::Kernel;
use crate::kernel::registry::get_string;

pub type wstr256 = str256;

#[derive(Clone, SmartDefault)]
pub struct WString {
    value: wstr256,
    default: wstr256,
    #[default(_code = "get_id()")]
    id: usize,
    monitor: bool,
    read_only: bool,
    alias: Option<usize>,
    path: usize
}

impl_primitive_all!(WString, wstr256);

impl TryFrom<&Value> for WString {
    type Error = Stop;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_str() {
            None => Err(error!(format!("Invalid value {} for WString", value))),
            Some(a) => Ok(WString::new(
                &wstr256::from_str(a).map_err(|e| error!(format!("{}", e)))?,
            )?),
        }
    }
}
