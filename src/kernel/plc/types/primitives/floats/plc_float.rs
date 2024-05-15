use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::kernel::plc::types::primitives::traits::family_traits::GetRawPointerPrimitive;
use crate::kernel::plc::types::primitives::traits::primitive_traits::{AsMutPrimitive, Primitive, PrimitiveTrait, RawMut};
use crate::kernel::plc::types::primitives::floats::lreal::{LReal};
use crate::kernel::plc::types::primitives::floats::real::Real;
use crate::kernel::plc::types::primitives::string::wchar::wchar;
use crate::{create_family, error, impl_primitive_traits, key_reader};
use crate::kernel::plc::types::primitives::floats::checked_float::TryIntoCheck;
use camelpaste::paste;
use serde::Serializer;
use serde_json::{Map, Value};
use crate::kernel::plc::types::primitives::string::_string::plcstr;
use crate::kernel::plc::types::primitives::string::wstring::plcwstr;

create_family!(
    #[enum_dispatch(MetaData, SetMetaData, ToggleMonitor)]
    PlcFloat(Real, LReal)
);

impl_primitive_traits!(PlcFloat, {
    bool, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    char, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    wchar, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    plcstr, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
    plcwstr, [direct false], [stop Err(error!(format!("0")))], [none Err(error!(format!("0")))],
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

/*impl From<f64> for PlcFloat {
    fn from(value: f64) -> Self {
        Self::LReal(LReal::new(&value).unwrap())
    }
}*/

impl TryFrom<&Map<String, Value>> for PlcFloat {
    type Error = Stop;

    fn try_from(data: &Map<String, Value>) -> Result<Self, Self::Error> {
        key_reader!(
           format!("Parse PlcFloat"),
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
                "Real" => Ok(PlcFloat::Real(Real::new_default(id))),
                "LReal" => Ok(PlcFloat::LReal(LReal::new_default(id))),
                _ => Err(error!(
                    format!("Invalid PlcFloat type: {}", ty),
                    "Parse PlcFloat".to_string()
                )),
            },
            Some(value) => {
                if let Some(v) = value.as_f64() {
                    match ty {
                        "Real" => Ok(PlcFloat::Real(Real::new(&TryIntoCheck::try_into(v).unwrap(), id)?)),
                        "LReal" => Ok(PlcFloat::LReal(LReal::new(
                            &v, id
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
