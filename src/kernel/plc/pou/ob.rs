use crate::parser::interface::interface::parse_struct_interface;
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::kernel::plc::interface::section::Section;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::interface::status::{BodyStatus, InterfaceStatus};
use crate::kernel::plc::interface::struct_interface::StructInterface;
use crate::kernel::plc::interface::traits::DeferredBuilder;
use crate::kernel::plc::operations::operations::{RunTimeOperation, RuntimeOperationTrait};
use crate::kernel::registry::Kernel;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::{create_block_interface, error, key_reader};
use serde_json::{Map, Value};
use crate::parser::body::body::parse_json_target;
use crate::kernel::registry::get_string;

pub struct Ob {
    json: Map<String, Value>,
    interface: SectionInterface,
    interface_status: InterfaceStatus,
    body_status: BodyStatus,
    body: Vec<RunTimeOperation>,
    trace: Option<FileTrace>,
}

impl Ob {
    pub fn get_section(&mut self, section: &Section) -> Option<&StructInterface> {
        self.interface.get(section)
    }

    pub fn execute(&mut self, channel: &Broadcast) -> Result<(), Stop> {
        self.body.iter_mut().try_for_each(|op| {
            op.with_void(channel).map_err(|e| {
                e.add_sim_trace(&format!("Ob Start"))
                    .maybe_file_trace(&self.trace)
            })?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn get_interface(&self) -> &SectionInterface {
        &self.interface
    }
}

impl FileTraceBuilder for Ob {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl DeferredBuilder for Ob {
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
            body_status: BodyStatus::Default,
            body: Vec::new(),
            trace,
        }
    }

    fn build_interface(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        self.interface_status = InterfaceStatus::Pending;
        let data = &self.json;

        key_reader!(
            format!("Parse Ob interface"),
            data {
                interface => {
                    src => as_object,
                }
            }
        );

        create_block_interface!(
            src, self.interface, registry, channel,
            { Temp },
            { Constant }
        )
        .map_err(|e| {
            e.add_sim_trace(&format!("Build Ob Interface"))
                .maybe_file_trace(&self.trace)
        })?;

        self.interface_status = InterfaceStatus::Solved;
        Ok(())
    }

    fn build_body(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        match(|| {
            self.body_status = BodyStatus::Pending;
            let data = &self.json;

            key_reader!(
                format!("Parse Ob interface"),
                data {
                    body => as_array,
                }
            );

            let mut operations = Vec::new();

            body.iter().try_for_each(|operation| {
                operations.push(parse_json_target(&operation)?.solve_as_operation(
                    &self.interface,
                    None,
                    registry,
                    channel
                )?);
                Ok(())
            })?;
            self.body = operations;
            Ok::<(), Stop>(())
        })() {
            Ok(_) => {
                self.body_status = BodyStatus::Solved;
                Ok(())
            }
            Err(e) => Err(e.add_sim_trace("Build Ob Body").maybe_file_trace(&self.trace))
        }
    }

    fn get_interface_status(&self) -> InterfaceStatus {
        self.interface_status
    }

    fn get_body_status(&self) -> BodyStatus {
        self.body_status
    }
}
