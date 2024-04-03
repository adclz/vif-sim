use crate::plc::interface::struct_interface::StructInterface;
use crate::registry::local::pointer::LocalPointer;
use crate::registry::registry::Kernel;
use crate::container::error::error::Stop;
use crate::{error};
use serde_json::{Value};
use std::collections::HashMap;
use crate::container::broadcast::broadcast::Broadcast;
use crate::parser::local_type::local_type::parse_local_type;
use crate::plc::interface::section::Section;

pub fn parse_struct_interface(
    json: &Value,
    registry: &Kernel,
    channel: &Broadcast,
    section: &Option<Section>
) -> Result<StructInterface, Stop> {
    let mut section_to_fill = HashMap::new();
    // Get all members
    let fields = json.as_object().ok_or(error!(
        format!("Data for section of interface is not of type Object"),
        format!("Parse interface section")
    ))?;

    let section = match section {
        None => Section::NONE,
        Some(a) => a.clone()
    };

    // Check if section is constant
    let read_only = matches!(section, Section::Constant);

    // Add the local pointer to each member
    fields.iter().try_for_each(|(name, value)| {
        let json = value.as_object().ok_or(error!(format!(
            "member '{}' is not a valid struct interface object: {:?}",
            name, value
        )))?;

        let mut pointer = LocalPointer::from(parse_local_type(json, registry, channel)?);

        // Checks if type is allowed
        registry.check_excluded_type(&pointer)?;

        // Checks if type is allowed in section
        registry.check_excluded_type_in_section(&section, &pointer)?;

        // Set constant if section is constant
        if read_only {
          pointer.set_read_only(true);
        };
        // Fails if the member is already present
        match section_to_fill.insert(name.to_string(), pointer) {
            None => Ok(()),
            Some(_) => Err(error!(
                format!("Could not create type '{}' because it is already present", name),
                format!("Parse member")
            )),
        }?;
        Ok(())
    })?;
    Ok(StructInterface::from(section_to_fill))
}
