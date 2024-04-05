﻿use crate::parser::body::json_target::JsonTarget;
use crate::plc::interface::section::Section;
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::interface::traits::Cloneable;
use crate::plc::operations::operations::{Operation, RunTimeOperation, RuntimeOperationTrait};
use crate::plc::pou::fc::Fc;
use crate::registry::registry::Kernel;
use crate::container::error::error::{Stop};
use std::collections::HashMap;
use std::rc::Rc;
use serde::{Serialize, Serializer};
use crate::plc::complex::instance::private::{PrivateInstanceAccessors, PrivateInstanceTrait};
use crate::plc::complex::instance::public::PublicInstanceAccessors;
use crate::container::broadcast::broadcast::Broadcast;

/// Only from Fc
#[derive(Clone)]
pub struct FcInstance {
    interface: SectionInterface,
    body: Vec<JsonTarget>,
    name: String
}

impl PrivateInstanceAccessors for FcInstance {
    fn get_mut_interface(&mut self) -> &mut SectionInterface {
        &mut self.interface
    }
}

impl PublicInstanceAccessors for FcInstance {
    fn get_interface(&self) -> &SectionInterface {
        &self.interface
    }

    fn get_body(&self) -> &Vec<JsonTarget> {
        &self.body
    }
}

impl Serialize for FcInstance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.interface.serialize(serializer)
    }
}

impl FcInstance {
    pub fn from(name: &str, value: &mut Fc, registry: &Kernel, channel: &Broadcast) -> Result<Self, Stop> {
        Ok(Self {
            interface: value.clone_interface(registry, channel)?,
            body: value.clone_body(registry, channel)?,
            name: name.into()
        })
    }

    pub fn build_executable(
        &mut self,
        match_interface: &HashMap<Section, Vec<(Vec<String>, JsonTarget)>>,
        parent_interface: &SectionInterface,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {

        let mut input_actions = self.define_input_actions(match_interface, parent_interface, registry, channel)?;
        let mut output_actions = self.define_output_actions(match_interface, parent_interface, registry, channel)?;
        let mut body = self.build_operations(registry, channel)?;
        self.save_raw_pointers(registry, channel)?;

        let _return = self.interface.get_return().as_ref().cloned();
        let name = self.name.clone();

        Ok(Box::new(Operation::new(
            &"Call FbInstance",
            move |channel| {
            let index = channel
                .get_cycle_stack()
                .borrow_mut()
                .add_section(&name, "Fc");

            input_actions.iter_mut().try_for_each(|assign| {
                assign.with_void(channel)?;
                Ok(())
            })?;

            if body.is_empty() {
                channel.add_warning("Function body is empty");
            };

            for operation in &mut body {
                // In case of early returns
                operation.with_void(channel)?;
                if operation.return_early() {
                    break;
                };
            }

            // Output
            output_actions.iter_mut().try_for_each(|assign| {
                assign.with_void(channel)?;
                Ok(())
            })?;

            channel
                .get_cycle_stack()
                .borrow_mut()
                .go_back_to_section(index);
            Ok(())
        }, _return, false, &None)))
    }
}