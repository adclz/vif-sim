use crate::parser::body::json_target::JsonTarget;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::kernel::plc::operations::operations::{BuildJsonOperation, NewJsonOperation, RunTimeOperation};
use crate::kernel::registry::Kernel;
use crate::container::error::error::Stop;
use crate::{key_reader};
use serde_json::{Map, Value};
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;
use crate::kernel::rust::operations::{box_shl_plc_primitive};


#[derive(Clone)]
pub struct Shl {
    shl: JsonTarget,
    shl_with: JsonTarget,
}

impl NewJsonOperation for Shl {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Shl"),
            json {
                shl,
                shl_with,
            }
        );

        let shl = parse_json_target(&shl)?;
        let shl_with = parse_json_target(&shl_with)?;

        Ok(Self {
            shl,
            shl_with
        })
    }
}

impl BuildJsonOperation for Shl {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let shl = self.shl.solve_to_ref(interface, template, None, registry, channel)?;
        let shl_with = self.shl_with.solve_to_ref(interface, template, None, registry, channel)?;
        box_shl_plc_primitive(&shl, &shl_with, &None, registry)
    }
}
