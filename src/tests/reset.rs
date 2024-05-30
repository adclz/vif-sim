#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use core::ops::Deref;
    use serde_json::Value;
    use uuid::Uuid;
    use crate::container::broadcast::broadcast::Broadcast;
    use crate::kernel::plc::types::primitives::traits::family_traits::WithRefFamily;
    use crate::kernel::plc::types::primitives::traits::primitive_traits::PrimitiveTrait;
    use crate::kernel::registry::{get_or_insert_global_string, Kernel};
    use crate::parser::main::program::parse_program;
    use crate::parser::main::provider::parse_provider;

    #[test]
    pub fn reset_provider() {
        let data = r#"
        {
            "file:///MyFb": {
                "ty": "fb",
                "src": {
                    "interface": {
                        "ty": "interface",
                        "src": {
                            "temp": {
                                "test": {
                                    "ty": "Bool",
                                    "src": {
                                        "value": true
                                    }
                                }
                            }
                        }
                    },
                    "body": []
                }
            },
            "file://MyInstance": {
                "ty": "instance_db",
                "src": {
                    "of": "MyFb",
                    "interface": "{}"
                }
            }
        }"#;

        let json: HashMap<String, Value> = serde_json::from_str(data).unwrap();

        let uuid = Uuid::default();
        let mut kernel = Kernel::default();
        let channel = Broadcast::new(&uuid);

        let base = parse_provider(&json, &mut kernel, &channel).unwrap();
        kernel.try_build_resources_interfaces(&channel).unwrap();
        kernel.try_build_resources_bodies(&channel).unwrap();
        kernel.swap_pointers_collector_to_resources();

        let fb = kernel.get(&get_or_insert_global_string(&"MyFb".to_string())).unwrap();
        let static_test = fb.as_ref_fb().unwrap().try_get_nested(&[get_or_insert_global_string(&"test".to_string())]).unwrap();
        assert!(static_test.with_plc_bool(&channel,|a| a.as_bool().unwrap().get(&channel).unwrap()).unwrap());

        assert!(!kernel.provider.is_empty());
        assert!(!kernel.provider_raw_pointers.borrow().deref().is_empty());

        kernel.clear_all(&channel);

        assert!(kernel.provider.is_empty());
        assert!(kernel.provider_raw_pointers.borrow().deref().is_empty());
    }
    #[test]
    pub fn reset_program() {
        let data = r#"
        {
            "file:///MyFb": {
                "ty": "fb",
                "src": {
                    "interface": {
                        "ty": "interface",
                        "src": {
                            "temp": {
                                "test": {
                                    "ty": "Bool",
                                    "src": {
                                        "value": true
                                    }
                                }
                            }
                        }
                    },
                    "body": []
                }
            },
            "file://MyInstance": {
                "ty": "instance_db",
                "src": {
                    "of": "MyFb",
                    "interface": "{}"
                }
            }
        }"#;

        let json: HashMap<String, Value> = serde_json::from_str(data).unwrap();

        let uuid = Uuid::default();
        let mut kernel = Kernel::default();
        let channel = Broadcast::new(&uuid);

        let base = parse_program(&json, &mut kernel, &channel).unwrap();
        kernel.try_build_program_interfaces(&channel).unwrap();
        kernel.try_build_program_bodies(&channel).unwrap();
        kernel.swap_pointers_collector_to_program();

        let fb = kernel.get(&get_or_insert_global_string(&"MyFb".to_string())).unwrap();
        let static_test = fb.as_ref_fb().unwrap().try_get_nested(&[get_or_insert_global_string(&"test".to_string())]).unwrap();
        assert!(static_test.with_plc_bool(&channel,|a| a.as_bool().unwrap().get(&channel).unwrap()).unwrap());

        assert!(!kernel.program.is_empty());
        assert!(!kernel.program_raw_pointers.borrow().deref().is_empty());

        kernel.clear_program(&channel);
        
        assert!(kernel.program.is_empty());
        assert!(kernel.program_raw_pointers.borrow().deref().is_empty());
    }
}
