use js_sys::Int32Array;
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "node")]
#[wasm_bindgen(module = "/src/js/typed_array/node_shift.js")]
extern "C" {
    pub fn shiftRight(collection: &Int32Array, steps: u32) -> Int32Array;
    pub fn shiftLeft(collection: &Int32Array, steps: u32) -> Int32Array;
}

#[cfg(not(feature = "node"))]
#[wasm_bindgen(module = "/src/js/typed_array/browser_shift.js")]
extern "C" {
    pub fn shiftRight(collection: &Int32Array, steps: u32) -> Int32Array;
    pub fn shiftLeft(collection: &Int32Array, steps: u32) -> Int32Array;
}