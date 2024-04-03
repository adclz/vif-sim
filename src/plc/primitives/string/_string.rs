use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::plc::primitives::family_traits::*;
use crate::plc::primitives::primitive_traits::*;
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
use crate::registry::registry::Kernel;

#[derive(SmartDefault, Clone)]
pub struct _String {
    value: str256,
    default: str256,
    #[default(_code = "get_id()")]
    id: usize,
    monitor: bool,
    read_only: bool,
    alias: Option<usize>
}

impl_primitive_all!(_String, str256);

impl TryFrom<&Value> for _String {
    type Error = Stop;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_str() {
            None => Err(error!(format!("Invalid value {} for String", value))),
            Some(a) => Ok(_String::new(
                &str256::from_str(a).map_err(|e| error!(format!("{}", e)))?,
            )?),
        }
    }
}

/*
impl RawMut for _String {
    fn reset_ptr(&mut self, channel: &Broadcast) {
        self.reset(channel)
    }
}

impl PrimitiveTrait for _String {
    type Native = str256;
    type PlcPrimitive = _String;

    fn new(value: &Self::Native) -> Result<Self::PlcPrimitive, Stop> {
        Ok(Self {
            default: value.into(),
            value: value.into(),
            id: get_id(),
            monitor: false,
        })
    }

    fn get(&self) -> &Self::Native {
        &self.value
    }

    fn set_as(&mut self, value: &Self::PlcPrimitive, channel: &Broadcast) {
        self.value = value.get().into();
        if self.monitor {
            self.monitor(channel)
        }
    }

    fn set(&mut self, value: &str256, channel: &Broadcast) {
        self.value = value.into();
        self.monitor(channel)
    }

    fn set_default(&mut self, value: &Self::Native) {
        self.default = value.into()
    }

    fn reset(&mut self, channel: &Broadcast) {
        self.value = self.default.clone();
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

impl Serialize for _String {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut data = serializer.serialize_struct("data", 3)?;
        data.serialize_field("ty", &"String")?;
        data.serialize_field("id", &self.id)?;
        data.serialize_field("value", &self.value.to_string())?;
        data.end()
    }
}

impl Display for _String {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
*/
