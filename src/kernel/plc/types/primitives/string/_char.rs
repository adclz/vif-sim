﻿use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::*;
use crate::kernel::plc::types::primitives::traits::primitive_traits::*;
use crate::kernel::plc::types::primitives::traits::meta_data::*;
use crate::{error, impl_primitive_all, impl_primitive_base, impl_primitive_display, impl_primitive_raw_mut, impl_primitive_serialize, impl_primitive_type_name};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::Value;
use smart_default::SmartDefault;
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::borrow::Cow;
use crate::kernel::plc::types::primitives::string::_string::_String;
use crate::kernel::registry::Kernel;
use crate::kernel::registry::get_string;

#[derive(Clone, SmartDefault)]
pub struct _Char {
    value: char,
    default: char,
    #[default(_code = "get_id()")]
    id: usize,
    monitor: bool,
    read_only: bool,
    alias: Option<usize>,
    path: usize,
}

impl_primitive_all!(_Char, char);

impl TryFrom<&Value> for _Char {
    type Error = Stop;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_str() {
            None => Err(error!(format!("Invalid value {} for Char", value))),
            Some(a) => Ok(_Char::new(
                &char::from_str(a).map_err(|e| error!(format!("{}", e)))?,
            )?),
        }
    }
}
