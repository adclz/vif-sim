use crate::parser::body::json_target::JsonTarget;
use crate::kernel::plc::types::complex::instance::fb_instance::FbInstance;
use crate::kernel::plc::types::complex::instance::fc_instance::FcInstance;
use crate::kernel::plc::interface::section::Section;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::kernel::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation, RuntimeOperationTrait,
};
use crate::kernel::plc::types::primitives::traits::family_traits::{IsFamily, WithMutFamily, WithRefFamily};
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic};use crate::kernel::registry::{get_or_insert_global_string, get_string, Kernel};
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::str::FromStr;
use crate::parser::body::body::parse_json_target;
use crate::parser::body::path::parse_path;
use crate::container::broadcast::broadcast::Broadcast;

#[derive(Clone)]
pub struct Call {
    call: JsonTarget,
    name: usize,
    interface: HashMap<Section, Vec<(Vec<String>, JsonTarget)>>,
    id: u64,
}

#[macro_export]
macro_rules! insert_section {
    ($data: expr, $interface: expr, $({ $section_string: ident, $section: ident }),+) => {
        $data
            .iter()
            .try_for_each(|(section, value)| match section.as_str() {
                $(stringify!($section_string) => {
                    let as_section =
                        $interface.entry(Section::$section)
                        .or_insert_with(|| Vec::new());

                    let section_record = value.as_object()
                        .ok_or_else(move || error!(format!("Data for section '{}' of call is not of type Object: {:?}", section.as_str(), value), format!("Build Call Interface")))?;

                    section_record.iter().try_for_each(|(name, member)| {
                        as_section.push((vec![name.clone()], parse_json_target(&member)?));
                        Ok(())
                    })?;
                    Ok(())
                })+,
                _ => Err(error!(format!("Invalid section for call: {}", section.as_str())))
            })
    };
}

impl NewJsonOperation for Call {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Call"),
            json {
                call,
                interface => as_object,
                id => as_u64,
            }
        );

        let call_interface = interface["src"].as_object().unwrap();

        let to = parse_json_target(&call).map_err(|e| {
            e.add_sim_trace("Build call operation -> parse 'call'")
                .add_id(id)
        })?;
        let name = parse_path(&json["call"]["src"]["path"])?
            .last()
            .unwrap()
            .to_string();
        
        let name = get_or_insert_global_string(&name);

        let mut as_interface = HashMap::new();

        insert_section!(
            call_interface, as_interface,
            { input, Input },
            { inout, InOut },
            { output, Output }
        )
        .map_err(|e| {
            e.add_sim_trace("Build call operation -> parse interface of calling block")
                .add_id(id)
        })?;

        Ok(Self {
            call: to,
            interface: as_interface,
            name,
            id,
        })
    }
}
impl BuildJsonOperation for Call {
    fn build(
        &self,
        parent_interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let name = self.name.clone();
        if self.call.is_global() {
            let global_pointer = self.call.solve_as_global_pointer(registry).unwrap(); //<-- Safe (is_global)

            // GLOBAL

            // Creating an instance from fc
            if global_pointer.is_fc() {
                FcInstance::from(
                    name,
                    self.id,
                    global_pointer
                        .as_mut_fc()? //<-- Safe (is_fc)
                        .deref_mut(),
                    registry,
                    channel
                )?
                .build_executable(&self.interface, parent_interface, registry, channel)
                .map_err(|e| {
                    e.add_sim_trace("Build call operation -> build fc instance")
                        .add_id(self.id)
                })
            }
            // Cloning instance Db pointer
            else if global_pointer.is_db() {
                let mut db = global_pointer.as_mut_db()?; //<-- Safe (is_db)
                if db.is_instance_db() {
                    let executable = db
                        .as_mut_instance_db()? //<-- Safe (is_instance_db)
                        .build_executable(&self.interface, &parent_interface, registry, channel)
                        .map_err(|e| {
                            e.add_sim_trace(
                                &"Build call operation -> build instance db".to_string(),
                            ).add_id(self.id)
                        })?;
                    
                    Ok(Box::new(Operation::new(
                        MaybeHeapOrStatic(Some(HeapOrStatic::Heap(format!("Call {}",  get_string(name))))),
                        move |channel| {
                            let index = channel
                                .get_cycle_stack()
                                .borrow_mut()
                                .add_section(name, "Fb");

                            executable.with_void(channel)?;

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
                } else {
                    Err(error!(
                        format!(
                            "Call operation refers to an invalid db type: '{}'",
                            global_pointer
                        ),
                        format!("Build call operation -> build instance db")
                    ))
                }
            } else {
                Err(error!(
                    format!(
                        "Call operation refers to an invalid global type: '{}'",
                        global_pointer
                    ),
                    format!("Build call operation -> build global instance")
                ))
            }
        // LOCAL
        } else if self.call.is_local() {
            let local_pointer = self
                .call
                .solve_as_local_pointer(parent_interface, None, registry, channel)
                .unwrap(); //<-- Safe (is_local)

            // cloning the local pointer
            if local_pointer.is_fb_instance() {
                let executable = local_pointer.with_mut_fb_instance(channel, &mut |a| {
                    a.build_executable(&self.interface, parent_interface, registry, channel)
                        .map_err(|e| {
                            e.add_sim_trace(&"Build Call Operation".to_string())
                                .add_id(self.id)
                        })
                })??;

                Ok(Box::new(Operation::new(
                    MaybeHeapOrStatic(Some(HeapOrStatic::Heap(format!("Call {}", name)))),
                    move |channel| {
                        let index = channel
                            .get_cycle_stack()
                            .borrow_mut()
                            .add_section(name, "Fb");

                        executable.with_void(channel)?;

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
            } else {
                Err(error!(
                    format!(
                        "Call operation is bind to an invalid local type '{}'",
                        local_pointer
                    ),
                    format!("Build call operation -> build local instance")
                ))
            }
        } else {
            Err(error!(
                format!("Call operation is bind to an invalid target"),
                format!("Build call operation -> build instance")
            ))
        }
    }
}
