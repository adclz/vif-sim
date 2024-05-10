use ansi_term::Color::Green;
use crate::{key_reader};
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation,
};
use crate::kernel::registry::Kernel;
use crate::container::error::error::Stop;
use crate::container::container::{DELAYED_TIMERS, get_id,};
#[cfg(target_arch = "wasm32")]
use crate::container::container::{read_sab_commands};
use serde_json::{Map, Value};
use ansi_term::Colour::Yellow;
use tsify::Tsify;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use wasm_bindgen::prelude::wasm_bindgen;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::container::container::SimulationStatus;
use web_time::Instant;
use crate::container::broadcast::broadcast::Broadcast;
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic};

#[derive(Tsify)]
#[wasm_bindgen(skip_typescript)]
#[derive(Clone)]
pub struct BreakPoint {
    id: u64,
    status: BreakPointStatus,
}

#[wasm_bindgen]
impl BreakPoint {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            status: BreakPointStatus::Inactive,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn get_id(&self) -> u64 {
        self.id
    }

    #[wasm_bindgen(getter)]
    pub fn get_status(&self) -> BreakPointStatus {
        self.status
    }
}

impl BreakPoint {
    pub fn set_status(&mut self, status: BreakPointStatus) {
        self.status = status
    }
}

pub fn pause_simulation(channel: &Broadcast, id: Option<u64>) -> Result<(), Stop> {
    channel.add_message(
        &Yellow.paint("[Pause] Simulation paused").to_string());
    let earlier = Instant::now();

    channel.push_cycle_stack();
    channel.set_simulation_status(&SimulationStatus::Pause);

    #[cfg(not(target_arch = "wasm32"))]
    channel.add_warning("Pause is not available on OS targets.");

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(a) = id {
            channel.add_breakpoint_status(&BreakPointUpdateStatus::new(
                a,
                BreakPointStatus::Active,
            ));
        }
        channel.publish();
        js_sys::Atomics::wait(&channel.get_pause_int32(), 0, 1).unwrap_throw();

        (*DELAYED_TIMERS.lock().unwrap())
            .iter_mut()
            .for_each(|(_ptr, dur)| {
                *dur += Instant::now().duration_since(earlier);
            });

        channel.add_message(&Green.paint("[Pause] Simulation resumed").to_string());
        channel.set_simulation_status(&SimulationStatus::Start);
        if let Some(a) = id {
            channel.add_breakpoint_status(&BreakPointUpdateStatus::new(
                a,
                BreakPointStatus::Inactive,
            ));
        }
        channel.publish();

        let stop = read_sab_commands(&channel);
        if stop {
            return Err(Stop::new("Manual stop before cycle end".into(), &None, id))
        }
    }
    Ok(())
}

pub fn enableBreakpoint(channel: &Broadcast, bp: u64) {
    channel.add_breakpoint_status(&BreakPointUpdateStatus::new(
        bp,
        BreakPointStatus::Inactive,
    ));
    channel.add_message(&format!("Enabled breakpoint {}", bp))
}

pub fn disableBreakpoint(channel: &Broadcast, bp: u64) {
    channel.add_breakpoint_status(&BreakPointUpdateStatus::new(
        bp,
        BreakPointStatus::Disabled,
    ));
    channel.add_message(&format!("Disabled breakpoint {}", bp))
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum BreakPointStatus {
    Inactive,
    Active,
    Disabled,
}

#[derive(Tsify)]
#[wasm_bindgen(skip_typescript)]
#[derive(Clone)]
pub struct BreakPointUpdateStatus {
    id: u64,
    status: BreakPointStatus,
}

#[wasm_bindgen]
impl BreakPointUpdateStatus {
    pub fn new(id: u64, status: BreakPointStatus) -> Self {
        Self {
            id,
            status,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn get_id(&self) -> u64 {
        self.id
    }

    #[wasm_bindgen(getter)]
    pub fn get_status(&self) -> BreakPointStatus {
        self.status
    }
}


pub struct BreakpointJson {
    id: u64,
}

impl Clone for BreakpointJson {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
        }
    }
}

impl NewJsonOperation for BreakpointJson {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Breakpoint"),
            json {
                id => as_u64,
            }
        );

        Ok(Self {
            id,
        })
    }
}

impl BuildJsonOperation for BreakpointJson {
    fn build(
        &self,
        _parent_interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast,
    ) -> Result<RunTimeOperation, Stop> {
        if !registry.should_ignore_operation() {
            channel.add_breakpoint(&BreakPoint::new(
                self.id
            ));
        }
        let id = self.id;
        Ok(Box::new(Operation::new(
            MaybeHeapOrStatic(Some(HeapOrStatic::Static(&"Breakpoint"))),
            move |channel| {
                if channel.is_breakpoint_enabled(id) {
                    pause_simulation(channel, Some(id))?;
                }
                Ok(())
            },
            None,
            false,
            self.id
        )))
    }
}


