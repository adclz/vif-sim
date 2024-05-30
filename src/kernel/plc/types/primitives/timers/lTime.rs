use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::*;
use crate::kernel::plc::types::primitives::traits::primitive_traits::*;
use crate::kernel::plc::types::primitives::traits::meta_data::*;
use crate::kernel::plc::types::primitives::timers::traits::TimeDuration;
use crate::{error, impl_primitive_all, key_reader};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use core::any::{Any, TypeId};
use core::fmt::{Display, Formatter};
use core::time::Duration;
use std::borrow::Cow;
use crate::kernel::registry::Kernel;
use crate::kernel::registry::get_string;

#[derive(Clone)]
pub struct LTime {
    value: i64,
    default: i64,

    id: u32,
    read_only: bool,
    alias: Option<usize>,
    path: usize
}

impl_primitive_all!(LTime, i64);

impl TryFrom<&Map<String, Value>> for LTime {
    type Error = Stop;

    fn try_from(data: &Map<String, Value>) -> Result<Self, Self::Error> {
        key_reader!(
            format!("Parse Time"),
            data {
                value => as_i64,
                id => as_u64,
            }
        );
        let id = id as u32;
        LTime::new(&value.try_into().map_err(|e| error!(format!("{}", e)))?, id)
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
