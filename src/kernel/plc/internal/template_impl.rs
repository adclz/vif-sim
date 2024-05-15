use crate::parser::interface::interface::parse_struct_interface;
use crate::parser::body::json_target::JsonTarget;
use crate::kernel::plc::types::complex::boxed::set::box_set_plc_complex;
use crate::kernel::plc::interface::section::Section;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation, RuntimeOperationTrait,
};
use crate::kernel::rust::set::box_set_plc_primitive;
use crate::kernel::arch::global::pointer::GlobalPointer;
use crate::kernel::arch::local::pointer::LocalPointer;
use crate::kernel::registry::{convert_string_path_to_usize, get_or_insert_global_string, Kernel};
use crate::container::error::error::Stop;
use crate::{create_block_interface, error, insert_section, key_reader};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;
use crate::kernel::rust::auto_set::box_set_auto;
use crate::kernel::plc::types::complex::instance::public::PublicInstanceTrait;
use crate::kernel::plc::interface::traits::InterfaceAccessors;
use crate::kernel::plc::types::primitives::traits::meta_data::MaybeHeapOrStatic;

#[enum_dispatch::enum_dispatch(InterfaceAccessors)]
pub enum TemplateMemory {
    Local(LocalPointer),
    Global(GlobalPointer),
}

#[derive(Clone)]
pub struct TemplateImpl {
    of: String,
    call_interface: HashMap<Section, Vec<(Vec<String>, JsonTarget)>>,
    inner: Value,
    id: u32,
}

impl NewJsonOperation for TemplateImpl {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop>
    where
        Self: Clone,
    {
        key_reader!(
            format!("Parse Template"),
            json {
                of => as_str,
                inner,
                id => as_u64,
                call_interface => {
                    src => as_object,
                }
            }
        );
        
        let id = id as u32;

        let call_interface = call_interface["src"].as_object().unwrap();

        let mut as_interface = HashMap::new();

        insert_section!(
            call_interface, as_interface,
            { input, Input },
            { inout, InOut },
            { output, Output }
        )
        .map_err(|e| {
            e.add_sim_trace("Build template -> parse interface of calling block")
                .add_id(id)
        })?;

        Ok(Self {
            of: of.to_string(),
            //interface: interface.clone(),
            call_interface: as_interface,
            inner: inner.clone(),
            id,
        })
    }
}

impl BuildJsonOperation for TemplateImpl {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let template_origin = registry.program_templates.get(&self.of).ok_or_else(move || error!(
            format!("Template '{}' could not be found", &self.of),
            format!("Parse template -> find target")
        ))?;

        let target = parse_json_target(&self.inner)?;

        // Resolve the inner memory
        let inner_memory = match target {
            _ if target.is_local() || target.is_local_out() => Ok(TemplateMemory::Local(
                target
                    .solve_as_local_pointer(interface, template, registry, channel)
                    .ok_or_else(move || error!(format!(
                        "Could not find a valid local reference for template memory {}",
                        target
                    )))?,
            )),
            _ if target.is_global() => Ok(TemplateMemory::Global(
                target
                    .solve_as_global_pointer(registry)
                    .ok_or_else(move || error!(format!(
                        "Could not find a valid global reference for template memory {}",
                        target
                    )))?,
            )),
            _ => Err(error!("The target for template memory is neither local or global".to_string())),
        }?;

        let mut input_actions = Vec::new();
        let mut output_actions = Vec::new();

        // Tries to find all the references in call_interface inside
        self.call_interface
            .iter()
            .try_for_each(|(section, members)| match section {
                Section::Input => {
                    members
                        .iter()
                        .try_for_each(|(target_path, source)| {
                            let target = inner_memory.try_get_nested(&convert_string_path_to_usize(target_path))
                                .ok_or_else(move || error!(format!("Could not find a valid reference in instance interface for path {:?}, Current interface: {}", &target_path, interface)))?;
                            input_actions.push(box_set_auto(&target, &source.solve_to_ref(interface, None, Some(target.as_ref().borrow().deref().clone()), registry, channel)?, 0, registry)?);
                            Ok(())
                        })
                },
                Section::InOut => {
                    members
                        .iter()
                        .try_for_each(|(target_path, source)| {
                            let mut target = inner_memory.try_get_nested(&convert_string_path_to_usize(target_path))
                                .ok_or_else(move || error!(format!("Could not find a valid reference in instance interface for path {:?}, Current interface: {}", &target_path, interface)))?;

                            let source = source.solve_as_local_pointer(interface, None, registry, channel)
                                .ok_or_else(move || error!(format!("Invalid InOut parameter in calling block, expected a reference, got {}", source)))?;

                            // Swapping Pointers
                            target.replace_pointer(&source);

                            Ok(())
                        })
                },
                Section::Output => {
                    members
                        .iter()
                        .try_for_each(|(target_path, source)| {
                            let target = inner_memory.try_get_nested(&convert_string_path_to_usize(target_path))
                                .ok_or_else(move || error!(format!("Could not find a valid reference in instance interface for path {:?}, Current interface: {}", &target_path, interface)))?;
                            output_actions.push(box_set_auto(&source.solve_as_local_pointer(interface, None, registry, channel).unwrap(), &target, 0, registry)?);
                            Ok(())
                        })
                },
                _ => Ok(())
            })?;

        let operations = template_origin
            .borrow_mut()
            .deref_mut()
            .get_body(registry, channel)?
            .iter()
            .map(|instruction| {
                parse_json_target(instruction)?.solve_as_operation(
                    &interface,
                    Some(&inner_memory),
                    registry,
                    channel
                )
            })
            .collect::<Result<Vec<RunTimeOperation>, Stop>>()
            .map_err(|e: Stop| {
                e.add_sim_trace("Build template -> body")
                    .add_id(self.id)
            })?;

        let name = get_or_insert_global_string(&self.of);
        Ok(Box::new(Operation::new(
            MaybeHeapOrStatic(None),
            move |channel| {
                let index = channel
                    .get_cycle_stack()
                    .borrow_mut()
                    .add_section(name, "[Template]");

                input_actions.iter_mut().try_for_each(|assign| {
                    assign.with_void(channel)?;
                    Ok(())
                })?;

                for operation in &operations {
                    // In case of early returns
                    operation.with_void(channel)?;
                    if operation.return_early() {
                        break;
                    };
                }

                output_actions.iter_mut().try_for_each(|assign| {
                    assign.with_void(channel)?;
                    Ok(())
                })?;

                channel
                    .get_cycle_stack()
                    .borrow_mut()
                    .go_back_to_section(index);
                Ok(())
            },
            None,
            false,
            self.id
        )))
    }
}
