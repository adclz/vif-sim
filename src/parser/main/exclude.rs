use std::collections::HashSet;
use crate::kernel::registry::{Kernel};
use serde_json::{Map, Value};
use crate::container::error::error::Stop;
use crate::error;
use crate::parser::local_type::constant_type::{create_default_constant_from_str};
use crate::kernel::plc::interface::section::Section;

fn forbidden_alias(alias: &str) -> bool {
    matches!(alias, "Ob" | "Fb" | "Fc" |
        "AnyInteger" | "AnyUnsignedInteger" | "AnySignedInteger" | "AnyBinary" | "AnyFloat" | "AnyString" | "AnyTime" | "AnyTod" |
        "Udt" | "Struct" | "Array" | "Instance" |
        "USInt" | "SInt" | "UInt" | "Int" | "UDInt" | "DInt" | "ULInt" | "LInt" |
        "Byte" | "Word" | "DWord" | "LWord" |
        "Real" | "LReal" |
        "String" | "WString" | "Char" | "WChar" |
        "Time" | "LTime" |
        "Tod" | "LTod")
}

fn turn_family_into_types(identifier: &str) -> Option<HashSet<String>> {
    match identifier {
        "AnyInteger" => Some(HashSet::from(["USInt".into(), "SInt".into(), "UInt".into(), "Int".into(), "UDInt".into(), "DInt".into(), "ULInt".into(), "LInt".into()])),
        "AnyUnsignedInteger" => Some(HashSet::from(["USInt".into(), "UInt".into(), "UDInt".into(), "ULInt".into()])),
        "AnySignedInteger" => Some(HashSet::from(["SInt".into(), "Int".into(), "DInt".into(), "LInt".into()])),
        "AnyBinary" => Some(HashSet::from(["Byte".into(), "Word".into(), "DWord".into(), "LWord".into()])),
        "AnyFloat" => Some(HashSet::from(["Real".into(), "LReal".into()])),
        "AnyString" => Some(HashSet::from(["String".into(), "WString".into(), "Char".into(), "WChar".into()])),
        "AnyTime" => Some(HashSet::from(["Time".into(), "LTime".into()])),
        "AnyTod" => Some(HashSet::from(["Tod".into(), "LTod".into()])),
        _ => None
    }
}

pub fn parse_type_aliases(type_aliases: Option<&Map<String, Value>>, registry: &mut Kernel) -> Result<(), Stop> {
    if let Some(a) = type_aliases {
        a
            .iter()
            .try_for_each(|x| {
                if forbidden_alias(&x.0) {
                    Err(error!(format!("Alias name {} is forbidden", x.0)))
                } else if let Some(identifier) = x.1.as_str() {
                    registry.add_type_alias(&x.0, create_default_constant_from_str(identifier)?);
                    Ok(())
                } else {
                    Err(error!(format!("Invalid identifier for alias {}, expected string got {:?}", x.0, x.1)))
                }
            })
    } else { Ok(()) }
}

pub fn parse_exclude_sections(exclude_sections: Option<&Map<String, Value>>, registry: &mut Kernel) -> Result<(), Stop> {
    if let Some(a) = exclude_sections {
        a
            .iter()
            .try_for_each(|(section, all_types)| {
                let section_to_s = match section.as_str() {
                    "input" => Ok(Section::Input),
                    "output" => Ok(Section::Output),
                    "inout" => Ok(Section::InOut),
                    "static" => Ok(Section::Static),
                    "temp" => Ok(Section::Temp),
                    "constant" => Ok(Section::Constant),
                    "return" => Ok(Section::Return),
                    _ => Err(error!(format!("[Exclude] Invalid section {}", section)))
                }?;

                let entry_section = registry.get_mut_excluded_types_in_section(&section_to_s);

                if let Some(all_types) = all_types.as_array() {
                    all_types
                        .iter()
                        .try_for_each(|a_type| {
                            if let Some(a_type) = a_type.as_str() {
                                entry_section.insert(a_type.into());
                                Ok(())
                            } else {
                                Err(error!(format!("[Exclude] Invalid type {} for section: {}", a_type, section.as_str())))
                            }
                        })
                } else { Ok(()) }
            })
    } else { Ok(()) }
}

pub fn parse_exclude_types(exclude_types: Option<&Vec<Value>>, registry: &mut Kernel) -> Result<(), Stop> {
    if let Some(a) = exclude_types {
        a
            .iter()
            .try_for_each(|x| {
                if let Some(a) = x.as_str() {
                    registry.get_mut_excluded_types().push(a.into());
                    Ok(())
                } else {
                    Err(error!(format!("[Exclude] Invalid entry for type: {}", x)))
                }
            })
    } else { Ok(()) }
}

// Definitely O²

pub fn parse_filter_operations(filter_operations: Option<&Map<String, Value>>, registry: &mut Kernel) -> Result<(), Stop> {
    if let Some(a) = filter_operations {
        a
            // Iter in all operations sent by the user
            .iter()
            .try_for_each(|(op, to_ban)| {
                // Get the operation or create it
                let operation = registry.get_mut_excluded_operation(op);

                if let Some(ban_by_type) = to_ban.as_object() {
                    // Iter through the ban entries by first type
                    ban_by_type
                        .iter()
                        .try_for_each(|(initial_type, array)| {
                            let type_entries = match turn_family_into_types(&initial_type) {
                                None => HashSet::from([initial_type.into()]),
                                Some(a) => a
                            };

                            // Creates the entries for types
                            type_entries.iter().try_for_each(|entry| {
                                let add_to = operation.entry(entry.into()).or_default();
                                // Checks if ban list is array
                                if let Some(ban_list) = array.as_array() {
                                    // Iter through the ban list
                                    ban_list
                                        .iter()
                                        .try_for_each(|a| {
                                            if let Some(t) = a.as_str() {
                                                let type_entries = match turn_family_into_types(&t) {
                                                    None => HashSet::from([t.into()]),
                                                    Some(a) => a
                                                };

                                                type_entries.iter().for_each(|x| {
                                                    add_to.insert(x.into());
                                                });
                                                Ok(())
                                            } else {
                                                Ok(())
                                            }
                                        })
                                } else {
                                    Err(error!(format!("[Exclude] Types for filter_operation {} is not an array, got {:?}", op, ban_by_type)))
                                }
                            })
                        })
                } else {
                    Err(error!(format!("[Exclude] Type entry for filter_operation {} is not an object, got {:?}", op, to_ban)))
                }
            })
    } else { Ok(()) }
}

pub fn parse_return_operations(filter_operations: Option<&Map<String, Value>>, registry: &mut Kernel) -> Result<(), Stop> {
    if let Some(a) = filter_operations {
        a
            // Iter in all operations sent by the user
            .iter()
            .try_for_each(|(op, to_ban)| {
                // Get the operation or create it

                if let Some(ban_by_type) = to_ban.as_object() {
                    // Iter through the ban entries by first type
                    ban_by_type
                        .iter()
                        .try_for_each(|(initial_type, array)| {

                                if let Some(ban_list) = array.as_array() {
                                    // Iter through the ban list
                                    ban_list
                                        .iter()
                                        .try_for_each(|a| {
                                            if let Some(tuple) = a.as_array() {
                                                let type1 = match tuple[0].as_str() {
                                                    None => Err(error!(format!("{} is not a string", tuple[0]))),
                                                    Some(a) => Ok(a)
                                                }?;

                                                let type2 = match tuple[1].as_str() {
                                                    None => Err(error!(format!("{} is not a string", tuple[1]))),
                                                    Some(a) => Ok(a)
                                                }?;

                                                let constant = match registry.get_type_alias_as_constant_type(initial_type.as_str()) {
                                                    None =>  match create_default_constant_from_str(initial_type.as_str()) {
                                                        Ok(a) => Ok(a),
                                                        Err(_) => Err(error!(format!("Could not find a valid type or alias for {}", initial_type)))
                                                    },
                                                    Some(a) => Ok(a.clone())
                                                }?;

                                                let operation = registry.get_mut_return_operation(op);
                                                let add_to = operation.entry(type1.into()).or_default();
                                                add_to.insert(type2.into(), constant);
                                                Ok(())
                                            } else {
                                                Ok(())
                                            }
                                        })
                                } else {
                                    Err(error!(format!("[Exclude] Types for filter_operation {} is not an array, got {:?}", op, ban_by_type)))
                                }
                            })
                } else {
                    Err(error!(format!("[Exclude] Type entry for filter_operation {} is not an object, got {:?}", op, to_ban)))
                }
            })
    } else { Ok(()) }
}