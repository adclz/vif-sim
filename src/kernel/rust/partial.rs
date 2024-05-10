use crate::error;
use crate::kernel::plc::types::primitives::traits::primitive_traits::PrimitiveTrait;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use camelpaste::paste;
use std::cmp::Ordering;
use std::fmt::Display;
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive};
use crate::kernel::registry::Kernel;
use crate::kernel::plc::types::primitives::traits::meta_data::MetaData;

pub fn ord<T: Sized + PartialOrd + Clone>(o1: T, o2: T) -> Option<Ordering> {
    o1.partial_cmp(&o2)
}

macro_rules! box_ord_primitive {
    ($({
        $primitive: ident,
        [$($associated: ident),+]
    }),+
    ) => {
        pub fn box_ord_plc_primitive<T: 'static + MetaData + Primitive + Clone + Display, 
        Y : 'static + MetaData + Primitive + Clone + Display>(variable1: &T, variable2: &Y, trace: u64, kernel: &Kernel) -> Result<Box<dyn Fn(&Broadcast) -> Result<Option<Ordering>, Stop>>, Stop>{
            paste! {
                kernel.check_filtered_operation(&"cmp", variable1, variable2)?;
                $(
                    if variable1.[<is_$primitive>]() {
                        $(
                           if variable2.[<is_$associated>]() {
                               let o1_clone = variable1.clone();
                               let o2_clone = variable2.clone();
                               let trace = trace.clone();

                               return Ok(Box::new(move |channel| {
                                    Ok(ord(o1_clone.[<as_$primitive>](channel).unwrap(),
                                    o2_clone.[<as_$associated>](channel)?.try_into()
                                        .map_err(|_| error!(format!("Failed comparison of {} with {}", o1_clone, o2_clone)))
                                        .map_err(|e| e.add_id(trace))?
                                   ))
                               }))
                           }
                        )+
                    }
                )+
                Err(error!(format!("Invalid operation: Can not compare {} with {}", variable1, variable2)))
            }
        }
    };
}

box_ord_primitive!(
    { bool, [bool] },
    { u8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u32, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u64, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i32, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i64, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { f32, [f32] },
    { f64, [f32, f64] },
    { str256, [str256] },
    { char, [char] },
    { wstr256, [wstr256] },
    { wchar, [wchar] }
);