use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::kernel::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation,
};
use crate::kernel::registry::Kernel;
use crate::container::error::error::Stop;
use serde_json::{Map, Value};
use crate::container::broadcast::broadcast::Broadcast;
use crate::key_reader;
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic};

#[derive(Clone)]
pub struct Return {
    trace: Option<FileTrace>
}

impl FileTraceBuilder for Return {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl NewJsonOperation for Return {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop>
    where
        Self: Clone,
    {
        key_reader!(
            format!("Parse Return"),
            json {
                trace? => as_object,
            }
        );

        let trace = match trace {
            None => None,
            Some(a) => Self::build_trace(a),
        };

        Ok(Self {trace})
    }
}

impl BuildJsonOperation for Return {
    fn build(
        &self,
        _interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        _registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop>
    where
        Self: Clone,
    {
        Ok(Box::new(Operation::new(
            MaybeHeapOrStatic(Some(HeapOrStatic::Static(&"Return"))),
            move |_channel| Ok(()),
            None,
            false,
            &self.trace
        )))
    }
}
