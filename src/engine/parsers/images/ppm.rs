use crate::engine::parsers::images::{read_decimal_int, skip_bytes};

use super::ImageParser;
use std::path::Path;
use std::fs::File;
use super::read_bytes;


const MAGIC_NUMBER: [u8; 2] = ['P' as u8, '6' as u8];

const PLAIN_MAGIC_NUMBER: [u8; 2] = ['P' as u8, '6' as u8];


pub struct PPMParser {}


impl ImageParser for PPMParser {
    fn parse(path: &Path) -> Result<((u32, u32), u64, Vec<u8>), Box<dyn std::error::Error>> {
        let mut file: File = File::open(path)?;


        let magic_number: [u8; 2] = read_bytes(&mut file, [0u8;2])?;

        let plain = magic_number == PLAIN_MAGIC_NUMBER;

        if magic_number != MAGIC_NUMBER && !plain {panic!("Called PPMParser::parse on non-ppm file (wrong magic number)");}


        skip_bytes(&mut file, 1);


        let mut dimensions = [0;2];

        for dimension in dimensions.iter_mut() {*dimension = read_decimal_int(&mut file)?;}


        todo!();
    }
}
