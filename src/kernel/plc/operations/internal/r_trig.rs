use crate::key_reader;
use crate::parser::body::json_target::JsonTarget;
use crate::kernel::plc::interface::section_interface::SectionInterface;
use crate::kernel::plc::internal::template_impl::TemplateMemory;
use crate::kernel::plc::operations::operations::{
    BuildJsonOperation, NewJsonOperation, Operation, RunTimeOperation,
};
use crate::kernel::plc::types::primitives::boolean::bool::Bool;
use crate::kernel::plc::types::primitives::boolean::plc_bool::PlcBool;
use crate::kernel::plc::types::primitives::traits::family_traits::{WithMutFamily, WithRefFamily};
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic};
use crate::kernel::plc::types::primitives::traits::primitive_traits::PrimitiveTrait;
use crate::kernel::arch::local::pointer::LocalPointer;
use crate::kernel::arch::local::r#type::LocalType;
use crate::kernel::registry::Kernel;
use crate::container::error::error::Stop;
use serde_json::{Map, Value};
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};
use std::rc::Rc;
use crate::parser::body::body::parse_json_target;
use crate::container::broadcast::broadcast::Broadcast;

#[derive(Clone)]
pub struct R_Trig {
    input: JsonTarget,
    stat_bit: Rc<RefCell<bool>>,
    id: u32,
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
                id => as_u64,
            }
        );

        let id = id as u32;

        let input = parse_json_target(input).map_err(|e| {
            e.add_sim_trace("Build #R_Trig -> input")
                .add_id(id)
        })?;

        Ok(Self {
            input,
            stat_bit: Rc::new(RefCell::new(false)),
            id,
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

        let return_trig = LocalPointer::new(LocalType::PlcBool(PlcBool::Bool(Bool::new_default(0))));
        let return_trig_clone = return_trig.clone();

        Ok(Box::new(Operation::new(
            MaybeHeapOrStatic(Some(HeapOrStatic::Static(&"Rising Edge"))),
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
            self.id
        )))
    }
}
