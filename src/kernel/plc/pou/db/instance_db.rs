use crate::kernel::plc::types::complex::instance::private::{PrivateInstanceAccessors, PrivateInstanceTrait};
use crate::kernel::plc::types::complex::instance::public::PublicInstanceAccessors;
use crate::kernel::plc::interface::section::Section;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::interface::status::{BodyStatus, InterfaceStatus};
use crate::kernel::plc::interface::struct_interface::StructInterface;
use crate::kernel::plc::interface::traits::{Cloneable, DeferredBuilder};
use crate::kernel::plc::operations::operations::{Operation, RunTimeOperation, RuntimeOperationTrait};
use crate::kernel::arch::local::pointer::LocalPointer;
use crate::kernel::registry::{get_or_insert_global_string, Kernel};
use crate::container::error::error::Stop;
use crate::{create_block_interface, error, key_reader};
use serde_json::{Map, Value};
use std::collections::HashMap;
use crate::parser::body::json_target::JsonTarget;
use crate::container::broadcast::broadcast::Broadcast;
use crate::kernel::plc::types::primitives::traits::meta_data::MaybeHeapOrStatic;
use crate::kernel::plc::types::primitives::traits::primitive_traits::ToggleMonitor;
use crate::parser::interface::interface::parse_struct_interface;
use crate::kernel::registry::get_string;

pub struct InstanceDb {
    json: Map<String, Value>,
    interface: SectionInterface,
    interface_status: InterfaceStatus,
    body_status: BodyStatus,
    body: Vec<JsonTarget>,
    id: u32
}

impl PrivateInstanceAccessors for InstanceDb {
    fn get_mut_interface(&mut self) -> &mut SectionInterface {
        &mut self.interface
    }
}

impl PublicInstanceAccessors for InstanceDb {
    fn get_interface(&self) -> &SectionInterface {
        &self.interface
    }

    fn get_body(&self) -> &Vec<JsonTarget> {
        &self.body
    }
}

impl InstanceDb {
    pub fn try_replace_pointer_nested(&mut self, path: &[usize], other: &LocalPointer) -> Option<LocalPointer> {
        self.interface.try_replace_pointer_nested(path, other)
    }

    pub fn try_get_nested(&self, path: &[usize]) -> Option<LocalPointer> {
        self.interface.try_get_nested(path)
    }

    pub fn get_section(&mut self, section: &Section) -> Option<&StructInterface> {
        self.interface.get(section)
    }

    pub fn build_executable(
        &mut self,
        match_interface: &HashMap<Section, Vec<(Vec<String>, JsonTarget)>>,
        parent_interface: &SectionInterface,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        match self.body_status {
            BodyStatus::Default => self.build_body(registry, channel).map_err(|e| {
                e.add_sim_trace(&format!("Build instance db -> executable"))
                    .add_id(self.id)
            })?,
            BodyStatus::Pending => {
                return Err(error!(
                    format!(
                        "Could not build instance db executable, check for recursive reference"
                    ),
                    format!("Build instance db -> executable")
                ))
            }
            BodyStatus::Solved => {}
        }

        let input_actions =
            self.define_input_actions(match_interface, parent_interface, registry, channel)?;
        let output_actions =
            self.define_output_actions(match_interface, parent_interface, registry, channel)?;
        let body = self.build_operations(registry, channel)?;
        self.save_raw_pointers(registry, channel)?;

        Ok(Box::new(Operation::new(
            MaybeHeapOrStatic(None),
            move |channel| {
                input_actions.iter().try_for_each(|assign| {
                    assign.with_void(channel)?;
                    Ok(())
                })?;

                if body.is_empty() {
                    channel.add_warning("Function body is empty");
                };
                for operation in &body {
                    // In case of early returns
                    operation.with_void(channel)?;
                    if operation.return_early() {
                        break;
                    };
                }

                // Output
                output_actions.iter().try_for_each(|assign| {
                    assign.with_void(channel)?;
                    Ok(())
                })?;
                Ok(())
            },
            None,
            false,
            self.id
        )))
    }
}

impl DeferredBuilder for InstanceDb {
    fn default(json: &Map<String, Value>) -> Self {
        Self {
            json: json.clone(),
            interface: SectionInterface::new(),
            interface_status: InterfaceStatus::Default,
            body_status: BodyStatus::Default,
            body: Vec::new(),
            id: json["id"].as_u64().unwrap() as u32,
        }
    }

    fn build_interface(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        self.interface_status = InterfaceStatus::Pending;
        let data = &self.json;

        key_reader!(
            format!("Parse instance db -> interface"),
            data {
                of => as_str,
                interface => {
                    src => as_object,
                }
            }
        );

        let as_type = registry.get(&get_or_insert_global_string(&of.to_string())).ok_or_else(|| error!(
            format!("Could not find a block named '{}' to create instance", of),
            format!("Parse instance db -> interface"),
            Some(self.id)
        ))?;
        if as_type.is_fb() {
            create_block_interface!(
            src, self.interface, registry, channel,
            { Input },
            { Output },
            { InOut },
            { Static },
            { Temp },
            { Constant },
            true
        )
                .map_err(|e| {
                    e.add_sim_trace("Build Fb Interface")
                        .add_id(self.id)
                })?;
            self.interface.set_monitor(registry);
            self.interface_status = InterfaceStatus::Solved;
            Ok(())
        } else {
            Err(error!(
                format!("Invalid block for InstanceDb, expected Fb, got {}", as_type),
                format!("Parse instance db -> interface"),
                Some(self.id))
            )
        }
    }

    fn build_body(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        self.body_status = BodyStatus::Pending;
        let data = &self.json;

        key_reader!(
            format!("Parse instance db -> body"),
            data {
                of => as_str,
            }
        );

        let as_type = registry.get(&get_or_insert_global_string(&of.to_string())).ok_or_else(|| error!(
            format!("Could not find a block named '{}' to create instance", of),
            format!("Parse instance db -> body"),
            Some(self.id)
        ))?;
        if as_type.is_fb() {
            // Safe (is_fb)
            self.body = as_type.as_mut_fb()?.clone_body(registry, channel).map_err(|e| {
                e.add_sim_trace(&format!("Parse instance db -> body"))
                    .add_id(self.id)
            })?;
            self.body_status = BodyStatus::Solved;
            Ok(())
        } else {
            Err(error!(
                format!("Invalid block for InstanceDb, expected Fb, got {}", as_type),
                format!("Parse instance db -> body"),
                Some(self.id))
            )
        }
    }

    fn get_interface_status(&self) -> InterfaceStatus {
        self.interface_status
    }

    fn get_body_status(&self) -> BodyStatus {
        self.body_status
    }
}
