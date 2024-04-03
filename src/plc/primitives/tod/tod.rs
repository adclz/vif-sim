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
pub struct Tod {
    value: u32,
    default: u32,
    #[default(_code = "get_id()")]
    id: usize,
    monitor: bool,
    read_only: bool,
    alias: Option<usize>
}

impl_primitive_all!(Tod, u32);

impl TryFrom<&Value> for Tod {
    type Error = Stop;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_u64() {
            None => Err(error!(format!("Invalid value {} for Tod", value))),
            Some(a) => Ok(Tod::new(
                &a.try_into().map_err(|e| error!(format!("{}", e)))?,
            )?),
        }
    }
}

/*impl RawMut for Tod {
    fn reset_ptr(&mut self, channel: &Broadcast) {
        self.reset(channel)
    }
}

impl PrimitiveTrait for Tod {
    type Native = u32;
    type PlcPrimitive = Tod;

    fn new(value: &u32) -> Result<Tod, Stop> {
        if *value > 86_399_999 {
            return Err(error!(format!(
                "Time of day can not be superior than 24*60*60*1000-1: {}",
                value
            )));
        }
        Ok(Self {
            default: *value,
            value: *value,
            id: get_id(),
        })
    }

    fn get(&self) -> &u32 {
        &self.value
    }

    fn set_as(&mut self, value: &Tod, channel: &Broadcast) {
        let v = *value.get();
        if v > 86_399_999 {
            channel.add_warning("[Warn] Tod value exceeds 86_399_999");
        }
        self.value = v;
        self.monitor(channel)
    }

    fn set(&mut self, value: u32, channel: &Broadcast) {
        if value > 86_399_999 {
            channel.add_warning("[Warn] Tod value exceeds 86_399_999");
        }
        self.value = value;
        self.monitor(channel)
    }


    fn set_default(&mut self, value: Self::Native) {
        self.default = value
    }

    fn reset(&mut self, channel: &Broadcast) {
        self.value = self.default;
        self.monitor(channel)
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_type_id(&self) -> TypeId {
        self.value.type_id()
    }

    fn monitor(&self, channel: &Broadcast) {
        channel.add_monitor_change(&MonitorChange::new(self.id, format!("{}", self)))
    }
}

impl Serialize for Tod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut data = serializer.serialize_struct("data", 3)?;
        data.serialize_field("ty", &"Tod")?;
        data.serialize_field("id", &self.id)?;
        data.serialize_field("value", &format!("{}", self))?;
        data.end()
    }
}

impl Display for Tod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.value > 86_399_999 {
            write!(f, "{}", UDInt::new(self.get()).unwrap())
        } else {
            let hours = (self.value / 3_600_000) as u8;
            let remaining_ms = self.value % 3_600_000;

            let minutes = (remaining_ms / 60_000) as u8;
            let remaining_ms = remaining_ms % 60_000;

            let seconds = (remaining_ms / 1000) as u8;
            let milliseconds = (remaining_ms % 1000) as u16;

            write!(
                f,
                "TOD#{:02}:{:02}:{:02}.{:03}",
                hours, minutes, seconds, milliseconds
            )
        }
    }
}
*/
