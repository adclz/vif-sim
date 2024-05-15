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
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic};
use crate::kernel::arch::any::any_type::AnyRefType;


#[derive(Clone)]
pub struct For {
    _for: JsonTarget,
    with: JsonTarget,
    to: JsonTarget,
    by: Option<JsonTarget>,
    _do: Vec<JsonTarget>,
    id: u32,
}

impl NewJsonOperation for For {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse For"),
            json {
                _for,
                with,
                to,
                by?,
                _do => as_array,
                id => as_u64,
            }
        );

        let id = id as u32;

        let _for = parse_json_target(_for).map_err(|e|e.add_id(id))?;
        let with = parse_json_target(with).map_err(|e|e.add_id(id))?;
        let to = parse_json_target(to).map_err(|e|e.add_id(id))?;
        let by= match by {
            Some(a) => Some(parse_json_target(a).map_err(|e|e.add_id(id))?),
            None => None
        };

        let _do = _do
            .iter()
            .map(|f| parse_json_target(&f))
            .collect::<Result<Vec<JsonTarget>, Stop>>().map_err(|e|e.add_id(id))?;

        Ok(Self {
            _for,
            with,
            to,
            by,
            _do,
            id,
        })
    }
}

impl BuildJsonOperation for For {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {

        let _for = self
            ._for
            .solve_as_local_pointer(interface, template, registry, channel)
            .ok_or_else(move || error!(format!("ForOf first argument has to be a reference, got {}", self._for), format!("Build For -> for"))).map_err(|e|e.add_id(self.id))?;

        let with = self
            .with
            .solve_to_ref(interface, template, Some(_for.as_ref().borrow().deref().clone()), registry, channel)?;

        let for_with = box_set_plc_primitive(&_for, &with, self.id, true, registry)?;

        let to = self.to.solve_to_ref(interface, template, Some(_for.as_ref().borrow().deref().clone()), registry, channel).map_err(|e|e.add_id(self.id))?;

        let by = match &self.by {
            None => None,
            Some(a) => Some(a.solve_to_ref(interface, template, Some(_for.as_ref().borrow().deref().clone()), registry, channel).map_err(|e|e.add_id(self.id))?)
        };

        let mut _do: Vec<RunTimeOperation> = self
            ._do
            .iter()
            .map(|i| i.solve_as_operation(interface, template, registry, channel))
            .collect::<Result<Vec<RunTimeOperation>, Stop>>().map_err(|e|e.add_id(self.id))?;

        let for_to = box_ord_plc_primitive(&_for, &to, self.id, registry).map_err(|e|e.add_id(self.id))?;

        let incr_by = match &by {
            None => None,
            Some(a) => {
                let add = box_add_plc_primitive(&_for, a, self.id, registry).map_err(|e| e.add_id(self.id))?;
                let set = box_set_plc_primitive(&_for, &add, self.id, true, registry).map_err(|e| e.add_id(self.id))?;
                Some(set)
            }
        };

        let _for_clone = _for.clone();
        let with_clone = with.clone();
        let to_clone = to.clone();

        let display = match by {
            None => MaybeHeapOrStatic(Some(HeapOrStatic::Closure(Rc::new(RefCell::new(
                    move || format!("For {} := {} to {}", _for_clone, with_clone, to_clone)))))),
            Some(ref incr_by) => {
                let incr_by_clone = incr_by.clone();
                MaybeHeapOrStatic(Some(HeapOrStatic::Closure(Rc::new(RefCell::new(
                    move || format!("For {} := {} to {} by {}", _for_clone, with_clone, to_clone, incr_by_clone))))))
            }
        };

        let id = self.id;

        Ok(Box::new(Operation::new(
            display,
            move |channel| {
                for_with.with_void(channel)?;
                let earlier = Instant::now();

                while for_to(channel)?.unwrap().is_ne() {
                    for operation in &_do {
                        operation.with_void(channel).map_err(|e|e.add_id(id))?;
                    }

                    // Increment
                    if let Some(incr) = &incr_by {
                        incr.with_void(channel).map_err(|e|e.add_id(id))?;
                    }

                    let elapsed = Instant::now().duration_since(earlier);
                    if elapsed > THOUSAND_MS {
                        return match &by {
                            None => Err(error!(format!("For of loop took longer than 100 ms to execute. \nStatus of loop: FOR {} := {} TO {}", _for, with, to)))
                                .map_err(|e|e.add_id(id)),
                            Some(a) => Err(error!(format!("For of loop took longer than 100 ms to execute. \nStatus of loop: FOR {} := {} TO {} BY {}", _for, with, to, a)))
                                .map_err(|e|e.add_id(id))
                        }
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
