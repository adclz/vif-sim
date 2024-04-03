use crate::plc::interface::traits::DeferredBuilder;
use crate::plc::internal::template::Template;
use crate::plc::pou::db::db::Db::{Global, Instance};
use crate::plc::pou::db::global_db::GlobalDb;
use crate::plc::pou::db::instance_db::InstanceDb;
use crate::plc::pou::fb::Fb;
use crate::plc::pou::fc::Fc;
use crate::plc::pou::udt::Udt;
use crate::registry::global::pointer::GlobalPointer;
use crate::registry::global::r#type::GlobalType;
use crate::registry::registry::Kernel;
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::container::broadcast::broadcast::Broadcast;

pub fn parse_provider(json: &HashMap<String, Value>, registry: &mut Kernel, channel: &Broadcast) -> Result<(), Stop> {
    for (key, value) in json {
        if !value.is_object() {
            continue;
        };
        let block_data = value.as_object().unwrap();
        let name = match key.rsplit_once('/') {
            None => continue,
            Some(a) => a.1,
        };

        if key.starts_with("file") {
            key_reader!(
                "Parse program pack".to_string(),
                block_data {
                    ty => as_str,
                    src => as_object,
                }
            );
            match ty {
                "template" => {
                    registry.program_templates.insert(
                        name.to_string(),
                        Rc::new(RefCell::new(Template::default(src))),
                    );
                    Ok(())
                }
                "ob" => Err(error!("Ob type is not allowed in provider code".to_string())),
                "fb" => registry.resources.add_new_global(
                    name.to_string(),
                    GlobalPointer::new(GlobalType::Fb(Fb::default(src))),
                ),
                "fc" => registry.resources.add_new_global(
                    name.to_string(),
                    GlobalPointer::new(GlobalType::Fc(Fc::default(src))),
                ),
                "global_db" => registry.resources.add_new_global(
                    name.to_string(),
                    GlobalPointer::new(GlobalType::Db(Global(GlobalDb::default(src)))),
                ),
                "instance_db" => registry.resources.add_new_global(
                    name.to_string(),
                    GlobalPointer::new(GlobalType::Db(Instance(InstanceDb::default(src)))),
                ),
                "udt" => registry.resources.add_new_global(
                    name.to_string(),
                    GlobalPointer::new(GlobalType::Udt(Udt::default(src))),
                ),
                _ => Err(error!(
                    format!("Unknown type provided: '{}'", ty),
                    "Parse program pack".to_string()
                )),
            }?;
        }
    }
    Ok(())
}
