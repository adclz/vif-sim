﻿use crate::container::broadcast::broadcast::Broadcast;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::kernel::plc::operations::basics::compare::{box_cmp, get_cmp_targets};
use crate::kernel::plc::operations::internal::timer_sm::TimerStateMachine;
use crate::kernel::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation,
};
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic};
use crate::kernel::plc::types::primitives::traits::primitive_traits::PrimitiveTrait;
use crate::kernel::registry::Kernel;
use crate::key_reader;
use crate::parser::body::body::parse_json_target;
use crate::parser::body::json_target::JsonTarget;
use ansi_term::Color::Yellow;
use ansi_term::Colour::{Blue, Green, Red};
use serde_json::{Map, Value};
use core::cell::RefCell;
use core::fmt::{Display, Formatter};
use std::rc::Rc;
use tsify::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[derive(Tsify)]
#[wasm_bindgen(skip_typescript)]
#[derive(Clone)]
pub struct UnitTest {
    description: String,
    id: u32,
    status: UnitTestStatus,
}

#[wasm_bindgen]
impl UnitTest {
    pub fn new(id: u32, description: String) -> Self {
        Self {
            id,
            description,
            status: UnitTestStatus::Unreached,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn get_description(&self) -> String {
        self.description.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn get_id(&self) -> u32 {
        self.id
    }

    #[wasm_bindgen(getter)]
    pub fn get_status(&self) -> UnitTestStatus {
        self.status
    }
}

impl UnitTest {
    pub fn set_status(&mut self, status: UnitTestStatus) {
        self.status = status
    }
}

impl Display for UnitTest {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "[Unit tests] {} -> {}",
            &self.description,
            match &self.status {
                UnitTestStatus::Unreached => Yellow.paint("Unreached"),
                UnitTestStatus::Failed => Red.paint("Failed"),
                UnitTestStatus::Succeed => Green.paint("Succeed"),
            }
        )
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum UnitTestStatus {
    Unreached,
    Failed,
    Succeed,
}

#[derive(Tsify)]
#[wasm_bindgen(skip_typescript)]
#[derive(Clone)]
pub struct UnitTestUpdateStatus {
    id: u32,
    status: UnitTestStatus,
    fail_message: Option<String>,
}

#[wasm_bindgen]
impl UnitTestUpdateStatus {
    pub fn new(id: u32, status: UnitTestStatus, fail_message: Option<String>) -> Self {
        Self {
            id,
            status,
            fail_message,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn get_id(&self) -> u32 {
        self.id
    }

    #[wasm_bindgen(getter)]
    pub fn get_status(&self) -> UnitTestStatus {
        self.status
    }

    #[wasm_bindgen(getter)]
    pub fn get_fail_message(&self) -> Option<String> {
        self.fail_message.clone()
    }
}

pub struct UnitTestJson {
    description: String,
    expect: JsonTarget,
    with: JsonTarget,
    operator: String,
    id: u32,
}

impl Clone for UnitTestJson {
    fn clone(&self) -> Self {
        Self {
            description: self.description.clone(),
            expect: self.expect.clone(),
            with: self.with.clone(),
            operator: self.operator.clone(),
            id: self.id,
        }
    }
}

impl NewJsonOperation for UnitTestJson {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Unit"),
            json {
                description => as_str,
                expect,
                with,
                operator => as_str,
                id => as_u64,
            }
        );

        let id = id as u32;

        let expect = parse_json_target(&expect).map_err(|e| {
            e.add_sim_trace(&"Parse Unit tests -> Parse expect param".to_string())
                .add_id(id)
        })?;

        let with = parse_json_target(&with).map_err(|e| {
            e.add_sim_trace(&"Parse Compare -> Parse with param".to_string())
                .add_id(id)
        })?;

        Ok(Self {
            description: description.to_string(),
            expect,
            with,
            operator: operator.to_string(),
            id,
        })
    }
}

impl BuildJsonOperation for UnitTestJson {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast,
    ) -> Result<RunTimeOperation, Stop> {
        let targets = get_cmp_targets(
            &self.expect,
            &self.with,
            &interface,
            template,
            &registry,
            &channel,
        )
        .map_err(|e| e.add_id(self.id))?;

        let (compare, with) = targets.clone();

        let expect = box_cmp(
            self.id,
            &compare,
            &with,
            &self.operator,
            &interface,
            template,
            &registry,
            &channel,
        )
        .map_err(|e| e.add_id(self.id))?;

        let operator = self.operator.clone();

        let description = self.description.clone();
        let id = self.id;

        if !registry.should_ignore_operation() {
            channel.add_unit_test(&UnitTest::new(
                id,
                self.description.clone(),
            ));
        }

        let description_clone = description.clone();

        Ok(Box::new(Operation::new(
            MaybeHeapOrStatic(Some(HeapOrStatic::Closure(Rc::new(RefCell::new(
                move || format!("Test {}", description_clone),
            ))))),
            move |channel| {
                let curr_section = channel
                    .get_cycle_stack()
                    .borrow_mut()
                    .get_current_section()
                    .unwrap();

                curr_section
                    .borrow_mut()
                    .insert_log(&Blue.paint("[Unit Test]: Running ...").to_string());

                if expect(channel)? {
                    curr_section
                        .borrow_mut()
                        .insert_log(&Green.paint("[Unit Test]: -> Passed").to_string());
                    channel.add_unit_test_status(&UnitTestUpdateStatus::new(
                        id,
                        UnitTestStatus::Succeed,
                        None,
                    ));
                    Ok(())
                } else {
                    curr_section
                        .borrow_mut()
                        .insert_log(&Red.paint("[Unit Test]: -> Failed").to_string());
                    channel.add_unit_test_status(&UnitTestUpdateStatus::new(
                        id,
                        UnitTestStatus::Failed,
                        Some(format!("Expected {} to be {} {}", compare, operator, with)),
                    ));
                    Ok(())
                }
            },
            None,
            false,
            self.id,
        )))
    }
}
