pub mod dispatcher;

use wasm_bindgen::prelude::*;
use crate::container::broadcast::store::Store;

#[cfg(feature = "node")]
#[wasm_bindgen(module = "/src/js/dispatcher/node_dispatcher.js")]
extern "C" {
    pub type NodeDispatcher;

    #[wasm_bindgen(constructor)]
    pub fn new(server_id: &str) -> NodeDispatcher;

    #[wasm_bindgen(method)]
    pub fn addPlugin(this: &NodeDispatcher, interval: u32, name: &str) -> JsValue;

    #[wasm_bindgen(method)]
    pub fn publishStore(this: &NodeDispatcher, store: Store);
}

#[cfg(not(feature = "node"))]
#[wasm_bindgen(module = "/src/js/dispatcher/browser_dispatcher.js")]
extern "C" {
    pub type BrowserDispatcher;

    #[wasm_bindgen(constructor)]
    pub fn new(server_id: &str) -> BrowserDispatcher;

    #[wasm_bindgen(method)]
    pub fn addPlugin(this: &BrowserDispatcher, interval: u32, name: &str) -> JsValue;

    #[wasm_bindgen(method)]
    pub fn publishStore(this: &BrowserDispatcher, store: Store);
}
