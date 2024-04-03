use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::plc::primitives::family_traits::{ToggleMonitor, MetaData, SetMetaData};
use crate::plc::primitives::primitive_traits::*;
use crate::{error, impl_primitive_raw_mut, impl_primitive_serialize, impl_primitive_type_name};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::Value;
use smart_default::SmartDefault;
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use crate::plc::primitives::floats::checked_float::TryIntoCheck;
use std::borrow::Cow;
use crate::registry::registry::Kernel;

#[derive(Clone, Copy, SmartDefault)]
pub struct Real {
    default: f32,
    value: f32,
    #[default(_code = "get_id()")]
    id: usize,
    monitor: bool,
    read_only: bool,
    alias: Option<usize>
}

impl TryFrom<&Value> for Real {
    type Error = Stop;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_f64() {
            None => Err(error!(format!("Invalid value {} for Real", value))),
            Some(a) => Ok(Real::new(&TryIntoCheck::try_into(a)?)?),
        }
    }
}

impl_primitive_serialize!(Real, Decimal);
impl_primitive_type_name!(Real, Decimal);
impl_primitive_raw_mut!(Real, Decimal);

impl ToggleMonitor for Real {
    fn set_monitor(&mut self, activate: bool) {
        self.monitor = activate
    }
}

pub fn scale(f: f32) -> usize {
    match f as i64 {
        0 => 7,
        1..=10 => 6,
        11..=100 => 5,
        101..=1000 => 4,
        1001..=10000 => 3,
        10001..=100000 => 2,
        _ => 1
    }
}

impl Display for Real {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{float:.scale$}", float = self.value, scale = scale(self.value))
    }
}

/*
 * @deprecated
 */
/*fn rescale_to_real(decimal: &mut Decimal) -> &mut Decimal {
    let trunc: u32 = match decimal.trunc().try_into().unwrap() {
        0 => 0,
        Gt => (Gt as u32).checked_ilog10().unwrap_or(0) + 1,
    };

    let should_rescale = 7_u32.checked_sub(trunc).unwrap_or(1);

    if should_rescale > 0 {
        decimal.rescale(should_rescale);
    };
    decimal
}*/

impl PrimitiveTrait for Real {
    type Native = f32;
    type PlcPrimitive = Real;

    fn new(value: &Self::Native) -> Result<Self::PlcPrimitive, Stop> {
        Ok(Self {
            default: *value,
            value: *value,
            id: get_id(),
            monitor: false,
            read_only: false,
            alias: None
        })
    }

    fn get(&self, channel: &Broadcast) -> Result<Self::Native, Stop> {
        Ok(self.value)
    }

    fn set(&mut self, value: Self::Native, channel: &Broadcast) -> Result<(), Stop> {
        self.value = value;
        self.monitor(channel);
        Ok(())
    }

    fn set_default(&mut self, value: Self::Native) -> Result<(), Stop> {
        self.default = value;
        self.value = self.default;
        Ok(())
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
