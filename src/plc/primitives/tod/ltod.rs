use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::plc::primitives::family_traits::*;
use crate::plc::primitives::primitive_traits::*;
use crate::{error, impl_primitive_all};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::Value;
use smart_default::SmartDefault;
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use std::borrow::Cow;
use crate::registry::registry::Kernel;

#[derive(Clone, Copy, SmartDefault)]
pub struct LTod {
    value: u64,
    default: u64,
    #[default(_code = "get_id()")]
    id: usize,
    monitor: bool,
    read_only: bool,
    alias: Option<usize>
}

impl_primitive_all!(LTod, u64);

impl TryFrom<&Value> for LTod {
    type Error = Stop;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_u64() {
            None => Err(error!(format!("Invalid value {} for LTod", value))),
            Some(a) => Ok(LTod::new(
                &a.try_into().map_err(|e| error!(format!("{}", e)))?,
            )?),
        }
    }
}
/*
impl Serialize for LTod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut data = serializer.serialize_struct("data", 3)?;
        data.serialize_field("ty", &"LTod")?;
        data.serialize_field("id", &self.id)?;
        data.serialize_field("value", &format!("{}", self))?;
        data.end()
    }
}

impl Display for LTod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hours = (self.value / 3_600_000_000_000) as u8;
        let remaining_ms = self.value % 3_600_000_000_000;

        let minutes = (remaining_ms / 60_000_000_000) as u8;
        let remaining_ms = remaining_ms % 60_000_000_000;

        let seconds = (remaining_ms / 1_000_000_000) as u8;
        let nanoseconds = (remaining_ms % 1_000_000_000) as u32;

        write!(
            f,
            "LTOD#{:02}:{:02}:{:02}.{:09}",
            hours, minutes, seconds, nanoseconds
        )
    }
}
*/
