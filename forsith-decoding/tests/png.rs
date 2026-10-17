use std::error::Error;
use std::io::BufRead;
use std::{fs::File, io::BufReader, path::PathBuf};
use forsith_decoding::{DecodingError, image::{ImageDecoder, PixelFormat, PngDecoder}};

include!("generated_png_tests.rs");

fn test_image(path: &str, solution_filepath: &str) -> Result<(), Box<dyn Error>> {

    let test_file = File::open(path).unwrap();

    let id = PathBuf::from(path).file_name().unwrap().to_str().unwrap()[..3].to_string();

    let mut decoder = match PngDecoder::<u8, {PixelFormat::TruecolorAlpha as u8}>::open(BufReader::new(test_file)) {
        Ok(d) => d,
        Err(e) => {if is_correct_err(&e, &id) {return Ok(())} else {return Err(Box::from(e))}}
    };

    let solution: Option<Vec<u8>> =
        if decoder.source_bit_depth() <= 8 {parse_solution(solution_filepath, |s| s.parse::<u8>().unwrap())}
        else {parse_solution(solution_filepath, |s| s.parse::<u16>().unwrap()).map(|s| s.iter().map(|&v| (v as u32 * 255 / u16::MAX as u32) as u8).collect())};

    let mut decoded_bytes = 0;
    let mut decoded_buf: Vec<u8> = vec![0u8; decoder.min_buf_size()];
    while match decoder.read(&mut decoded_buf) {
        Ok(len) => {
            let solution = solution.as_ref().unwrap_or_else(|| panic!("test that should fail succeeded!"));

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
            if solution.is_some() {return Err(Box::from(e));}

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
        | ("xhd", DecodingError::CRCMismatch(_, _))
        | ("xc1" | "xc9" | "xd0" | "xd3" | "xd9", DecodingError::InvalidChunk(_))
        | ("xdt", DecodingError::NoIDAT)
        | ("xcs", DecodingError::CRCMismatch(_, _))
        | ("xcr" | "xlf", _)
    )
}

fn parse_solution<T>(filepath: &str, parse_t: fn(&str) -> T) -> Option<Vec<T>> {
    let file = File::open(filepath).ok()?;
    let mut reader = BufReader::new(file);
    reader.skip_until(b'[').unwrap();

    let mut solution = vec![];

    let mut buf = vec![];
    loop {
        reader.read_until(b',', &mut buf).unwrap();

        let start = buf.iter().position(|&b| !b.is_ascii_whitespace()).unwrap();
        let end = buf[start..].iter().position(|&b| b.is_ascii_whitespace() || b == b',' || b == b']').unwrap() + start;

        solution.push(parse_t(str::from_utf8(&buf[start..end]).unwrap()));

        if buf.contains(&b']') {
            break;
        }

        buf.clear();
    }

    Some(solution)
}

