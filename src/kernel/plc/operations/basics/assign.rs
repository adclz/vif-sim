use core::ops::Deref;
use crate::parser::body::json_target::JsonTarget;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::kernel::plc::operations::operations::{BuildJsonOperation, NewJsonOperation, RunTimeOperation};
use crate::kernel::rust::set::box_set_plc_primitive;
use crate::kernel::registry::Kernel;
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use serde_json::{Map, Value};
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;

#[derive(Clone)]
pub struct Assign {
    assign: JsonTarget,
    to: JsonTarget,
    id: u32,
}

impl NewJsonOperation for Assign {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Assign"),
            json {
                id => as_u64, 
            }
        );

        let id = id as u32;

        match(|| {
            key_reader!(
            format!("Parse Assign"),
            json {
                assign,
                to,
            }
        );

            let assign = parse_json_target(assign).map_err(|e| {
                e.add_sim_trace(&format!("Parse Assign Operation [assign]"))
            })?;

            let to = parse_json_target(to).map_err(|e| {
                e.add_sim_trace(&format!("Parse Assign Operation [to]"))
            })?;

            if assign.is_constant() {
                return Err(error!(
                format!("Cannot assign a constant value")
            ));
            };

            Ok(Self { assign, to, id })
        })() {
            Ok(a) => Ok(a),
            Err(e) => Err(e.add_sim_trace("Parse Assign").add_id(id))
        }

    }
}

impl BuildJsonOperation for Assign {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let a1 = self
            .assign
            .solve_as_local_pointer(interface, template, registry, channel)
            .ok_or_else(move || error!(format!("Expected a valid reference, got {}", self.assign), "Build assign -> source".to_string()).add_id(self.id))?;

        if a1.is_read_only() {
            return Err(error!(format!("Attempt to assign a constant value"), "Build assign -> source".to_string(), Some(self.id)))
        }

        let a2 = self
            .to
            .solve_to_ref(interface, template, Some(a1.as_ref().borrow().deref().clone()), registry, channel)
            .map_err(|e| {
                e.add_sim_trace("Build assign -> target")
                    .add_id(self.id)
            })?;

        box_set_plc_primitive(&a1, &a2, self.id, false, registry)
    }
}
