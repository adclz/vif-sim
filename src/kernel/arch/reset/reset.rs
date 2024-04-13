use crate::kernel::plc::types::primitives::traits::primitive_traits::{RawMut};
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;
use camelpaste::paste;

#[derive(Default)]
pub struct SectionPointers(Vec<*mut dyn RawMut>);
impl SectionPointers {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn push(&mut self, rst: *mut dyn RawMut) {
        self.0.push(rst);
    }

    pub fn append(&mut self, rst: &mut Vec<*mut dyn RawMut>) {
        self.0.append(rst);
    }

    pub fn reset(&mut self, channel: &Broadcast) -> Result<(), Stop> {
        self.0
            .iter_mut()
            .try_for_each(|f| {
                unsafe { (**f).reset_ptr(channel); }
                Ok(())
            })
    }
}

macro_rules! impl_section_pointers {
    ($($section: ident),+) => {
        paste! {
        #[derive(Default)]
        pub struct RawPointers {
            $($section: SectionPointers),+
        }

        impl RawPointers {
                $(
                    pub fn [<clear_$section:lower>](&mut self) {
                        self.$section.clear();
                    }

                    pub fn [<push_$section:lower>](&mut self, rst: *mut dyn RawMut) {
                        self.$section.push(rst);
                    }

                    pub fn [<append_$section:lower>](&mut self, rst: &mut Vec<*mut dyn RawMut>) {
                        self.$section.append(rst);
                    }

                    pub fn [<reset_$section:lower>](&mut self, channel: &Broadcast) -> Result<(), Stop> {
                        self.$section.reset(channel)
                    }
                )+

                pub fn clear_all(&mut self) {
                    $(
                        self.[<clear_$section:lower>]();
                    )+
                }

                pub fn reset_all(&mut self, channel: &Broadcast) -> Result<(), Stop> {
                    $(
                        self.[<reset_$section:lower>](channel)?;
                    )+
                    Ok(())
                }

                pub fn filter_dangling(&mut self) {
                    $(
                        self.$section.0.retain(|x| !x.is_null());
                    )+
                }
            }
        }
    };
}

impl_section_pointers!(
    Input, Output, Static, InOut, Constant, Temp
);
