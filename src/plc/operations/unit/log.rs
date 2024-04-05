﻿use crate::parser::body::json_target::JsonTarget;
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::internal::template_impl::TemplateMemory;
use crate::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation,
};
use crate::registry::global::pointer::GlobalPointer;
use crate::registry::local::pointer::LocalPointer;
use crate::registry::registry::Kernel;
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::plc::operations::unit::test::UnitTestJson;

#[derive(Clone)]
enum StringOrJsonTarget {
    String(String),
    Indice(JsonTarget),
}

enum StringOrRuntimeTarget {
    String(String),
    Global(GlobalPointer),
    Local(LocalPointer),
    Constant(Map<String, Value>),
}

impl Display for StringOrRuntimeTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StringOrRuntimeTarget::String(string) => write!(f, "{}", string),
            StringOrRuntimeTarget::Global(global) => write!(f, "{}", global),
            StringOrRuntimeTarget::Local(local) => write!(f, "{}", local),
            StringOrRuntimeTarget::Constant(constant) => write!(f, "{:?}", constant),
        }
    }
}

#[derive(Clone)]
pub struct UnitLog {
    fmt: Vec<StringOrJsonTarget>,
    trace: Option<FileTrace>,
}

impl FileTraceBuilder for UnitLog {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl NewJsonOperation for UnitLog {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
           format!("Parse log"),
           json {
                message => as_str,
                format => as_array,
                trace? => as_object,
            }
        );

        let trace = match trace {
            None => None,
            Some(a) => Self::build_trace(a),
        };

        let mut fmt = Vec::new();
        let mut curr_index = 0;
        let indices = message.match_indices("{}").collect::<Vec<_>>();

        if indices.is_empty() {
            fmt.push(StringOrJsonTarget::String(message.into()))
        } else {
            for i in 0..indices.len() {
                let indice = indices.get(i).ok_or(error!(format!("Invalid index in log argument {}", i)))?;
                let target = format.get(i).ok_or(error!(format!("Invalid index in log argument {}", i)))?;
                let str;
                if curr_index == 0 {
                    str = &message[curr_index..indice.0];
                } else {
                    str = &message[curr_index + 2..indice.0];
                }
                fmt.push(StringOrJsonTarget::String(str.into()));
                fmt.push(StringOrJsonTarget::Indice(parse_json_target(target)?));
                curr_index = indice.0;
            }
            let str = &message[curr_index + 2..message.len()];
            fmt.push(StringOrJsonTarget::String(str.into()));
        }

        Ok(Self { fmt, trace })
    }
}

impl BuildJsonOperation for UnitLog {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let mut messages = Vec::new();
        self.fmt.iter().try_for_each(|s| {
            match s {
                StringOrJsonTarget::String(string) => {
                    messages.push(StringOrRuntimeTarget::String(string.into()))
                }
                StringOrJsonTarget::Indice(indice) => {
                    if indice.is_global() {
                        messages.push(StringOrRuntimeTarget::Global(
                            indice
                                .solve_as_global_pointer(registry)
                                .ok_or(error!(format!(
                                    "Could not find a global reference to log {}",
                                    indice
                                )))?
                                .clone(),
                        ))
                    } else if indice.is_local() || indice.is_local_out() || indice.is_inner() {
                        messages.push(StringOrRuntimeTarget::Local(
                            indice
                                .solve_as_local_pointer(interface, template, registry, channel)
                                .ok_or(error!(format!(
                                    "Could not find a local reference to log {} in {}",
                                    indice, interface
                                )))?
                                .clone(),
                        ))
                    } else if indice.is_constant() {
                        messages.push(StringOrRuntimeTarget::Constant(
                            indice
                                .get_raw_constant()
                                .ok_or(error!(format!(
                                    "Could not find a constant reference to log {}",
                                    indice
                                )))?
                                .clone(),
                        ))
                    }
                }
            }
            Ok(())
        })?;

        Ok(Box::new(Operation::new(
            &"Log",
            move |channel| {
                let mut display = String::new();
                messages.iter().try_for_each(|m| {
                    display += &format!("{}", m);
                    Ok(())
                })?;

                let curr_section = channel
                    .get_cycle_stack()
                    .borrow_mut()
                    .get_current_section()
                    .unwrap();
                curr_section
                    .borrow_mut()
                    .insert_log(&display);
                channel.add_message(&display);
                Ok(())
            },
            None,
            false,
            &self.trace
        )))
    }
}
