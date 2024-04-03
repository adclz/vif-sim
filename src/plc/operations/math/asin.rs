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
use crate::plc::primitives::boxed::operations::{box_asin_plc_primitive, box_sin_plc_primitive};
use crate::plc::primitives::floats::plc_float::PlcFloat;
use crate::plc::primitives::floats::real::Real;
use crate::registry::local::r#type::LocalType;


#[derive(Clone)]
pub struct ASin {
    asin: JsonTarget,
}

impl NewJsonOperation for ASin {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse ASin"),
            json {
                asin,
            }
        );

        let asin = parse_json_target(&asin)?;

        Ok(Self {
            asin,
        })
    }
}

impl BuildJsonOperation for ASin {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let asin = self.asin.solve_to_ref(interface, template, Some(LocalType::PlcFloat(PlcFloat::Real(Real::default()))), registry, channel)?;
        box_asin_plc_primitive(&asin, &None, registry)
    }
}
