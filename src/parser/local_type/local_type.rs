use std::ops::{Deref, DerefMut};
use serde_json::{Map, Value};
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use crate::{error, key_reader};
use crate::parser::interface::interface::parse_struct_interface;
use crate::parser::local_type::constant_type::parse_constant_type;
use crate::plc::complex::array::PlcArray;
use crate::plc::complex::instance::fb_instance::FbInstance;
use crate::plc::complex::r#struct::PlcStruct;
use crate::plc::interface::status::InterfaceStatus;
use crate::plc::interface::traits::{Cloneable, DeferredBuilder};
use crate::plc::primitives::binaries::plc_binary::PlcBinary;
use crate::plc::primitives::boolean::plc_bool::PlcBool;
use crate::plc::primitives::boxed::set::box_set_plc_primitive_default_once;
use crate::plc::primitives::floats::plc_float::PlcFloat;
use crate::plc::primitives::integers::plc_integer::PlcInteger;
use crate::plc::primitives::string::plc_string::PlcString;
use crate::plc::primitives::timers::plc_time::PlcTime;
use crate::plc::primitives::tod::plc_tod::PlcTod;
use crate::registry::local::r#type::{IntoLocalType, LocalType};
use crate::registry::registry::Kernel;

pub fn parse_local_type(
    json: &Map<String, Value>,
    registry: &Kernel,
    channel: &Broadcast,
) -> Result<LocalType, Stop> {
    key_reader!(
            format!("Parse value"),
            json {
                ty => as_str,
                src => as_object,
            }
        );

    match ty {

        // Alias
        "alias" => {
            key_reader!(
                format!("Parse type alias"),
                src {
                    name => as_str,
                    data => as_object,
                }
            );
            // Checks if alias exists
            match registry.get_type_alias_as_constant_type(&name) {
                None => Err(error!(format!("Type alias {} is not registered", name))),
                Some(as_constant) => {
                    match parse_constant_type(data, registry, Some(as_constant.transform()?)) {
                        Ok(b) => Ok(b.transform()?),
                        Err(e) => Err(error!(format!("Type alias do not match the registered type alias, expected {}, got {}", as_constant, name)))
                    }
                }
            }
        },

        // Bool
        "Bool" => Ok(LocalType::PlcBool(PlcBool::try_from((src, ty))?)),

        // Binary
        "Byte" | "Word" | "DWord" | "LWord" => Ok(LocalType::PlcBinary(PlcBinary::try_from((src, ty))?)),

        // Integers
        "SInt" | "Int" | "DInt" | "LInt" | "USInt" | "UInt" | "UDInt" | "ULInt" =>
            Ok(LocalType::PlcInteger(PlcInteger::try_from((src, ty))?)),

        // Floats
        "Real" | "LReal" => Ok(LocalType::PlcFloat(PlcFloat::try_from((src, ty))?)),

        // Time
        "Time" | "LTime" => Ok(LocalType::PlcTime(PlcTime::try_from((src, ty))?)),

        // TOD
        "Tod" | "LTod" => Ok(LocalType::PlcTod(PlcTod::try_from((src, ty))?)),

        //String
        "String" | "Char" | "WString" | "WChar" => Ok(LocalType::PlcString(PlcString::try_from((src, ty))?)),

        // Struct
        "Struct" => Ok(LocalType::PlcStruct(PlcStruct::from_json(src, registry, channel)?)),

        // Array
        "array" => Ok(LocalType::PlcArray(PlcArray::from_json(src, registry, channel)?)),

        // From

        // Udt
        "udt_impl" => {
            key_reader!(
                format!("Udt name is not of type string, {:?}", src),
                src {
                    of => as_str,
                    interface?,
                }
            );

            let udt = registry.get(of).ok_or(error!(
                format!("The Udt '{}' could not be found", of),
                "Parse interface section".to_string()
            ))?;

            if udt.is_udt() {
                let mut as_udt = udt.as_mut_udt()?;
                match as_udt.get_interface_status() {
                    InterfaceStatus::Default => as_udt.build_interface(registry, channel),
                    InterfaceStatus::Pending => Err(error!(
                        format!("Interface is recursive"),
                        format!("Parse member")
                    )),
                    InterfaceStatus::Solved => Ok(()),
                }?;

                let udt_interface = as_udt.clone_interface(registry, channel)?;

                match interface {
                    Some(a) => {
                        println!("{}", a);
                        // Parse the received interface
                        parse_struct_interface(a, registry, channel, &None)?
                            .get_pointers_with_path(&vec!(), &vec!())
                            .iter()
                            .try_for_each(|x| {
                                box_set_plc_primitive_default_once(
                                    &udt_interface
                                        .try_get_nested(&x.0.1)
                                        .ok_or(error!(format!("Could not find {:?} in {}", &x.0.1, udt_interface)))?, &x.0.0)?(channel)
                            })?;
                        Ok(LocalType::PlcStruct(PlcStruct::from_interface(
                            Some(of.into()),
                            udt_interface,
                            registry, channel,
                        )?))
                    },
                    None => {
                        Ok(LocalType::PlcStruct(PlcStruct::from_interface(
                            Some(of.into()),
                            udt_interface,
                            registry, channel,
                        )?))
                    }
                }
            } else {
                Err(error!(
                    format!("'{}' can not be created", of),
                    format!("Parse interface section")
                ))
            }
        }

        // Instance of Fb
        "instance" => {
            key_reader!(
                format!("Instance name is not of type string, {:?}", src),
                src {
                    of => as_str,
                    interface?,
                }
            );

            let instance = registry.get(of).ok_or(error!(
                format!("The Fb '{}' could not be found", of),
                format!("Parse Fb instance")
            ))?;

            if instance.is_fb() {
                let mut as_fb = instance.as_mut_fb()?;
                match as_fb.get_interface_status() {
                    InterfaceStatus::Default => as_fb.build_interface(registry, channel),
                    InterfaceStatus::Pending => Err(error!(format!("Fb '{}' could not be built. Watch out for recursive types in interfaces", of), format!("Parse interface section"))),
                    InterfaceStatus::Solved => Ok(())
                }?;

                let fb_interface = as_fb.clone_interface(registry, channel)?;
                let fb_body = as_fb.clone_body(registry, channel)?;

                match interface {
                    Some(a) => {
                        // Parse the received interface
                        parse_struct_interface(a, registry, channel, &None)?
                            .get_pointers_with_path(&vec!(), &vec!())
                            .iter()
                            .try_for_each(|x| {
                                box_set_plc_primitive_default_once(
                                    &fb_interface.try_get_nested(&x.0.1)
                                        .ok_or(error!(format!("0")))?, &x.0.0)?(channel)
                            })?;
                        Ok(LocalType::FbInstance(
                            FbInstance::from_fb(Some(of.to_string()), fb_interface, fb_body, registry, channel)?,
                        ))
                    },
                    None => {
                        Ok(LocalType::FbInstance(
                            FbInstance::from_fb(Some(of.to_string()), fb_interface, fb_body, registry, channel)?,
                        ))
                    }
                }
            } else {
                Err(error!(
                    format!("'{}' can not be instantiated", of),
                    format!("Parse interface section")
                ))
            }
        }
        _ => Err(error!(format!("Unknown local type: {}", ty))),
    }
}