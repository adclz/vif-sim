use std::borrow::Cow;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::parser::body::json_target::JsonTarget;
use crate::kernel::plc::types::complex::array::PlcArray;
use crate::kernel::plc::types::complex::instance::private::{PrivateInstanceAccessors, PrivateInstanceTrait};
use crate::kernel::plc::types::complex::instance::public::PublicInstanceAccessors;
use crate::kernel::plc::interface::section::Section;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::interface::traits::Cloneable;
use crate::kernel::plc::operations::operations::{Operation, RunTimeOperation, RuntimeOperationTrait};
use crate::kernel::plc::pou::fb::Fb;
use crate::kernel::plc::types::primitives::traits::primitive_traits::{RawMut, ToggleMonitor};
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::kernel::plc::types::primitives::string::wstring::wstr256;
use crate::kernel::arch::local::pointer::LocalPointerAndPath;
use crate::kernel::registry::{get_full_path, get_string, Kernel};
use crate::{error, impl_primitive_traits};
use camelpaste::paste;
use fixedstr::str256;

use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::kernel::plc::types::primitives::traits::meta_data::{MaybeHeapOrStatic, MetaData, SetMetaData};
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive};

/// Only from Fb
#[derive(Clone)]
pub struct FbInstance {
    interface: SectionInterface,
    body: Vec<JsonTarget>,
    of: Option<String>,
    read_only: bool,
    path: usize,
    id: u64
}

impl_primitive_traits!(FbInstance, {
    bool, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    char, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    wchar, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    str256, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    wstr256, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    f32, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    f64, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    u8, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    u16, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    u32, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    u64, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    i8, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    i16, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    i32, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))],
    i64, [direct false], [stop Err(error!(format!("Can't convert an instance into a primitive")))], [none Err(error!(format!("Can't convert an instance into a primitive")))]
});

impl PrivateInstanceAccessors for FbInstance {
    fn get_mut_interface(&mut self) -> &mut SectionInterface {
        &mut self.interface
    }
}

impl PublicInstanceAccessors for FbInstance {
    fn get_interface(&self) -> &SectionInterface {
        &self.interface
    }

    fn get_body(&self) -> &Vec<JsonTarget> {
        &self.body
    }
}

impl MetaData for FbInstance {
    fn name(&self) -> &'static str {
        &"Instance"
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

impl SetMetaData for FbInstance {
    fn set_alias(&mut self, alias: &str, kernel: &Kernel){
        // do nothing
    }

    fn set_read_only(&mut self, value: bool) {
        self.read_only = value;
        self.get_mut_interface()
            .iter_mut()
            .for_each(|a| a.1.iter_mut().for_each(|b| b.1.set_read_only(value)))
    }

    fn set_name(&mut self, path: usize) {
        self.path = path;
    }
}

impl ToggleMonitor for FbInstance {
    fn set_monitor(&mut self, activate: bool) {
        self.interface.set_monitor(activate)
    }
}

impl Display for FbInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.interface)
    }
}

impl Serialize for FbInstance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.interface.serialize(serializer)
    }
}

impl FbInstance {
    pub fn from_fb(of: Option<String>, id: u64, interface: SectionInterface, body: Vec<JsonTarget>, registry: &Kernel, channel: &Broadcast) -> Result<Self, Stop> {
        Ok(Self {
            interface,
            body,
            read_only: false,
            of,
            path: 0,
            id
        })
    }

    pub fn get_raw_pointers(&self) -> Vec<*mut dyn RawMut> {
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

    pub fn build_executable(
        &mut self,
        match_interface: &HashMap<Section, Vec<(Vec<String>, JsonTarget)>>,
        parent_interface: &SectionInterface,
        registry: &Kernel,
        channel: &Broadcast,
    ) -> Result<RunTimeOperation, Stop> {
        let input_actions =
            self.define_input_actions(match_interface, parent_interface, registry, channel)?;
        let output_actions =
            self.define_output_actions(match_interface, parent_interface, registry, channel)?;
        let body = self.build_operations(registry, channel)?;
        self.save_raw_pointers(registry, channel)?;

        Ok(Box::new(Operation::new(
            MaybeHeapOrStatic(None),
            move |channel| {
                input_actions.iter().try_for_each(|assign| {
                    assign.with_void(channel)?;
                    Ok(())
                })?;

                if body.is_empty() {
                    channel.add_warning("Function body is empty");
                };

                for operation in &body {
                    // In case of early returns
                    operation.with_void(channel)?;
                    if operation.return_early() {
                        break;
                    };
                }

                // Output
                output_actions.iter().try_for_each(|assign| {
                    assign.with_void(channel)?;
                    Ok(())
                })?;
                Ok(())
            },
            None,
            false,
            self.id
        )))
    }
}
