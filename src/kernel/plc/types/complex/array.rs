use std::borrow::Cow;
use crate::{error, impl_primitive_traits, key_reader};
use crate::kernel::plc::interface::array_interface::ArrayInterface;
use crate::kernel::plc::types::primitives::traits::primitive_traits::{RawMut, ToggleMonitor};
use crate::kernel::arch::local::pointer::{LocalPointer, LocalPointerAndPath};
use crate::kernel::registry::{convert_string_path_to_usize, get_full_path, get_string, Kernel};
use crate::container::error::error::Stop;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use crate::parser::body::body::parse_json_target;
use crate::parser::body::json_target::JsonTarget;
use crate::kernel::rust::set::box_set_plc_primitive_default_once;
use camelpaste::paste;
use crate::container::broadcast::broadcast::Broadcast;
use crate::kernel::plc::types::primitives::string::_string::plcstr;
use crate::parser::local_type::constant_type::parse_constant_type;

use crate::parser::local_type::local_type::parse_local_type;
use crate::kernel::rust::auto_set::box_set_auto_default_once;

use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::kernel::plc::types::primitives::string::wstring::plcwstr;
use crate::kernel::plc::types::primitives::traits::meta_data::{MetaData, SetMetaData};
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive};

#[derive(Clone)]
pub struct PlcArray {
    interface: ArrayInterface,
    read_only: bool,
    path: usize,
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

    fn get_path(&self) -> String {
        get_string(self.path)
    }
}

impl SetMetaData for PlcArray {
    fn set_alias(&mut self, alias: &str, kernel: &Kernel) {
        //do nothing
    }

    fn set_read_only(&mut self, value: bool) {
        self.read_only = value;
        self.get_mut_interface()
            .iter_mut()
            .for_each(|a| a.set_read_only(value))
    }

    fn set_name(&mut self, path: usize) {
        self.path = path;
    }
}

impl ToggleMonitor for PlcArray {
    fn set_monitor(&self, kernel: &Kernel) {
        self.interface.set_monitor(kernel)
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
    plcstr, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
    plcwstr, [direct false], [stop Err(error!(format!("Can't convert an array into a primitive")))], [none Err(error!(format!("Can't convert an array into a primitive")))],
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
    pub fn from_json(data: &Map<String, Value>, registry: &Kernel, channel: &Broadcast, monitor: bool) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Array"),
            data {
                length => as_i64,
                of => as_object,
                values => as_array,
            }
        );

        let mut of = LocalPointer::from(parse_local_type(of, registry, channel, monitor)?);
        let mut interface = Vec::new();
        
        for i in 0..length {
            match values.get(i as usize) {
                None => {
                    return Err(error!(format!("Array value is not provided at index {}", i)))
                }
                Some(value) => {
                    if let Some(object) = value.as_object() {
                        let value_at_index = LocalPointer::from(parse_local_type(object, registry, channel, monitor)?);
                        if monitor {
                            value_at_index.set_monitor(registry);
                        }
                        interface.push(value_at_index);
                    } else {
                        return Err(error!(format!("Array value is not of type object")))
                    }
                }
            }
        }
        
        Ok(Self {
            interface: ArrayInterface::from(interface),
            read_only: false,
            path: 0_usize,
        })
    }

    pub fn get_raw_pointers(&mut self) -> Vec<*mut dyn RawMut> {
        self.get_interface().get_raw_pointers()
    }

    pub fn get_pointers_with_path(&self, full_path: &[usize], start_with: &[usize]) -> Vec<LocalPointerAndPath> {
        self.get_interface().get_pointers_with_path(full_path, start_with)
    }

    pub fn len(&self) -> usize {
        self.interface.len()
    }

    pub fn try_replace_pointer_nested(&mut self, path: &[usize], other: &LocalPointer) -> Option<LocalPointer> {
        self.interface.try_replace_pointer_nested(path, other)
    }

    pub fn try_get_nested(&self, path: &[usize]) -> Option<LocalPointer> {
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
