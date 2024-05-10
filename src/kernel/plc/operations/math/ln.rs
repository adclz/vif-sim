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
use crate::kernel::rust::operations::{box_ln_plc_primitive};
use crate::kernel::plc::types::primitives::floats::plc_float::PlcFloat;
use crate::kernel::plc::types::primitives::floats::real::Real;
use crate::kernel::arch::local::r#type::LocalType;


#[derive(Clone)]
pub struct Ln {
    ln: JsonTarget,
    id: u64
}

impl NewJsonOperation for Ln {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Ln"),
            json {
                ln,
                id => as_u64,
            }
        );

        let ln = parse_json_target(&ln)?;

        Ok(Self {
            ln,
            id
        })
    }
}

impl BuildJsonOperation for Ln {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let ln = self.ln.solve_to_ref(interface, template, Some(LocalType::PlcFloat(PlcFloat::Real(Real::default()))), registry, channel)?;
        box_ln_plc_primitive(&ln, self.id, registry)
    }
}
