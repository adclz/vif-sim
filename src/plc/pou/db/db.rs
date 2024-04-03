use crate::error;
use crate::plc::complex::instance::public::PublicInstanceAccessors;
use crate::plc::interface::section::Section;
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::interface::struct_interface::StructInterface;
use crate::plc::pou::db::global_db::GlobalDb;
use crate::plc::pou::db::instance_db::InstanceDb;
use crate::registry::local::pointer::{LocalPointer, LocalPointerAndPath};
use crate::container::error::error::Stop;

pub enum Db {
    Global(GlobalDb),
    Instance(InstanceDb)
}

impl Db {
    pub fn get_interface(&self) -> &SectionInterface {
        match self {
            Db::Global(global) => global.get_interface(),
            Db::Instance(instance) => instance.get_interface(),
        }
    }

    pub fn get_pointers_with_path(&self, full_path: &[String], start_with: &[String]) -> Vec<LocalPointerAndPath> {
        match self {
            Db::Global(global) => global.get_interface().get_pointers_with_path(full_path, start_with),
            Db::Instance(instance) => instance.get_interface().get_pointers_with_path(full_path, start_with),
        }
    }

    pub fn try_replace_pointer_nested(&mut self, path: &[String], other: &LocalPointer) -> Option<LocalPointer> {
        match self {
            Db::Global(global) => global.try_replace_pointer_nested(path, other),
            Db::Instance(instance) => instance.try_replace_pointer_nested(path, other),
        }
    }

    pub fn try_get_nested(&self, path: &[String]) -> Option<LocalPointer> {
        match self {
            Db::Global(global) => global.try_get_nested(path),
            Db::Instance(instance) => instance.try_get_nested(path),
        }
    }

    pub fn get_section(&mut self, section: &Section) -> Option<&StructInterface> {
        match self {
            Db::Global(global) => global.get_section(section),
            Db::Instance(instance) => instance.get_section(section)
        }
    }
    
    pub fn is_global_db(&self) -> bool {
        matches!(self, Db::Global(_))
    }

    pub fn is_instance_db(&self) -> bool {
        matches!(self, Db::Instance(_))
    }

    pub fn as_ref_global_db(&self) -> Result<&GlobalDb, Stop> {
        match self {
            Db::Global(global) => Ok(global),
            _ => Err(error!(format!("Not a GlobalDb")))
        }
    }

    pub fn as_ref_instance_db(&self) -> Result<&InstanceDb, Stop> {
        match self {
            Db::Instance(instance) => Ok(instance),
            _ => Err(error!(format!("Not an InstanceDb")))
        }
    }

    pub fn as_mut_global_db(&mut self) -> Result<&mut GlobalDb, Stop> {
        match self {
            Db::Global(global) => Ok(global),
            _ => Err(error!(format!("Not a GlobalDb")))
        }
    }

    pub fn as_mut_instance_db(&mut self) -> Result<&mut InstanceDb, Stop> {
        match self {
            Db::Instance(instance) => Ok(instance),
            _ => Err(error!(format!("Not an InstanceDb")))
        }
    }
}