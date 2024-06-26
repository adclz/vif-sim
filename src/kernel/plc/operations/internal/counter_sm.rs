﻿use crate::key_reader;
use crate::parser::body::json_target::JsonTarget;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::kernel::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation, RuntimeOperationTrait,
};
use crate::kernel::rust::partial::box_ord_plc_primitive;
use crate::kernel::rust::set::box_set_plc_primitive;
use crate::kernel::plc::types::primitives::traits::family_traits::{WithMutFamily, WithRefFamily};
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic};
use crate::kernel::plc::types::primitives::integers::plc_integer::PlcInteger;
use crate::kernel::plc::types::primitives::integers::sint::SInt;
use crate::kernel::plc::types::primitives::traits::primitive_traits::PrimitiveTrait;
use crate::kernel::plc::types::primitives::traits::crement::Crement;
use crate::kernel::arch::constant::r#type::ConstantType;
use crate::kernel::registry::Kernel;
use crate::container::error::error::Stop;
use serde_json::{Map, Value};
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;
use crate::kernel::plc::operations::internal::timer_sm::TimerStateMachine;

#[derive(Clone)]
pub struct CounterStateMachine {
    increment: Option<JsonTarget>,
    decrement: Option<JsonTarget>,

    reset: Option<JsonTarget>,
    load: Option<JsonTarget>,

    preset_var: JsonTarget,
    counter_var: JsonTarget,

    on_counter_up: Vec<JsonTarget>,
    on_counter_down: Vec<JsonTarget>,
    on_counter_reset: Vec<JsonTarget>,
    
    id: u32,
}

impl NewJsonOperation for CounterStateMachine {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop>
    where
        Self: Clone,
    {
        key_reader!(
            format!("Invalid Counter source"),
            json {
                increment?,
                decrement?,
                reset?,
                load?,
                preset_var,
                counter_var,
                on_counter_up => as_array,
                on_counter_down => as_array,
                on_counter_reset => as_array,
                id => as_u64,
            }
        );

        let id = id as u32;

        Ok(Self {
            increment: increment.map(parse_json_target).transpose()?,
            decrement: decrement.map(parse_json_target).transpose()?,
            reset: reset.map(parse_json_target).transpose()?,
            load: load.map(parse_json_target).transpose()?,
            counter_var: parse_json_target(counter_var)?,
            preset_var: parse_json_target(preset_var)?,
            on_counter_up: on_counter_up
                .iter()
                .map(parse_json_target)
                .collect::<Result<Vec<JsonTarget>, Stop>>()?,
            on_counter_down: on_counter_down
                .iter()
                .map(parse_json_target)
                .collect::<Result<Vec<JsonTarget>, Stop>>()?,
            on_counter_reset: on_counter_reset
                .iter()
                .map(parse_json_target)
                .collect::<Result<Vec<JsonTarget>, Stop>>()?,
            id
        })
    }
}

impl BuildJsonOperation for CounterStateMachine {
    fn build(
        &self,
        _interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let increment = self
            .increment
            .as_ref()
            .map(|x| x.solve_as_operation(_interface, template, registry, channel))
            .transpose()?;
        let decrement = self
            .decrement
            .as_ref()
            .map(|x| x.solve_as_operation(_interface, template, registry, channel))
            .transpose()?;
        let reset = self
            .reset
            .as_ref()
            .map(|x| x.solve_as_operation(_interface, template, registry, channel))
            .transpose()?;
        let load = self
            .load
            .as_ref()
            .map(|x| x.solve_as_operation(_interface, template, registry, channel))
            .transpose()?;

        let counter_var = self
            .counter_var
            .solve_as_local_pointer(_interface, template, registry, channel)
            .unwrap();
        let preset_var = self
            .preset_var
            .solve_as_local_pointer(_interface, template, registry, channel)
            .unwrap();
        let on_counter_up = self
            .on_counter_up
            .iter()
            .map(|x| x.solve_as_operation(_interface, template, registry, channel))
            .collect::<Result<Vec<RunTimeOperation>, Stop>>()?;
        let on_counter_down = self
            .on_counter_down
            .iter()
            .map(|x| x.solve_as_operation(_interface, template, registry, channel))
            .collect::<Result<Vec<RunTimeOperation>, Stop>>()?;
        let on_counter_reset = self
            .on_counter_reset
            .iter()
            .map(|x| x.solve_as_operation(_interface, template, registry, channel))
            .collect::<Result<Vec<RunTimeOperation>, Stop>>()?;

        let counter_down = box_ord_plc_primitive(&counter_var, &preset_var, self.id, registry)?;
        let counter_up = box_ord_plc_primitive(&counter_var, &preset_var, self.id, registry)?;

        let load_counter = box_set_plc_primitive(&counter_var, &preset_var,  self.id, true, registry)?;

        Ok(Box::new(Operation::new(
            MaybeHeapOrStatic(None),
            move |channel| {
                // If reset
                if let Some(a) = reset.as_ref() {
                    if a.with_plc_bool(channel, |a| Ok(a.as_bool()?.get(channel)?))?? {
                        on_counter_reset
                            .iter()
                            .try_for_each(|x| x.with_void(channel))?;
                    }
                }

                // Load the inner value
                if let Some(a) = load.as_ref() {
                    if a.with_plc_bool(channel, |a| Ok(a.as_bool()?.get(channel)?))?? {
                        load_counter.with_void(channel)?;
                    }
                }

                // Increment
                if let Some(a) = increment.as_ref() {
                    if a.with_plc_bool(channel, |a| Ok(a.as_bool()?.get(channel)?))?? {
                        counter_var
                            .with_mut_plc_integer(channel, &mut |a| a.increment(channel))??;
                    }
                }

                // Decrement
                if let Some(a) = decrement.as_ref() {
                    if a.with_plc_bool(channel, |a| Ok(a.as_bool()?.get(channel)?))?? {
                        counter_var
                            .with_mut_plc_integer(channel, &mut |a| a.decrement(channel))??;
                    }
                }

                // When counter reached up value
                if counter_up(channel)?.unwrap().is_ge() {
                    on_counter_up
                        .iter()
                        .try_for_each(|x| x.with_void(channel))?;
                }

                // When counter reached down value
                if counter_down(channel)?.unwrap().is_le() {
                    on_counter_down
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
