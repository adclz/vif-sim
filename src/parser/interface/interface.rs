use crate::kernel::plc::interface::struct_interface::StructInterface;
use crate::kernel::arch::local::pointer::LocalPointer;
use crate::kernel::registry::{get_or_insert_global_string, Kernel};
use crate::container::error::error::Stop;
use crate::{error};
use serde_json::{Value};
use std::collections::HashMap;
use crate::container::broadcast::broadcast::Broadcast;
use crate::parser::local_type::local_type::parse_local_type;
use crate::kernel::plc::interface::section::Section;
use crate::kernel::plc::types::primitives::traits::meta_data::SetMetaData;

pub fn parse_struct_interface(
    json: &Value,
    registry: &Kernel,
    channel: &Broadcast,
    section: &Option<Section>,
    previous_path: &[usize]
) -> Result<StructInterface, Stop> {
    let mut section_to_fill = HashMap::new();
    // Get all members
    let fields = json.as_object().ok_or_else(move || error!(
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
        let json = value
            .as_object()
            .ok_or_else(move || error!(format!(
            "member '{}' is not a valid struct interface object: {:?}",
            name, value
        )))?;

        let name = get_or_insert_global_string(name);
        let mut current_path = previous_path.to_vec();

        let mut pointer = LocalPointer::from(parse_local_type(json, registry, channel, &current_path)?);

        // Checks if type is allowed
        registry.check_excluded_type(&pointer)?;

        // Checks if type is allowed in section
        registry.check_excluded_type_in_section(&section, &pointer)?;

        // Set constant if section is constant
        if read_only {
          pointer.set_read_only(true);
        };

        current_path.push(name);

        // Set path
        pointer.set_path(current_path);

        // Fails if the member is already present

        match section_to_fill.insert(name, pointer) {
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
