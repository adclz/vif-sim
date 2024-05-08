#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::container::broadcast::broadcast::Broadcast;
    use crate::kernel::registry::Kernel;
    use crate::parser::main::program::parse_program;

    #[test]
    pub fn assign() {
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
                    "body": [
                        { 
                            "ty": "asg",
                            "src": {
                                "assign": {
                                    "ty": "local",
                                    "src": {
                                        "path": ["test"]
                                    }
                                },
                                "to": {
                                    "ty": "Implicit",
                                    "src": {
                                        "ty": "Bool",
                                        "value": true
                                    }
                                }
                            }
                        }
                    ]
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

        let uuid = Uuid::default();
        let mut kernel = Kernel::default();
        let channel = Broadcast::new(&uuid);
        parse_program(&serde_json::from_str(data).unwrap(), &mut kernel, &channel).unwrap();
        kernel.try_build_program_interfaces(&channel).unwrap();
        kernel.try_build_program_bodies(&channel).unwrap();
    }
}