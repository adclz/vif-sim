use crate::kernel::plc::interface::traits::DeferredBuilder;
use crate::kernel::plc::internal::template::Template;
use crate::kernel::plc::pou::db::db::Db::{Global, Instance};
use crate::kernel::plc::pou::db::global_db::GlobalDb;
use crate::kernel::plc::pou::db::instance_db::InstanceDb;
use crate::kernel::plc::pou::fb::Fb;
use crate::kernel::plc::pou::fc::Fc;
use crate::kernel::plc::pou::udt::Udt;
use crate::kernel::arch::global::pointer::GlobalPointer;
use crate::kernel::arch::global::r#type::GlobalType;
use crate::kernel::registry::{get_or_insert_global_string, Kernel};
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use serde_json::Value;
use core::cell::RefCell;
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
                "Parse provider".to_string(),
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
                "ob" => Err(error!("Ob type is not allowed in provider".to_string())),
                "fb" => registry.provider.add_new_global(
                    get_or_insert_global_string(&name.to_string()),
                    GlobalPointer::new(GlobalType::Fb(Fb::default(src)))
                ),
                "fc" => registry.provider.add_new_global(
                    get_or_insert_global_string(&name.to_string()),
                    GlobalPointer::new(GlobalType::Fc(Fc::default(src)))
                ),
                "global_db" => registry.provider.add_new_global(
                    get_or_insert_global_string(&name.to_string()),
                    GlobalPointer::new(GlobalType::Db(Global(GlobalDb::default(src)))),
                ),
                "instance_db" => registry.provider.add_new_global(
                    get_or_insert_global_string(&name.to_string()),
                    GlobalPointer::new(GlobalType::Db(Instance(InstanceDb::default(src))))
                ),
                "udt" => registry.provider.add_new_global(
                    get_or_insert_global_string(&name.to_string()),
                    GlobalPointer::new(GlobalType::Udt(Udt::default(src)))
                ),
                _ => Err(error!(
                    format!("Unknown type provided: '{}'", ty),
                    "Parse provider".to_string()
                )),
            }?;
        }
    }
    Ok(())
}
