use crate::{error, key_reader};
use crate::parser::body::json_target::JsonTarget;
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::kernel::plc::operations::operations::{BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation};
use crate::kernel::plc::types::primitives::traits::primitive_traits::{PrimitiveTrait, RawMut};
use crate::kernel::arch::local::pointer::LocalPointer;
use crate::kernel::registry::Kernel;
use crate::container::error::error::Stop;
use serde_json::{Map, Value};
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;
use crate::kernel::plc::types::primitives::traits::family_traits::{WithMutFamily, WithRefFamily};
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic};

#[derive(Clone)]
pub struct Reset {
    reset: Vec<JsonTarget>,
    trace: Option<FileTrace>,
}

impl FileTraceBuilder for Reset {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl NewJsonOperation for Reset {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop>
    where
        Self: Clone,
    {
        key_reader!(
            format!("Parse #reset"),
            json {
                trace? => as_object,
                reset => as_array,
            }
        );

        let trace = match trace {
            None => None,
            Some(a) => Self::build_trace(a),
        };


        Ok(Self {
            reset: reset
                .iter()
                .map(parse_json_target)
                .collect::<Result<Vec<JsonTarget>, Stop>>()?,
            trace,
        })
    }
}

impl BuildJsonOperation for Reset {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let reset = self
            .reset
            .iter()
            .map(|x| x.solve_as_local_pointer(interface, template, registry, channel))
            .collect::<Option<Vec<LocalPointer>>>()
            .ok_or_else(move || error!(format!("Invalid reference for internal reset"), format!("Build Reset -> reset references")))?;

        let raw_pointers = reset
            .iter()
            .fold(vec![], |_all, p| p.get_raw_pointers());

        Ok(Box::new(Operation::new(
            MaybeHeapOrStatic(Some(HeapOrStatic::Static(&"Reset"))),
            move |channel| {
                raw_pointers
                    .iter()
                    .for_each(|x| unsafe { (**x).reset_ptr(channel) });
                Ok(())
            },
            None,
            false,
            &self.trace
        )))
    }
}