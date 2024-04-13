use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::container::container::{ParseStatus, SimulationStatus};
use crate::container::error::error::Stop;
use camelpaste::paste;
use tsify::Tsify;
use crate::container::broadcast::log_stack::Stack;
use crate::kernel::plc::operations::unit::breakpoint::{BreakPoint, BreakPointUpdateStatus};
use crate::kernel::plc::operations::unit::test::{UnitTest, UnitTestUpdateStatus};

#[derive(Tsify)]
#[wasm_bindgen(skip_typescript)]
#[derive(Clone)]
pub struct MonitorSchema {
    path: Vec<String>,
    #[tsify(type = "{id: number, value: string}")]
    value: JsValue,
}

#[wasm_bindgen]
impl MonitorSchema {
    pub fn new(path: Vec<String>, value: JsValue) -> Self {
        Self {
            path,
            value,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn get_path(&self) -> Vec<String> {
        self.path.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn get_value(&self) -> JsValue {
        self.value.clone()
    }
}

#[derive(Tsify)]
#[wasm_bindgen(skip_typescript)]
#[derive(Clone)]
pub struct MonitorChange {
    id: usize,
    value: String,
}

#[wasm_bindgen]
impl MonitorChange {
    pub fn new(id: usize, value: String) -> Self {
        Self {
            id,
            value,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn get_id(&self) -> usize {
        self.id
    }

    #[wasm_bindgen(getter)]
    pub fn get_value(&self) -> String {
        self.value.clone()
    }
}

macro_rules! impl_store {
    ($($field: ident => $value: ty),+) => {
        #[derive(Default, Clone)]
        #[wasm_bindgen(skip_typescript)]
        pub struct Store {
            $($field: $value),+,
        }
    };
}

macro_rules! impl_take {
    ($($field: ident => $value: ty),+) => {
        paste! {
            #[wasm_bindgen(skip_typescript)]
            impl Store {
            $(#[wasm_bindgen(getter)]
                pub fn [<get_$field>](&mut self) -> $value {
                    std::mem::take(&mut self.$field)
                }
            )+}
        }
    }
}

macro_rules! impl_serialize {
    ($($field: ident => $value: ty),+) => {
        paste! {
            #[wasm_bindgen(skip_typescript)]
            impl Store {
            $(
                #[wasm_bindgen(getter)]
                pub fn [<get_$field>](&mut self) -> $value {
                    match &self.$field {
                        Some(a) => a.serialize(),
                        None => JsValue::null()
                    }
                }
            )+}
        }
    }
}

impl_store!(
    stack => Option<Stack>,
    messages => Option<Vec<String>>,
    warnings => Option<Vec<String>>,
    error => Option<Stop>,
    monitor_schemas => Option<Vec<MonitorSchema>>,
    monitor_changes => Option<Vec<MonitorChange>>,
    breakpoints => Option<Vec<BreakPoint>>,
    breakpoints_statuses => Option<Vec<BreakPointUpdateStatus>>,
    unit_tests => Option<Vec<UnitTest>>,
    unit_tests_statuses => Option<Vec<UnitTestUpdateStatus>>,
    entry_points => Option<Vec<String>>,
    simulation_status => Option<SimulationStatus>,
    parse_provider_status => Option<ParseStatus>,
    parse_program_status => Option<ParseStatus>
);

impl_serialize!(
    stack => JsValue,
    error => JsValue
);

impl_take!(
    messages => Option<Vec<String>>,
    warnings => Option<Vec<String>>,
    monitor_schemas => Option<Vec<MonitorSchema>>,
    monitor_changes => Option<Vec<MonitorChange>>,
    breakpoints => Option<Vec<BreakPoint>>,
    breakpoints_statuses => Option<Vec<BreakPointUpdateStatus>>,
    unit_tests => Option<Vec<UnitTest>>,
    unit_tests_statuses => Option<Vec<UnitTestUpdateStatus>>,
    entry_points => Option<Vec<String>>,
    simulation_status => Option<SimulationStatus>,
    parse_provider_status => Option<ParseStatus>,
    parse_program_status => Option<ParseStatus>
);

impl Store {
    pub fn move_and_reset(&mut self) -> Store {
        std::mem::take(self)
    }

    pub fn reset_store(&mut self) {
        *self = std::mem::take(&mut Store::default());
    }

    pub fn add_message(&mut self, message: &str) {
        self.messages.get_or_insert_with(Vec::new).push(message.into());
    }

    pub fn add_warning(&mut self, warning: &str) {
        self.warnings.get_or_insert_with(Vec::new).push(warning.into());
    }

    pub fn add_error(&mut self, error: &Stop) {
        self.error = Some(error.clone())
    }

    pub fn add_monitor_schema(&mut self, schema: &MonitorSchema) {
        self.monitor_schemas.get_or_insert_with(Vec::new).push(schema.clone());
    }

    pub fn add_monitor_change(&mut self, change: &MonitorChange) {
        self.monitor_changes.get_or_insert_with(Vec::new).push(change.clone());
    }

    pub fn add_breakpoint(&mut self, location: &BreakPoint) {
        self.breakpoints.get_or_insert_with(Vec::new).push(location.clone());
    }

    pub fn add_breakpoint_status(&mut self, status: &BreakPointUpdateStatus) {
        self.breakpoints_statuses.get_or_insert_with(Vec::new).push(status.clone());
    }

    pub fn add_unit_test(&mut self, location: &UnitTest) {
        self.unit_tests.get_or_insert_with(Vec::new).push(location.clone());
    }

    pub fn add_unit_test_status(&mut self, status: &UnitTestUpdateStatus) {
        self.unit_tests_statuses.get_or_insert_with(Vec::new).push(status.clone());
    }

    pub fn set_simulation_status(&mut self, status: &SimulationStatus) {
        self.simulation_status = Some(status.clone())
    }

    pub fn set_parse_provider_status(&mut self, status: &ParseStatus) {
        self.parse_provider_status = Some(status.clone())
    }

    pub fn set_parse_program_status(&mut self, status: &ParseStatus) {
        self.parse_program_status = Some(status.clone())
    }

    pub fn add_entry_point(&mut self, point: &str) {
        self.entry_points.get_or_insert_with(Vec::new).push(point.into());
    }

    pub fn clear_entry_points(&mut self) {
        self.entry_points = None;
    }

    pub fn set_stack(&mut self, stack: Stack) {
        self.stack = Some(stack)
    }
}
