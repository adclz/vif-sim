use crate::container::broadcast::broadcast::Broadcast;
use crate::container::broadcast::store::MonitorChange;
use crate::container::container::get_id;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::*;
use crate::kernel::plc::types::primitives::traits::primitive_traits::*;
use crate::kernel::plc::types::primitives::traits::meta_data::*;
use crate::{error, impl_primitive_all, key_reader};
use fixedstr::str256;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use smart_default::SmartDefault;
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::borrow::Cow;
use crate::kernel::registry::Kernel;
use crate::kernel::registry::get_string;

#[derive(Clone)]
pub struct _String {
    value: plcstr,
    default: plcstr,

    id: u32,
    read_only: bool,
    alias: Option<usize>,
    path: usize,
}

impl_primitive_all!(_String, plcstr);

#[derive(Debug, Clone, Copy, PartialOrd, Default, PartialEq)]
pub struct plcstr(pub str256);

impl Display for plcstr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for plcstr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(self.0.as_str())
    }
}

impl TryFrom<&Map<String, Value>> for _String {
    type Error = Stop;

    fn try_from(data: &Map<String, Value>) -> Result<Self, Self::Error> {
        key_reader!(
                format!("Parse String"),
                data {
                    value => as_str,
                    id => as_u64,
                }
            );
        let id = id as u32;
        _String::new(&plcstr(str256::from_str(value).map_err(|e| error!(format!("{}", e)))?), id)
    }
}