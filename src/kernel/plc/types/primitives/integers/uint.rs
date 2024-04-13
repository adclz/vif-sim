﻿use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::*;
use crate::kernel::plc::types::primitives::traits::primitive_traits::*;
use crate::kernel::plc::types::primitives::traits::meta_data::*;
use crate::kernel::plc::types::primitives::traits::crement::Crement;
use crate::{error, impl_primitive_all, impl_primitive_crement};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::Value;
use smart_default::SmartDefault;
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use std::borrow::Cow;
use crate::kernel::registry::Kernel;
use crate::kernel::registry::get_full_path;

#[derive(Clone, SmartDefault)]
pub struct UInt {
    default: u16,
    value: u16,
    #[default(_code = "get_id()")]
    id: usize,
    monitor: bool,
    read_only: bool,
    alias: Option<usize>,
    path: Vec<usize>
}

impl_primitive_crement!(UInt);
impl_primitive_all!(UInt, u16);

impl TryFrom<&Value> for UInt {
    type Error = Stop;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_u64() {
            None => Err(error!(format!("Invalid value {} for UInt", value))),
            Some(a) => Ok(UInt::new(
                &a.try_into().map_err(|e| error!(format!("{}", e)))?,
            )?),
        }
    }
}