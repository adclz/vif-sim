use crate::container::broadcast::broadcast::Broadcast;
#[cfg(target_arch = "wasm32")]
use crate::container::broadcast::store::MonitorSchema;
use crate::parser::body::path::parse_path;
use crate::kernel::registry::{convert_string_path_to_usize, get_string, GlobalOrLocal, Kernel};
use serde_json::Value;
use std::ops::DerefMut;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::primitive_traits::ToggleMonitor;

pub fn parse_monitor(monitor: &Vec<Value>, channel: &Broadcast, registry: &mut Kernel) {
    monitor.iter().for_each(|x| {
        let path =
            &convert_string_path_to_usize(
            &parse_path(x)
            .map_err(|e| channel.add_warning(&format!("[Monitor] Invalid path {:?}", x)))
            .unwrap_or_default());


        match registry.get_and_find_nested(&path) {
            None => channel.add_warning(&format!("[Monitor] Could not find variable: {:?}", path)),
            Some(a) => match a {
                GlobalOrLocal::Global(a) => {
                    if let Ok(b) = a.as_ref_db() {
                        b.get_pointers_with_path(&path, &path).iter().for_each(|x| {
                            #[cfg(target_arch = "wasm32")]
                            {
                                x.0.0.as_ref().borrow_mut().deref_mut().set_monitor(true);
                                channel.add_monitor_schema(&MonitorSchema::new(
                                    x.0.1.iter().map(|x| get_string(*x)).collect(),
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
                            path.iter().map(|x| get_string(*x)).collect(),
                            serde_wasm_bindgen::to_value(&a).unwrap(),
                        ))
                    }
                }
            },
        }
    });
}

pub fn parse_set(monitor: &Vec<Value>, channel: &Broadcast, registry: &mut Kernel) -> Result<(), Stop> {
    monitor.iter().try_for_each(|x| {
        let path =
            &convert_string_path_to_usize(
                &parse_path(x)
                    .map_err(|e| channel.add_warning(&format!("[Set] Invalid path {:?}", x)))
                    .unwrap_or_default());
        
        match registry.get_and_find_nested(&path) {
            None => { channel.add_warning(&format!("[Set] Could not find variable: {:?}", path)); Ok(()) },
            Some(a) => match a {
                GlobalOrLocal::Global(a) => {
                    Ok(())
                }
                GlobalOrLocal::Local(a) => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        a.as_ref().borrow_mut().deref_mut().set_monitor(true);
                        channel.add_monitor_schema(&MonitorSchema::new(
                            path.iter().map(|x| get_string(*x)).collect(),
                            serde_wasm_bindgen::to_value(&a).unwrap(),
                        ))
                    }
                    Ok(())
                }
            },
        }
    })?;
    Ok(())
}
