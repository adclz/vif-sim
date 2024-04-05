use crate::parser::interface::interface::parse_struct_interface;
use crate::parser::body::json_target::JsonTarget;
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::plc::complex::boxed::set::box_set_plc_complex;
use crate::plc::interface::section::Section;
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation, RuntimeOperationTrait,
};
use crate::plc::primitives::boxed::set::box_set_plc_primitive;
use crate::registry::global::pointer::GlobalPointer;
use crate::registry::local::pointer::LocalPointer;
use crate::registry::registry::Kernel;
use crate::container::error::error::Stop;
use crate::{create_block_interface, error, insert_section, key_reader};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;
use crate::plc::auto_boxed::set::box_set_auto;
use crate::plc::complex::instance::public::PublicInstanceTrait;
use crate::plc::interface::traits::InterfaceAccessors;

#[enum_dispatch::enum_dispatch(InterfaceAccessors)]
pub enum TemplateMemory {
    Local(LocalPointer),
    Global(GlobalPointer),
}

#[derive(Clone)]
pub struct TemplateImpl {
    trace: Option<FileTrace>,
    of: String,
    call_interface: HashMap<Section, Vec<(Vec<String>, JsonTarget)>>,
    inner: Value,
}

impl FileTraceBuilder for TemplateImpl {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
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
                trace? => as_object,
                call_interface => {
                    src => as_object,
                }
            }
        );

        let call_interface = call_interface["src"].as_object().unwrap();

        let trace = match trace {
            None => None,
            Some(a) => Self::build_trace(a),
        };

        let mut as_interface = HashMap::new();

        insert_section!(
            call_interface, as_interface,
            { input, Input },
            { inout, InOut },
            { output, Output }
        )
        .map_err(|e| {
            e.add_sim_trace("Build template -> parse interface of calling block")
                .maybe_file_trace(&trace)
        })?;

        Ok(Self {
            of: of.to_string(),
            //interface: interface.clone(),
            call_interface: as_interface,
            inner: inner.clone(),
            trace,
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
        let template_origin = registry.program_templates.get(&self.of).ok_or(error!(
            format!("Template '{}' could not be found", &self.of),
            format!("Parse template -> find target")
        ))?;

        let target = parse_json_target(&self.inner)?;

        // Resolve the inner memory
        let inner_memory = match target {
            _ if target.is_local() || target.is_local_out() => Ok(TemplateMemory::Local(
                target
                    .solve_as_local_pointer(interface, template, registry, channel)
                    .ok_or(error!(format!(
                        "Could not find a valid local reference for template memory {}",
                        target
                    )))?,
            )),
            _ if target.is_global() => Ok(TemplateMemory::Global(
                target
                    .solve_as_global_pointer(registry)
                    .ok_or(error!(format!(
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
                            let target = inner_memory.try_get_nested(&target_path)
                                .ok_or(error!(format!("Could not find a valid reference in instance interface for path {:?}, Current interface: {}", &target_path, interface)))?;
                            input_actions.push(box_set_auto(&target, &source.solve_to_ref(interface, None, Some(target.as_ref().borrow().deref().clone()), registry, channel)?, &None, registry)?);
                            Ok(())
                        })
                },
                Section::InOut => {
                    members
                        .iter()
                        .try_for_each(|(target_path, source)| {
                            let mut target = inner_memory.try_get_nested(&target_path)
                                .ok_or(error!(format!("Could not find a valid reference in instance interface for path {:?}, Current interface: {}", &target_path, interface)))?;

                            let source = source.solve_as_local_pointer(interface, None, registry, channel)
                                .ok_or(error!(format!("Invalid InOut parameter in calling block, expected a reference, got {}", source)))?;

                            // Swapping Pointers
                            target.replace_pointer(&source);

                            Ok(())
                        })
                },
                Section::Output => {
                    members
                        .iter()
                        .try_for_each(|(target_path, source)| {
                            let target = inner_memory.try_get_nested(target_path)
                                .ok_or(error!(format!("Could not find a valid reference in instance interface for path {:?}, Current interface: {}", &target_path, interface)))?;
                            output_actions.push(box_set_auto(&source.solve_as_local_pointer(interface, None, registry, channel).unwrap(), &target, &None, registry)?);
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
                    .maybe_file_trace(&self.trace)
            })?;

        let name = self.of.clone();
        Ok(Box::new(Operation::new(
            &"Template",
            move |channel| {
                let index = channel
                    .get_cycle_stack()
                    .borrow_mut()
                    .add_section(&name, "[Template]");

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
            &self.trace
        )))
    }
}
