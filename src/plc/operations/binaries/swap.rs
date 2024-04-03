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
use crate::plc::primitives::boxed::operations::{box_swap_bytes_plc_primitive};


#[derive(Clone)]
pub struct Swap {
    swap: JsonTarget,
}

impl NewJsonOperation for Swap {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Swap (bytes)"),
            json {
                swap,
            }
        );

        let swap = parse_json_target(swap)?;

        Ok(Self {
            swap,
        })
    }
}

impl BuildJsonOperation for Swap {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let swap = self.swap.solve_to_ref(interface, template, None, registry, channel)?;
        box_swap_bytes_plc_primitive(&swap, &None, registry)
    }
}
