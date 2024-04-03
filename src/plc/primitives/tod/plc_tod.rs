#![allow(non_snake_case)]
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::plc::primitives::family_traits::GetRawPointerPrimitive;
use crate::plc::primitives::family_traits::{AsMutPrimitive, Primitive};

use crate::plc::primitives::primitive_traits::{PrimitiveTrait, RawMut};
use crate::plc::primitives::string::wchar::wchar;
use crate::plc::primitives::string::wstring::wstr256;
use crate::plc::primitives::tod::ltod::LTod;
use crate::plc::primitives::tod::tod::Tod;
use crate::{create_family, error, impl_primitive_traits, key_reader};
use camelpaste::paste;
use fixedstr::str256;

use serde::Serializer;
use serde_json::{Map, Value};

create_family!(
    #[enum_dispatch(MetaData, SetMetaData, ToggleMonitor)]
    PlcTod(Tod, LTod)
);

impl_primitive_traits!(PlcTod, {
    bool, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    char, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wchar, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    str256, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wstr256, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
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

impl From<u64> for PlcTod {
    fn from(value: u64) -> Self {
        match value {
            0..=4_294_967_295 => Self::Tod(Tod::new(&(value as u32)).unwrap()),
            _ => Self::LTod(LTod::new(&value).unwrap()),
        }
    }
}

impl TryFrom<(&Map<String, Value>, &str)> for PlcTod {
    type Error = Stop;

    fn try_from(src: (&Map<String, Value>, &str)) -> Result<Self, Self::Error> {
        let _src = src.0;
        let ty = src.1;
        key_reader!(
            format!("Parse PlcTod {}", ty),
            _src {
                value?,
            }
        );

        match value {
            None => match ty {
                "Tod" => Ok(PlcTod::Tod(Tod::default())),
                "LTod" => Ok(PlcTod::LTod(LTod::default())),
                _ => Err(error!(
                    format!("Invalid PlcTod type: {}", ty),
                    format!("Parse PlcTod")
                )),
            },
            Some(value) => {
                if let Some(v) = value.as_u64() {
                    match ty {
                        "Tod" => Ok(PlcTod::Tod(Tod::new(&(v as u32))?)),
                        "LTod" => Ok(PlcTod::LTod(LTod::new(&v)?)),
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
