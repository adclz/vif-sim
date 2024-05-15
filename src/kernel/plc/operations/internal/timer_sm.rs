use crate::container::broadcast::broadcast::Broadcast;
use crate::container::container::{get_id, DELAYED_TIMERS};
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use crate::parser::body::body::parse_json_target;
use crate::parser::body::json_target::JsonTarget;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::kernel::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation, RuntimeOperationTrait,
};
use crate::kernel::rust::partial::box_ord_plc_primitive;
use crate::kernel::plc::types::primitives::traits::family_traits::{WithMutFamily, WithRefFamily};
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic};
use crate::kernel::plc::types::primitives::traits::primitive_traits::PrimitiveTrait;
use crate::kernel::plc::types::primitives::timers::traits::TimeDuration;
use crate::kernel::registry::Kernel;
use serde_json::{Map, Value};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::time::Duration;
use web_time::Instant;
use crate::kernel::plc::operations::unit::test::UnitTestJson;

pub struct TimerStateMachine {
    start: JsonTarget,
    reset: Option<JsonTarget>,
    preset_var: JsonTarget,
    timer_var: JsonTarget,

    on_timer_start: Vec<JsonTarget>,
    on_timer_elapsed: Vec<JsonTarget>,
    on_timer_reset: Vec<JsonTarget>,

    started: Rc<RefCell<bool>>,
    previous_duration: Rc<RefCell<Instant>>,
    id: u32,
}

impl Clone for TimerStateMachine {
    fn clone(&self) -> Self {
        Self {
            start: self.start.clone(),
            reset: self.reset.clone(),
            preset_var: self.preset_var.clone(),
            timer_var: self.timer_var.clone(),

            on_timer_start: self.on_timer_start.clone(),
            on_timer_elapsed: self.on_timer_elapsed.clone(),
            on_timer_reset: self.on_timer_reset.clone(),

            started: Rc::new(RefCell::new(false)),
            previous_duration: Rc::new(RefCell::new(Instant::now())),
            id: self.id
        }
    }
}

impl NewJsonOperation for TimerStateMachine {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop>
    where
        Self: Clone,
    {
        key_reader!(
            format!("Invalid Timer source"),
            json {
                start,
                reset?,
                preset_var,
                timer_var,
                on_timer_start => as_array,
                on_timer_elapsed => as_array,
                on_timer_reset => as_array,
                id => as_u64,
            }
        );

        let id = id as u32;
        
        Ok(Self {
            start: parse_json_target(start)?,
            reset: reset.map(parse_json_target).transpose()?,
            preset_var: parse_json_target(preset_var)?,
            timer_var: parse_json_target(timer_var)?,
            on_timer_start: on_timer_start
                .iter()
                .map(parse_json_target)
                .collect::<Result<Vec<JsonTarget>, Stop>>()?,
            on_timer_elapsed: on_timer_elapsed
                .iter()
                .map(parse_json_target)
                .collect::<Result<Vec<JsonTarget>, Stop>>()?,
            on_timer_reset: on_timer_reset
                .iter()
                .map(parse_json_target)
                .collect::<Result<Vec<JsonTarget>, Stop>>()?,
            started: Rc::new(RefCell::new(false)),
            previous_duration: Rc::new(RefCell::new(Instant::now())),
            id,
        })
    }
}

impl BuildJsonOperation for TimerStateMachine {
    fn build(
        &self,
        _interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast,
    ) -> Result<RunTimeOperation, Stop> {
        let start = self
            .start
            .solve_as_operation(_interface, template, registry, channel)?;
        let reset = self
            .reset
            .as_ref()
            .map(|x| x.solve_as_operation(_interface, template, registry, channel))
            .transpose()?;
        let preset_var = self
            .preset_var
            .solve_as_local_pointer(_interface, template, registry, channel)
            .ok_or_else(move || error!(format!("Invalid preset var for timer {}", self.preset_var), format!("Build Timer -> preset var")))?;
        let timer_var = self
            .timer_var
            .solve_as_local_pointer(_interface, template, registry, channel)
            .ok_or_else(move || error!(format!("Invalid timer var for timer {}", self.preset_var), format!("Build Timer -> timer var")))?;
        let on_timer_start = self
            .on_timer_start
            .iter()
            .map(|x| x.solve_as_operation(_interface, template, registry, channel))
            .collect::<Result<Vec<RunTimeOperation>, Stop>>()
            .map_err(|e| e.add_sim_trace(&"Build Timer -> on timer start operations"))?;
        let on_timer_elapsed = self
            .on_timer_elapsed
            .iter()
            .map(|x| x.solve_as_operation(_interface, template, registry, channel))
            .collect::<Result<Vec<RunTimeOperation>, Stop>>()
            .map_err(|e| e.add_sim_trace(&"Build Timer -> on timer elapsed operations"))?;
        let on_timer_reset = self
            .on_timer_reset
            .iter()
            .map(|x| x.solve_as_operation(_interface, template, registry, channel))
            .collect::<Result<Vec<RunTimeOperation>, Stop>>()
            .map_err(|e| e.add_sim_trace(&"Build Timer -> on timer reset operations"))?;

        let started = self.started.clone();
        let previous_duration = self.previous_duration.clone();
        let id = self.id;

        let elapsed = box_ord_plc_primitive(&timer_var, &preset_var, id, registry)?;

        Ok(Box::new(Operation::new(
            MaybeHeapOrStatic(None),
            move |channel| {
                // If reset
                if let Some(a) = reset.as_ref() {
                    if a.with_plc_bool(channel, |a| Ok(a.as_bool()?.get(channel)?))?? {
                        DELAYED_TIMERS.lock().unwrap().remove(&id);
                        *started.borrow_mut().deref_mut() = false;
                        *previous_duration.borrow_mut().deref_mut() = Instant::now();
                        on_timer_reset
                            .iter()
                            .try_for_each(|x| x.with_void(channel))?;
                    }
                }

                // If timer started
                if *started.borrow().deref() {
                    let instant = Instant::now()
                        .duration_since(*previous_duration.borrow().deref())
                        - *DELAYED_TIMERS.lock().unwrap().get(&id).unwrap();
                    timer_var
                        .with_mut_plc_time(channel, &mut |a| a.set_duration(&instant, channel))??;

                    // If elapsed
                    if elapsed(channel)?.unwrap().is_ge() {
                        on_timer_elapsed
                            .iter()
                            .try_for_each(|x| x.with_void(channel))?;
                    }
                }
                // Else should the timer start
                else if start.with_plc_bool(channel, |a| Ok(a.as_bool()?.get(channel)?))?? {
                    DELAYED_TIMERS
                        .lock()
                        .unwrap()
                        .insert(id, Duration::from_millis(0));
                    *started.borrow_mut().deref_mut() = true;
                    *previous_duration.borrow_mut().deref_mut() = Instant::now();
                    on_timer_start
                        .iter()
                        .try_for_each(|x| x.with_void(channel))?;
                }
                Ok(())
            },
            None,
            false,
            self.id
        )))
    }
}