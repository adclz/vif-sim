#[cfg(target_arch = "wasm32")]
use crate::js::dispatcher::dispatcher::Dispatcher;
use crate::container::broadcast::stack::Stack;
use crate::container::error::error::Stop;
use crate::container::container::{ParseStatus, SimulationStatus};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use js_sys::{Int32Array, SharedArrayBuffer};
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
use crate::container::broadcast::store::{MonitorChange, MonitorSchema, Store};
use crate::kernel::plc::operations::unit::breakpoint::{BreakPoint, BreakPointStatus, BreakPointUpdateStatus};
use crate::kernel::plc::operations::unit::test::{UnitTest, UnitTestStatus, UnitTestUpdateStatus};

pub struct Broadcast {
    #[cfg(target_arch = "wasm32")]
    dispatcher: Dispatcher,

    #[cfg(target_arch = "wasm32")]
    pause_int32: js_sys::Int32Array,

    #[cfg(target_arch = "wasm32")]
    command_lock_int32: js_sys::Int32Array,

    #[cfg(target_arch = "wasm32")]
    runtime_commands_int32: Option<js_sys::Int32Array>,

    store: Rc<RefCell<Store>>,

    unit_tests: Rc<RefCell<HashMap<usize, UnitTest>>>,
    breakpoints: Rc<RefCell<HashMap<usize, BreakPoint>>>,

    stack: Rc<RefCell<Stack>>,
}


impl Broadcast {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(id: &Uuid) -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            dispatcher: Dispatcher::new(&id.to_string()),

            store: Rc::new(RefCell::new(Store::default())),

            unit_tests: Rc::new(RefCell::new(HashMap::new())),
            breakpoints: Rc::new(RefCell::new(HashMap::new())),

            stack: Rc::new(RefCell::new(Stack::new())),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(id: &Uuid, pause_sab: &js_sys::SharedArrayBuffer, command_lock_sab: &js_sys::SharedArrayBuffer) -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            dispatcher: Dispatcher::new(&id.to_string()),

            #[cfg(target_arch = "wasm32")]
            command_lock_int32: Int32Array::new(&command_lock_sab),

            #[cfg(target_arch = "wasm32")]
            pause_int32: Int32Array::new(&pause_sab),

            #[cfg(target_arch = "wasm32")]
            runtime_commands_int32: None,

            store: Rc::new(RefCell::new(Store::default())),

            unit_tests: Rc::new(RefCell::new(HashMap::new())),
            breakpoints: Rc::new(RefCell::new(HashMap::new())),

            stack: Rc::new(RefCell::new(Stack::new())),
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn set_runtime_commands_sab(&mut self, sab: &SharedArrayBuffer) {
        self.runtime_commands_int32 = Some(js_sys::Int32Array::new(sab))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_runtime_commands_sab(&self) -> &Option<Int32Array> {
        &self.runtime_commands_int32
    }
    
    #[cfg(target_arch = "wasm32")]
    pub fn get_pause_int32(&self) -> &Int32Array {
        &self.pause_int32
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_command_lock_int32(&self) -> &Int32Array {
        &self.command_lock_int32
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_dispatcher(&self) -> &Dispatcher {
        &self.dispatcher
    }

    pub fn move_and_publish(&self) {
        let store = self.store.borrow_mut().move_and_reset();
        #[cfg(target_arch = "wasm32")]
        self.dispatcher.publish_store(store)
    }

    pub fn publish(&self) {
        let store = self.store.borrow_mut().clone();
        #[cfg(target_arch = "wasm32")]
        self.dispatcher.publish_store(store)
    }

    pub fn add_message(&self, message: &str) {
        self.store.borrow_mut().add_message(message);
        println!("{} \n", message);
    }

    pub fn add_warning(&self, warning: &str) {
        self.store.borrow_mut().add_warning(warning);
        println!("WARNING: {} \n", warning);
    }

    pub fn add_error(&self, error: &Stop) {
        self.store.borrow_mut().add_error(error);
        println!("ERROR: {} \n", error);
        #[cfg(not(target_arch = "wasm32"))]
        panic!("{}", error)
    }

    pub fn add_breakpoint(&self, breakpoint: &BreakPoint) {
        self.store.borrow_mut().add_breakpoint(breakpoint);
        self.breakpoints.borrow_mut().insert(breakpoint.get_id(), breakpoint.clone());
    }

    pub fn add_breakpoint_status(&self, breakpoint: &BreakPointUpdateStatus) {
        self.store.borrow_mut().add_breakpoint_status(breakpoint);
        self.breakpoints.borrow_mut().get_mut(&breakpoint.get_id()).unwrap().set_status(breakpoint.get_status())
    }

    pub fn is_breakpoint_enabled(&self, id: usize) -> bool {
        !matches!(self.breakpoints.borrow().deref().get(&id).unwrap().get_status(), BreakPointStatus::Disabled)
    }

    pub fn add_monitor_schema(&self, schema: &MonitorSchema) {
        self.store.borrow_mut().add_monitor_schema(schema)
    }

    pub fn add_monitor_change(&self, schema: &MonitorChange) {
        self.store.borrow_mut().add_monitor_change(schema)
    }

    pub fn set_simulation_status(&self, status: &SimulationStatus) {
        self.store.borrow_mut().set_simulation_status(status)
    }

    pub fn set_parse_provider_status(&self, status: &ParseStatus) {
        self.store.borrow_mut().set_parse_provider_status(status)
    }

    pub fn set_parse_program_status(&self, status: &ParseStatus) {
        self.store.borrow_mut().set_parse_program_status(status)
    }

    pub fn add_entry_point(&self, entry: &str) {
        self.store.borrow_mut().add_entry_point(entry)
    }

    pub fn clear_entry_points(&self) {
        self.store.borrow_mut().clear_entry_points();
    }

    pub fn add_unit_test(&self, location: &UnitTest) {
        self.store.borrow_mut().add_unit_test(location);
        self.unit_tests.borrow_mut().insert(location.get_id(), location.clone());
    }

    pub fn get_unit_tests(&self) -> Vec<UnitTest> {
        self.unit_tests.borrow_mut().deref_mut().iter().map(|x| x.1.clone()).collect()
    }

    pub fn add_unit_test_status(&self, status: &UnitTestUpdateStatus) {
        self.store.borrow_mut().add_unit_test_status(status);
        self.unit_tests.borrow_mut().get_mut(&status.get_id()).unwrap().set_status(status.get_status());
    }

    pub fn clear_unit_tests(&self) {
        *self.unit_tests.borrow_mut().deref_mut() = HashMap::new()
    }

    pub fn reset_unit_tests(&self) {
        self.unit_tests.borrow_mut().iter_mut().for_each(|a| a.1.set_status(UnitTestStatus::Unreached))
    }

    pub fn clear_breakpoints(&self) {
        *self.breakpoints.borrow_mut().deref_mut() = HashMap::new();
    }

    pub fn breakpoints_len(&self) -> usize {
        self.breakpoints.borrow_mut().len()
    }

    pub fn reset_breakpoints(&self) {
        let mut breakpoints_ids = vec!();
        self.breakpoints
            .borrow_mut()
            .deref_mut()
            .iter_mut()
            .for_each(|x| {
                if let BreakPointStatus::Disabled = x.1.get_status() { return; }
                breakpoints_ids.push(*x.0);
            });
        breakpoints_ids.iter().for_each(|x| {
            self.add_breakpoint_status(&BreakPointUpdateStatus::new(
                *x,
                BreakPointStatus::Inactive,
            ))
        })
    }

    pub fn enable_all_breakpoints(&self) {
        let mut breakpoints_ids = vec!();
        self.breakpoints
            .borrow_mut()
            .deref_mut()
            .iter_mut()
            .for_each(|x| {
                breakpoints_ids.push(*x.0)
            });
        breakpoints_ids.iter().for_each(|x| {
            self.add_breakpoint_status(&BreakPointUpdateStatus::new(
                *x,
                BreakPointStatus::Inactive,
            ))
        })
    }

    pub fn disable_all_breakpoints(&self) {
        let mut breakpoints_ids = vec!();
        self.breakpoints
            .borrow_mut()
            .deref_mut()
            .iter_mut()
            .for_each(|x| {
                breakpoints_ids.push(*x.0)
            });
        breakpoints_ids.iter().for_each(|x| {
            self.add_breakpoint_status(&BreakPointUpdateStatus::new(
                *x,
                BreakPointStatus::Disabled,
            ))
        })
    }

    pub fn push_cycle_stack(&self) {
        self.store.borrow_mut().set_stack(self.stack.borrow().deref().clone());
    }

    pub fn get_cycle_stack(&self) -> &Rc<RefCell<Stack>> {
        &self.stack
    }

    pub fn reset_cycle_stack(&self) {
        self.stack.borrow_mut().clear();
    }
}
