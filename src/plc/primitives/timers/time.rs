use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::plc::primitives::family_traits::*;
use crate::plc::primitives::primitive_traits::*;
use crate::plc::primitives::timers::traits::TimeDuration;
use crate::{error, impl_primitive_all};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::Value;
use smart_default::SmartDefault;
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use std::time::Duration;
use std::borrow::Cow;
use crate::registry::registry::Kernel;

#[derive(Clone, Copy, SmartDefault)]
pub struct Time {
    value: i32,
    default: i32,
    #[default(_code = "get_id()")]
    id: usize,
    monitor: bool,
    read_only: bool,
    alias: Option<usize>
}

impl_primitive_all!(Time, i32);

impl TryFrom<&Value> for Time {
    type Error = Stop;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_i64() {
            None => Err(error!(format!("Invalid value {} for Time", value))),
            Some(a) => Ok(Time::new(
                &a.try_into().map_err(|e| error!(format!("{}", e)))?,
            )?),
        }
    }
}

impl TimeDuration for Time {
    fn set_duration(&mut self, duration: &Duration, channel: &Broadcast) -> Result<(), Stop> {
        self.set(duration.as_millis() as i32, channel)
    }

    fn get_duration(&self) -> Duration {
        Duration::from_millis(self.value as u64)
    }
}
