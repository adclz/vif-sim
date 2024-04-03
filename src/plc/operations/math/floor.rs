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
use crate::plc::primitives::boxed::operations::{box_floor_plc_primitive};
use crate::plc::primitives::floats::plc_float::PlcFloat;
use crate::plc::primitives::floats::real::Real;
use crate::registry::local::r#type::LocalType;


#[derive(Clone)]
pub struct Floor {
    floor: JsonTarget,
}

impl NewJsonOperation for Floor {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Floor"),
            json {
                floor,
            }
        );

        let floor = parse_json_target(&floor)?;

        Ok(Self {
            floor
        })
    }
}

impl BuildJsonOperation for Floor {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let floor = self.floor.solve_to_ref(interface, template, Some(LocalType::PlcFloat(PlcFloat::Real(Real::default()))), registry, channel)?;
        box_floor_plc_primitive(&floor, &None, registry)
    }
}
