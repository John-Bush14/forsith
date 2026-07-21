use std::error::Error;
use std::{fs::File, io::BufReader, path::PathBuf};
use crate::{DecodingError, ImageDecoder, PixelFormat, PngDecoder};


include!("pngsuite/generated_tests.rs");

fn test_image(path: &str, solution_filepath: &str) -> Result<(), Box<dyn Error>> {

    let test_file = File::open(path).unwrap();

    let solution = File::open(solution_filepath);
    let solution: Result<Vec<u8>, _> = solution.map(|f| serde_json::from_reader(f).unwrap());

    let id = PathBuf::from(path).file_name().unwrap().to_str().unwrap()[..3].to_string();

    let mut decoder = match PngDecoder::<_, u8, {PixelFormat::TruecolorAlpha as u8}>::open(BufReader::new(test_file)) {
        Ok(d) => d,
        Err(e) => {if is_correct_err(&e, &id) {return Ok(())} else {return Err(Box::from(e))}}
    };

    let mut decoded_bytes = 0;
    let mut decoded_buf: Vec<u8> = vec![0u8; decoder.min_buf_size()];
    while match decoder.read(&mut decoded_buf) {
        Ok(len) => {
            let solution = solution.as_ref().unwrap_or_else(|e| panic!("test that should fail succeeded!"));

            let decoded = &decoded_buf[..len];

            if decoded_bytes+len > solution.len() {
                panic!("decoded image contained more data than solution?");
            }

            let solution = &solution[decoded_bytes..decoded_bytes+len];

            if !decoded.iter().zip(solution.iter()).all(|(d, &c)| d.abs_diff(c) <= 1) {
                for i in 0..len {
                    if decoded[i] != solution[i] {
                        panic!("Mismatch at byte {}: decoded = {}, solution = {}", i, decoded[i], solution[i]);
                    }
                }

                panic!("Decoded data does not match solution data");
            } else {
                println!("Decoded data matches solution data for {} bytes", len);
            }

            decoded_bytes += len; len
        },
        Err(e) => {
            if is_correct_err(&e, &id) {return Ok(());}
            if solution.is_ok() {return Err(Box::from(e));}

            panic!("Failed with incorrect error {e}");
        }
    } > 0 {}

    if decoded_bytes != solution.unwrap().len() {
        panic!("Solution file has more data than decoded data");
    }

    Ok(())
}

fn is_correct_err(err: &DecodingError, id: &str) -> bool {
    matches!((id, err),
        ("xs1", DecodingError::InccorectHeader(_))
        | ("xs2", DecodingError::InccorectHeader(_))
        | ("xs4", DecodingError::InccorectHeader(_))
        | ("xs7", DecodingError::InccorectHeader(_))
        | ("xhd", DecodingError::CRCMismatch(1443964200, 1129534797))
        | ("xc1" | "xc9" | "xd0" | "xd3" | "xd9", DecodingError::InvalidChunk(super::ChunkType::Ihdr))
        | ("xdt", DecodingError::NoIDAT)
        | ("xcs", DecodingError::CRCMismatch(3492746441, 1129534797))
        | ("xcr" | "xlf", _)
    )
}
