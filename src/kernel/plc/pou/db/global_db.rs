use serde_json::{Map, Value};
use crate::parser::interface::interface::parse_struct_interface;
use crate::kernel::plc::interface::section::Section;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::interface::status::{BodyStatus, InterfaceStatus};
use crate::kernel::plc::interface::struct_interface::StructInterface;
use crate::kernel::plc::interface::traits::DeferredBuilder;
use crate::kernel::arch::local::pointer::LocalPointer;
use crate::kernel::registry::Kernel;
use crate::{create_block_interface, error, key_reader};
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::{Stop};
use crate::kernel::registry::get_string;

pub struct GlobalDb {
    json: Map<String, Value>,
    interface: SectionInterface,
    interface_status: InterfaceStatus,
    body_status: BodyStatus,
    trace: Option<FileTrace>
}

impl GlobalDb {
    pub fn get_interface(&self) -> &SectionInterface {
        &self.interface
    }

    pub fn try_replace_pointer_nested(&mut self, path: &[usize], other: &LocalPointer) -> Option<LocalPointer> {
        self.interface.try_replace_pointer_nested(path, other)
    }
    
    pub fn try_get_nested(&self, path: &[usize]) -> Option<LocalPointer> {
        self.interface.try_get_nested(path)
    }

    pub fn get_section(&mut self, section: &Section) -> Option<&StructInterface> {
        self.interface.get(section)
    }
}

impl FileTraceBuilder for GlobalDb {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl DeferredBuilder for GlobalDb {
    fn default(json: &Map<String, Value>) -> Self {
        let mut trace = None;
        if json.contains_key("trace") {
            if let Some(a) = json["trace"].as_object() {
                trace = Self::build_trace(a);
            }
        }

        Self {
            json: json.clone(),
            interface: SectionInterface::new(),
            interface_status: InterfaceStatus::Default,
            body_status: BodyStatus::Solved,
            trace
        }
    }

    fn build_interface(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        self.interface_status = InterfaceStatus::Pending;
        let data = &self.json;

        key_reader!(
            "Parse GlobalDb interface".to_string(),
            data {
                interface => {
                    src => as_object,
                }
            }
        );

        create_block_interface!(
            src, self.interface, registry, channel,
            { Static }
        ).map_err(|e| e.add_sim_trace(&format!("Build Global Db")).maybe_file_trace(&self.trace))?;
        
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