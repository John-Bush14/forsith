use std::{error::Error, fmt};


#[derive(Debug)]
pub enum ForsithError {
}


impl fmt::Display for ForsithError {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[allow(unused_imports)]
        use ForsithError::*;

        return write!(f, "{}", match self {
            _ => "unreachable"
        });
    }
}

impl Error for ForsithError {}
