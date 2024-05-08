use std::borrow::Cow;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::parser::interface::interface::parse_struct_interface;
use crate::kernel::plc::interface::struct_interface::StructInterface;
use crate::kernel::plc::pou::udt::Udt;
use crate::kernel::plc::types::primitives::traits::family_traits::{IsFamily};
use crate::kernel::plc::types::primitives::traits::primitive_traits::ToggleMonitor;
use crate::kernel::plc::types::primitives::traits::primitive_traits::RawMut;
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::kernel::plc::types::primitives::string::wstring::wstr256;
use crate::kernel::arch::local::pointer::{LocalPointer, LocalPointerAndPath};
use crate::kernel::registry::{get_full_path, get_string, Kernel};
use crate::{error, impl_primitive_traits, key_reader, required_key};
use camelpaste::paste;
use fixedstr::str256;

use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};
use crate::kernel::plc::types::primitives::traits::meta_data::{MetaData, SetMetaData};
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive};

#[derive(Clone)]
pub struct PlcStruct {
    interface: StructInterface,
    of: Option<String>,
    read_only: bool,
    path: usize
}

impl MetaData for PlcStruct {
    fn name(&self) -> &'static str {
        &"Struct"
    }

    fn get_alias_str<'a>(&'a self, kernel: &'a Kernel) -> Option<&'a String> {
        self.of.as_ref()
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

impl SetMetaData for PlcStruct {
    fn set_alias(&mut self, alias: &str, kernel: &Kernel) {
        // do nothing
    }

    fn set_read_only(&mut self, value: bool) {
        self.read_only = value;
        self.get_mut_interface()
            .iter_mut()
            .for_each(|a| a.1.set_read_only(value))
    }

    fn set_name(&mut self, path: usize) {
        self.path = path;
    }
}

impl ToggleMonitor for PlcStruct {
    fn set_monitor(&mut self, activate: bool) {
        self.interface.set_monitor(activate)
    }
}

impl Display for PlcStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.interface)
    }
}

impl Serialize for PlcStruct {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.interface.serialize(serializer)
    }
}

impl_primitive_traits!(PlcStruct, {
    bool, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    char, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    wchar, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    str256, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    wstr256, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    f32, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    f64, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    u8, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    u16, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    u32, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    u64, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    i8, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    i16, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    i32, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))],
    i64, [direct false], [stop Err(error!(format!("Can't convert a struct into a primitive")))], [none Err(error!(format!("Can't convert a struct into a primitive")))]
});

impl PlcStruct {
    pub fn from_interface(of: Option<String>, value: StructInterface, registry: &Kernel, channel: &Broadcast) -> Result<Self, Stop> {
        Ok(Self { of, interface: value, read_only: false, path: 0_usize })
    }

    pub fn get_raw_pointers(&mut self) -> Vec<*mut dyn RawMut> {
        self.get_interface().get_raw_pointers()
    }

    pub fn get_pointers_with_path(
        &self,
        full_path: &[usize],
        start_with: &[usize],
    ) -> Vec<LocalPointerAndPath> {
        self.get_interface()
            .get_pointers_with_path(full_path, start_with)
    }

    pub fn from_json(
        json: &Map<String, Value>,
        registry: &Kernel,
        channel: &Broadcast,
    ) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Struct interface"),
            json {
                interface,
            }
        );

        let mut _self = Self {
            of: None,
            interface: StructInterface::new(),
            read_only: false,
            path: 0_usize
        };

        parse_struct_interface(&interface, registry, channel, &None)?
            .as_ref()
            .iter()
            .for_each(|(name, pointer)| {
                _self
                    .interface
                    .as_mut()
                    .entry(name.clone())
                    .or_insert_with(|| pointer.clone());
            });

        Ok(_self)
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

    pub fn get_interface(&self) -> &StructInterface {
        &self.interface
    }

    pub fn get_mut_interface(&mut self) -> &mut StructInterface {
        &mut self.interface
    }

    pub fn get(&self, name: &usize) -> Option<&LocalPointer> {
        self.interface.get(&name)
    }
}
