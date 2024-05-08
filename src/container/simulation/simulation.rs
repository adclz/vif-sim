use crate::kernel::arch::global::r#type::GlobalType;
use crate::kernel::registry::{get_or_insert_global_string, get_string, Kernel};
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::container::container::{CONTAINER_PARAMS, ContainerParams, StopOn};
use crate::{error};
use std::ops::DerefMut;
use ansi_term::Colour::{Blue, Green, Purple};
use crate::kernel::plc::operations::unit::test::UnitTestStatus;

pub struct Simulation<'a> {
    registry: &'a Kernel,
    channel: &'a Broadcast,
    params: ContainerParams,
}

impl<'a> Simulation<'a> {
    pub fn new(
        registry: &'a Kernel,
        channel: &'a Broadcast,
        params: &ContainerParams,
    ) -> Self {
        Self {
            registry,
            channel,
            params: params.clone(),
        }
    }


    pub async fn start(&mut self, entry: &str) -> Result<bool, Stop> {
        self.channel.reset_cycle_stack();
        
        let entry = get_or_insert_global_string(&entry.to_string());

        let index = self
            .channel
            .get_cycle_stack()
            .borrow_mut()
            .add_section(entry, "ob");

        let curr_section = self.channel
            .get_cycle_stack()
            .borrow_mut()
            .get_current_section()
            .unwrap();
        
        let entry_block = self.registry.get(&entry);
        if entry_block.is_some() {
            match entry_block.unwrap().as_ref().borrow_mut().deref_mut() {
                GlobalType::Ob(ref mut ob) => {
                    match ob.execute(self.channel) {
                        Ok(_) => {}
                        Err(e) => {
                            self.channel.push_cycle_stack();
                            self.channel.reset_cycle_stack();
                            return Err(e);
                        }
                    };
                }
                _ => return Err(error!(format!("{} is not an OB block!", get_string(entry)))),
            }
        } else {
            return Err(error!(format!("Invalid entry block '{}'", get_string(entry))));
        }

        self.channel.add_message(&Purple.paint("--- End of Cycle ---").to_string());
        curr_section
            .borrow_mut()
            .insert_log(&Purple.paint("--- End of Cycle ---").to_string());

        self.registry
            .provider_raw_pointers
            .borrow_mut()
            .reset_temp(self.channel)?;

        self.registry
            .program_raw_pointers
            .borrow_mut()
            .reset_temp(self.channel)?;

        let mut unit_tests_done = false;
        match self.params.stopOn {
            StopOn::Infinite => {}
            StopOn::UnitTestsPassed => {
                curr_section
                    .borrow_mut()
                    .insert_log(&Blue.paint("Checking Unit tests...").to_string());
                self.channel
                    .get_unit_tests()
                    .iter()
                    .for_each(|test| {
                        curr_section.borrow_mut().insert_log(&format!("{}", test));
                        if let UnitTestStatus::Unreached = test.get_status() { unit_tests_done = true }
                    });
            }
        }
        
        if !unit_tests_done && StopOn::UnitTestsPassed == (*CONTAINER_PARAMS.lock().unwrap()).stopOn {
            self.channel.add_message(&format!(
                "Simulation stopped: {}", &Blue.paint("All unit tests reached"),
            ));
            curr_section
                .borrow_mut()
                .insert_log(&Green.paint("End of simulation").to_string());
            self.channel.push_cycle_stack();
            return Ok(false);
        }
        Ok(true)
    }
}
