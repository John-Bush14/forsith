use std::fmt;


pub enum ForsithError {}


impl fmt::Display for ForsithError {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", match *self {
            _ => "Unreachable!"
        });
    }
}
