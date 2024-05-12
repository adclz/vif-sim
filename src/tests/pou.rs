#[cfg(test)]
mod tests {
    use crate::kernel::registry::{get_or_insert_global_string, get_string, Kernel};
    use crate::container::broadcast::broadcast::Broadcast;
    use crate::parser::main::program::parse_program;
    use uuid::Uuid;
    use crate::kernel::plc::interface::traits::InterfaceAccessors;
    use crate::kernel::plc::types::primitives::traits::family_traits::WithRefFamily;
    use crate::kernel::plc::types::primitives::traits::primitive_traits::PrimitiveTrait;
    use crate::container::error::error::Stop;

    #[test]
    fn parse_ob_interface() {
        let data = r#"
        {
            "file:///MyOb": {
                "ty": "ob",
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
            }
        }"#;

        let uuid = Uuid::default();
        let mut kernel = Kernel::default();
        let channel = Broadcast::new(&uuid);
        let base = parse_program(&serde_json::from_str(data).unwrap(), &mut kernel, &channel).unwrap();
        kernel.try_build_program_interfaces(&channel).unwrap();
        kernel.try_build_program_bodies(&channel).unwrap();

        let ob = kernel.get(&get_or_insert_global_string(&"MyOb".to_string())).unwrap();
    }

    #[test]
    fn parse_fb_interface() {
        let data = r#"
        {
            "file:///MyFb": {
                "ty": "fb",
                "src": {
                    "interface": {
                        "ty": "interface",
                        "src": {
                            "static": {
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
            }
        }"#;

        let uuid = Uuid::default();
        let mut kernel = Kernel::default();
        let channel = Broadcast::new(&uuid);
        let base = parse_program(&serde_json::from_str(data).unwrap(), &mut kernel, &channel).unwrap();
        kernel.try_build_program_interfaces(&channel).unwrap();
        kernel.try_build_program_bodies(&channel).unwrap();

        let fb = kernel.get(&get_or_insert_global_string(&"MyFb".to_string())).unwrap();
        let static_test = fb.as_ref_fb().unwrap().try_get_nested(&[get_or_insert_global_string(&"test".to_string())]).unwrap();
        assert!(static_test.with_plc_bool(&channel,|a| a.as_bool().unwrap().get(&channel).unwrap()).unwrap());
    }

    #[test]
    fn parse_fc_interface() {
        let data = r#"
        {
            "file:///MyFc": {
                "ty": "fc",
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
            }
        }"#;

        let uuid = Uuid::default();
        let mut kernel = Kernel::default();
        let channel = Broadcast::new(&uuid);
        let base = parse_program(&serde_json::from_str(data).unwrap(), &mut kernel, &channel).unwrap();
        kernel.try_build_program_interfaces(&channel).unwrap();
        kernel.try_build_program_bodies(&channel).unwrap();

        let fc = kernel.get(&get_or_insert_global_string(&"MyFc".to_string())).unwrap();
        let static_test = fc.as_ref_fc().unwrap().try_get_nested(&[get_or_insert_global_string(&"test".to_string())]).unwrap();
        assert!(static_test.with_plc_bool(&channel,|a| a.as_bool().unwrap().get(&channel).unwrap()).unwrap());
    }

    #[test]
    fn parse_fb_interface_invalid() {
        let data = r#"
        {
            "file:///MyFb": {
                "ty": "fb",
                "src": {
                    "interface": {
                        "ty": "interface",
                        "src": {
                            "static": {
                                "test": {
                                    "ty": "Bool",
                                    "src": {
                                        "value": true
                                    }
                                }
                            },
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
            }
        }"#;

        let uuid = Uuid::default();
        let mut kernel = Kernel::default();
        let channel = Broadcast::new(&uuid);
        parse_program(&serde_json::from_str(data).unwrap(), &mut kernel, &channel).unwrap();
        assert_eq!(kernel.try_build_program_interfaces(&channel), Err(Stop::new("Invalid member name: test already exists".into(), &None, None)));
    }
}
