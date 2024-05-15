extern crate console_error_panic_hook;

use crate::kernel::registry::Kernel;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::simulation::simulation::Simulation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use wasm_bindgen::prelude::wasm_bindgen;

use ansi_term::Colour::{Blue, Green, Purple, Yellow, Red};

use web_time::{Duration, Instant};
use crate::parser::main::program::parse_program;
use crate::parser::main::provider::parse_provider;

#[cfg(target_arch = "wasm32")]
use gloo_timers::callback::Timeout;
use js_sys::{Int32Array, SharedArrayBuffer};
use serde_json::Value;
use tsify::Tsify;
use uuid::Uuid;
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use crate::js::typed_array::shiftLeft;
use crate::{error, key_reader};
use crate::container::error::error::Stop;
use crate::parser::main::exclude::{parse_type_aliases, parse_return_operations, parse_exclude_sections, parse_exclude_types, parse_filter_operations};
use crate::container::simulation::pause::{enableBreakpoint, pause_simulation, disableBreakpoint};

pub static DELAYED_TIMERS: Lazy<Arc<Mutex<HashMap<u32, Duration>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub static COUNTER: AtomicUsize = AtomicUsize::new(1);
pub static BACKUP_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub fn reset_counter_to_backup() {
    COUNTER.store(BACKUP_COUNTER.load(Ordering::Relaxed), Ordering::Relaxed);
}

pub fn reset_counter_to_default() {
    COUNTER.store(1_usize, Ordering::Relaxed);
}

pub fn set_counter_backup() {
    BACKUP_COUNTER.store(COUNTER.load(Ordering::Relaxed), Ordering::Relaxed)
}

pub static IS_RUNNING: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));

pub static CONTAINER_PARAMS: Lazy<Arc<Mutex<ContainerParams>>> = Lazy::new(|| Arc::new(Mutex::new(ContainerParams::default())));


#[wasm_bindgen]
pub fn is_running() -> bool {
    *IS_RUNNING.lock().unwrap()
}


#[wasm_bindgen]
pub struct Container {
    registry: Kernel,
    channel: Broadcast,
    id: Uuid,
    #[cfg(target_arch = "wasm32")]
    pause_int32: js_sys::Int32Array,
    #[cfg(target_arch = "wasm32")]
    command_lock_int32: js_sys::Int32Array,
    #[cfg(target_arch = "wasm32")]
    runtime_commands_sab: Option<js_sys::SharedArrayBuffer>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn boot_container(params: Option<ContainerParams>, pause_sab: js_sys::SharedArrayBuffer, command_lock_sab: js_sys::SharedArrayBuffer) -> Container {
    console_error_panic_hook::set_once();
    Container::new(params, pause_sab, command_lock_sab)
}

#[cfg(not(target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn boot_container(params: Option<ContainerParams>) -> Container {
    console_error_panic_hook::set_once();
    Container::new(params)
}

impl Container {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(params: Option<ContainerParams>) -> Self {
        let id = Uuid::new_v4();
        Self {
            registry: Kernel::default(),
            channel: Broadcast::new(&id),
            id,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(params: Option<ContainerParams>, pause_sab: js_sys::SharedArrayBuffer, command_lock_sab: js_sys::SharedArrayBuffer) -> Self {
        let id = Uuid::new_v4();
        Self {
            registry: Kernel::default(),
            channel: Broadcast::new(&id, &pause_sab, &command_lock_sab),
            id,
            pause_int32: js_sys::Int32Array::new(&pause_sab),
            command_lock_int32: js_sys::Int32Array::new(&command_lock_sab),
            runtime_commands_sab: None,
        }
    }
}

pub const FOUR_MS: Duration = Duration::from_millis(4);
pub const FIVE_MS: Duration = Duration::from_millis(5);
pub const THOUSAND_MS: Duration = Duration::from_millis(100);

#[wasm_bindgen]
impl Container {
    pub fn get_id(&self) -> String {
        self.id.to_string()
    }

    pub fn load_server_params(&mut self, json: &str) {
        let json: HashMap<String, Value> = match serde_json::from_str(json) {
            Ok(a) => a,
            Err(a) => {
                self.channel.add_error(&error!(format!("Invalid container params"), format!("Parse container params")));
                self.registry.clear_all(&self.channel);
                self.channel.move_and_publish();
                return;
            }
        };

        key_reader!(
            format!("Could not read container params"),
            json {
                stopOn? => as_u64,
                stopAfter? => as_u64,
                microTaskFlush? => as_u64,
            }
        );

        let params = ContainerParams {
            stopOn: StopOn::from(stopOn.unwrap_or(0)),
            stopAfter: stopAfter.unwrap_or_default(),
            microTaskFlush: Some(microTaskFlush.unwrap_or(1000))
        };

        let current_params = CONTAINER_PARAMS.lock().unwrap().clone();

        if current_params.stopOn != params.stopOn {
            self.channel.add_message(&format!(
                "[Parameter changed] StopOn {} -> {}",
                &Yellow.paint(format!("{}", current_params.stopOn)),
                &Blue.paint(format!("{}", params.stopOn))
            ));
        }

        if current_params.stopAfter != params.stopAfter {
            self.channel.add_message(&format!(
                "[Parameter changed] StopAfter {} ms -> {} ms",
                &Yellow.paint(format!("{}", current_params.stopAfter)),
                &Blue.paint(format!("{}", params.stopAfter))
            ));
        }

        if current_params.microTaskFlush != params.microTaskFlush {
            self.channel.add_message(&format!(
                "[Parameter changed] MicroTaskFlush {} ms -> {} ms",
                &Yellow.paint(format!("{:?}", current_params.microTaskFlush)),
                &Blue.paint(format!("{:?}", params.microTaskFlush))
            ));
        }

        (*CONTAINER_PARAMS.lock().unwrap()) = params;
        self.channel.move_and_publish();
    }

    pub fn load_provider(&mut self, data: &str) -> ParseStatus {
        let mut status = ParseStatus::Empty;
        if !self.registry.provider.is_empty() {
            self.channel.add_warning("A provider has already been loaded, clean the provider first before uploading a new one".into());
            self.channel.move_and_publish();
            return status;
        };

        let json: HashMap<String, Value> = match serde_json::from_str(data) {
            Ok(a) => a,
            Err(_) => {
                self.channel.add_error(&error!(format!("Invalid provider data"), format!("Parse provider")));
                self.registry.clear_all(&self.channel);
                self.channel.move_and_publish();
                return status;
            }
        };

        key_reader!(
            format!("Invalid provider data"),
            json {
                exclude_types? => as_array,
                type_aliases? => as_object,
                filter_operations? => as_object,
                exclude_sections? => as_object,
                filter_operations? => as_object,
                override_return? => as_object,
            }
        );

        match (|| {
            parse_exclude_types(exclude_types, &mut self.registry)?;
            parse_filter_operations(filter_operations, &mut self.registry)?;
            parse_exclude_sections(exclude_sections, &mut self.registry)?;
            parse_type_aliases(type_aliases, &mut self.registry)?;
            parse_return_operations(override_return, &mut self.registry)?;
            Ok(())
        })() {
            Ok(_) => {},
            Err(e) => {
                self.channel.add_error(&e);
                self.registry.clear_all(&self.channel);
                self.channel.move_and_publish();
                return status;
            }
        }

        match parse_provider(&json, &mut self.registry, &self.channel) {
            Ok(_r) =>
                self.channel.add_message(
                    &format!(
                        "Found {} blocks in provider",
                        &Blue.paint(format!("{}", self.registry.provider.len()))
                    )
                ),
            Err(e) => {
                self.channel.add_error(&e);
                self.registry.clear_all(&self.channel);
                self.channel.move_and_publish();
                return status;
            }
        }

        self.channel.add_message("Parsing interfaces...");
        match self.registry.try_build_resources_interfaces(&self.channel) {
            Ok(_r) => {}
            Err(e) => {
                self.channel.add_error(&e);
                self.registry.clear_all(&self.channel);
                self.channel.move_and_publish();
                return status;
            }
        };

        self.channel.add_message("Parsing bodies...");
        match self.registry.try_build_resources_bodies(&self.channel) {
            Ok(_r) => {}
            Err(e) => {
                self.channel.add_error(&e);
                self.registry.clear_all(&self.channel);
                self.channel.move_and_publish();
                return status;
            }
        };
        self.channel.add_message(&Green.paint("Parsing done").to_string());
        status = ParseStatus::Loaded;
        self.registry.swap_pointers_collector_to_resources();
        self.channel.set_parse_provider_status(&status);
        self.channel.move_and_publish();
        set_counter_backup();
        status
    }

    pub fn load_program(&mut self, data: &str) -> ParseStatus {
        let mut status = ParseStatus::Empty;
        if self.registry.provider.is_empty() {
            self.channel.add_warning("No provider found".into());
        };

        if !self.registry.program.is_empty() {
            self.channel.add_warning("A program has already been loaded, clean the program first before uploading a new one".into());
            self.channel.move_and_publish();
            return status;
        };

        let json: HashMap<String, Value> = match serde_json::from_str(data) {
            Ok(a) => a,
            Err(_) => {
                self.channel.add_error(&error!(format!("Invalid user program data"), format!("Parse user program")));
                self.registry.clear_all(&self.channel);
                self.channel.move_and_publish();
                return status;
            }
        };

        key_reader!(
            format!("Invalid user program data"),
            json {
                monitor? => as_array,
                signature? => as_str,
            }
        );

        match parse_program(&json, &mut self.registry, &self.channel) {
            Ok(_r) => self.channel.add_message(
                &format!(
                    "Found {} blocks in program",
                    &Blue.paint(format!("{}", &self.registry.program.len()))
                )
            ),
            Err(e) => {
                self.channel.add_error(&e);
                self.registry.clear_program(&self.channel);
                self.channel.move_and_publish();
                reset_counter_to_backup();
                return status;
            }
        }

        self.channel.add_message("Parsing interfaces...");
        match self.registry.try_build_program_interfaces(&self.channel) {
            Ok(_r) => {}
            Err(e) => {
                self.channel.add_error(&e);
                self.registry.clear_program(&self.channel);
                self.channel.move_and_publish();
                reset_counter_to_backup();
                return status;
            }
        };

        self.channel.add_message("Parsing bodies...");
        match self.registry.try_build_program_bodies(&self.channel) {
            Ok(_) => {}
            Err(e) => {
                self.channel.add_error(&e);
                self.registry.clear_program(&self.channel);
                self.channel.move_and_publish();
                reset_counter_to_backup();
                return status;
            }
        };

        self.channel.add_message(&Green.paint("Parsing done").to_string());

        if let Some(a) = signature {
            self.channel.add_message(a);
        };

        status = ParseStatus::Loaded;
        self.registry.swap_pointers_collector_to_program();
        self.channel.set_parse_program_status(&status);
        self.channel.move_and_publish();

        #[cfg(target_arch = "wasm32")]
        {
            // 6 orders (Stop, Pause, EnableAll, DisableAll)
            // 0 = Empty
            // 1 = Stop
            // 2 = Pause
            // 3 = EnableAll
            // 4 = DisableAll
            // 5 = Enable
            // 6 = Disable
            // + 1 To leave some space for empty
            // Since we always use 2 indexes for each order
            let sab_length =
                8 // min byte value for int32array
                    * (5 // all possible orders + empty
                    + self.channel.breakpoints_len()); // each breakpoint is an individual order
            let sab = js_sys::SharedArrayBuffer::new(sab_length as u32);
            self.runtime_commands_sab = Some(sab);
            self.channel.set_runtime_commands_sab(&self.runtime_commands_sab.as_ref().unwrap());
        }
        status
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_runtime_command_sab(&self) -> Option<SharedArrayBuffer> {
        self.runtime_commands_sab.as_ref().cloned()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn add_plugin(&mut self, name: &str, interval: u32) -> JsValue {
        self.channel.get_dispatcher().add_plugin(interval, name)
    }

    pub async fn start(&mut self, entry: &str) {
        self.channel
            .set_simulation_status(&SimulationStatus::Start);
        self.channel.publish();
        *IS_RUNNING.lock().unwrap() = true;

        let params = &CONTAINER_PARAMS.lock().unwrap().clone();
        let mut sim = Simulation::new(&self.registry, &self.channel, &params);
        self.channel.add_message(
            &format!("--- Starting simulation with [{}] ---",
                     &Purple.paint((&entry).to_string()
                     )));

        let mut eventLoopLastRefreshInterval = Instant::now();
        let stopAfter = match params.stopAfter {
            0 => None,
            _ => Some(Duration::from_millis(params.stopAfter))
        };
        let simulationStartInstant = Instant::now();

        loop {
            if let Some(a) = stopAfter {
                if simulationStartInstant.elapsed() > a {
                    self.channel.add_message(&format!(
                        "Simulation stopped: {}",
                        Blue.paint(&format!("Reached {:?}", a)))
                    );
                    break;
                };
            };

            if let Some(a) = &params.microTaskFlush {
                let elapsed = Instant::now().duration_since(eventLoopLastRefreshInterval);
                if elapsed > Duration::from_millis(*a) {
                    #[cfg(target_arch = "wasm32")]
                    gloo_timers::future::sleep(Duration::from_millis(0)).await;
                    eventLoopLastRefreshInterval = Instant::now();
                }
            }

            #[cfg(target_arch = "wasm32")]
                let mut must_stop = read_sab_commands(&self.channel);
            #[cfg(target_arch = "wasm32")]
            if must_stop {
                self.channel.add_message(&format!(
                    "Simulation stopped: Manual stop"
                ));
                break;
            };

            let earlier = Instant::now();

            match sim.start(entry).await {
                Ok(should_continue) => match should_continue {
                    true => {
                        self.channel.build_monitor(&self.registry);
                        self.channel.move_and_publish();
                    },
                    false => {
                        self.channel.build_monitor(&self.registry);
                        break; 
                    }
                },
                Err(ref e) => {
                    self.channel.add_message(&format!(
                        "Simulation stopped: {}", &Red.paint("Error")
                    ));
                    self.channel.add_error(e);
                    //self.channel.move_and_publish();
                    break;
                }
            }

            let elapsed = earlier.duration_since(Instant::now());

            #[cfg(not(target_arch = "wasm32"))]
            if elapsed < FIVE_MS {
                spin_sleep::sleep(FIVE_MS - elapsed);
                self.channel.add_message(&format!(
                    "--- Cycle [Slowed] ~{} ---",
                    Blue.paint(&format!("{:?}", (elapsed + (FIVE_MS + elapsed))))
                ));
            } else {
                self.channel.add_message(&format!(
                    "--- Cycle ~{} ---",
                    Blue.paint(&format!("{:?}", elapsed))
                ));
            };

            #[cfg(target_arch = "wasm32")]
            #[cfg(not(feature = "node"))]
            if elapsed < FIVE_MS {
                gloo_timers::future::sleep((FIVE_MS - elapsed)).await;
                eventLoopLastRefreshInterval = Instant::now();
                self.channel.add_message(&format!(
                    "--- Cycle [Slowed] ~{} ---",
                    Blue.paint(&format!("{:?}", (elapsed + (FIVE_MS + elapsed))))
                ));
            } else {
                self.channel.add_message(&format!(
                    "--- Cycle ~{} ---",
                    Blue.paint(&format!("{:?}", elapsed))
                ));
            };

            #[cfg(target_arch = "wasm32")]
            #[cfg(feature = "node")]
            if elapsed < FIVE_MS {
                gloo_timers::future::sleep((FIVE_MS - elapsed)).await;
                eventLoopLastRefreshInterval = Instant::now();
                self.channel.add_message(&format!(
                    "--- Cycle [Slowed] ~{} ---",
                    Blue.paint(&format!("{:?}", (elapsed + (FIVE_MS + FOUR_MS + elapsed))))
                ));
            } else {
                self.channel.add_message(&format!(
                    "--- Cycle ~{} ---",
                    Blue.paint(&format!("{:?}", elapsed))
                ));
            };
        }

        *IS_RUNNING.lock().unwrap() = false;
        self.channel.add_message(&Purple.paint("--- End of simulation ---").to_string());

        self.channel.push_cycle_stack();
        self.channel.set_simulation_status(&SimulationStatus::Stop);
        self.channel.move_and_publish();
        self.registry.reset_all(&self.channel)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_runtime_commands_int32(&self) -> Option<js_sys::SharedArrayBuffer> {
        self.runtime_commands_sab.as_ref().cloned()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn read_sab_commands(&self) {
        read_sab_commands(&self.channel);
    }

    pub fn disable_breakpoint(&self, data: u32) {
        self.channel.disable_breakpoint();
        self.channel.add_message(&format!("Disabled breakpoint"));
        self.channel.publish();
    }

    pub fn enable_breakpoint(&self, data: u32) {
        self.channel.activate_breakpoint(data);
        self.channel.add_message(&format!("Enabled breakpoint {}", data));
        self.channel.publish();
    }

    pub fn disable_all_breakpoints(&self) {
        self.channel.clear_breakpoints();
        self.channel.add_message(&format!("Disabled all breakpoints"));
        self.channel.publish();
    }

    pub fn clear_program(&mut self) {
        self.registry.clear_program(&self.channel);
        self.channel.set_simulation_status(&SimulationStatus::Unavailable);
        self.channel.add_message(&Yellow.paint("--- Program reset ---").to_string());
        self.channel.move_and_publish();
        reset_counter_to_backup();
    }

    pub fn clear_provider(&mut self) {
        self.registry.clear_all(&self.channel);
        self.channel.set_simulation_status(&SimulationStatus::Unavailable);
        self.channel.add_message(&Yellow.paint("--- Full reset ---").to_string());
        self.channel.move_and_publish();
        reset_counter_to_default();
    }
}

#[cfg(target_arch = "wasm32")]
pub fn read_sab_commands(channel: &Broadcast) -> bool {
    let mut must_stop = false;
    js_sys::Atomics::store(&channel.get_command_lock_int32(), 0, 1).unwrap();
    if let Some(a) = channel.get_runtime_commands_sab() {
        let vec = a.to_vec();
        let chunks = vec.chunks_exact(2);
        for window in chunks {
            shiftLeft(&a, 1);
            match window[0] {
                0 => { // 0 = Empty
                    a.fill(0, 0, a.length());
                    break;
                }
                1 => { // 1 = Stop
                    must_stop = true;
                    a.fill(0, 0, a.length());
                    break;
                }
                2 => { // 2 = Pause
                    if pause_simulation(channel, None).is_err() {
                        must_stop = true;
                        a.fill(0, 0, a.length());
                        break;
                    }
                }
                3 => { // 3 = Enable all breakpoints
                    //channel.clear_breakpoints();
                    channel.add_message(&"Enabled all breakpoints".to_string());
                    channel.publish();
                }
                4 => { // 4 = Disable all breakpoints
                    channel.clear_breakpoints();
                    channel.add_message(&"Disabled all breakpoints".to_string());
                    channel.publish();
                }
                5 => { // 5 Enable breakpoint
                    if (is_running()) {
                        enableBreakpoint(channel, window[1] as u32);
                        channel.publish();
                    }
                }
                6 => { // 6 Disable breakpoint
                    if (is_running()) {
                        disableBreakpoint(channel, window[1] as u32);
                        channel.publish();
                    }
                }
                _ => {}
            }
        }
    }
    js_sys::Atomics::notify(&channel.get_command_lock_int32(), 0).unwrap();
    js_sys::Atomics::store(&channel.get_command_lock_int32(), 0, 0).unwrap();
    must_stop
}

pub trait Discriminant {
    fn discriminant(&self) -> u8 {
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[repr(u8)]
#[wasm_bindgen]
pub enum ParseStatus {
    Empty,
    Loaded,
}

impl Default for ParseStatus {
    fn default() -> Self {
        Self::Empty
    }
}

impl Discriminant for ParseStatus {}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[repr(u8)]
#[wasm_bindgen]
pub enum SimulationStatus {
    Start,
    Stop,
    Pause,
    Unavailable,
}

impl Default for SimulationStatus {
    fn default() -> Self {
        Self::Unavailable
    }
}

impl Discriminant for SimulationStatus {}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq)]
#[wasm_bindgen]
pub enum StopOn {
    Infinite,
    UnitTestsPassed,
}

impl Discriminant for StopOn {}

impl From<u64> for StopOn {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::Infinite,
            _ => Self::UnitTestsPassed
        }
    }
}

impl Display for StopOn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StopOn::Infinite => write!(f, "Infinite"),
            StopOn::UnitTestsPassed => write!(f, "UnitTestsReached")
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[wasm_bindgen]
pub enum MonitorFormat {
    Array,
    Recursive,
}

impl Discriminant for MonitorFormat{}

#[derive(Deserialize, Serialize, Clone, Tsify)]
#[wasm_bindgen(skip_typescript)]
pub struct ContainerParams {
    #[tsify(optional)]
    pub stopOn: StopOn,
    #[tsify(optional)]
    pub stopAfter: u64,
    #[tsify(optional)]
    pub microTaskFlush: Option<u64>,
}

impl Default for ContainerParams {
    fn default() -> Self {
        Self {
            stopOn: StopOn::UnitTestsPassed,
            stopAfter: 0,
            microTaskFlush: Some(1000),
        }
    }
}
