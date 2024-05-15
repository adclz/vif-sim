use serde_json::{Map, Value};
use crate::{error, key_reader};
use crate::parser::interface::interface::parse_struct_interface;
use crate::kernel::plc::interface::status::{BodyStatus, InterfaceStatus};
use crate::kernel::plc::interface::struct_interface::StructInterface;
use crate::kernel::plc::interface::traits::{DeferredBuilder};
use crate::kernel::arch::local::pointer::LocalPointer;
use crate::kernel::registry::Kernel;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;

pub struct Udt {
    json: Map<String, Value>,
    interface: StructInterface,
    interface_status: InterfaceStatus,
    body_status: BodyStatus,
    id: u32,
}

impl Udt {
    pub fn clone_interface(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<StructInterface, Stop> {
        match self.interface_status {
            InterfaceStatus::Default => self.build_interface(registry, channel),
            InterfaceStatus::Pending => Err(error!(format!("Recursive !"))),
            InterfaceStatus::Solved => Ok(()),
        }?;
        Ok(self.interface.clone())
    }

    pub fn try_replace_pointer_nested(&mut self, path: &[usize], other: &LocalPointer) -> Option<LocalPointer> {
        self.interface.try_replace_pointer_nested(path, other)
    }
    
    pub fn try_get_nested(&self, path: &[usize]) -> Option<LocalPointer> {
        self.interface.try_get_nested(path)
    }
}

impl DeferredBuilder for Udt {
    fn default(json: &Map<String, Value>) -> Self {
        Self {
            json: json.clone(),
            interface: StructInterface::new(),
            interface_status: InterfaceStatus::Default,
            body_status: BodyStatus::Solved,
            id: json["id"].as_u64().unwrap() as u32
        }
    }

    fn build_interface(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        self.interface_status = InterfaceStatus::Pending;
        let data = &self.json;

        key_reader!(
            format!("Parse Udt interface"),
            data {
                interface,
            }
        );

        parse_struct_interface(&interface, registry, channel, &None, false)?
            .as_ref().iter().for_each(|(name, pointer)| {
            self.interface.as_mut().entry(name.clone()).or_insert_with(|| pointer.clone());
        });
        
        self.interface_status = InterfaceStatus::Solved;
        Ok(())
    }

    fn build_body(&mut self, _registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        Ok(())
    }

    fn get_interface_status(&self) -> InterfaceStatus {
        self.interface_status
    }

    fn get_body_status(&self) -> BodyStatus {
        self.body_status
    }
}
