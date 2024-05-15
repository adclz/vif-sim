/// Creates a block interface.
///
/// All sections are created, except for return.
///
/// $data_interface has to be a [`HashMap<String, Value>`].
///
/// Appends all sections and members into $interface.
///
/// Appends all pointers to kernel.
///
#[macro_export]
macro_rules! create_block_interface {
    ($data_interface: expr,
    $interface: expr,
    $registry: ident,
    $channel: ident,
    $({ $section: ident }),+,
    $monitor: ident) => {
        camelpaste::paste! {
            {
                let mut all_names: Vec<usize> = vec!();
            
            $data_interface
            .iter()
            .try_for_each(|(section, value)| match section.as_str() {
                "return" => Ok(()),
                $(stringify!([<$section:lower>]) => {
                    // Creates the section
                    let section_struct = parse_struct_interface(value, $registry, $channel, &Some(Section::$section), $monitor)?;
                    
                    // Checks if names exists in another section
                    let names = section_struct.get_names();
                    
                    if let Some(name) = names.iter().find(|name| all_names.contains(name)) {
                        return Err(error!(format!("Invalid member name: {} already exists", get_string(*name))));
                    }
                    
                    // Saves the names
                    all_names.extend(names);
                    
                    // Insert it
                    let section = $interface.entry(Section::$section).or_insert_with(|| section_struct);
                    
                    // Saves the pointers
                        $registry
                            .raw_pointers_collector
                            .borrow_mut()
                            .[<append_$section:lower>](&mut section.get_raw_pointers());

                    Ok(())
                })+,
                _ => Err(error!(format!("Invalid section name: '{}'", section), format!("Parse Section")))
            })
        }
        }
    };
}