#[cfg(target_arch = "wasm32")]
use crate::js::dispatcher::dispatcher::Dispatcher;
use crate::container::broadcast::stack::Stack;
use crate::container::error::error::Stop;
use crate::container::container::{ParseStatus, SimulationStatus};
use core::cell::RefCell;
use std::collections::{HashMap, HashSet};
use core::ops::{Deref, DerefMut};
use std::rc::Rc;
use js_sys::{Int32Array, SharedArrayBuffer};
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
use crate::container::broadcast::store::{MonitorChange, MonitorSchema, Store};
use crate::kernel::plc::operations::unit::test::{UnitTest, UnitTestStatus, UnitTestUpdateStatus};
use crate::kernel::registry::Kernel;

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

    unit_tests: Rc<RefCell<HashMap<u32, UnitTest>>>,
    breakpoints: Rc<RefCell<HashSet<u32>>>,

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
            breakpoints: Rc::new(RefCell::new(HashSet::new())),

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
            breakpoints: Rc::new(RefCell::new(HashSet::new())),
            
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

    /*pub fn add_breakpoint_status(&self, breakpoint: &BreakPointUpdateStatus) {
        self.store.borrow_mut().add_breakpoint_status(breakpoint);
        self.breakpoints.borrow_mut().get_mut(&breakpoint.get_id()).unwrap().set_status(breakpoint.get_status())
    }*/

    /// Checks if a breakpoint is enabled
    pub fn is_breakpoint_enabled(&self, id: u32) -> bool {
        self.breakpoints.borrow().deref().contains(&id)
    }

    // Adds a breakpoint (set from outside) 
    pub fn add_breakpoint(&self, id: u32) {
        self.breakpoints.borrow_mut().insert(id);
        self.store.borrow_mut().activate_breakpoint(id);
    }
    
    pub fn build_monitor(&self, kernel: &Kernel) {
        self.store.borrow_mut().build_monitor(kernel);
    }
    
    /// Removes a breakpoint (set from outside)
    pub fn remove_breakpoint(&self, id: u32) {
        self.breakpoints.borrow_mut().remove(&id);
        self.store.borrow_mut().disable_breakpoint()
    }
    
    /// Activate breakpoint
    pub fn activate_breakpoint(&self, id: u32) {
        self.store.borrow_mut().activate_breakpoint(id);
    }

    /// Disable breakpoint
    pub fn disable_breakpoint(&self) {
        self.store.borrow_mut().disable_breakpoint();
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
        self.breakpoints.borrow_mut().clear()
    }

    pub fn breakpoints_len(&self) -> usize {
        self.breakpoints.borrow_mut().len()
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
