use std::borrow::Cow;
use crate::{error, impl_primitive_traits, key_reader};
use crate::plc::interface::array_interface::ArrayInterface;
use crate::plc::primitives::primitive_traits::RawMut;
use crate::registry::local::pointer::{LocalPointer, LocalPointerAndPath};
use crate::registry::registry::Kernel;
use crate::container::error::error::Stop;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use crate::parser::body::body::parse_json_target;
use crate::parser::body::json_target::JsonTarget;
use crate::plc::primitives::boxed::set::box_set_plc_primitive_default_once;
use camelpaste::paste;
use crate::plc::primitives::family_traits::{Primitive, AsMutPrimitive, MetaData, ToggleMonitor, SetMetaData};
use crate::container::broadcast::broadcast::Broadcast;
use fixedstr::str256;
use crate::parser::local_type::constant_type::parse_constant_type;

use crate::parser::local_type::local_type::parse_local_type;
use crate::plc::auto_boxed::set::box_set_auto_default_once;

use crate::plc::primitives::string::wchar::wchar;
use crate::plc::primitives::string::wstring::wstr256;

#[derive(Clone)]
pub struct PlcArray {
    interface: ArrayInterface,
    read_only: bool,
}

impl MetaData for PlcArray {
    fn name(&self) -> &'static str {
        &"Array"
    }

    fn get_alias_str<'a>(&self, kernel: &'a Kernel) -> Option<&'a String> {
        None
    }

    fn get_alias_id(&self, kernel: &Kernel) -> Option<usize> {
        None
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }
}

impl SetMetaData for PlcArray {
    fn set_alias(&mut self, alias: &str, kernel: &Kernel) {
        //do nnothing
    }

    fn set_read_only(&mut self, value: bool) {
        self.read_only = value;
        self.get_mut_interface()
            .iter_mut()
            .for_each(|a| a.set_read_only(value))
    }
}

impl ToggleMonitor for PlcArray {
    fn set_monitor(&mut self, activate: bool) {
        self.interface.set_monitor(activate)
    }
}

impl Display for PlcArray {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.interface)
    }
}

impl Serialize for PlcArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        self.interface.serialize(serializer)
    }
}

impl_primitive_traits!(PlcArray, {
    bool, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    char, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    wchar, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    str256, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    wstr256, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    f32, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    f64, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    u8, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    u16, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    u32, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    u64, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    i8, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    i16, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    i32, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    i64, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))]
});

impl PlcArray {
    pub fn from_json(data: &Map<String, Value>, registry: &Kernel, channel: &Broadcast) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Array"),
            data {
                length => as_i64,
                values? => as_object,
                of => as_object,
            }
        );

        let mut interface = Vec::new();
        let array_type = vec!(LocalPointer::from(parse_local_type(of, registry, channel)?));

        println!("{}", array_type[0]);
        let array_pointer = array_type.first().unwrap();

        for _i in 0..length {
            interface.push(array_pointer.duplicate())
        }

        let interface = ArrayInterface::from(interface);

        if values.is_some() {
            let values = values.unwrap();
            key_reader!(
                format!("Parse Array default values"),
                values {
                    src => as_array,
                }
            );

            src
                .iter()
                .try_for_each(|a| {

                    let a = match a.as_object() {
                        None => Err(error!(format!("Array value is not an object, got {}", a))),
                        Some(a) => Ok(a)
                    }?;

                    key_reader!(
                    format!("Array default values"),
                    a {
                        target,
                        value,
                    }
                );

                    // Find the reference in array
                    let target = match parse_json_target(target)? {
                        JsonTarget::Local(a) => interface.try_get_nested(&a),
                        _ => None
                    }.ok_or(error!(format!("Invalid target for array default value, expected pointer, got {:?}", a)))?;

                    // Parse the value and solve it according to the previous reference
                    let value_raw = parse_json_target(value)?;

                   if value_raw.is_constant() {
                       let value = parse_constant_type(value.as_object().unwrap(), registry, Some(target.as_ref().borrow().deref().clone()))?;
                       box_set_auto_default_once(&target, &value)?(channel)
                   } else if value_raw.is_local() {
                       let value = parse_local_type(value.as_object().unwrap(), registry, channel)?;
                       box_set_auto_default_once(&target, &value)?(channel)
                   } else {
                        Err(error!(format!("A default value has to be a local or constant type")))
                   }
                })?
        }

        Ok(Self {
            interface,
            read_only: false
        })
    }

    pub fn get_raw_pointers(&mut self) -> Vec<*mut dyn RawMut> {
        self.get_interface().get_raw_pointers()
    }

    pub fn get_pointers_with_path(&self, full_path: &[String], start_with: &[String]) -> Vec<LocalPointerAndPath> {
        self.get_interface().get_pointers_with_path(full_path, start_with)
    }

    pub fn len(&self) -> usize {
        self.interface.len()
    }

    pub fn try_replace_pointer_nested(&mut self, path: &[String], other: &LocalPointer) -> Option<LocalPointer> {
        self.interface.try_replace_pointer_nested(path, other)
    }

    pub fn try_get_nested(&self, path: &[String]) -> Option<LocalPointer> {
        self.interface.try_get_nested(path)
    }

    pub fn get_interface(&self) -> &ArrayInterface {
        &self.interface
    }

    pub fn get_mut_interface(&mut self) -> &mut ArrayInterface {
        &mut self.interface
    }

    pub fn mut_interface(&mut self) -> &mut ArrayInterface {
        &mut self.interface
    }
}
