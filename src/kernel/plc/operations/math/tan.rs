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
use crate::kernel::rust::operations::{box_tan_plc_primitive};
use crate::kernel::plc::types::primitives::floats::plc_float::PlcFloat;
use crate::kernel::plc::types::primitives::floats::real::Real;
use crate::kernel::arch::local::r#type::LocalType;
use crate::kernel::plc::types::primitives::traits::primitive_traits::PrimitiveTrait;


#[derive(Clone)]
pub struct Tan {
    tan: JsonTarget,
    id: u32,
}

impl NewJsonOperation for Tan {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Tan"),
            json {
                tan,
                id => as_u64,
            }
        );

        let id = id as u32;

        let tan = parse_json_target(&tan)?;

        Ok(Self {
            tan,
            id
        })
    }
}

impl BuildJsonOperation for Tan {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let tan = self.tan.solve_to_ref(interface, template, Some(LocalType::PlcFloat(PlcFloat::Real(Real::new_default(0)))), registry, channel)?;
        box_tan_plc_primitive(&tan, self.id, registry)
    }
}
