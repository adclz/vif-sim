use crate::container::broadcast::broadcast::Broadcast;
#[cfg(target_arch = "wasm32")]
use crate::container::broadcast::store::MonitorSchema;
use crate::parser::body::path::parse_path;
use crate::kernel::registry::{convert_string_path_to_usize, get_string, GlobalOrLocal, Kernel};
use serde_json::Value;
use std::ops::DerefMut;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::primitive_traits::ToggleMonitor;


