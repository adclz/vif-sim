﻿use crate::container::broadcast::broadcast::Broadcast;
#[cfg(target_arch = "wasm32")]
use crate::container::broadcast::store::MonitorSchema;
use crate::parser::body::path::parse_path;
use crate::registry::registry::{GlobalOrLocal, Kernel};
use serde_json::Value;
use std::ops::DerefMut;
use crate::plc::primitives::family_traits::ToggleMonitor;

pub fn parse_monitor(monitor: &Vec<Value>, channel: &Broadcast, registry: &mut Kernel) {
    monitor.iter().for_each(|x| {
        let path = parse_path(x)
            .map_err(|e| channel.add_warning(&format!("[Monitor] Invalid path {:?}", x)))
            .unwrap_or_default();

        match registry.get_and_find_nested(&path) {
            None => channel.add_warning(&format!("[Monitor] Could not find variable: {:?}", path)),
            Some(a) => match a {
                GlobalOrLocal::Global(a) => {
                    if let Ok(b) = a.as_ref_db() {
                        b.get_pointers_with_path(&path, &path).iter().for_each(|x| {
                            #[cfg(target_arch = "wasm32")]
                            {
                                x.0 .0.as_ref().borrow_mut().deref_mut().set_monitor(true);
                                channel.add_monitor_schema(&MonitorSchema::new(
                                    x.0 .1.clone(),
                                    serde_wasm_bindgen::to_value(&x.0 .0).unwrap(),
                                ))
                            }
                        })
                    } else {
                        channel.add_warning(&format!(
                            "[Monitor] Global type is not of type DataBlock {:?}",
                            path
                        ))
                    };
                }
                GlobalOrLocal::Local(a) => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        a.as_ref().borrow_mut().deref_mut().set_monitor(true);
                        channel.add_monitor_schema(&MonitorSchema::new(
                            path,
                            serde_wasm_bindgen::to_value(&a).unwrap(),
                        ))
                    }
                }
            },
        }
    });
}