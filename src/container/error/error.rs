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
    id_stack: Vec<u32>,
    sim_stack: Vec<String>,
}

impl PartialEq for Stop {
    fn eq(&self, other: &Self) -> bool {
        self.error == other.error
    }
}

impl Serialize for Stop {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut state = serializer.serialize_struct("error", 3)?;
        state.serialize_field("error", &self.error)?;
        state.serialize_field("id_stack", &self.id_stack)?;
        state.serialize_field("sim_stack", &self.sim_stack)?;
        state.end()
    }
}

impl Display for Stop {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\x1B[31mError: {}\x1b[0m", self.error)?;
        writeln!(f, "SimStack")?;
        self.sim_stack
            .iter()
            .try_for_each(|s| writeln!(f, "at - {}", s))?;
        writeln!(f, "FileStack")?;
        self.id_stack.iter().try_for_each(|s| {
            writeln!(
                f,
                "at - {}",
                s
            )
        })
    }
}

impl Stop {
    pub fn serialize(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self).unwrap()
    }
    pub fn new(message: String, sim_trace: &Option<String>, id: Option<u32>) -> Self {
        let mut id_vec = Vec::new();
        if id.is_some() {
            id_vec.push(*id.as_ref().unwrap());
        }

        let mut sim_stack_vec = Vec::new();
        if sim_trace.is_some() {
            sim_stack_vec.push(sim_trace.as_ref().unwrap().clone());
        }

        let err = Self {
            error: message,
            id_stack: id_vec,
            sim_stack: sim_stack_vec
        };
        
        /*#[cfg(not(target_arch = "wasm32-unknown-unknown"))]
        panic!("{}", err);*/
        err
    }

    pub fn add_sim_trace(mut self, stack: &str) -> Self {
        self.sim_stack.push(stack.into());
        self
    }

    pub fn add_id(mut self, id: u32) -> Self {
        self.id_stack.push(id);
        self
    }
}
