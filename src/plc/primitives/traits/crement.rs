#[warn(unused_imports)]
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;

use crate::plc::primitives::integers::sint::SInt;
use crate::plc::primitives::integers::usint::USInt;
use crate::plc::primitives::integers::uint::UInt;
use crate::plc::primitives::integers::int::Int;
use crate::plc::primitives::integers::dint::DInt;
use crate::plc::primitives::integers::udint::UDInt;
use crate::plc::primitives::integers::lint::LInt;
use crate::plc::primitives::integers::ulint::ULInt;
use crate::plc::primitives::integers::plc_integer::PlcInteger;

use crate::plc::primitives::binaries::byte::Byte;
use crate::plc::primitives::binaries::word::Word;
use crate::plc::primitives::binaries::dword::DWord;
use crate::plc::primitives::binaries::lword::LWord;
use crate::plc::primitives::binaries::plc_binary::PlcBinary;


#[enum_dispatch::enum_dispatch]
pub trait Crement {
    fn increment(&mut self, channel: &Broadcast) -> Result<(), Stop>;
    fn decrement(&mut self, channel: &Broadcast) -> Result<(), Stop>;
}

#[macro_export]
macro_rules! impl_primitive_crement {
    ($primitive: ident) => {
        impl Crement for $primitive {
            fn increment(&mut self, channel: &Broadcast) -> Result<(), Stop> {
                self.set(self.value + 1, channel)
            }

            fn decrement(&mut self, channel: &Broadcast) -> Result<(), Stop> {
                self.set(self.value - 1, channel)
            }
        }
    };
}
