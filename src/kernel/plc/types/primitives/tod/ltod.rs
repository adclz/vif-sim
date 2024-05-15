use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::*;
use crate::kernel::plc::types::primitives::traits::primitive_traits::*;
use crate::kernel::plc::types::primitives::traits::meta_data::*;
use crate::{error, impl_primitive_all, key_reader};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use smart_default::SmartDefault;
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use std::borrow::Cow;
use crate::kernel::registry::Kernel;
use crate::kernel::registry::get_string;

#[derive(Clone)]
pub struct LTod {
    value: u64,
    default: u64,

    id: u32,
    read_only: bool,
    alias: Option<usize>,
    path: usize
}

impl_primitive_all!(LTod, u64);

impl TryFrom<&Map<String, Value>> for LTod {
    type Error = Stop;

    fn try_from(data: &Map<String, Value>) -> Result<Self, Self::Error> {
        key_reader!(
            format!("Parse Tod"),
            data {
                value => as_u64,
                id => as_u64,
            }
        );
        let id = id as u32;
        LTod::new(&value.try_into().map_err(|e| error!(format!("{}", e)))?, id)
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
