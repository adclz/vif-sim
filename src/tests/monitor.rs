#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use serde_json::Value;
    use uuid::Uuid;
    use crate::container::broadcast::broadcast::Broadcast;
    use crate::kernel::plc::types::primitives::traits::meta_data::MetaData;
    use crate::kernel::registry::{convert_string_path_to_usize, GlobalOrLocal, Kernel};
    use crate::key_reader;
    use crate::parser::body::path::parse_path;
    use crate::parser::main::monitor::parse_monitor;
    use crate::parser::main::program::parse_program;

    #[test]
    fn test_monitor() {
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
            },
            "monitor": [
                ["MyInstance", "test"]
            ]
        }"#;

        let json: HashMap<String, Value> = serde_json::from_str(data).unwrap();

        key_reader!(
            format!("Invalid user program data"),
            json {
                monitor? => as_array,
            }
        );

        let monitor = monitor.unwrap();

        let uuid = Uuid::default();
        let mut kernel = Kernel::default();
        let channel = Broadcast::new(&uuid);

        let base = parse_program(&json, &mut kernel, &channel).unwrap();
        kernel.try_build_program_interfaces(&channel).unwrap();
        kernel.try_build_program_bodies(&channel).unwrap();

        parse_monitor(&monitor, &channel, &mut kernel);

        monitor.iter().for_each(|x| {
            let path =
                &convert_string_path_to_usize(
                    &parse_path(x)
                        .map_err(|e| channel.add_warning(&format!("[Monitor] Invalid path {:?}", x)))
                        .unwrap_or_default());

            let exists = kernel.get_and_find_nested(&path).unwrap();
            match exists {
                GlobalOrLocal::Global(a) => panic!(""),
                GlobalOrLocal::Local(a) => {
                    assert_eq!(format!("{}", a), "test(Bool: true)");
                }
            }
        })
    }
}
