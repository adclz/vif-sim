use crate::parser::trace::trace::FileTrace;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::fmt::{Display, Formatter};
use tsify::Tsify;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Tsify)]
#[derive(Clone, Debug)]
#[wasm_bindgen(skip_typescript)]
pub struct Stop {
    error: String,
    file_stack: Vec<FileTrace>,
    sim_stack: Vec<String>,
}

impl Display for Stop {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\x1B[31mError: {}\x1b[0m", self.error)?;
        writeln!(f, "SimStack")?;
        self.sim_stack
            .iter()
            .try_for_each(|s| writeln!(f, "at - {}", s))?;
        writeln!(f, "FileStack")?;
        self.file_stack.iter().try_for_each(|s| {
            writeln!(
                f,
                "at - {}:{}:{}",
                s.get_filename(),
                s.get_line(),
                s.get_column()
            )
        })
    }
}

impl Serialize for Stop {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("error", 3)?;
        state.serialize_field("error", &self.error)?;
        state.serialize_field("file_stack", &self.file_stack)?;
        state.serialize_field("sim_stack", &self.sim_stack)?;
        state.end()
    }
}

impl Stop {
    pub fn serialize(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self).unwrap()
    }
    pub fn new(message: String, sim_trace: &Option<String>, trace: &Option<FileTrace>) -> Self {
        let mut trace_vec = Vec::new();
        if trace.is_some() {
            trace_vec.push(trace.as_ref().unwrap().clone());
        }

        let mut sim_stack_vec = Vec::new();
        if sim_trace.is_some() {
            sim_stack_vec.push(sim_trace.as_ref().unwrap().clone());
        }

//        #[cfg(not(target_arch = "wasm32"))]
//        panic!("{}", message);

        Self {
            error: message,
            file_stack: trace_vec,
            sim_stack: sim_stack_vec
        }
    }

    pub fn add_sim_trace(mut self, stack: &str) -> Self {
        self.sim_stack.push(stack.into());
        self
    }

    pub fn add_file_trace(mut self, stack: &FileTrace) -> Self {
        self.file_stack.push(stack.clone());
        self
    }

    pub fn maybe_file_trace(mut self, stack: &Option<FileTrace>) -> Self {
        match stack {
            None => {}
            Some(a) => self.file_stack.push(a.clone()),
        }
        self
    }
}
