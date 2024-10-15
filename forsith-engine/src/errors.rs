use std::fmt;


#[derive(Debug)]
pub enum ForsithError {
    HandleCreationFailed(String)
}


impl fmt::Display for ForsithError {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ForsithError::*;

        return write!(f, "{}", match self {
            HandleCreationFailed(handle) => format!("tried to create {} handle, but handle was still 0 after func call", handle),
        });
    }
}

impl Error for ForsithError {}
