use crate::plc::interface::section::Section;
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::interface::status::{BodyStatus, InterfaceStatus};
use crate::plc::operations::operations::Operation;
use crate::registry::local::pointer::LocalPointer;
use crate::registry::registry::Kernel;
use crate::container::error::error::Stop;
use serde_json::{Map, Value};
use crate::parser::body::json_target::JsonTarget;
use crate::container::broadcast::broadcast::Broadcast;

pub trait DeferredBuilder {
    fn default(json: &Map<String, Value>) -> Self;

    fn build_interface(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop>;
    fn build_body(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop>;

    fn get_interface_status(&self) -> InterfaceStatus;
    fn get_body_status(&self) -> BodyStatus;
}

pub trait Cloneable {
    fn clone_interface(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<SectionInterface, Stop>;
    fn clone_body(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<Vec<JsonTarget>, Stop>;
}

pub trait Executable {
    fn build_executable(&mut self) -> Operation;
}

pub trait HasSectionInterface {
    fn try_get_nested(&self, path: &[String]) -> Option<LocalPointer>;
    fn get_section(&mut self, section: &Section) -> Option<&LocalPointer>;
    fn get_interface(&self) -> &SectionInterface;
}

#[enum_dispatch::enum_dispatch]
pub trait InterfaceAccessors {
    fn try_get_nested(&self, path: &[String]) -> Option<LocalPointer>;
}
