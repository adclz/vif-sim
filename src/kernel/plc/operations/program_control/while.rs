use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use crate::parser::body::json_target::JsonTarget;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::kernel::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation, RuntimeOperationTrait,
};
use crate::kernel::rust::partial::box_ord_plc_primitive;
use crate::kernel::rust::set::box_set_plc_primitive;
use crate::kernel::registry::Kernel;
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use serde_json::{Map, Value};
use web_time::Instant;
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::container::THOUSAND_MS;
use crate::kernel::rust::operations::box_add_plc_primitive;
use crate::kernel::plc::types::primitives::traits::primitive_traits::{Primitive, PrimitiveTrait};
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic};

#[derive(Clone)]
pub struct While {
    _while: JsonTarget,
    _do: Vec<JsonTarget>,
    id: u64,
}

impl NewJsonOperation for While {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse While"),
            json {
                _while,
                _do => as_array,
                id => as_u64,
            }
        );

        let _while = parse_json_target(_while).map_err(|e|e.add_id(id))?;

        let _do = _do
            .iter()
            .map(|f| parse_json_target(&f))
            .collect::<Result<Vec<JsonTarget>, Stop>>().map_err(|e|e.add_id(id))?;

        Ok(Self {
            _while,
            _do,
            id,
        })
    }
}

impl BuildJsonOperation for While {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {

        let _while = self
            ._while
            .solve_to_ref(interface, template, None, registry, channel)
            .map_err(|e|e.add_id(self.id))?;

        let _do: Vec<RunTimeOperation> = self
            ._do
            .iter()
            .map(|i| i.solve_as_operation(interface, template, registry, channel))
            .collect::<Result<Vec<RunTimeOperation>, Stop>>().map_err(|e|e.add_id(self.id))?;

        let _while_clone = _while.clone();
        let id = self.id;

        Ok(Box::new(Operation::new(
            MaybeHeapOrStatic(Some(HeapOrStatic::Closure(Rc::new(RefCell::new(move || format!("While {}", _while_clone)))))),
            move |channel| {
                let earlier = Instant::now();

                while _while.as_bool(channel)? {
                    for operation in &_do {
                        operation.with_void(channel).map_err(|e|e.add_id(id))?;
                    }

                    let elapsed = Instant::now().duration_since(earlier);
                    if elapsed > THOUSAND_MS {
                      return  Err(error!(format!("While loop took longer than 100 ms to execute.")))
                            .map_err(|e|e.add_id(id))
                    };
                }
                Ok(())
            },
            None,
            false,
            self.id
        )))
    }
}
