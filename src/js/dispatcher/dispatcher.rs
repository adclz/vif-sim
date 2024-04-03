#[cfg(not(feature = "node"))]
use crate::js::dispatcher::BrowserDispatcher;
#[cfg(feature = "node")]
use crate::js::dispatcher::NodeDispatcher;
use wasm_bindgen::JsValue;
use crate::container::broadcast::store::Store;
use crate::container::container::SimulationStatus;
use crate::container::error::error::Stop;

pub enum Dispatcher {
    #[cfg(feature = "node")]
    Node(NodeDispatcher),
    #[cfg(not(feature = "node"))]
    Browser(BrowserDispatcher),
}

impl Dispatcher {
    pub fn new(server_id: &str) -> Self {
        #[cfg(feature = "node")]
        return Self::Node(NodeDispatcher::new(server_id));

        #[cfg(not(feature = "node"))]
        return Self::Browser(BrowserDispatcher::new(server_id));
    }

    pub fn add_plugin(&self, interval: u32, name: &str) -> JsValue {
        match self {
            #[cfg(feature = "node")]
            Dispatcher::Node(a) => a.addPlugin(interval, name),
            #[cfg(not(feature = "node"))]
            Dispatcher::Browser(a) => a.addPlugin(interval, name),
        }
    }

    pub fn publish_store(&self, store: Store) {
        match self {
            #[cfg(feature = "node")]
            Dispatcher::Node(a) => a.publishStore(store),
            #[cfg(not(feature = "node"))]
            Dispatcher::Browser(a) => a.publishStore(store),
        }
    }
}
