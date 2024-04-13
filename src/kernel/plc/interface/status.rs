use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, PartialEq)]
pub enum InterfaceStatus {
    Default,
    Pending,
    Solved
}

impl Display for InterfaceStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InterfaceStatus::Default => write!(f, "Default"),
            InterfaceStatus::Pending => write!(f, "Pending"),
            InterfaceStatus::Solved => write!(f, "Solved"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum BodyStatus {
    Default,
    Pending,
    Solved
}

impl Display for BodyStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BodyStatus::Default => write!(f, "Default"),
            BodyStatus::Pending => write!(f, "Pending"),
            BodyStatus::Solved => write!(f, "Solved"),
        }
    }
}