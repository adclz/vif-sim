use serde_json::{Map, Value};
use crate::{error, key_reader};
use crate::parser::interface::interface::parse_struct_interface;
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::plc::interface::status::{BodyStatus, InterfaceStatus};
use crate::plc::interface::struct_interface::StructInterface;
use crate::plc::interface::traits::{DeferredBuilder};
use crate::registry::local::pointer::LocalPointer;
use crate::registry::registry::Kernel;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;

pub struct Udt {
    json: Map<String, Value>,
    interface: StructInterface,
    interface_status: InterfaceStatus,
    body_status: BodyStatus,
    trace: Option<FileTrace>
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

    pub fn try_replace_pointer_nested(&mut self, path: &[String], other: &LocalPointer) -> Option<LocalPointer> {
        self.interface.try_replace_pointer_nested(path, other)
    }
    
    pub fn try_get_nested(&self, path: &[String]) -> Option<LocalPointer> {
        self.interface.try_get_nested(path)
    }
}

impl FileTraceBuilder for Udt {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl DeferredBuilder for Udt {
    fn default(json: &Map<String, Value>) -> Self {
        let mut trace = None;
        if json.contains_key("trace") {
            if let Some(a) = json["trace"].as_object() {
                trace = Self::build_trace(a);
            }
        }

        Self {
            json: json.clone(),
            interface: StructInterface::new(),
            interface_status: InterfaceStatus::Default,
            body_status: BodyStatus::Solved,
            trace
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

        parse_struct_interface(&interface, registry, channel, &None)?
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
