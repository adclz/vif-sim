use std::ops::Deref;
use crate::parser::body::json_target::JsonTarget;
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::internal::template_impl::TemplateMemory;
use crate::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation, RuntimeOperationTrait,
};
use crate::plc::primitives::boxed::partial::box_ord_plc_primitive;
use crate::plc::primitives::boxed::set::box_set_plc_primitive;
use crate::registry::registry::Kernel;
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use serde_json::{Map, Value};
use web_time::Instant;
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::container::THOUSAND_MS;
use crate::plc::primitives::boxed::operations::box_add_plc_primitive;
use crate::registry::any::any_type::AnyRefType;


#[derive(Clone)]
pub struct For {
    _for: JsonTarget,
    with: JsonTarget,
    to: JsonTarget,
    by: Option<JsonTarget>,
    _do: Vec<JsonTarget>,
    trace: Option<FileTrace>,
}

impl FileTraceBuilder for For {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl NewJsonOperation for For {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse For"),
            json {
                _for,
                with,
                to,
                by?,
                _do => as_array,
                trace? => as_object,
            }
        );

        let trace = match trace {
            None => None,
            Some(a) => Self::build_trace(a),
        };

        let _for = parse_json_target(_for).map_err(|e|e.maybe_file_trace(&trace))?;
        let with = parse_json_target(with).map_err(|e|e.maybe_file_trace(&trace))?;
        let to = parse_json_target(to).map_err(|e|e.maybe_file_trace(&trace))?;
        let by= match by {
            Some(a) => Some(parse_json_target(a).map_err(|e|e.maybe_file_trace(&trace))?),
            None => None
        };

        let _do = _do
            .iter()
            .map(|f| parse_json_target(&f))
            .collect::<Result<Vec<JsonTarget>, Stop>>().map_err(|e|e.maybe_file_trace(&trace))?;

        Ok(Self {
            _for,
            with,
            to,
            by,
            _do,
            trace,
        })
    }
}

impl BuildJsonOperation for For {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let trace = self.trace.clone();

        let _for = self
            ._for
            .solve_as_local_pointer(interface, template, registry, channel)
            .ok_or(error!(format!("ForOf first argument has to be a reference, got {}", self._for), format!("Build For -> for"))).map_err(|e|e.maybe_file_trace(&trace))?;

        let with = self
            .with
            .solve_to_ref(interface, template, Some(_for.as_ref().borrow().deref().clone()), registry, channel)?;

        let for_with = box_set_plc_primitive(&_for, &with, &trace, registry)?;

        let to = self.to.solve_to_ref(interface, template, Some(_for.as_ref().borrow().deref().clone()), registry, channel).map_err(|e|e.maybe_file_trace(&trace))?;

        let by = match &self.by {
            None => None,
            Some(a) => Some(a.solve_to_ref(interface, template, Some(_for.as_ref().borrow().deref().clone()), registry, channel).map_err(|e|e.maybe_file_trace(&trace))?)
        };

        let mut _do: Vec<RunTimeOperation> = self
            ._do
            .iter()
            .map(|i| i.solve_as_operation(interface, template, registry, channel))
            .collect::<Result<Vec<RunTimeOperation>, Stop>>().map_err(|e|e.maybe_file_trace(&trace))?;

        let for_to = box_ord_plc_primitive(&_for, &to, &None, registry).map_err(|e|e.maybe_file_trace(&trace))?;

        let incr_by = match &by {
            None => None,
            Some(a) => {
                let add = box_add_plc_primitive(&_for, a, &trace, registry).map_err(|e| e.maybe_file_trace(&trace))?;
                let set = box_set_plc_primitive(&_for, &add, &trace, registry).map_err(|e| e.maybe_file_trace(&trace))?;
                Some(set)
            }
        };

        Ok(Box::new(Operation::new(
            move |channel| {
                for_with.with_void(channel)?;
                let earlier = Instant::now();

                while for_to(channel)?.unwrap().is_ne() {
                    for operation in &_do {
                        operation.with_void(channel).map_err(|e|e.maybe_file_trace(&trace))?;
                    }

                    // Increment
                    if let Some(incr) = &incr_by {
                        incr.with_void(channel).map_err(|e|e.maybe_file_trace(&trace))?;
                    }

                    let elapsed = Instant::now().duration_since(earlier);
                    if elapsed > THOUSAND_MS {
                        return match &by {
                            None => Err(error!(format!("For of loop took longer than 100 ms to execute. \nStatus of loop: FOR {} := {} TO {}", _for, with, to)))
                                .map_err(|e|e.maybe_file_trace(&trace)),
                            Some(a) => Err(error!(format!("For of loop took longer than 100 ms to execute. \nStatus of loop: FOR {} := {} TO {} BY {}", _for, with, to, a)))
                                .map_err(|e|e.maybe_file_trace(&trace))
                        }
                    };
                }
                Ok(())
            },
            None,
            false,
            &self.trace
        )))
    }
}
