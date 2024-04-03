use crate::key_reader;
use crate::parser::body::json_target::JsonTarget;
use crate::parser::trace::trace::{FileTrace, FileTraceBuilder};
use crate::plc::interface::section_interface::SectionInterface;
use crate::plc::internal::template_impl::TemplateMemory;
use crate::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation,
};
use crate::plc::primitives::boolean::bool::Bool;
use crate::plc::primitives::boolean::plc_bool::PlcBool;
use crate::plc::primitives::family_traits::{WithMutFamily, WithRefFamily};
use crate::plc::primitives::primitive_traits::PrimitiveTrait;
use crate::registry::local::pointer::LocalPointer;
use crate::registry::local::r#type::LocalType;
use crate::registry::registry::Kernel;
use crate::container::error::error::Stop;
use serde_json::{Map, Value};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;

#[derive(Clone)]
pub struct R_Trig {
    input: JsonTarget,
    stat_bit: Rc<RefCell<bool>>,
    trace: Option<FileTrace>,
}

impl FileTraceBuilder for R_Trig {
    fn get_trace(&self) -> &Option<FileTrace> {
        &self.trace
    }
}

impl NewJsonOperation for R_Trig {
    fn new(json: &Map<String, Value>) -> Result<Self, Stop>
    where
        Self: Clone,
    {
        key_reader!(
            format!("Parse #R_Trig"),
            json {
                input,
                trace? => as_object,
            }
        );

        let trace = match trace {
            None => None,
            Some(a) => Self::build_trace(a),
        };

        let input = parse_json_target(input).map_err(|e| {
            e.add_sim_trace("Build #R_Trig -> input")
                .maybe_file_trace(&trace)
        })?;

        Ok(Self {
            input,
            stat_bit: Rc::new(RefCell::new(false)),
            trace,
        })
    }
}

impl BuildJsonOperation for R_Trig {
    fn build(
        &self,
        interface: &SectionInterface,
        template: Option<&TemplateMemory>,
        registry: &Kernel,
        channel: &Broadcast
    ) -> Result<RunTimeOperation, Stop> {
        let input = self
            .input
            .solve_as_local_pointer(interface, template, registry, channel)
            .unwrap();
        let input = input.clone();
        let stat_bit = self.stat_bit.clone();

        let return_trig = LocalPointer::new(LocalType::PlcBool(PlcBool::Bool(Bool::default())));
        let return_trig_clone = return_trig.clone();

        Ok(Box::new(Operation::new(
            move |channel| {
                let clk_deref = input.with_plc_bool(channel, |a| Ok(a.as_bool()?.get(channel)?))??;
                let stat_bit_deref = *stat_bit.borrow().deref();

                if (clk_deref != stat_bit_deref) & clk_deref {
                    return_trig_clone.with_mut_plc_bool(channel, &mut |a| {
                        a.as_mut_bool()?.set(true, channel)?;
                        Ok(())
                    })?
                } else {
                    return_trig_clone.with_mut_plc_bool(channel, &mut |a| {
                        a.as_mut_bool()?.set(false, channel)?;
                        Ok(())
                    })?
                }?;
                *stat_bit.borrow_mut().deref_mut() = clk_deref;
                Ok(())
            },
            Some(return_trig),
            false,
            &self.trace
        )))
    }
}
