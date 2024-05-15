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
use crate::kernel::rust::operations::{box_abs_plc_primitive};


#[derive(Clone)]
pub struct Abs {
    abs: JsonTarget,
    id: u32,
}

impl NewJsonOperation for Abs {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Abs"),
            json {
                abs,
                id => as_u64,
            }
        );

        let id = id as u32;

        let abs = parse_json_target(&abs)?;

        Ok(Self {
            abs,
            id
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
        box_abs_plc_primitive(&abs, self.id, registry)
    }
}
