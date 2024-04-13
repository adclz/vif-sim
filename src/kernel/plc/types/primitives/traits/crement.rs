#[warn(unused_imports)]
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;

use crate::kernel::plc::types::primitives::integers::sint::SInt;
use crate::kernel::plc::types::primitives::integers::usint::USInt;
use crate::kernel::plc::types::primitives::integers::uint::UInt;
use crate::kernel::plc::types::primitives::integers::int::Int;
use crate::kernel::plc::types::primitives::integers::dint::DInt;
use crate::kernel::plc::types::primitives::integers::udint::UDInt;
use crate::kernel::plc::types::primitives::integers::lint::LInt;
use crate::kernel::plc::types::primitives::integers::ulint::ULInt;
use crate::kernel::plc::types::primitives::integers::plc_integer::PlcInteger;

use crate::kernel::plc::types::primitives::binaries::byte::Byte;
use crate::kernel::plc::types::primitives::binaries::word::Word;
use crate::kernel::plc::types::primitives::binaries::dword::DWord;
use crate::kernel::plc::types::primitives::binaries::lword::LWord;
use crate::kernel::plc::types::primitives::binaries::plc_binary::PlcBinary;


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
