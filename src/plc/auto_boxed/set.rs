use std::fmt::Display;
use crate::container::broadcast::broadcast::Broadcast;
use crate::plc::complex::boxed::set::{box_set_plc_complex, box_set_plc_complex_default_once};
use crate::plc::operations::operations::RunTimeOperation;
use crate::plc::primitives::boxed::set::{box_set_plc_primitive, box_set_plc_primitive_default_once};
use crate::plc::primitives::family_traits::{WithTypeFamily, IsFamily, WithMutFamily, WithRefFamily, AsMutPrimitive, Primitive, MetaData};
use crate::container::error::error::Stop;
use crate::parser::trace::trace::FileTrace;
use crate::registry::registry::Kernel;

pub fn box_set_auto<T: 'static + MetaData + WithMutFamily + WithTypeFamily + Clone + Display + IsFamily + AsMutPrimitive + Primitive, Y: 'static + MetaData + WithRefFamily + WithTypeFamily + Clone + Display + IsFamily + Primitive>(o1: &T, o2: &Y, trace: &Option<FileTrace>, registry: &Kernel) -> Result<RunTimeOperation, Stop> {
    if o1.is_complex() && o2.is_complex() {
        box_set_plc_complex(o1, o2, trace, registry)
    }
    else {
        box_set_plc_primitive(o1, o2, trace, registry)
    }
}

pub fn box_set_auto_default_once<T: 'static + MetaData + WithMutFamily + WithTypeFamily + Clone + Display + IsFamily + AsMutPrimitive + Primitive,
    Y: 'static + MetaData + WithRefFamily + WithTypeFamily + Clone + Display + IsFamily + Primitive>(o1: &T, o2: &Y) -> Result<Box<dyn FnMut (&Broadcast) -> Result <(), Stop >>, Stop> {
    if o1.is_complex() && o2.is_complex() {
        box_set_plc_complex_default_once(o1, o2)
    }
    else {
        box_set_plc_primitive_default_once(o1, o2)
    }
}