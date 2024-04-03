use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use std::any::TypeId;

pub trait RawMut {
    fn reset_ptr(&mut self, channel: &Broadcast);
}

pub trait PrimitiveTrait {
    type Native;
    type PlcPrimitive;
    /// Creates a new PlcPrimitive from a native type.
    ///
    /// The provided value is also the default value.
    fn new(value: &Self::Native) -> Result<Self::PlcPrimitive, Stop>;

    /// Borrows the value field.
    fn get(&self, channel: &Broadcast) -> Result<Self::Native, Stop>;

    /// Sets the value from native.
    ///
    /// If monitor is set, this will trigger a monitor event.
    fn set(&mut self, value: Self::Native, channel: &Broadcast) -> Result<(), Stop>;

    fn set_default(&mut self, value: Self::Native) -> Result<(), Stop>;

    /// Resets the native value.
    ///
    /// Basically the value field copies the default field.
    fn reset(&mut self, channel: &Broadcast);

    /// Returns the id of this type.
    ///
    /// Since ids are defined with an AtomicUsize, they are all unique
    fn get_id(&self) -> usize;

    fn get_type_id(&self) -> TypeId;

    /// Sends an event to the main broadcast with the id of this type and the field value.
    fn monitor(&self, channel: &Broadcast);
}
