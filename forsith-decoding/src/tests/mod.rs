#[cfg(test)]
mod decoding_tests {
    use std::{fs::File, io::{BufReader, Read}, path::PathBuf};

    use crate::{DecodingError, ImageDecoder, PixelFormat, PngDecoder};

    #[test]
    fn image_decoding_tests() -> Result<(), DecodingError> {
        let assets_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("tests")
            .join("assets");

        let test_png = File::open(assets_path.clone().join("test.png")).unwrap();
        let mut decoder = PngDecoder::<_, 8, {PixelFormat::TruecolorAlpha as u8}>::open(BufReader::new(test_png))?;

        let mut solution_file = File::open(assets_path.join("test.raw")).unwrap();

        let mut len = 1;
        let mut solution_buf = vec![0u8; 0];
        while len > 0 {
            let decoded_buf = decoder.fill_buf()?;
            len = decoded_buf.len();
            solution_buf.resize(len, 0);

            solution_file.read_exact(&mut solution_buf)?;

            if !decoded_buf.iter().eq(solution_buf.iter()) {
                panic!("Decoded data does not match solution data");
            } else {
                println!("Decoded data matches solution data for {} bytes", len);
            }

            decoder.consume(len);
        }

        let mut rest_of_solution = Vec::new();
        solution_file.read_to_end(&mut rest_of_solution)?;
        if !rest_of_solution.is_empty() {
            panic!("Solution file has more data than decoded data");
        }

        Ok(())
    }
}
