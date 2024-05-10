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
use crate::kernel::rust::operations::{box_swap_bytes_plc_primitive};


#[derive(Clone)]
pub struct Swap {
    swap: JsonTarget,
    id: u64,
}

impl NewJsonOperation for Swap {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Swap (bytes)"),
            json {
                swap,
                id => as_u64,
            }
        );

        let swap = parse_json_target(swap)?;

        Ok(Self {
            swap,
            id
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
        box_swap_bytes_plc_primitive(&swap, self.id, registry)
    }
}
