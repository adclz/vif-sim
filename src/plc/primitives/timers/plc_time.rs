#![allow(non_snake_case)]
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::plc::primitives::family_traits::GetRawPointerPrimitive;
use crate::plc::primitives::family_traits::{AsMutPrimitive, Primitive};

use crate::plc::primitives::primitive_traits::{PrimitiveTrait, RawMut};
use crate::plc::primitives::string::wchar::wchar;
use crate::plc::primitives::string::wstring::wstr256;
use crate::plc::primitives::timers::lTime::LTime;
use crate::plc::primitives::timers::time::Time;
use crate::{create_family, error, impl_primitive_traits, key_reader};
use camelpaste::paste;
use fixedstr::str256;

use serde::Serializer;
use serde_json::{Map, Value};

create_family!(
    #[enum_dispatch(TimeDuration, MetaData, SetMetaData, ToggleMonitor)]
    PlcTime(Time, LTime)
);

impl_primitive_traits!(PlcTime, {
    bool, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    char, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wchar, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    str256, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wstr256, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
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

impl From<i64> for PlcTime {
    fn from(value: i64) -> Self {
        match value {
            -2_147_483_648..=2_147_483_647 => PlcTime::Time(Time::new(&(value as i32)).unwrap()),
            -9_223_372_036_854_775_808..=9_223_372_036_854_775_807 => {
                PlcTime::LTime(LTime::new(&value).unwrap())
            }
        }
    }
}

impl TryFrom<(&Map<String, Value>, &str)> for PlcTime {
    type Error = Stop;

    fn try_from(src: (&Map<String, Value>, &str)) -> Result<Self, Self::Error> {
        let _src = src.0;
        let ty = src.1;
        key_reader!(
            format!("Parse PlcTime {}", ty),
            _src {
                value?,
            }
        );

        match value {
            None => match ty {
                "Time" => Ok(PlcTime::Time(Time::default())),
                "LTime" => Ok(PlcTime::LTime(LTime::default())),
                _ => Err(error!(
                    format!("Invalid PlcTime type: {}", ty),
                    "Parse PlcTime".to_string()
                )),
            },
            Some(value) => {
                if let Some(v) = value.as_i64() {
                    match ty {
                        "Time" => Ok(PlcTime::Time(Time::new(&(v as i32))?)),
                        "LTime" => Ok(PlcTime::LTime(LTime::new(&(v))?)),
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
