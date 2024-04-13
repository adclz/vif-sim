﻿use serde_json::{Map, Value};
use crate::parser::interface::interface::{parse_struct_interface};
use crate::kernel::plc::interface::section::Section;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::interface::status::{BodyStatus, InterfaceStatus};
use crate::kernel::plc::interface::struct_interface::StructInterface;
use crate::kernel::plc::interface::traits::{Cloneable, DeferredBuilder};
use crate::kernel::arch::local::pointer::LocalPointer;
use crate::kernel::registry::Kernel;
use crate::{create_block_interface, required_key, error, key_reader};
use crate::parser::body::body::parse_json_target;
use crate::parser::body::json_target::JsonTarget;
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::{Stop};
use crate::parser::local_type::local_type::parse_local_type;

pub struct Fc {
    json: Map<String, Value>,
    interface: SectionInterface,
    interface_status: InterfaceStatus,
    body_status: BodyStatus,
    body: Vec<JsonTarget>,
    trace: Option<FileTrace>
}

impl Fc {
    pub fn try_replace_pointer_nested(&mut self, path: &[usize], other: &LocalPointer) -> Option<LocalPointer> {
        self.interface.try_replace_pointer_nested(path, other)
    }

    pub fn try_get_nested(&self, path: &[usize]) -> Option<LocalPointer> {
        self.interface.try_get_nested(path)
    }

    pub fn get_section(&mut self, section: &Section) -> Option<&StructInterface> {
        self.interface.get(section)
    }
    
    pub fn get_interface(&self) -> &SectionInterface {
        &self.interface
    }
}

impl Cloneable for Fc {
    fn clone_interface(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<SectionInterface, Stop> {
        match self.interface_status {
            InterfaceStatus::Default => self.build_interface(registry, channel),
            InterfaceStatus::Pending => Err(error!(format!("Recursive !"))),
            InterfaceStatus::Solved => Ok(()),
        }?;
        Ok(self.interface.clone())
    }

    fn clone_body(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<Vec<JsonTarget>, Stop> {
        match self.body_status {
            BodyStatus::Default => self.build_body(registry, channel),
            BodyStatus::Pending => Err(error!(format!("Recursive !"))),
            BodyStatus::Solved => Ok(())
        }?;
        Ok(self.body.clone())
    }
}

impl FileTraceBuilder for Fc {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl DeferredBuilder for Fc {
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
            trace
        }
    }

    fn build_interface(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        self.interface_status = InterfaceStatus::Pending;
        let data = &self.json;

        key_reader!(
            format!("Build Fb: Parse Fb Interface"),
            data {
                interface => {
                    src => as_object,
                }
            }
        );

        create_block_interface!(
            src, self.interface, registry, channel,
            { Input },
            { Output },
            { InOut },
            { Temp },
            { Constant }
        ).map_err(|e| e.add_sim_trace(&format!("Build Fc")).maybe_file_trace(&self.trace))?;

        // Get the return
        src
            .iter()
            .try_for_each(|(section, value)| match section.as_str() {
                "return" => {
                    let json = value.as_object().unwrap();

                    self.interface.swap_return(LocalPointer::from(parse_local_type(json, registry, channel, &vec!())?));
                    Ok(())
                },
                _ => Ok(())
            })?;
        
        self.interface_status = InterfaceStatus::Solved;
        Ok(())
    }

    fn build_body(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        match (|| {
            self.body_status = BodyStatus::Pending;
            let data = &self.json;

            key_reader!(
            "Build Fc -> Parse Fc body".to_string(),
            data {
                body => as_array,
            }
        );

            registry.set_ignore_operation(true);
            body.iter()
                .try_for_each(|operation| {
                    // Check if operation is valid, even if we don't use it
                    let op = parse_json_target(&operation)?;

                    op.solve_as_operation(
                        &self.interface,
                        None,
                        registry,
                        channel,
                    )?;
                    self.body.push(op);

                    Ok::<(), Stop>(())
                })?;
            registry.set_ignore_operation(false);
            Ok::<(), Stop>(())
        })() {
            Ok(_) => {
                self.body_status = BodyStatus::Solved;
                Ok(())
            }
            Err(e) => Err(e.add_sim_trace("Build Fc Body").maybe_file_trace(&self.trace))
        }
    }

    fn get_interface_status(&self) -> InterfaceStatus {
        self.interface_status
    }

    fn get_body_status(&self) -> BodyStatus {
        self.body_status
    }
}