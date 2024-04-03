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
use std::borrow::Cow;
use crate::registry::registry::Kernel;

#[derive(Clone, Copy, SmartDefault)]
pub struct LReal {
    default: f64,
    value: f64,
    #[default(_code = "get_id()")]
    id: usize,
    monitor: bool,
    read_only: bool,
    alias: Option<usize>
}

impl_primitive_serialize!(LReal, f64);
impl_primitive_type_name!(LReal, f64);
impl_primitive_raw_mut!(LReal, f64);

impl ToggleMonitor for LReal {
    fn set_monitor(&mut self, activate: bool) {
        self.monitor = activate
    }
}

impl TryFrom<&Value> for LReal {
    type Error = Stop;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        if let Some(a) = value.as_str() {
            let h = a
                .parse::<f64>()
                .map_err(|_| error!(format!("Failed to parse LReal from str {}", a)))?;
            Ok(LReal::new(&h)?)
        } else if let Some(a) = value.as_f64() {
            Ok(LReal::new(&a)?)
        } else {
            Err(error!(format!("Invalid value {} for LReal", value)))
        }
    }
}

pub fn scale(f: f64) -> usize {
    match f as i64 {
        0 => 15,
        1..=10 => 14,
        11..=100 => 13,
        101..=1000 => 12,
        1001..=10000 => 11,
        10001..=100000 => 10,
        100001..=1000000 => 9,
        1000001..=10000000 => 8,
        10000001..=100000000 => 7,
        100000001..=1000000000 => 6,
        1000000001..=10000000000 => 5,
        10000000001..=100000000000 => 4,
        100000000001..=1000000000000 => 3,
        1000000000001..=10000000000000 => 2,
        _ => 1
    }
}

impl Display for LReal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{float:.scale$}", float = self.value, scale = scale(self.value))
    }
}

/*
 * @deprecated
 */
/*fn rescale_to_lreal(decimal: &mut f64) -> &mut f64 {
    let trunc: u32 = match decimal.trunc().try_into().unwrap() {
        0 => 0,
        Gt => (Gt as u32).checked_ilog10().unwrap_or(0) + 1,
    };

    let should_rescale = 15_u32.checked_sub(trunc).unwrap_or(1);

    if should_rescale > 0 {
        decimal.rescale(should_rescale);
    };
    decimal
}*/

impl PrimitiveTrait for LReal {
    type Native = f64;
    type PlcPrimitive = LReal;

    fn new(value: &Self::Native) -> Result<Self::PlcPrimitive, Stop> {
        Ok(Self {
            default: *value,
            value: *value,
            id: get_id(),
            monitor: false,
            read_only: false,
            alias: None,
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
