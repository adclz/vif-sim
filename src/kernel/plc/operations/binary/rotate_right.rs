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
use crate::kernel::rust::operations::{box_rotate_right_plc_primitive};

#[derive(Clone)]
pub struct RotateRight {
    rotate: JsonTarget,
    rotate_with: JsonTarget,
    id: u64,
}

impl NewJsonOperation for RotateRight {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Rotate right"),
            json {
                rotate,
                rotate_with,
                id => as_u64,
            }
        );

        let rotate = parse_json_target(&rotate)?;
        let rotate_with = parse_json_target(&rotate_with)?;

        Ok(Self {
            rotate,
            rotate_with,
            id
        })
    }
}

impl BuildJsonOperation for RotateRight {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let rotate = self.rotate.solve_to_ref(interface, template, None, registry, channel)?;
        let rotate_with = self.rotate_with.solve_to_ref(interface, template, None, registry, channel)?;
        box_rotate_right_plc_primitive(&rotate, &rotate_with, self.id, registry)
    }
}
