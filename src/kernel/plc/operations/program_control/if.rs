use std::cell::RefCell;
use std::rc::Rc;
use crate::key_reader;
use crate::parser::body::json_target::JsonTarget;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::kernel::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation, RuntimeOperationTrait,
};
use crate::kernel::plc::types::primitives::traits::family_traits::{WithMutFamily, WithRefFamily};
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic};
use crate::kernel::plc::types::primitives::traits::primitive_traits::PrimitiveTrait;
use crate::kernel::registry::Kernel;
use crate::container::error::error::Stop;
use serde_json::{Map, Value};
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;

#[derive(Clone)]
pub struct If {
    _if: JsonTarget,
    then: Vec<JsonTarget>,
    _else: Option<Vec<JsonTarget>>,
    id: u32
}

impl NewJsonOperation for If {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse If"),
            json {
                _if,
                then => as_array,
                _else? => as_array,
                id => as_u64,
            }
        );

        let id = id as u32;

        let _if = parse_json_target(&_if).map_err(|e| {
            e.add_sim_trace(&format!("Parse If Operation"))
                .add_id(id)
        })?;

        let then = then
            .iter()
            .map(|f| parse_json_target(&f))
            .collect::<Result<Vec<JsonTarget>, Stop>>()?;

        let maybe_else;

        if _else.is_some() {
            maybe_else = Some(
                _else
                    .unwrap()
                    .iter()
                    .map(|f| parse_json_target(&f))
                    .collect::<Result<Vec<JsonTarget>, Stop>>()?,
            );
        } else {
            maybe_else = None;
        }

        Ok(Self {
            _if,
            then,
            _else: maybe_else,
            id,
        })
    }
}

impl BuildJsonOperation for If {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let mut _if = self
            ._if
            .solve_as_operation(interface, template, registry, channel)
            .map_err(|e| {
                e.add_sim_trace(&format!("Build If -> Build first condition"))
                    .add_id(self.id)
            })?;

        let then = self
            .then
            .iter()
            .map(|i| i.solve_as_operation(interface, template, registry, channel))
            .collect::<Result<Vec<RunTimeOperation>, Stop>>()
            .map_err(|e| {
                e.add_sim_trace(&format!("Build If -> Build then operations"))
                    .add_id(self.id)
            })?;

        let _else: Option<Vec<RunTimeOperation>> = match &self._else {
            Some(target) => Some(
                target
                    .iter()
                    .map(|i| i.solve_as_operation(interface, template, registry, channel))
                    .collect::<Result<Vec<RunTimeOperation>, Stop>>()
                    .map_err(|e| {
                        e.add_sim_trace(&format!("Build If -> Build else condition(s)"))
                            .add_id(self.id)
                    })?,
            ),
            None => None,
        };

        let if_clone = self._if.clone();

        Ok(Box::new(Operation::new(
            MaybeHeapOrStatic(Some(HeapOrStatic::Closure(Rc::new(RefCell::new(move || format!("If {}", if_clone)))))),
            move |channel| {
                _if.with_plc_bool(channel, |a| {
                    if a.as_bool().unwrap().get(channel)? {
                        for operation in &then {
                            operation.with_void(channel)?;
                        }
                    } else if _else.is_some() {
                        let else_operations = _else.as_ref().unwrap();
                        for operation in else_operations {
                            operation.with_void(channel)?;
                        }
                    };
                    Ok(())
                })??;
                Ok(())
            },
            None,
            false,
            self.id
        )))
    }
}
