use std::{error::Error, ffi::CString, fmt};


#[derive(Debug)]
pub enum ForsithError {
    InstanceExtensionNotPresent(CString)
}


impl fmt::Display for ForsithError {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[allow(unused_imports)]
        use ForsithError::*;

        return write!(f, "{}", match self {
            InstanceExtensionNotPresent(extension) => format!("Instance extension {:?} not present", extension)
        });
    }
}

impl Error for ForsithError {}
