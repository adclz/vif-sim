use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tsify::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Tsify, Clone, Serialize, Deserialize, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FileTrace {
    file: String,
    column: u64,
    line: u64,
}

impl FileTrace {
    pub fn get_filename(&self) -> &str {
        &self.file
    }

    pub fn get_column(&self) -> &u64 {
        &self.column
    }

    pub fn get_line(&self) -> &u64 {
        &self.line
    }
}

pub trait FileTraceBuilder {
    fn build_trace(json: &Map<String, Value>) -> Option<FileTrace> {
        let filename = json["file"].as_str().unwrap();
        let column = json["column"].as_u64().unwrap();
        let line = json["line"].as_u64().unwrap();

        Some(FileTrace {
            file: filename.to_string(),
            column,
            line,
        })
    }
    fn get_trace(&self) -> &Option<FileTrace>;
}
