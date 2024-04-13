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
    $({ $section: ident }),+) => {
        camelpaste::paste! {
            $data_interface
            .iter()
            .try_for_each(|(section, value)| match section.as_str() {
                "return" => Ok(()),
                $(stringify!([<$section:lower>]) => {
                    let h = parse_struct_interface(value, $registry, $channel, &Some(Section::$section), &vec!())?;
                    let section = $interface.entry(Section::$section).or_insert_with(|| h);

                        $registry
                            .raw_pointers_collector
                            .borrow_mut()
                            .[<append_$section:lower>](&mut section.get_raw_pointers());

                    Ok(())
                })+,
                _ => Err(error!(format!("Invalid section name: '{}'", section), format!("Parse Section")))
            })
        }
    };
}