#[cfg(test)]
mod tests {
    use fixedstr::str256;
    use crate::container::error::error::Stop;
    use crate::kernel::plc::types::primitives::traits::meta_data::MetaData;
    use crate::kernel::plc::types::primitives::traits::primitive_traits::{PrimitiveTrait, RawDisplay};
    use crate::kernel::plc::types::primitives::binaries::byte::Byte;
    use crate::kernel::plc::types::primitives::binaries::dword::DWord;
    use crate::kernel::plc::types::primitives::binaries::lword::LWord;
    use crate::kernel::plc::types::primitives::binaries::word::Word;
    use crate::kernel::plc::types::primitives::boolean::bool::Bool;
    use crate::kernel::plc::types::primitives::integers::dint::DInt;
    use crate::kernel::plc::types::primitives::integers::int::Int;
    use crate::kernel::plc::types::primitives::integers::lint::LInt;
    use crate::kernel::plc::types::primitives::integers::sint::SInt;
    use crate::kernel::plc::types::primitives::integers::udint::UDInt;
    use crate::kernel::plc::types::primitives::integers::uint::UInt;
    use crate::kernel::plc::types::primitives::integers::ulint::ULInt;
    use crate::kernel::plc::types::primitives::integers::usint::USInt;
    use crate::kernel::plc::types::primitives::floats::real::Real;
    use crate::kernel::plc::types::primitives::floats::lreal::LReal;
    use crate::kernel::plc::types::primitives::timers::time::Time;
    use crate::kernel::plc::types::primitives::timers::lTime::LTime;
    use crate::kernel::plc::types::primitives::tod::tod::Tod;
    use crate::kernel::plc::types::primitives::tod::ltod::LTod;
    use crate::kernel::plc::types::primitives::string::_string::_String;
    use crate::kernel::plc::types::primitives::string::_char::_Char;
    use crate::kernel::plc::types::primitives::string::wstring::WString;
    use crate::kernel::plc::types::primitives::string::wchar::WChar;

    #[test]
    fn check_names() {
        assert_eq!(Bool::new(&true).unwrap().name(), "Bool");
        assert_eq!(Byte::new(&0).unwrap().name(), "Byte");
        assert_eq!(Word::new(&0).unwrap().name(), "Word");
        assert_eq!(DWord::new(&0).unwrap().name(), "DWord");
        assert_eq!(LWord::new(&0).unwrap().name(), "LWord");
        assert_eq!(USInt::new(&0).unwrap().name(), "USInt");
        assert_eq!(SInt::new(&0).unwrap().name(), "SInt");
        assert_eq!(UInt::new(&0).unwrap().name(), "UInt");
        assert_eq!(Int::new(&0).unwrap().name(), "Int");
        assert_eq!(UDInt::new(&0).unwrap().name(), "UDInt");
        assert_eq!(DInt::new(&0).unwrap().name(), "DInt");
        assert_eq!(ULInt::new(&0).unwrap().name(), "ULInt");
        assert_eq!(LInt::new(&0).unwrap().name(), "LInt");
        assert_eq!(Real::new(&0.0).unwrap().name(), "Real");
        assert_eq!(LReal::new(&0.0).unwrap().name(), "LReal");
        assert_eq!(Time::new(&0).unwrap().name(), "Time");
        assert_eq!(LTime::new(&0).unwrap().name(), "LTime");
        assert_eq!(Tod::new(&0).unwrap().name(), "Tod");
        assert_eq!(LTod::new(&0).unwrap().name(), "LTod");
        assert_eq!(_String::new(&str256::new()).unwrap().name(), "_String");
        assert_eq!(_Char::new(&'0').unwrap().name(), "_Char");
        assert_eq!(WString::new(&str256::new()).unwrap().name(), "WString");
        assert_eq!(WChar::new(&'0').unwrap().name(), "WChar");
    }

    #[test]
    fn check_display() {
        assert_eq!(format!("{}", Bool::new(&true).unwrap()), "(Bool: true)");
        assert_eq!(format!("{}", Byte::new(&0).unwrap()), "(Byte: 0)");
        assert_eq!(format!("{}", Word::new(&0).unwrap()), "(Word: 0)");
        assert_eq!(format!("{}", DWord::new(&0).unwrap()), "(DWord: 0)");
        assert_eq!(format!("{}", LWord::new(&0).unwrap()), "(LWord: 0)");
        assert_eq!(format!("{}", USInt::new(&0).unwrap()), "(USInt: 0)");
        assert_eq!(format!("{}", SInt::new(&0).unwrap()), "(SInt: 0)");
        assert_eq!(format!("{}", UInt::new(&0).unwrap()), "(UInt: 0)");
        assert_eq!(format!("{}", Int::new(&0).unwrap()), "(Int: 0)");
        assert_eq!(format!("{}", UDInt::new(&0).unwrap()), "(UDInt: 0)");
        assert_eq!(format!("{}", DInt::new(&0).unwrap()), "(DInt: 0)");
        assert_eq!(format!("{}", ULInt::new(&0).unwrap()), "(ULInt: 0)");
        assert_eq!(format!("{}", LInt::new(&0).unwrap()), "(LInt: 0)");
        assert_eq!(format!("{}", Real::new(&0.0).unwrap()), "(Real: 0.0000000)");
        assert_eq!(format!("{}", LReal::new(&0.0).unwrap()), "(LReal: 0.000000000000000)");
        assert_eq!(format!("{}", Time::new(&0).unwrap()), "(Time: 0)");
        assert_eq!(format!("{}", LTime::new(&0).unwrap()), "(LTime: 0)");
        assert_eq!(format!("{}", Tod::new(&0).unwrap()), "(Tod: 0)");
        assert_eq!(format!("{}", LTod::new(&0).unwrap()), "(LTod: 0)");
        assert_eq!(format!("{}", _String::new(&str256::new()).unwrap()), "(_String: )");
        assert_eq!(format!("{}", _Char::new(&'0').unwrap()), "(_Char: '0')");
        assert_eq!(format!("{}", WString::new(&str256::new()).unwrap()), "(WString: )");
        assert_eq!(format!("{}", WChar::new(&'0').unwrap()), "(WChar: '0')");
    }

    #[test]
    fn check_raw_display() {
        assert_eq!(format!("{}", Bool::new(&true).unwrap().raw_display()), "true");
        assert_eq!(format!("{}", Byte::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", Word::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", DWord::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", LWord::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", USInt::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", SInt::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", UInt::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", Int::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", UDInt::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", DInt::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", ULInt::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", LInt::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", Real::new(&0.0).unwrap().raw_display()), "0.0000000");
        assert_eq!(format!("{}", LReal::new(&0.0).unwrap().raw_display()), "0.000000000000000");
        assert_eq!(format!("{}", Time::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", LTime::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", Tod::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", LTod::new(&0).unwrap().raw_display()), "0");
        assert_eq!(format!("{}", _String::new(&str256::new()).unwrap().raw_display()), "");
        assert_eq!(format!("{}", _Char::new(&'0').unwrap().raw_display()), "0");
        assert_eq!(format!("{}", _String::new(&str256::new()).unwrap().raw_display()), "");
        assert_eq!(format!("{}", _Char::new(&'0').unwrap().raw_display()), "0");
    }
}

