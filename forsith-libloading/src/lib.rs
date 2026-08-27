#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::option_map_unit_fn)]

mod parsing;

// for warnings
pub fn use_parsing() {
    let _ = parsing::XmlDocument::parse(b"<root></root>".into());
}
