use crate::parser::body::json_target::JsonTarget;
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::internal::template_impl::TemplateMemory;
use crate::plc::operations::operations::{BuildJsonOperation, NewJsonOperation, RunTimeOperation};
use crate::registry::registry::Kernel;
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use serde_json::{Map, Value};
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;
use crate::plc::primitives::boxed::operations::box_add_plc_primitive;
use crate::plc::primitives::boxed::operations::box_div_plc_primitive;
use crate::plc::primitives::boxed::operations::box_mul_plc_primitive;
use crate::plc::primitives::boxed::operations::box_rem_plc_primitive;
use crate::plc::primitives::boxed::operations::box_sub_plc_primitive;
use crate::registry::local::r#type::IntoLocalType;

pub struct Calc {
    calc: JsonTarget,
    with: JsonTarget,
    operator: String,
}

impl Clone for Calc {
    fn clone(&self) -> Self {
        Self {
            calc: self.calc.clone(),
            with: self.with.clone(),
            operator: self.operator.clone(),
        }
    }
}

impl NewJsonOperation for Calc {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Calc"),
            json {
                calc,
                with,
                operator => as_str,
            }
        );

        let calc = parse_json_target(&calc)?;
        let with = parse_json_target(&with)?;

        Ok(Self {
            calc,
            with,
            operator: operator.to_string(),
        })
    }
}

impl BuildJsonOperation for Calc {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let o1 = self.calc.solve_to_ref(interface, template, None, registry, channel)?;
        let o2 = self.with.solve_to_ref(interface, template, Some(o1.transform()?), registry, channel)?;

        match self.operator.as_str() {
            "+" => box_add_plc_primitive(&o1, &o2, &None, registry),
            "-" => box_sub_plc_primitive(&o1, &o2, &None, registry),
            "*" => box_mul_plc_primitive(&o1, &o2, &None, registry),
            "/" => box_div_plc_primitive(&o1, &o2, &None, registry),
            "MOD" => box_rem_plc_primitive(&o1, &o2, &None, registry),
            _ => Err(error!(format!("Invalid calc operator {}", self.operator.as_str()), format!("0"))),
        }
    }
}
