use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::*;
use crate::kernel::plc::types::primitives::traits::primitive_traits::*;
use crate::kernel::plc::types::primitives::traits::meta_data::*;
use crate::kernel::plc::types::primitives::timers::traits::TimeDuration;
use crate::{error, impl_primitive_all};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::Value;
use smart_default::SmartDefault;
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use std::time::Duration;
use std::borrow::Cow;
use crate::kernel::registry::Kernel;
use crate::kernel::registry::get_full_path;

#[derive(Clone, SmartDefault)]
pub struct LTime {
    value: i64,
    default: i64,
    #[default(_code = "get_id()")]
    id: usize,
    monitor: bool,
    read_only: bool,
    alias: Option<usize>,
    path: Vec<usize>
}

impl_primitive_all!(LTime, i64);

impl TryFrom<&Value> for LTime {
    type Error = Stop;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_i64() {
            None => Err(error!(format!("Invalid value {} for LTime", value))),
            Some(a) => Ok(LTime::new(
                &a.try_into().map_err(|e| error!(format!("{}", e)))?,
            )?),
        }
    }
}

impl TimeDuration for LTime {
    fn set_duration(&mut self, duration: &Duration, channel: &Broadcast) -> Result<(), Stop> {
        self.set(duration.as_millis() as i64, channel)
    }

    fn get_duration(&self) -> Duration {
        Duration::from_millis(self.value as u64)
    }
}
