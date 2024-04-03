use crate::error;
use crate::plc::operations::operations::{Operation, RunTimeOperation};
use crate::plc::primitives::family_traits::{Primitive, AsMutPrimitive};
use crate::plc::primitives::primitive_traits::PrimitiveTrait;
use crate::container::error::error::Stop;
use camelpaste::paste;
use std::fmt::Display;
use crate::container::broadcast::broadcast::Broadcast;
use crate::parser::trace::trace::FileTrace;
use crate::registry::registry::Kernel;
use crate::plc::primitives::family_traits::MetaData;

macro_rules! primitive_links {
    ($({
        $primitive: ident,
        [$($associated: ident),+]
    }),+
    ) => {
        pub fn box_set_plc_primitive<T: 'static + MetaData + Primitive + AsMutPrimitive + Clone + Display, Y : 'static + MetaData + Primitive + Clone + Display>(variable1: &T, variable2: &Y, trace: &Option<FileTrace>, kernel: &Kernel) -> Result<RunTimeOperation, Stop>{
            kernel.check_filtered_operation(&"assign", variable1, variable2)?;
            paste! {
                $(
                    if variable1.[<is_$primitive>]() {
                        $(
                           if variable2.[<is_$associated>]() {
                               let mut o1_clone = variable1.clone();
                               let o2_clone = variable2.clone();
                               let trace = trace.clone();

                               return Ok(Box::new(Operation::new(move |channel| {
                                    o1_clone.[<set_$primitive>](
                                        o2_clone.[<as_$associated>](channel)?.try_into()
                                            .map_err(|_| error!(format!("Failed assignment of {} with {}", o1_clone, o2_clone)))?,
                                        channel
                                    )?;
                                    Ok(())
                               }, None, false, &trace)))
                           }
                        )+
                    }
                )+
                Err(error!(format!("Invalid assignment: Can not set {} with {}", variable1, variable2)))
            }
        }
        pub fn box_set_plc_primitive_default_once<T: 'static + MetaData + Primitive + AsMutPrimitive + Clone + Display, Y : 'static + MetaData + Primitive + Clone + Display>(variable1: &T, variable2: &Y) -> Result<Box<dyn FnMut(&Broadcast) -> Result<(), Stop>>, Stop> {
            paste! {
                $(
                    if variable1.[<is_$primitive>]() {
                        $(
                           if variable2.[<is_$associated>]() {
                               let mut o1_clone = variable1.clone();
                               let o2_clone = variable2.clone();

                               return Ok(Box::new(move |channel| {
                                    o1_clone.[<set_default_$primitive>](
                                        o2_clone.[<as_$associated>](channel)?.try_into()
                                        .map_err(|_e| error!(format!("Invalid assignment: Can not set default {} with {}", o1_clone, o2_clone)))?
                                    )?;
                                    Ok(())
                               }))
                           }
                        )+
                    }
                )+
                Err(error!(format!("Invalid assignment: Can not set default {} with {}", variable1, variable2)))
            }
        }
    };
}

primitive_links!(
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
