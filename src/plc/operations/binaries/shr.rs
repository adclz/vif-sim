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
use crate::plc::primitives::boxed::operations::{box_shr_plc_primitive};


#[derive(Clone)]
pub struct Shr {
    shr: JsonTarget,
    shr_with: JsonTarget,
}

impl NewJsonOperation for Shr {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Shr"),
            json {
                shr,
                shr_with,
            }
        );

        let shr = parse_json_target(&shr)?;
        let shr_with = parse_json_target(&shr_with)?;

        Ok(Self {
            shr,
            shr_with
        })
    }
}

impl BuildJsonOperation for Shr {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let shr = self.shr.solve_to_ref(interface, template, None, registry, channel)?;
        let shr_with = self.shr_with.solve_to_ref(interface, template, None, registry, channel)?;
        box_shr_plc_primitive(&shr, &shr_with, &None, registry)
    }
}
