use core::time::Duration;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;

use crate::kernel::plc::types::primitives::timers::time::Time;
use crate::kernel::plc::types::primitives::timers::lTime::LTime;
use crate::kernel::plc::types::primitives::timers::plc_time::PlcTime;

#[enum_dispatch::enum_dispatch]
pub trait TimeDuration {
    fn set_duration(&mut self, duration: &Duration, channel: &Broadcast) -> Result<(), Stop> ;
    fn get_duration(&self) -> Duration;
}