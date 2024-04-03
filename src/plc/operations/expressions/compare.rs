﻿use std::ops::Deref;
use crate::parser::body::json_target::JsonTarget;
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::internal::template_impl::TemplateMemory;
use crate::plc::operations::operations::{BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation, RuntimeOperationTrait};
use crate::plc::primitives::boolean::bool::Bool;
use crate::plc::primitives::boolean::plc_bool::PlcBool;
use crate::plc::primitives::boxed::partial::{box_ord_plc_primitive};
use crate::plc::primitives::family_traits::{IsFamily, WithMutFamily, WithRefFamily};
use crate::plc::primitives::primitive_traits::PrimitiveTrait;
use crate::registry::local::pointer::LocalPointer;
use crate::registry::local::r#type::LocalType;
use crate::registry::registry::Kernel;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use serde_json::{Map, Value};
use crate::parser::body::body::parse_json_target;
use crate::registry::any::any_type::AnyRefType;

#[derive(Clone)]
pub struct Compare {
    compare: JsonTarget,
    with: JsonTarget,
    operator: String,
    cont: Option<String>,
    cont_with: Option<Map<String, Value>>,
    trace: Option<FileTrace>
}


impl FileTraceBuilder for Compare {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl NewJsonOperation for Compare {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop> {
        key_reader!(
            format!("Parse Compare -> interface"),
            json {
                compare,
                with,
                operator => as_str,
                cont? => as_str,
                trace? => as_object,
            }
        );

        let trace = match trace {
            None => None,
            Some(a) => Self::build_trace(a),
        };

        let compare = parse_json_target(&compare).map_err(|e| {
            e.add_sim_trace(&format!("Parse Compare -> Parse first param"))
                .maybe_file_trace(&trace)
        })?;

        let with = parse_json_target(&with).map_err(|e| {
            e.add_sim_trace(&format!("Parse Compare -> Parse with param"))
                .maybe_file_trace(&trace)
        })?;

        let mut cont_with = None;

        if cont.is_some() && json.contains_key("cont_with") {
            if let Some(a) = json["cont_with"].as_object() {
                cont_with = Some(a.clone());
            }
        }

        Ok(Self {
            compare,
            with,
            operator: operator.to_string(),
            cont: cont.map(|h| h.to_string()).or(None),
            cont_with,
            trace,
        })
    }
}

pub fn get_cmp_targets(
    compare: &JsonTarget,
    with: &JsonTarget,
    interface: &SectionInterface,
    template: Option<&TemplateMemory>,
    registry: &Kernel,
    channel: &Broadcast,
) -> Result<(AnyRefType, AnyRefType), Stop> {
    let compare = compare
        .solve_to_ref(interface, template, None, registry, channel)
        .map_err(|e| e.add_sim_trace(&format!("Build compare -> target")))?;

    let with = match &compare {
        AnyRefType::Local(a) => with
            .solve_to_ref(interface, template, Some(a.as_ref().borrow().deref().clone()), registry, channel)
            .map_err(|e| e.add_sim_trace(&format!("Build compare -> with"))),

        AnyRefType::Constant(a) => with
            .solve_to_ref(interface, template, Some(a.clone().into()), registry, channel)
            .map_err(|e| e.add_sim_trace(&format!("Build compare -> with"))),

        AnyRefType::Operation(ref a) => {
            match a.get_return_pointer() {
                None => Err(error!(format!("Cannot compare with a void operation"), format!("Build compare -> with"))),
                Some(a) => with
                    .solve_to_ref(interface, template, Some(a.as_ref().borrow().deref().clone()), registry, channel)
                    .map_err(|e| e.add_sim_trace(&format!("Build compare -> with")))
            }
        }
    }?;
    Ok((compare, with))
}

pub fn box_cmp(
    compare: &AnyRefType,
    with: &AnyRefType,
    operator: &str,
    interface: &SectionInterface,
    template: Option<&TemplateMemory>,
    registry: &Kernel,
    channel: &Broadcast,
) -> Result<impl Fn(&Broadcast) -> Result<bool, Stop>, Stop> {
    let op: Result<Box<dyn Fn(&Broadcast) -> Result<bool, Stop>>, Stop> = match operator {
        "=" => {
            let cmp = box_ord_plc_primitive(compare, with, &None, registry)
                .map_err(|e| e.add_sim_trace(&format!("Build compare -> compare operation")))?;
            Ok(Box::new(move |channel: &Broadcast| Ok(cmp(channel)?.unwrap().is_eq())))
        }
        "<>" => {
            let cmp = box_ord_plc_primitive(compare, with, &None, registry)
                .map_err(|e| e.add_sim_trace(&format!("Build compare -> compare operation")))?;
            Ok(Box::new(move |channel: &Broadcast| Ok(cmp(channel)?.unwrap().is_ne())))
        }
        "<" => {
            let cmp = box_ord_plc_primitive(compare, with, &None, registry)
                .map_err(|e| e.add_sim_trace(&format!("Build compare -> compare operation")))?;
            Ok(Box::new(move |channel: &Broadcast| Ok(cmp(channel)?.unwrap().is_lt())))
        }
        ">" => {
            let cmp = box_ord_plc_primitive(compare, with, &None, registry)
                .map_err(|e| e.add_sim_trace(&format!("Build compare -> compare operation")))?;
            Ok(Box::new(move |channel: &Broadcast| Ok(cmp(channel)?.unwrap().is_gt())))
        }
        "<=" => {
            let cmp = box_ord_plc_primitive(compare, with, &None, registry)
                .map_err(|e| e.add_sim_trace(&format!("Build compare -> compare operation")))?;
            Ok(Box::new(move |channel: &Broadcast| Ok(cmp(channel)?.unwrap().is_le())))
        }
        ">=" => {
            let cmp = box_ord_plc_primitive(compare, with, &None, registry)
                .map_err(|e| e.add_sim_trace(&format!("Build compare -> compare operation")))?;
            Ok(Box::new(move |channel: &Broadcast| Ok(cmp(channel)?.unwrap().is_ge())))
        }
        _ => Err(error!(format!("Invalid operator for compare {}", operator))),
    };
    op
}


impl BuildJsonOperation for Compare {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast,
    ) -> Result<RunTimeOperation, Stop> {
        let targets = get_cmp_targets(
            &self.compare,
            &self.with,
            &interface,
            template,
            &registry,
            &channel,
        )
            .map_err(|e| e.maybe_file_trace(&self.trace))?;

        let (compare, with) = targets.clone();

        let first = box_cmp(
            &compare,
            &with,
            &self.operator,
            &interface,
            template,
            &registry,
            &channel,
        )
            .map_err(|e| e.maybe_file_trace(&self.trace))?;

        let return_ptr = Some(LocalPointer::new(LocalType::PlcBool(PlcBool::Bool(
            Bool::new(&false)?,
        ))));
        let return_ptr_clone = return_ptr.clone();

        match &self.cont {
            Some(operator) => {
                let as_cont_with = match &self.cont_with {
                    Some(target) => Ok(target),
                    None => Err(error!(format!("A cont operator was defined, but no cont found"))),
                }?;

                key_reader!(
                    format!("Invalid compare next"),
                    as_cont_with {
                        src => as_object,
                    }
                );

                let other = Compare::new(&src)?.build(interface, template, registry, channel)?;
                if !other.is_plc_bool() {
                    return Err(error!(format!(
                        "Invalid compare return type, expect PlcBool, got {}",
                        other
                    )));
                }

                match operator.as_str() {
                    "AND" | "&" => Ok(Box::new(Operation::new(
                        move |channel| {
                            other.with_plc_bool(channel, |a| {
                                return_ptr_clone.as_ref().unwrap().with_mut_plc_bool(
                                    channel,
                                    &mut |c| {
                                        c.as_mut_bool()?
                                            .set(first(channel)? & a.as_bool()?.get(channel)?, channel)?;
                                        Ok(())
                                    },
                                )?
                            })?
                        },
                        return_ptr,
                        false,
                        &self.trace
                    ))),
                    "OR" => Ok(Box::new(Operation::new(
                        move |channel| {
                            other.with_plc_bool(channel, |a| {
                                return_ptr_clone.as_ref().unwrap().with_mut_plc_bool(
                                    channel,
                                    &mut |c| {
                                        c.as_mut_bool()?
                                            .set(first(channel)? | a.as_bool()?.get(channel)?, channel)?;
                                        Ok(())
                                    },
                                )?
                            })?
                        },
                        return_ptr,
                        false,
                        &self.trace
                    ))),
                    "XOR" => Ok(Box::new(Operation::new(
                        move |channel| {
                            other.with_plc_bool(channel, |a| {
                                return_ptr_clone.as_ref().unwrap().with_mut_plc_bool(
                                    channel,
                                    &mut |c| {
                                        c.as_mut_bool()?.set(
                                            matches!(
                                                (first(channel)?, a.as_bool()?.get(channel)?),
                                                (true, false) | (false, true)
                                            ),
                                            channel,
                                        )?;
                                        Ok(())
                                    },
                                )?
                            })?
                        },
                        return_ptr,
                        false,
                        &self.trace
                    ))),
                    _ => Err(error!(format!("Invalid Cwith operator"))),
                }
            }
            None => Ok(Box::new(Operation::new(
                move |channel| {
                    return_ptr_clone
                        .as_ref()
                        .unwrap()
                        .with_mut_plc_bool(channel, &mut |r| {
                            r.as_mut_bool()?.set(first(channel)?, channel)?;
                            Ok(())
                        })?
                },
                return_ptr,
                false,
                &self.trace
            ))),
        }
    }
}
