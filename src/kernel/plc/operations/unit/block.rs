use crate::parser::body::json_target::JsonTarget;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::kernel::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation, RuntimeOperationTrait,
};
use crate::kernel::registry::{get_or_insert_global_string, get_string, Kernel};
use crate::container::error::error::Stop;
use crate::{key_reader};
use serde_json::{Map, Value};
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic};

#[derive(Clone)]
pub struct UnitBlock {
    blocks: Vec<JsonTarget>,
    description: String,
    id: u64,
}

impl NewJsonOperation for UnitBlock {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            "Parse Unit block".to_string(),
            json {
                block => as_array,
                description => as_str,
                id => as_u64,
            }
        );

        Ok(Self {
            blocks: block
                .iter()
                .map(|f| parse_json_target(f))
                .collect::<Result<Vec<JsonTarget>, Stop>>()?,
            description: description.to_string(),
            id,
        })
    }
}

impl BuildJsonOperation for UnitBlock {
    fn build(
        &self,
        section: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let mut blocks = self
            .blocks
            .iter()
            .map(|f| f.solve_as_operation(section, template, registry, channel))
            .collect::<Result<Vec<RunTimeOperation>, Stop>>()
            .map_err(|e| {
                e.add_sim_trace(&format!(
                    "Build Unit block -> Build operations [{}]",
                    self.description
                ))
                .add_id(self.id)
            })?;

        let description = get_or_insert_global_string(&self.description.clone());

        Ok(Box::new(Operation::new(
            MaybeHeapOrStatic(Some(HeapOrStatic::Static(&"Unit Block"))),
            move |channel| {
                let index = channel
                    .get_cycle_stack()
                    .borrow_mut()
                    .add_section(description, "Unit_block");
                
                blocks.iter_mut().try_for_each(|f| {
                    f.with_void(channel)?;
                    if f.return_early() {
                        return Ok(());
                    }
                    Ok(())
                })?;

                channel
                    .get_cycle_stack()
                    .borrow_mut()
                    .go_back_to_section(index);
                Ok(())
            },
            None,
            false,
            self.id
        )))
    }
}
