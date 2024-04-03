use std::borrow::Cow;
use crate::error;
use crate::plc::interface::status::{BodyStatus, InterfaceStatus};
use crate::plc::interface::traits::DeferredBuilder;
use crate::plc::internal::template::Template;
use crate::registry::global::pointer::GlobalPointer;
use crate::registry::global::r#type::GlobalType;
use crate::registry::local::pointer::LocalPointer;
use crate::registry::observers::reset::RawPointers;
use crate::container::error::error::Stop;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use crate::container::broadcast::broadcast::Broadcast;
use crate::plc::interface::section::Section;
use crate::plc::primitives::family_traits::MetaData;
use crate::registry::constant::r#type::ConstantType;

pub enum GlobalOrLocal {
    Global(GlobalPointer),
    Local(LocalPointer),
}

pub struct GlobalPointerMap(HashMap<String, GlobalPointer>);

impl GlobalPointerMap {
    pub fn add_new_global(
        &mut self,
        name: String,
        pointer: GlobalPointer,
    ) -> Result<(), Stop> {
        match self.0.entry(name.clone()) {
            Entry::Occupied(_) => Err(error!(
                format!("Name '{}' already registered.", name),
                format!("Register new global type")
            )),
            Entry::Vacant(entry) => {
                entry.insert(pointer);
                Ok(())
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, name: &str) -> Option<GlobalPointer> {
        self.0.get(name).cloned()
    }

    /// Will only work if Db
    fn get_and_find_nested(&self, path: &[String]) -> Option<GlobalOrLocal> {
        match self.0.get(&path[0]) {
            None => None,
            Some(root) =>
                match root.as_ref().borrow().deref() {
                    GlobalType::Db(db) => {
                        if path.len() == 1 {
                            Some(GlobalOrLocal::Global(root.clone()))
                        } else {
                            db.try_get_nested(&path[1..]).map(|a| GlobalOrLocal::Local(a.clone()))
                        }
                    }
                    _ => None,
                },
        }
    }

    fn iter(&self) -> impl Iterator<Item=(&String, &GlobalPointer)> {
        self.0.iter()
    }
}

type Operation = String;
type FirstType = String;
type SecondType = String;

pub struct Kernel {
    pub resources: GlobalPointerMap,
    pub program: GlobalPointerMap,

    pub resources_templates: HashMap<String, Rc<RefCell<Template>>>,
    pub program_templates: HashMap<String, Rc<RefCell<Template>>>,

    pub raw_pointers_collector: RefCell<RawPointers>,
    pub resources_raw_pointers: RefCell<RawPointers>,
    pub program_raw_pointers: RefCell<RawPointers>,

    exclude_types: Vec<String>,
    filter_operations: HashMap<Operation, HashMap<FirstType, HashSet<SecondType>>>,
    operations_return: HashMap<Operation, HashMap<FirstType, HashMap<SecondType, ConstantType>>>,
    exclude_sections: HashMap<Section, HashSet<String>>,

    type_aliases: HashMap<String, ConstantType>,
    all_types_id: Vec<String>,

    ignore_operation: Rc<RefCell<bool>>
}

impl Default for Kernel {
    fn default() -> Self {
        Self {
            resources: GlobalPointerMap(HashMap::new()),
            program: GlobalPointerMap(HashMap::new()),

            program_templates: HashMap::new(),
            resources_templates: HashMap::new(),

            raw_pointers_collector: RefCell::new(RawPointers::default()),
            resources_raw_pointers: RefCell::new(RawPointers::default()),
            program_raw_pointers: RefCell::new(RawPointers::default()),
            exclude_types: vec!(),
            filter_operations: HashMap::default(),
            operations_return: HashMap::default(),
            exclude_sections: HashMap::default(),

            type_aliases: HashMap::default(),
            all_types_id: vec!(),

            ignore_operation: Rc::new(RefCell::new(false))
        }
    }
}

impl Kernel {

    pub fn should_ignore_operation(&self) -> bool {
        *self.ignore_operation.borrow().deref()
    }

    pub fn set_ignore_operation(&self, value: bool) {
        (*self.ignore_operation.borrow_mut().deref_mut()) = value;
    }

    pub fn get(&self, name: &str) -> Option<GlobalPointer> {
        match self.program.get(name) {
            None => self.resources.get(name),
            Some(a) => Some(a.clone()),
        }
    }

    pub fn add_type_alias(&mut self, name: &str, of: ConstantType) -> usize {
        let name = name.to_string();
        self.type_aliases.insert(name.clone(), of);
        self.all_types_id.push(name.clone());
        self.all_types_id.iter().position(|e| e == &name).unwrap()
    }

    pub fn get_type_alias_id(&self, name: &str) -> Option<usize> {
        self.all_types_id.iter().position(|e| e == &name.to_string())
    }

    pub fn get_type_alias_str(&self, id: usize) -> Option<&String> {
        self.all_types_id.get(id)
    }

    pub fn get_type_alias_as_constant_type(&self, name: &str) -> Option<&ConstantType> {
        self.type_aliases.get(name)
    }

    pub fn get_mut_excluded_operation(&mut self, operation: &str) -> & mut HashMap<FirstType, HashSet<SecondType>> {
        self.filter_operations.entry(operation.into()).or_default()
    }

    pub fn get_mut_excluded_types(&mut self) -> &mut Vec<String> {
        &mut self.exclude_types
    }

    pub fn get_mut_return_operation(&mut self, operation: &str) -> & mut HashMap<FirstType, HashMap<SecondType, ConstantType>> {
        self.operations_return.entry(operation.into()).or_default()
    }

    pub fn get_mut_excluded_types_in_section(&mut self, operation: &Section) -> &mut HashSet<String> {
        self.exclude_sections.entry(operation.clone()).or_default()
    }

    pub fn check_filtered_operation<T: MetaData, Y: MetaData>(&self, operation: &str, meta_data_t1: &T, meta_data_t2: &Y) -> Result<(), Stop> {
        // Find the operation
        match self.filter_operations.get(operation) {
            None => Ok(()),
            Some(a) => {

                let type1 = match meta_data_t1.get_alias_str(self) {
                    None => meta_data_t1.name().to_string(),
                    Some(a) => a.deref().to_string()
                };

                let type2 = match meta_data_t2.get_alias_str(self) {
                    None => meta_data_t2.name().to_string(),
                    Some(a) => a.deref().to_string()
                };

                // Find the first types in initial keys
                if let Some(types) = a.get(&type1) {
                    // Iterate through the allowed ones
                    if types.contains(&type2) {
                        Ok(())
                    } else {
                        Err(error!(format!("Operation {} is forbidden with types {} and {}", operation, type1, type2)))
                    }
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn check_excluded_type<T: MetaData>(&self, meta_data: &T) -> Result<(), Stop> {
        if let Some(alias) = meta_data.get_alias_str(self) {
            let alias = alias.to_string();
            if self.exclude_types.contains(&alias){
                return Err(error!(format!("Type {} is forbidden", alias)));
            }
        };

        let type_name: String = meta_data.name().into();
        if self.exclude_types.contains(&type_name){
            return Err(error!(format!("Type {} is forbidden", type_name)));
        };

        Ok(())
    }

    pub fn check_excluded_type_in_section<T: MetaData>(&self, section: &Section, meta_data: &T) -> Result<(), Stop> {
        match self.exclude_sections.get(section) {
            None => Ok(()),
            Some(exclude_by_section) => {
                if let Some(alias) = meta_data.get_alias_str(self) {
                    if exclude_by_section.contains::<String>(&alias.deref().to_string()){
                        return Err(error!(format!("Type {} is forbidden in section {}", alias, section)));
                    }
                };

                let type_name: String = meta_data.name().into();
                if exclude_by_section.contains(&type_name){
                    return Err(error!(format!("Type {} is forbidden in section {}", type_name, section)));
                };

                Ok(())
            }
        }
    }

    pub fn check_return_operation<T: MetaData, Y: MetaData>(&self, operation: &str, meta_data_t1: &T, meta_data_t2: &Y) -> Option<ConstantType> {
        match self.operations_return.get(operation) {
            None => None,
            Some(a) => {

                let type1 = match meta_data_t1.get_alias_str(self) {
                    None => meta_data_t1.name().to_string(),
                    Some(a) => a.deref().to_string()
                };

                let type2 = match meta_data_t2.get_alias_str(self) {
                    None => meta_data_t2.name().to_string(),
                    Some(a) => a.deref().to_string()
                };

                if let Some(type1) = a.get(&type1) {
                    if let Some(constant) = type1.get(&type2) {
                      return Some(constant.clone())
                    }
                };
                None
            }
        }
    }

    pub fn swap_pointers_collector_to_resources(&mut self) {
        *self.resources_raw_pointers.borrow_mut().deref_mut() = std::mem::take(&mut self.raw_pointers_collector.borrow_mut())
    }

    pub fn swap_pointers_collector_to_program(&mut self) {
        *self.program_raw_pointers.borrow_mut().deref_mut() = std::mem::take(&mut self.raw_pointers_collector.borrow_mut())
    }

    /// Will only work if Db
    pub fn get_and_find_nested(&self, path: &[String]) -> Option<GlobalOrLocal> {
        match self.resources.get_and_find_nested(path) {
            None => self.program.get_and_find_nested(path),
            Some(a) => Some(a),
        }
    }

    // Reset methods
    pub fn reset_all(&mut self, channel: &Broadcast) {
        self.resources_raw_pointers.borrow_mut().filter_dangling();
        self.resources_raw_pointers.borrow_mut().reset_all(channel).unwrap();
        self.program_raw_pointers.borrow_mut().filter_dangling();
        self.program_raw_pointers.borrow_mut().reset_all(channel).unwrap();
        channel.reset_unit_tests();
        channel.reset_breakpoints();
    }

    pub fn clear_program(&mut self, channel: &Broadcast) {
        self.program.0.clear();
        self.reset_all(channel);
        channel.clear_unit_tests();
        channel.clear_breakpoints();
        channel.clear_entry_points();
        self.program_raw_pointers.borrow_mut().clear_all();
    }

    pub fn clear_all(&mut self, channel: &Broadcast) {
        self.clear_program(channel);
        self.resources.0.clear();
        self.resources_raw_pointers.borrow_mut().clear_all();
        self.exclude_types.clear();
        self.exclude_sections.clear();
        self.filter_operations.clear();
    }

    pub fn try_build_program_interfaces(&mut self, channel: &Broadcast) -> Result<(), Stop> {
        self.program.iter().try_for_each(|(name, pointer)| {
            let status = pointer.as_ref().borrow().deref().get_interface_status();
            match status {
                InterfaceStatus::Default => pointer
                    .as_ref()
                    .borrow_mut()
                    .deref_mut()
                    .build_interface(self, channel),
                InterfaceStatus::Pending => Err(error!(format!("Still pending !"))),
                InterfaceStatus::Solved => Ok(()),
            }
                .map_err(|e: Stop| e.add_sim_trace(&format!("Build '{}' -> interface", name)))
        })?;

        self.program_templates
            .iter()
            .try_for_each(|(name, template)| {
                let status = template.borrow().deref().get_interface_status();
                match status {
                    InterfaceStatus::Default => {
                        template.borrow_mut().deref_mut().build_interface(self, channel)
                    }
                    InterfaceStatus::Pending => Err(error!(format!("Still pending !"))),
                    InterfaceStatus::Solved => Ok(()),
                }
                    .map_err(|e: Stop| e.add_sim_trace(&format!("Build '{}' -> interface", name)))
            })
    }

    pub fn try_build_program_bodies(&mut self, channel: &Broadcast) -> Result<(), Stop> {
        self.program.iter().try_for_each(|(name, pointer)| {
            let status = pointer.as_ref().borrow().deref().get_body_status();
            match status {
                BodyStatus::Default => pointer.as_ref().borrow_mut().deref_mut().build_body(self, channel),
                BodyStatus::Pending => Err(error!(format!("Still pending !"))),
                BodyStatus::Solved => Ok(()),
            }
                .map_err(|e: Stop| e.add_sim_trace(&format!("Build '{}' -> body", name)))
        })?;

        self.program_templates
            .iter()
            .try_for_each(|(name, template)| {
                let status = template.borrow().deref().get_body_status();
                match status {
                    BodyStatus::Default => template.borrow_mut().deref_mut().build_body(self, channel),
                    BodyStatus::Pending => Err(error!(format!("Still pending !"))),
                    BodyStatus::Solved => Ok(()),
                }
                    .map_err(|e: Stop| e.add_sim_trace(&format!("Build '{}' -> body", name)))
            })
    }

    pub fn try_build_resources_interfaces(&mut self, channel: &Broadcast) -> Result<(), Stop> {
        self.resources.iter().try_for_each(|(name, pointer)| {
            let status = pointer.as_ref().borrow().deref().get_interface_status();
            match status {
                InterfaceStatus::Default => pointer
                    .as_ref()
                    .borrow_mut()
                    .deref_mut()
                    .build_interface(self, channel),
                InterfaceStatus::Pending => Err(error!(format!("Still pending !"))),
                InterfaceStatus::Solved => Ok(()),
            }
                .map_err(|e: Stop| e.add_sim_trace(&format!("Build '{}' -> interface", name)))
        })?;

        self.program_templates
            .iter()
            .try_for_each(|(name, template)| {
                let status = template.borrow().deref().get_interface_status();
                match status {
                    InterfaceStatus::Default => {
                        template.borrow_mut().deref_mut().build_interface(self, channel)
                    }
                    InterfaceStatus::Pending => Err(error!(format!("Still pending !"))),
                    InterfaceStatus::Solved => Ok(()),
                }
                    .map_err(|e: Stop| e.add_sim_trace(&format!("Build '{}' -> interface", name)))
            })
    }

    pub fn try_build_resources_bodies(&mut self, channel: &Broadcast) -> Result<(), Stop> {
        self.resources.iter().try_for_each(|(name, pointer)| {
            let status = pointer.as_ref().borrow().deref().get_body_status();
            match status {
                BodyStatus::Default => pointer.as_ref().borrow_mut().deref_mut().build_body(self, channel),
                BodyStatus::Pending => Err(error!(format!("Still pending !"))),
                BodyStatus::Solved => Ok(()),
            }
                .map_err(|e: Stop| e.add_sim_trace(&format!("Build '{}' -> body", name)))
        })?;

        self.program_templates
            .iter()
            .try_for_each(|(name, template)| {
                let status = template.borrow().deref().get_body_status();
                match status {
                    BodyStatus::Default => template.borrow_mut().deref_mut().build_body(self, channel),
                    BodyStatus::Pending => Err(error!(format!("Still pending !"))),
                    BodyStatus::Solved => Ok(()),
                }
                    .map_err(|e: Stop| e.add_sim_trace(&format!("Build '{}' -> body", name)))
            })
    }
}
