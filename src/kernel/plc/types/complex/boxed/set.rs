use crate::container::broadcast::broadcast::Broadcast;
use crate::kernel::plc::operations::operations::{Operation, RunTimeOperation, RuntimeOperationTrait};
use crate::kernel::rust::auto_set::{box_set_auto, box_set_auto_default_once};
use crate::container::error::error::Stop;
use crate::error;
use crate::kernel::plc::types::complex::instance::public::PublicInstanceAccessors;
use crate::kernel::plc::types::primitives::traits::family_traits::{IsFamily, WithMutFamily, WithRefFamily, WithTypeFamily};
use crate::kernel::plc::types::primitives::traits::meta_data::{HeapOrStatic, MaybeHeapOrStatic};
use crate::kernel::plc::types::primitives::traits::primitive_traits::{Primitive, AsMutPrimitive};
use crate::kernel::registry::Kernel;

pub fn box_set_plc_complex<T: WithMutFamily + WithTypeFamily + Clone + IsFamily + AsMutPrimitive + Primitive, 
    Y: WithRefFamily + WithTypeFamily + Clone + IsFamily + Primitive>(o1: &T, o2: &Y, trace: u32, registry: &Kernel) -> Result<RunTimeOperation, Stop> {
    let mut opexs = vec![];

    if o1.is_plc_struct() && o2.is_plc_struct() {
        opexs = o1.with_type_plc_struct(|a| {
            a.get_interface().iter().map(|(name, i)| {
                o2.with_type_plc_struct(|b| {
                    match b.get_interface().get(name) {
                        Some(p) => box_set_auto(i, p, trace, registry),
                        None => Err(error!(format!("Structs are not equal")))
                    }
                })?
            }).collect::<Result<Vec<RunTimeOperation>, Stop>>()
        })??;
    } else if o1.is_plc_array() && o2.is_plc_array() {
        for i in 0..o1.with_type_plc_array(|a| a.get_interface().len())? {
            opexs.push(box_set_auto(
                &o1.with_type_plc_array(|a| a.try_get_nested(&[i]).unwrap())?,
                &o2.with_type_plc_array(|a| a.try_get_nested(&[i]).unwrap())?,
                trace,
                registry,
            )?);
        }
    } else if o1.is_fb_instance() && o2.is_fb_instance() {
        o1.with_type_fb_instance(|a| {
            a.get_interface().iter().try_fold(vec![], |_a, (s, f)| {
                f.iter().map(|(name, pointer)| {
                    o2.with_type_fb_instance(|a| match a.get_interface().get(s).unwrap().try_get_nested(vec![name.clone()].as_slice()) {
                        None => Err(error!(format!("Invalid set, could not find {}", name), format!("Set instance pointers"))),
                        Some(some) => box_set_auto(pointer, &some, trace, registry)
                    })?
                }).collect::<Result<Vec<RunTimeOperation>, Stop>>()
            })
        })??;
    } else {
        return Err(error!(format!("Provided type is not a complex type or invalid")));
    };

    Ok(Box::new(Operation::new(
        MaybeHeapOrStatic(Some(HeapOrStatic::Static(&"Assign"))),
        move |channel| {
        for closure in &opexs {
            closure.with_void(channel)?;
        }
        Ok(())
    }, None, false, trace)))
}

pub fn box_set_plc_complex_default_once<T: WithMutFamily + WithTypeFamily + Clone + IsFamily + AsMutPrimitive + Primitive, Y: WithRefFamily + WithTypeFamily + Clone + IsFamily + Primitive>
(o1: &T, o2: &Y) -> Result<Box<dyn FnMut(&Broadcast) -> Result<(), Stop>>, Stop> {
    let mut opexs = vec![];

    if o1.is_plc_struct() && o2.is_plc_struct() {
        opexs = o1.with_type_plc_struct(|a| {
            a.get_interface().iter().map(|(name, i)| {
                o2.with_type_plc_struct(|b| {
                    match b.get_interface().get(name) {
                        Some(p) => box_set_auto_default_once(i, p),
                        None => Err(error!(format!("Structs are not equal")))
                    }
                })?
            }).collect::<Result<Vec<Box<dyn FnMut(&Broadcast) -> Result<(), Stop>>>, Stop>>()
        })??;
    } else if o1.is_plc_array() && o2.is_plc_array() {
        for i in 0..o1.with_type_plc_array(|a| a.get_interface().len())? {
            opexs.push(box_set_auto_default_once(
                &o1.with_type_plc_array(|a| a.try_get_nested(&[i]).unwrap())?,
                &o2.with_type_plc_array(|a| a.try_get_nested(&[i]).unwrap())?,
            )?);
        }
    } else if o1.is_fb_instance() && o2.is_fb_instance() {
        o1.with_type_fb_instance(|a| {
            a.get_interface().iter().try_fold(vec![], |_a, (s, f)| {
                f.iter().map(|(name, pointer)| {
                    o2.with_type_fb_instance(|a| match a.get_interface().get(s).unwrap().try_get_nested(vec![name.clone()].as_slice()) {
                        None => Err(error!(format!("Invalid set, could not find {}", name), format!("Set instance pointers"))),
                        Some(some) => box_set_auto_default_once(pointer, &some)
                    })?
                }).collect::<Result<Vec<Box<dyn FnMut(&Broadcast) -> Result<(), Stop>>>, Stop>>()
            })
        })??;
    } else {
        return Err(error!(format!("Provided type is not a complex type or invalid")));
    };

    Ok(Box::new(move |channel| {
        for closure in &mut opexs {
            closure(channel)?;
        }
        Ok(())
    }))
}