use crate::parser::body::json_target::JsonTarget;
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::internal::template_impl::TemplateMemory;
use crate::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation, RuntimeOperationTrait,
};
use crate::registry::registry::Kernel;
use crate::container::error::error::Stop;
use crate::{key_reader};
use serde_json::{Map, Value};
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;

#[derive(Clone)]
pub struct UnitBlock {
    blocks: Vec<JsonTarget>,
    description: String,
    trace: Option<FileTrace>,
}

impl FileTraceBuilder for UnitBlock {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl NewJsonOperation for UnitBlock {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            "Parse Unit block".to_string(),
            json {
                block => as_array,
                description => as_str,
                trace? => as_object,
            }
        );

        let trace = match trace {
            None => None,
            Some(a) => Self::build_trace(a),
        };

        Ok(Self {
            blocks: block
                .iter()
                .map(|f| parse_json_target(f))
                .collect::<Result<Vec<JsonTarget>, Stop>>()?,
            description: description.to_string(),
            trace,
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
                .maybe_file_trace(&self.trace)
            })?;

        let description = self.description.clone();

        Ok(Box::new(Operation::new(
            move |channel| {
                let index = channel
                    .get_cycle_stack()
                    .borrow_mut()
                    .add_section(&description, "Unit_block");
                
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
            &self.trace
        )))
    }
}
