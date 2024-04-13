use crate::kernel::plc::interface::traits::DeferredBuilder;
use crate::kernel::plc::pou::db::db::Db::{Global, Instance};
use crate::kernel::plc::pou::db::global_db::GlobalDb;
use crate::kernel::plc::pou::db::instance_db::InstanceDb;
use crate::kernel::plc::pou::fb::Fb;
use crate::kernel::plc::pou::fc::Fc;
use crate::kernel::plc::pou::ob::Ob;
use crate::kernel::plc::pou::udt::Udt;
use crate::kernel::arch::global::pointer::GlobalPointer;
use crate::kernel::arch::global::r#type::GlobalType;
use crate::kernel::registry::{get_or_insert_global_string, Kernel};
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use serde_json::Value;
use std::collections::HashMap;
use crate::container::broadcast::broadcast::Broadcast;

pub fn parse_program(json: &HashMap<String, Value>, registry: &mut Kernel, channel: &Broadcast) -> Result<(), Stop> {
    for (key, value) in json {
        if !value.is_object() { continue; };
        let block_data = value.as_object().unwrap(); // safe with is_object
        let name = match key.rsplit_once('/') {
            None => continue,
            Some(a) => a.1,
        };

        if key.starts_with("file") {
            key_reader!(
                "Parse user program".to_string(),
                block_data {
                    ty => as_str,
                    src => as_object,
                }
            );
            match ty {
                "template" => Err(error!("Template is only allowed in provider, are you trying to load a provider as a user program ?".to_string())),
                "ob" => {
                    registry.program.add_new_global(
                        get_or_insert_global_string(&name.to_string()),
                        GlobalPointer::new(GlobalType::Ob(Ob::default(src))))?;
                    channel.add_entry_point(name);
                    Ok(())
                }
                "fb" => registry.program.add_new_global(
                    get_or_insert_global_string(&name.to_string()),
                    GlobalPointer::new(GlobalType::Fb(Fb::default(src)))
                ),
                "fc" => registry.program.add_new_global(
                    get_or_insert_global_string(&name.to_string()),
                    GlobalPointer::new(GlobalType::Fc(Fc::default(src)))
                ),
                "global_db" => registry.program.add_new_global(
                    get_or_insert_global_string(&name.to_string()),
                    GlobalPointer::new(GlobalType::Db(Global(GlobalDb::default(src))))
                ),
                "instance_db" => registry.program.add_new_global(
                    get_or_insert_global_string(&name.to_string()),
                    GlobalPointer::new(GlobalType::Db(Instance(InstanceDb::default(src))))
                ),
                "udt" => registry.program.add_new_global(
                    get_or_insert_global_string(&name.to_string()),
                    GlobalPointer::new(GlobalType::Udt(Udt::default(src)))
                ),
                _ => Err(error!(
                    format!("Unknown type provided: '{}'", ty),
                    "Parse user program".to_string()
                )),
            }?;
        }
    }
    Ok(())
}

