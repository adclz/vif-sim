use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::boolean::bool::Bool;
use crate::kernel::plc::types::primitives::boolean::bit_access::BitAccess;
use crate::kernel::plc::types::primitives::traits::family_traits::GetRawPointerPrimitive;
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive};
use crate::kernel::plc::types::primitives::traits::primitive_traits::{PrimitiveTrait, RawMut};
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::kernel::plc::types::primitives::string::wstring::wstr256;
use crate::{create_family, error, impl_primitive_traits, key_reader};
use camelpaste::paste;
use fixedstr::str256;

use serde::Serializer;
use serde_json::{Map, Value};


create_family!(
    #[enum_dispatch(MetaData, SetMetaData, ToggleMonitor)]
    PlcBool(Bool, BitAccess)
);

impl_primitive_traits!(PlcBool, {
    bool, [direct true], [get_mut as_mut_bool, get_mut as_mut_bit_access], [get as_bool, get as_bit_access],
    char, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))],
    wchar, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))],
    str256, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))],
    wstr256, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))],
    f32, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))],
    f64, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))],
    u8, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))],
    u16, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))],
    u32, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))],
    u64, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))],
    i8, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))],
    i16, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))],
    i32, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))],
    i64, [direct false], [stop Err(error!(format!("t")))], [none Err(error!(format!("t")))]
});

impl From<bool> for PlcBool {
    fn from(value: bool) -> Self {
        Self::Bool(Bool::new(&value).unwrap())
    }
}

impl TryFrom<(&Map<String, Value>, &str)> for PlcBool {
    type Error = Stop;

    fn try_from(data: (&Map<String, Value>, &str)) -> Result<Self, Self::Error> {
        let src = data.0;
        key_reader!(
            format!("Parse PlcBool"),
            src {
                value? => as_bool,
            }
        );

        match value {
            None => Ok(Self::Bool(Bool::default())),
            Some(a) => Ok(Self::Bool(Bool::new(&a)?)),
        }
    }
}
