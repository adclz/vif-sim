use camelpaste::paste;
use crate::container::error::error::Stop;
use crate::error;
use std::fmt::Display;
use crate::parser::trace::trace::FileTrace;
use crate::plc::primitives::primitive_traits::PrimitiveTrait;
use crate::plc::operations::operations::{Operation, RunTimeOperation};
use crate::plc::primitives::family_traits::{Primitive, AsMutPrimitive};
use crate::registry::local::pointer::LocalPointer;
use crate::registry::local::r#type::{IntoLocalType};
use std::ops::{DerefMut};
use crate::registry::registry::Kernel;
use crate::plc::primitives::family_traits::MetaData;
use crate::plc::primitives::floats::checked_float::CheckedFloat;

macro_rules! box_create_checked_operation_primitive {
    ($op_fn: expr,
     $({
        $primitive: ident,
        [$($associated: ident $(check $signed: ident)?),+]
    }),+
    ) => {
        paste! {
            pub fn [<box_$op_fn _plc_primitive>]<T: 'static + MetaData + Primitive + Clone + Display + IntoLocalType, Y : 'static + MetaData + Primitive + Clone + Display>(variable1: &T, variable2: &Y, trace: &Option<FileTrace>, kernel: &Kernel) -> Result<RunTimeOperation, Stop>{
                kernel.check_filtered_operation(&stringify!($op_fn), variable1, variable2)?;
                $(
                    if variable1.[<is_$primitive>]() {
                        $(
                           if variable2.[<is_$associated>]() {
                               let return_ptr = match kernel.check_return_operation(&stringify!($op_fn), variable1, variable2) {
                                   Some(a) => LocalPointer::new(a.transform()?),
                                   None => LocalPointer::new(variable1.transform()?)
                               };
                               let return_ptr_clone = return_ptr.clone();
                               let o1_clone = variable1.clone();
                               let o2_clone = variable2.clone();
                               let trace = trace.clone();

                               return Ok(Box::new(Operation::new(move |channel| {
                                    let result = (o1_clone.[<as_$primitive>](channel)?)
                                        .[<checked_$op_fn $(_$signed)?>](o2_clone.[<as_$associated>](channel)?
                                            .try_into()
                                            .map_err(|_| error!(format!("Failed {} of {} with {}", stringify!($op_fn), o1_clone, o2_clone)))?
                                    )
                                    .ok_or(error!(format!("Invalid operation: Can not {} {} with {}", stringify!($op_fn), o1_clone, o2_clone)))?;

                                    return_ptr.as_ref().borrow_mut().deref_mut().[<set_$primitive>](result, channel)?;
                                    Ok(())
                               }, Some(return_ptr_clone), false, &trace)))
                           }
                        )+
                    }
                )+
                Err(error!(format!("Invalid operation: Can not {} {} with {}", stringify!($op_fn), variable1, variable2)))
            }
        }
    };
}

macro_rules! box_create_checked_operation_primitive_on_self {
    ($op_fn: expr,
     $({
        $primitive: ident
    }),+
    ) => {
        paste! {
            pub fn [<box_$op_fn _plc_primitive>]<T: 'static + MetaData + Primitive + Clone + Display + IntoLocalType>(variable1: &T, trace: &Option<FileTrace>, kernel: &Kernel) -> Result<RunTimeOperation, Stop>{
                $(
                    if variable1.[<is_$primitive>]() {
                        let return_ptr = LocalPointer::new(variable1.transform()?);
                        let return_ptr_clone = return_ptr.clone();
                        let o1_clone = variable1.clone();
                        let trace = trace.clone();

                        return Ok(Box::new(Operation::new(move |channel| {
                            let result = o1_clone.[<as_$primitive>](channel)?.[<checked_$op_fn>]()
                            .ok_or(error!(format!("Invalid operation: Can not {} {}", stringify!($op_fn), o1_clone)))?;

                            return_ptr.as_ref().borrow_mut().deref_mut().[<set_$primitive>](result, channel)?;
                            Ok(())
                        }, Some(return_ptr_clone), false, &trace)))
                    }
                )+
                Err(error!(format!("Invalid operation: Can not {} {}", stringify!($op_fn), variable1)))
            }
        }
    };
}

macro_rules! box_create_operation_primitive {
    ($op_fn: expr,
     $({
        $primitive: ident,
        [$($associated: ident $(check $signed: ident)?),+]
    }),+
    ) => {
        paste! {
            pub fn [<box_$op_fn _plc_primitive>]<T: 'static + MetaData + Primitive + Clone + Display + IntoLocalType, Y : 'static + MetaData + Primitive + Clone + Display>(variable1: &T, variable2: &Y, trace: &Option<FileTrace>, kernel: &Kernel) -> Result<RunTimeOperation, Stop>{
                kernel.check_filtered_operation(&stringify!($op_fn), variable1, variable2)?;
                $(
                    if variable1.[<is_$primitive>]() {
                        $(
                           if variable2.[<is_$associated>]() {
                               let return_ptr = match kernel.check_return_operation(&stringify!($op_fn), variable1, variable2) {
                                   Some(a) => LocalPointer::new(a.transform()?),
                                   None => LocalPointer::new(variable1.transform()?)
                               };
                               let return_ptr_clone = return_ptr.clone();
                               let o1_clone = variable1.clone();
                               let o2_clone = variable2.clone();
                               let trace = trace.clone();

                               return Ok(Box::new(Operation::new(move |channel| {
                                    let result = (o1_clone.[<as_$primitive>](channel)?)
                                        .[<$op_fn $(_$signed)?>](o2_clone.[<as_$associated>](channel)?
                                            .try_into()
                                            .map_err(|_| error!(format!("Failed {} of {} with {}", stringify!($op_fn), o1_clone, o2_clone)))?
                                    );

                                    return_ptr.as_ref().borrow_mut().deref_mut().[<set_$primitive>](result, channel)?;
                                    Ok(())
                               }, Some(return_ptr_clone), false, &trace)))
                           }
                        )+
                    }
                )+
                Err(error!(format!("Invalid operation: Can not {} {} with {}", stringify!($op_fn), variable1, variable2)))
            }
        }
    };
}

macro_rules! box_create_operation_primitive_on_self {
    ($op_fn: expr,
     $({
        $primitive: ident
    }),+
    ) => {
        paste! {
            pub fn [<box_$op_fn _plc_primitive>]<T: 'static + MetaData + Primitive + Clone + Display + IntoLocalType>(variable1: &T, trace: &Option<FileTrace>, kernel: &Kernel) -> Result<RunTimeOperation, Stop>{
                $(
                    if variable1.[<is_$primitive>]() {
                        let return_ptr = LocalPointer::new(variable1.transform()?);
                        let return_ptr_clone = return_ptr.clone();
                        let o1_clone = variable1.clone();
                        let trace = trace.clone();

                        return Ok(Box::new(Operation::new(move |channel| {
                            let result = o1_clone.[<as_$primitive>](channel)?.[<$op_fn>]();

                            return_ptr.as_ref().borrow_mut().deref_mut().[<set_$primitive>](result, channel)?;
                            Ok(())
                        }, Some(return_ptr_clone), false, &trace)))
                    }
                )+
                Err(error!(format!("Invalid operation: Can not {} {}", stringify!($op_fn), variable1)))
            }
        }
    };
}


box_create_checked_operation_primitive!(
    add,
    { u8, [u8, u16, u32, u64, i8 check signed, i16 check signed, i32 check signed, i64 check signed] },
    { u16, [u8, u16, u32, u64, i8 check signed, i16 check signed, i32 check signed, i64 check signed] },
    { u32, [u8, u16, u32, u64, i8 check signed, i16 check signed, i32 check signed, i64 check signed] },
    { u64, [u8, u16, u32, u64, i8 check signed, i16 check signed, i32 check signed, i64 check signed] },
    { i8, [u8 check unsigned, u16 check unsigned, u32 check unsigned, u64 check unsigned, i8, i16, i32, i64] },
    { i16, [u8 check unsigned, u16 check unsigned, u32 check unsigned, u64 check unsigned, i8, i16, i32, i64] },
    { i32, [u8 check unsigned, u16 check unsigned, u32 check unsigned, u64 check unsigned, i8, i16, i32, i64] },
    { i64, [u8 check unsigned, u16 check unsigned, u32 check unsigned, u64 check unsigned, i8, i16, i32, i64] },
    { f32, [f32] },
    { f64, [f32, f64] }
);

box_create_checked_operation_primitive!(
    sub,
    { u8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u32, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u64, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i8, [u8 check unsigned, u16, u32 check unsigned, u64 check unsigned, i8, i16, i32, i64] },
    { i16, [u8 check unsigned, u16 check unsigned, u32 check unsigned, u64 check unsigned, i8, i16, i32, i64] },
    { i32, [u8 check unsigned, u16 check unsigned, u32 check unsigned, u64 check unsigned, i8, i16, i32, i64] },
    { i64, [u8 check unsigned, u16 check unsigned, u32 check unsigned, u64 check unsigned, i8, i16, i32, i64] },
    { f32, [f32] },
    { f64, [f32, f64] }
);

box_create_checked_operation_primitive!(
    mul,
    { u8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u32, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u64, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i32, [u8, u16, u32 , u64, i8, i16, i32, i64] },
    { i64, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { f32, [f32] },
    { f64, [f32, f64] }
);

box_create_checked_operation_primitive!(
    div,
    { u8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u32, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u64, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i32, [u8, u16, u32 , u64, i8, i16, i32, i64] },
    { i64, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { f32, [f32] },
    { f64, [f32, f64] }
);

box_create_checked_operation_primitive!(
    rem,
    { u8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u32, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u64, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i32, [u8, u16, u32 , u64, i8, i16, i32, i64] },
    { i64, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { f32, [f32] },
    { f64, [f32, f64] }
);

box_create_checked_operation_primitive_on_self!(
    cos,
    { f32 },
    { f64 }
);

box_create_checked_operation_primitive_on_self!(
    sin,
    { f32 },
    { f64 }
);

box_create_checked_operation_primitive_on_self!(
    tan,
    { f32 },
    { f64 }
);

box_create_checked_operation_primitive_on_self!(
    acos,
    { f32 },
    { f64 }
);

box_create_checked_operation_primitive_on_self!(
    asin,
    { f32 },
    { f64 }
);

box_create_checked_operation_primitive_on_self!(
    atan,
    { f32 },
    { f64 }
);

box_create_checked_operation_primitive_on_self!(
    exp,
    { f32 },
    { f64 }
);

box_create_checked_operation_primitive_on_self!(
    ln,
    { f32 },
    { f64 }
);

box_create_checked_operation_primitive_on_self!(
    fract,
    { f32 },
    { f64 }
);

box_create_checked_operation_primitive_on_self!(
    trunc,
    { f32 },
    { f64 }
);

box_create_checked_operation_primitive_on_self!(
    floor,
    { f32 },
    { f64 }
);

box_create_checked_operation_primitive_on_self!(
    ceil,
    { f32 },
    { f64 }
);

box_create_checked_operation_primitive_on_self!(
    sqrt,
    { f32 },
    { f64 }
);

box_create_checked_operation_primitive_on_self!(
    sqr,
    { f32 },
    { f64 }
);

box_create_operation_primitive_on_self!(
    round,
    { f32 },
    { f64 }
);

box_create_operation_primitive_on_self!(
    abs,
    { i8 },
    { i16 },
    { i32 },
    { i64 },
    { f32 },
    { f64 }
);


box_create_operation_primitive_on_self!(
    swap_bytes,
    { u8 },
    { u16 },
    { u32 },
    { u64 },
    { i8 },
    { i16 },
    { i32 },
    { i64 }
);

box_create_checked_operation_primitive!(
    shr,
    { u8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u32, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u64, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i32, [u8, u16, u32 , u64, i8, i16, i32, i64] },
    { i64, [u8, u16, u32, u64, i8, i16, i32, i64] }
);

box_create_checked_operation_primitive!(
    shl,
    { u8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u32, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u64, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i32, [u8, u16, u32 , u64, i8, i16, i32, i64] },
    { i64, [u8, u16, u32, u64, i8, i16, i32, i64] }
);

box_create_operation_primitive!(
    rotate_left,
    { u8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u32, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u64, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i32, [u8, u16, u32 , u64, i8, i16, i32, i64] },
    { i64, [u8, u16, u32, u64, i8, i16, i32, i64] }
);

box_create_operation_primitive!(
    rotate_right,
    { u8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u32, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { u64, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i8, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i16, [u8, u16, u32, u64, i8, i16, i32, i64] },
    { i32, [u8, u16, u32 , u64, i8, i16, i32, i64] },
    { i64, [u8, u16, u32, u64, i8, i16, i32, i64] }
);
