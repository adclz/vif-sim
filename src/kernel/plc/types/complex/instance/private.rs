use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use crate::{error};
use crate::parser::body::json_target::JsonTarget;
use crate::kernel::plc::types::complex::boxed::set::box_set_plc_complex;
use crate::kernel::plc::types::complex::instance::public::PublicInstanceAccessors;
use crate::kernel::plc::interface::section::Section;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::operations::operations::{Operation, RunTimeOperation};
use crate::kernel::registry::{convert_string_path_to_usize, Kernel};
use crate::container::error::error::{Stop};
use crate::kernel::rust::set::box_set_plc_primitive;
use crate::container::broadcast::broadcast::Broadcast;
use crate::kernel::rust::auto_set::box_set_auto;

pub trait PrivateInstanceAccessors {
    fn get_mut_interface(&mut self) -> &mut SectionInterface;
}

pub trait PrivateInstanceTrait {
    fn save_raw_pointers(&self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop>;
    fn build_operations(&self, registry: &Kernel, channel: &Broadcast) -> Result<Vec<RunTimeOperation>, Stop>;

    // Define the actions BEFORE doing any operations
    // Such as changing the values of the input sections
    // And give direct access to InOut references //todo!
    fn define_input_actions(
        &mut self,
        match_interface: &HashMap<Section, Vec<(Vec<String>, JsonTarget)>>,
        parent_interface: &SectionInterface,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<Vec<RunTimeOperation>, Stop>;

    // Define the actions AFTER the operations are done
    // Changing the output values
    fn define_output_actions(
        &self,
        match_interface: &HashMap<Section, Vec<(Vec<String>, JsonTarget)>>,
        parent_interface: &SectionInterface,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<Vec<RunTimeOperation>, Stop>;
}

impl<T: PrivateInstanceAccessors + PublicInstanceAccessors> PrivateInstanceTrait for T {
    fn save_raw_pointers(&self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        match self.get_interface().get(&Section::Temp) {
            None => Ok(()),
            Some(a) => {
                registry
                    .raw_pointers_collector
                    .borrow_mut()
                    .append_temp(&mut a.get_raw_pointers());
                Ok(())
            }
        }?;
        match self.get_interface().get(&Section::Constant) {
            None => Ok(()),
            Some(a) => {
                registry
                    .raw_pointers_collector
                    .borrow_mut()
                    .append_constant(&mut a.get_raw_pointers());
                Ok(())
            }
        }?;
        match self.get_interface().get(&Section::Input) {
            None => Ok(()),
            Some(a) => {
                registry
                    .raw_pointers_collector
                    .borrow_mut()
                    .append_input(&mut a.get_raw_pointers());
                Ok(())
            }
        }?;
        match self.get_interface().get(&Section::Output) {
            None => Ok(()),
            Some(a) => {
                registry
                    .raw_pointers_collector
                    .borrow_mut()
                    .append_output(&mut a.get_raw_pointers());
                Ok(())
            }
        }?;
        match self.get_interface().get(&Section::Static) {
            None => Ok(()),
            Some(a) => {
                registry
                    .raw_pointers_collector
                    .borrow_mut()
                    .append_static(&mut a.get_raw_pointers());
                Ok(())
            }
        }?;
        match self.get_interface().get(&Section::InOut) {
            None => Ok(()),
            Some(a) => {
                registry
                    .raw_pointers_collector
                    .borrow_mut()
                    .append_inout(&mut a.get_raw_pointers());
                Ok(())
            }
        }
    }

    fn build_operations(&self, registry: &Kernel, channel: &Broadcast) -> Result<Vec<RunTimeOperation>, Stop> {
        self
            .get_body()
            .iter()
            .map(|instruction| instruction.solve_as_operation(self.get_interface(), None, registry, channel))
            .collect()
    }

    fn define_input_actions(&mut self, match_interface: &HashMap<Section, Vec<(Vec<String>, JsonTarget)>>, parent_interface: &SectionInterface, registry: &Kernel, channel: &Broadcast) -> Result<Vec<RunTimeOperation>, Stop> {
        let mut assigners = Vec::new();

        match_interface
            .iter()
            .try_for_each(|(section, members)| match section {
                Section::Input => {
                    members
                        .iter()
                        .try_for_each(|(target_path, source)| {
                            let target = self.get_interface().try_get_nested(&convert_string_path_to_usize(target_path))
                                .ok_or_else(|| error!(format!("Could not find a valid reference in instance interface for path {:?}, Current interface: {}", &target_path, self.get_interface())))?;

                            assigners.push(box_set_auto(&target, &source.solve_to_ref(parent_interface, None, Some(target.as_ref().borrow().deref().clone()), registry, channel)?, &None, registry)?);
                            Ok(())
                        })
                }
                Section::InOut => {
                    members
                        .iter()
                        .try_for_each(|(target_path, source)| {
                            let target_path_to_usize = convert_string_path_to_usize(target_path);
                            let target = self.get_interface().try_get_nested(&target_path_to_usize)
                                .ok_or_else(|| error!(format!("Could not find a valid reference in instance interface for path {:?}, Current interface: {}", &target_path, self.get_interface())))?;

                            let source = source.solve_as_local_pointer(parent_interface, None, registry, channel)
                                .ok_or_else(|| error!(format!("Invalid InOut parameter in calling block, expected a reference, got {}", source)))?;

                            self.get_mut_interface().try_replace_pointer_nested(&target_path_to_usize, &source);
                            Ok(())
                        })
                },
                _ => Ok(())
            })?;
        Ok(assigners)
    }


    fn define_output_actions(&self, match_interface: &HashMap<Section, Vec<(Vec<String>, JsonTarget)>>, parent_interface: &SectionInterface, registry: &Kernel, channel: &Broadcast) -> Result<Vec<RunTimeOperation>, Stop> {
        let mut assigners = Vec::new();

        match_interface
            .iter()
            .try_for_each(|(section, members)| match section {
                Section::Output => {
                    members
                        .iter()
                        .try_for_each(|(target_path, source)| {
                            let target = self.get_interface().try_get_nested(&convert_string_path_to_usize(target_path))
                                .ok_or_else(move || error!(format!("Could not find a valid reference in instance interface for path {:?}, Current interface: {}", &target_path, self.get_interface())))?;
                            assigners.push(box_set_auto(&source.solve_as_local_pointer(parent_interface, None, registry, channel).unwrap(), &target, &None, registry)?);
                            Ok(())
                        })
                }
                _ => Ok(())
            })?;
        Ok(assigners)
    }
}

