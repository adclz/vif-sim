use crate::{error, key_reader};
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::kernel::plc::interface::status::{BodyStatus, InterfaceStatus};
use crate::kernel::plc::interface::traits::DeferredBuilder;
use crate::kernel::registry::Kernel;
use crate::container::error::error::Stop;
use serde_json::{Map, Value};
use crate::container::broadcast::broadcast::Broadcast;

pub struct Template {
    json: Map<String, Value>,
    interface_status: InterfaceStatus,
    body_status: BodyStatus,
    body: Vec<Value>,
    trace: Option<FileTrace>,
}

impl FileTraceBuilder for Template {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl DeferredBuilder for Template {
    fn default(json: &Map<String, Value>) -> Self {
        let mut trace = None;
        if json.contains_key("trace") {
            trace = Self::build_trace(json["trace"].as_object().unwrap());
        }

        Self {
            json: json.clone(),
            interface_status: InterfaceStatus::Default,
            body_status: BodyStatus::Default,
            body: Vec::new(),
            trace,
        }
    }

    fn build_interface(&mut self, _registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        self.interface_status = InterfaceStatus::Solved;
        Ok(())
    }

    fn build_body(&mut self, _registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        self.body_status = BodyStatus::Pending;
        let data = &self.json;

        key_reader!(
            format!("Build Template -> Parse Template body"),
            data {
                body => as_array,
            }
        );

        self.body = body.clone();

        self.body_status = BodyStatus::Solved;
        Ok(())
    }

    fn get_interface_status(&self) -> InterfaceStatus {
        self.interface_status
    }

    fn get_body_status(&self) -> BodyStatus {
        self.body_status
    }
}

impl Template {
    pub fn get_body(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<&Vec<Value>, Stop> {
        match self.body_status {
            BodyStatus::Default => {
                self.build_body(registry, channel)?;
                Ok(&self.body)
            }
            BodyStatus::Pending => {
                // Template cannot depend on other templates
                Err(error!(format!("Recursive template")))
            }
            BodyStatus::Solved => Ok(&self.body),
        }
    }
}
