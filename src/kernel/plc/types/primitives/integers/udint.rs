﻿use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::*;
use crate::kernel::plc::types::primitives::traits::primitive_traits::*;
use crate::kernel::plc::types::primitives::traits::meta_data::*;
use crate::kernel::plc::types::primitives::traits::crement::Crement;
use crate::{error, impl_primitive_all, impl_primitive_crement, key_reader};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use core::any::{Any, TypeId};
use core::fmt::{Display, Formatter};
use std::borrow::Cow;
use crate::kernel::registry::Kernel;
use crate::kernel::registry::get_string;

#[derive(Clone)]
pub struct UDInt {
    default: u32,
    value: u32,

    id: u32,
    read_only: bool,
    alias: Option<usize>,
    path: usize
}

impl_primitive_crement!(UDInt);
impl_primitive_all!(UDInt, u32);

impl TryFrom<&Map<String, Value>> for UDInt {
    type Error = Stop;

    fn try_from(data: &Map<String, Value>) -> Result<Self, Self::Error> {
        key_reader!(
            format!("Parse UDInt"),
            data {
                value => as_u64,
                id => as_u64,
            }
        );
        let id = id as u32;
        UDInt::new(&value.try_into().map_err(|e| error!(format!("{}", e)))?, id)
    }
}
