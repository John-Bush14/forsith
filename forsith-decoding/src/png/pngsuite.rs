#[cfg(test)]
use std::{error::Error, io::Seek};


#[cfg(test)]
#[test]
fn pngsuite_png_decoding_tests() -> Result<(), Box<dyn Error>> {
    use std::{fs::File, io::{BufReader, Read}, path::PathBuf};
    use crate::{ImageDecoder, PixelFormat, PngDecoder};
    use std::fs;
    use std::path::Path;

    const BUFFER_SIZE: usize = 1024 * 32;

    let pngsuite_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("png")
        .join("pngsuite");

    for entry in fs::read_dir(pngsuite_dir.clone().join("png"))? {
        let entry = entry?;

        let filename = entry.file_name();
        let test_file = File::open(pngsuite_dir.clone().join("png").join(filename.clone())).unwrap();

        let mut solution_filename = PathBuf::from(filename.clone());
        solution_filename.set_extension("json");
        if filename.as_encoded_bytes().ends_with(b"16.png") {
            let mut stem = solution_filename.file_stem().unwrap().to_os_string();

            stem.push("_to8.json");

            solution_filename = PathBuf::from(stem);
        }
        let solution_file = File::open(pngsuite_dir.join("json").join(solution_filename)).unwrap();
        let solution: Vec<u8> = serde_json::from_reader(&solution_file)?;

        println!("decoding {filename:?}!");

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

        println!("decoded {filename:?} correctly!");
    }

    Ok(())
}
