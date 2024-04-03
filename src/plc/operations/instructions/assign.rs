use std::ops::Deref;
use crate::parser::body::json_target::JsonTarget;
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::internal::template_impl::TemplateMemory;
use crate::plc::operations::operations::{BuildJsonOperation, NewJsonOperation, RunTimeOperation};
use crate::plc::primitives::boxed::set::box_set_plc_primitive;
use crate::registry::registry::Kernel;
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use serde_json::{Map, Value};
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;

#[derive(Clone)]
pub struct Assign {
    assign: JsonTarget,
    to: JsonTarget,
    trace: Option<FileTrace>,
}

impl FileTraceBuilder for Assign {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl NewJsonOperation for Assign {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Assign"),
            json {
                trace? => as_object,
            }
        );
        match(|| {
            key_reader!(
            format!("Parse Assign"),
            json {
                assign,
                to,
            }
        );

            let trace = match trace {
                None => None,
                Some(a) => Self::build_trace(a),
            };

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

            Ok(Self { assign, to, trace })
        })() {
            Ok(a) => Ok(a),
            Err(e) => Err(e.add_sim_trace("Parse Assign").maybe_file_trace(&match trace {
                None => None,
                Some(a) => Self::build_trace(a),
            }))
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
            .ok_or(error!(format!("Expected a valid reference, got {}", self.assign), "Build assign -> source".to_string()).maybe_file_trace(&self.trace))?;

        if a1.is_read_only() {
            return Err(error!(format!("Attempt to assign a constant value"), "Build assign -> source".to_string(), &self.trace))
        }

        let a2 = self
            .to
            .solve_to_ref(interface, template, Some(a1.as_ref().borrow().deref().clone()), registry, channel)
            .map_err(|e| {
                e.add_sim_trace("Build assign -> target")
                    .maybe_file_trace(&self.trace)
            })?;

        box_set_plc_primitive(&a1, &a2, &self.trace, registry)
    }
}
