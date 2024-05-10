use std::fmt::Display;
use crate::container::broadcast::broadcast::Broadcast;
use crate::kernel::plc::types::complex::boxed::set::{box_set_plc_complex, box_set_plc_complex_default_once};
use crate::kernel::plc::operations::operations::RunTimeOperation;
use crate::kernel::rust::set::{box_set_plc_primitive, box_set_plc_primitive_default_once};
use crate::kernel::plc::types::primitives::traits::family_traits::{WithTypeFamily, IsFamily, WithMutFamily, WithRefFamily};
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::meta_data::MetaData;
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive};
use crate::kernel::registry::Kernel;

pub fn box_set_auto<T: 'static + MetaData + WithMutFamily + WithTypeFamily + Clone + Display + IsFamily + AsMutPrimitive + Primitive, 
    Y: 'static + MetaData + WithRefFamily + WithTypeFamily + Clone + Display + IsFamily + Primitive>(o1: &T, o2: &Y, trace: u64, registry: &Kernel) -> Result<RunTimeOperation, Stop> {
    if o1.is_complex() && o2.is_complex() {
        box_set_plc_complex(o1, o2, trace, registry)
    }
    else {
        box_set_plc_primitive(o1, o2, trace, false, registry)
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