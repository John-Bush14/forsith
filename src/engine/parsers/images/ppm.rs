use crate::engine::parsers::images::{read_decimal_int, read_decimal_word};

use super::ImageParser;
use std::path::Path;
use std::fs::File;
use super::read_bytes;


pub struct PPMParser {}


impl ImageParser for PPMParser {
    fn parse(path: &Path) -> Result<((u32, u32), u64, Vec<u8>), Box<dyn std::error::Error>> {
        let mut file: File = File::open(path)?;


        let magic_number: String = read_decimal_word(&mut file)?;

        let plain = &magic_number == "P3";

        if &magic_number != "P6" && !plain {panic!("Called PPMParser::parse on non-ppm file (wrong magic number)");}


        let mut dimensions = [0u32;2];

        for dimension in dimensions.iter_mut() {*dimension = read_decimal_int(&mut file)? as u32;}

        let size = (dimensions[0] * dimensions[1]*4 * std::mem::size_of::<u8>() as u32) as u64;


        let maxval = read_decimal_int(&mut file)?; let sample_size = if maxval < 256 {1} else {2};


        let mut pixels: Vec<u8> = Vec::with_capacity(size as usize);


        for _y in 0..dimensions[0] {
            for _x in 0..dimensions[1] {
                for sample in 0..4 {
                    if sample == 3 {pixels.push(255u8); continue;}

                    pixels.push(
                        ((match (plain, sample_size) {
                            (true, _) => read_decimal_int(&mut file)?,
                            (false, 1) => read_bytes(&mut file, [0u8])?[0] as u16,
                            (false, 2) => u16::from_le_bytes(read_bytes(&mut file, [0u8;2])?),
                            _ => unreachable!()
                        } as f32 / maxval as f32) * 255.0) as u8
                    );
                }
            }
        }

        return Ok((dimensions.into(), size, pixels));
    }
}
