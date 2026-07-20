use std::error::Error;
use std::{fs::File, io::BufReader};
use crate::{ImageDecoder, PixelFormat, PngDecoder};

const BUFFER_SIZE: usize = 1024 * 32;


include!("pngsuite/generated_tests.rs");


fn test_image(path: &str, solution_filepath: &str) -> Result<(), Box<dyn Error>> {

    let test_file = File::open(path).unwrap();

    let solution_file = File::open(solution_filepath).unwrap();
    let solution: Vec<u8> = serde_json::from_reader(&solution_file)?;

    let mut decoder = PngDecoder::<_, u8, {PixelFormat::TruecolorAlpha as u8}>::open(BufReader::new(test_file))?;

    let mut len = 1;
    let mut decoded_bytes = 0;
    let mut decoded_buf = [0u8; BUFFER_SIZE];
    while len > 0 {
        len = decoder.read(&mut decoded_buf)?;

        let decoded = &decoded_buf[..len];

        if decoded_bytes+len > solution.len() {
            panic!("decoded image contained more data than solution?");
        }

        let solution = &solution[decoded_bytes..decoded_bytes+len];

        if !decoded.iter().eq(solution.iter()) {
            for i in 0..len {
                if decoded[i] != solution[i] {
                    panic!("Mismatch at byte {}: decoded = {}, solution = {}", i, decoded[i], solution[i]);
                }
            }

            panic!("Decoded data does not match solution data");
        } else {
            println!("Decoded data matches solution data for {} bytes", len);
        }

        decoded_bytes += len;
    }

    if decoded_bytes != solution.len() {
        panic!("Solution file has more data than decoded data");
    }

    Ok(())
}
