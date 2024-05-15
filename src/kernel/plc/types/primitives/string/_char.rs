use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::*;
use crate::kernel::plc::types::primitives::traits::primitive_traits::*;
use crate::kernel::plc::types::primitives::traits::meta_data::*;
use crate::{error, impl_primitive_all, impl_primitive_base, impl_primitive_display, impl_primitive_raw_mut, impl_primitive_serialize, impl_primitive_type_name, key_reader};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use smart_default::SmartDefault;
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::borrow::Cow;
use crate::kernel::plc::types::primitives::string::_string::_String;
use crate::kernel::registry::Kernel;
use crate::kernel::registry::get_string;

#[derive(Clone)]
pub struct _Char {
    value: char,
    default: char,
    id: u32,
    read_only: bool,
    alias: Option<usize>,
    path: usize,
}

impl_primitive_all!(_Char, char);

impl TryFrom<&Map<String, Value>> for _Char {
    type Error = Stop;

    fn try_from(data: &Map<String, Value>) -> Result<Self, Self::Error> {
        key_reader!(
            format!("Parse LInt"),
            data {
                value => as_str,
                id => as_u64,
            }
        );
        let id = id as u32;
        _Char::new(&char::from_str(value).map_err(|e| error!(format!("{}", e)))?, id)
    }
}
