use std::fmt::{Debug, Display};


#[derive(Debug)]
pub enum DPError {
    ImpossibleTask,
    NoPathFound,
}

impl std::error::Error for DPError {}

impl Display for DPError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DPError::ImpossibleTask => write!(f, "task impossible to solve with given parameters"),
            DPError::NoPathFound => write!(f, "no valid path found"),
        }
    }
}


#[derive(Debug)]
pub enum ValueError<T> {
    NegativeValue(T),
}

impl<T: Debug + Display + Copy> std::error::Error for ValueError<T> {}

impl<T: Debug + Copy> Display for ValueError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ValueError::NegativeValue(val) => write!(f, "negative value not allowed: {:?}", val),
        }
    }
}
