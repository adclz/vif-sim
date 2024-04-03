use crate::parser::body::json_target::JsonTarget;
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::internal::template_impl::TemplateMemory;
use crate::plc::operations::operations::{BuildJsonOperation, NewJsonOperation, RunTimeOperation};
use crate::registry::registry::Kernel;
use crate::container::error::error::Stop;
use crate::{key_reader};
use serde_json::{Map, Value};
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;
use crate::plc::primitives::boxed::operations::{box_abs_plc_primitive};


#[derive(Clone)]
pub struct Abs {
    abs: JsonTarget,
}

impl NewJsonOperation for Abs {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Abs"),
            json {
                abs,
            }
        );

        let abs = parse_json_target(&abs)?;

        Ok(Self {
            abs
        })
    }
}

impl BuildJsonOperation for Abs {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let abs = self.abs.solve_to_ref(interface, template, None, registry, channel)?;
        box_abs_plc_primitive(&abs, &None, registry)
    }
}
