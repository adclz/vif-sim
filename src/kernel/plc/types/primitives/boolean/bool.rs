use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::*;
use crate::kernel::plc::types::primitives::traits::primitive_traits::*;
use crate::kernel::plc::types::primitives::traits::meta_data::*;
use crate::kernel::plc::types::primitives::traits::meta_data::*;
use crate::{error, impl_primitive_all, key_reader};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use core::any::{Any, TypeId};
use core::fmt::{Display, Formatter};
use std::borrow::Cow;
use crate::kernel::registry::Kernel;
use crate::kernel::registry::get_string;

#[derive(Clone)]
pub struct Bool {
    default: bool,
    value: bool,
    id: u32,
    read_only: bool,
    alias: Option<usize>,
    path: usize,
}

impl_primitive_all!(Bool, bool);

impl TryFrom<&Map<String, Value>> for Bool {
    type Error = Stop;

    fn try_from(data: &Map<String, Value>) -> Result<Self, Self::Error> {
        key_reader!(
            format!("Parse bool"),
            data {
                value => as_bool,
                id => as_u64,
            }
        );
        let id = id as u32;
        Bool::new(&value, id)
    }
}
