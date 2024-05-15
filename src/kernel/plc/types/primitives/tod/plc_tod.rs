#![allow(non_snake_case)]
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::GetRawPointerPrimitive;
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive, PrimitiveTrait, RawMut};
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::kernel::plc::types::primitives::tod::ltod::LTod;
use crate::kernel::plc::types::primitives::tod::tod::Tod;
use crate::{create_family, error, impl_primitive_traits, key_reader};
use camelpaste::paste;
use fixedstr::str256;

use serde::Serializer;
use serde_json::{Map, Value};
use crate::kernel::plc::types::primitives::string::_string::plcstr;
use crate::kernel::plc::types::primitives::string::wstring::plcwstr;

create_family!(
    #[enum_dispatch(MetaData, SetMetaData, ToggleMonitor)]
    PlcTod(Tod, LTod)
);

impl_primitive_traits!(PlcTod, {
    bool, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    char, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wchar, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    plcstr, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    plcwstr, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    f32, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    f64, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u8, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u16, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u32, [self.is_tod], [get_mut as_mut_tod], [get as_tod],
    u64, [self.is_l_tod], [get_mut as_mut_l_tod], [get as_l_tod],
    i8, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i16, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i32, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i64, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))]
});

/*
impl From<u64> for PlcTod {
    fn from(value: u64) -> Self {
        match value {
            0..=4_294_967_295 => Self::Tod(Tod::new(&(value as u32)).unwrap()),
            _ => Self::LTod(LTod::new(&value).unwrap()),
        }
    }
}*/

impl TryFrom<&Map<String, Value>> for PlcTod {
    type Error = Stop;

    fn try_from(data: &Map<String, Value>) -> Result<Self, Self::Error> {
        key_reader!(
           format!("Parse PlcBinary"),
           data {
                ty => as_str,
                src => {
                    value?,
                    id => as_u64,
                }
            }
        );
        let id = id as u32;
        match value {
            None => match ty {
                "Tod" => Ok(PlcTod::Tod(Tod::new_default(id))),
                "LTod" => Ok(PlcTod::LTod(LTod::new_default(id))),
                _ => Err(error!(
                    format!("Invalid PlcTod type: {}", ty),
                    format!("Parse PlcTod")
                )),
            },
            Some(value) => {
                if let Some(v) = value.as_u64() {
                    match ty {
                        "Tod" => Ok(PlcTod::Tod(Tod::new(&(v as u32), id)?)),
                        "LTod" => Ok(PlcTod::LTod(LTod::new(&v, id)?)),
                        _ => Err(error!(
                            format!("Invalid PlcTod type: {}", ty),
                            format!("Parse PlcTod")
                        )),
                    }
                } else {
                    Err(error!(
                        format!("Invalid PlcTod value: {}", value),
                        format!("Parse PlcTod")
                    ))
                }
            }
        }
    }
}
