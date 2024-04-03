use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::plc::primitives::family_traits::GetRawPointerPrimitive;
use crate::plc::primitives::family_traits::{AsMutPrimitive, Primitive};
use crate::plc::primitives::floats::lreal::{LReal};
use crate::plc::primitives::floats::real::Real;
use crate::plc::primitives::primitive_traits::{PrimitiveTrait, RawMut};
use crate::plc::primitives::string::wchar::wchar;
use crate::plc::primitives::string::wstring::wstr256;
use crate::{create_family, error, impl_primitive_traits, key_reader};
use crate::plc::primitives::floats::checked_float::TryIntoCheck;
use camelpaste::paste;
use fixedstr::str256;
use serde::Serializer;
use serde_json::{Map, Value};

create_family!(
    #[enum_dispatch(MetaData, SetMetaData, ToggleMonitor)]
    PlcFloat(Real, LReal)
);

impl_primitive_traits!(PlcFloat, {
    bool, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    char, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wchar, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    str256, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wstr256, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    f32, [self.is_real], [get_mut as_mut_real], [get as_real],
    f64, [self.is_l_real], [get_mut as_mut_l_real], [get as_l_real],
    u8, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u16, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u32, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    u64, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i8, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i16, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i32, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    i64, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))]
});

impl From<f64> for PlcFloat {
    fn from(value: f64) -> Self {
        Self::LReal(LReal::new(&value).unwrap())
    }
}

impl TryFrom<(&Map<String, Value>, &str)> for PlcFloat {
    type Error = Stop;

    fn try_from(src: (&Map<String, Value>, &str)) -> Result<Self, Self::Error> {
        let _src = src.0;
        let ty = src.1;
        key_reader!(
            format!("Parse PlcFloat {}", ty),
            _src {
                value?,
            }
        );

        match value {
            None => match ty {
                "Real" => Ok(PlcFloat::Real(Real::default())),
                "LReal" => Ok(PlcFloat::LReal(LReal::default())),
                _ => Err(error!(
                    format!("Invalid PlcFloat type: {}", ty),
                    "Parse PlcFloat".to_string()
                )),
            },
            Some(value) => {
                if let Some(v) = value.as_f64() {
                    match ty {
                        "Real" => Ok(PlcFloat::Real(Real::new(&TryIntoCheck::try_into(v).unwrap())?)),
                        "LReal" => Ok(PlcFloat::LReal(LReal::new(
                            &v,
                        )?)),
                        _ => Err(error!(
                            format!("Invalid PlcFloat type: {}", ty),
                            "Parse PlcFloat".to_string()
                        )),
                    }
                } else {
                    Err(error!(
                        format!("Invalid PlcFloat value: {}", value),
                        "Parse PlcFloat".to_string()
                    ))
                }
            }
        }
    }
}
