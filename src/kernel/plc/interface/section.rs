use std::fmt::{Display, Formatter};

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum Section {
    Input,
    Output,
    InOut,
    Static,
    Temp,
    Constant,
    Return,
    NONE,
}

impl Display for Section {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Section::Input => write!(f, "Input"),
            Section::Output => write!(f, "Output"),
            Section::InOut => write!(f, "InOut"),
            Section::Static => write!(f, "Static"),
            Section::Temp => write!(f, "Temp"),
            Section::Constant => write!(f, "Constant"),
            Section::Return => write!(f, "Return"),
            Section::NONE => write!(f, "None"),
        }
    }
}