#![allow(non_snake_case)]
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::GetRawPointerPrimitive;
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive, PrimitiveTrait, RawMut};
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::kernel::plc::types::primitives::timers::lTime::LTime;
use crate::kernel::plc::types::primitives::timers::time::Time;
use crate::{create_family, error, impl_primitive_traits, key_reader};
use camelpaste::paste;
use fixedstr::str256;

use serde::Serializer;
use serde_json::{Map, Value};
use crate::kernel::plc::types::primitives::string::_string::plcstr;
use crate::kernel::plc::types::primitives::string::wstring::plcwstr;

create_family!(
    #[enum_dispatch(TimeDuration, MetaData, SetMetaData, ToggleMonitor)]
    PlcTime(Time, LTime)
);

impl_primitive_traits!(PlcTime, {
    bool, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    char, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wchar, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    plcstr, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    plcwstr, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    f32, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    f64, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u8, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u16, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u32, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u64, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i8, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i16, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i32, [self.is_time], [get_mut as_mut_time], [get as_time],
    i64, [self.is_l_time], [get_mut as_mut_l_time], [get as_l_time]
});

/*
impl From<i64> for PlcTime {
    fn from(value: i64) -> Self {
        match value {
            -2_147_483_648..=2_147_483_647 => PlcTime::Time(Time::new(&(value as i32)).unwrap()),
            -9_223_372_036_854_775_808..=9_223_372_036_854_775_807 => {
                PlcTime::LTime(LTime::new(&value).unwrap())
            }
        }
    }
}*/

impl TryFrom<&Map<String, Value>> for PlcTime {
    type Error = Stop;

    fn try_from(data: &Map<String, Value>) -> Result<Self, Self::Error> {
        key_reader!(
           format!("Parse PlcTime"),
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
                "Time" => Ok(PlcTime::Time(Time::new_default(id))),
                "LTime" => Ok(PlcTime::LTime(LTime::new_default(id))),
                _ => Err(error!(
                    format!("Invalid PlcTime type: {}", ty),
                    "Parse PlcTime".to_string()
                )),
            },
            Some(value) => {
                if let Some(v) = value.as_i64() {
                    match ty {
                        "Time" => Ok(PlcTime::Time(Time::new(&(v as i32), id)?)),
                        "LTime" => Ok(PlcTime::LTime(LTime::new(&(v), id)?)),
                        _ => Err(error!(
                            format!("Invalid PlcTime type: {}", ty),
                            "Parse PlcTime".to_string()
                        )),
                    }
                } else {
                    Err(error!(
                        format!("Invalid PlcTime value: {}", value),
                        "Parse PlcTime".to_string()
                    ))
                }
            }
        }
    }
}
