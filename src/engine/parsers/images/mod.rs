use std::path::Path;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use crate::DynError;


mod ppm;


pub trait ImageParser {
    fn parse(file: &Path) -> Result<((u32, u32), u64, Vec<u8>), DynError>;
}


pub(crate) fn read_bytes<T: AsMut<[u8]>>(file: &mut File, mut buffer: T) -> Result<T, DynError> {
    file.read_exact(buffer.as_mut())?;

    return Ok(buffer);
}

pub(crate) fn read_decimal_int(file: &File) -> Result<u16, DynError> {return Ok(read_decimal_word(file)?.parse::<u16>()?);}

pub(crate) fn read_decimal_word(file: &File) -> Result<String, DynError> {todo!();}

#[allow(unreachable_code, dead_code, non_snake_case)]
pub fn parse_image(file: &Path) -> Result<((u32, u32), u64, Vec<u8>), Box<dyn std::error::Error>> {
    let file_extension: Option<&OsStr> = file.extension();

    return match file_extension {
        None => panic!("parse_image: path {:?} doesn't have file extension?", file),
        Some(extension) => match extension.to_str().unwrap() {
            "ppm" => return ppm::PPMParser::parse(file),
            _ => panic!("unsupported format (parse_image): {:?}", file)
        }
    }
}
