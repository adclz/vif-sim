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
use crate::kernel::rust::operations::{box_exp_plc_primitive};
use crate::kernel::plc::types::primitives::floats::plc_float::PlcFloat;
use crate::kernel::plc::types::primitives::floats::real::Real;
use crate::kernel::arch::local::r#type::LocalType;
use crate::kernel::plc::types::primitives::traits::primitive_traits::PrimitiveTrait;


#[derive(Clone)]
pub struct Exp {
    exp: JsonTarget,
    id: u32,
}

impl NewJsonOperation for Exp {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Exp"),
            json {
                exp,
                id => as_u64,
            }
        );

        let id = id as u32;

        let exp = parse_json_target(&exp)?;

        Ok(Self {
            exp,
            id
        })
    }
}

impl BuildJsonOperation for Exp {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let exp = self.exp.solve_to_ref(interface, template, Some(LocalType::PlcFloat(PlcFloat::Real(Real::new_default(0)))), registry, channel)?;
        box_exp_plc_primitive(&exp, self.id, registry)
    }
}
