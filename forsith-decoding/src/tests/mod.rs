#[cfg(test)]
mod decoding_tests {
    use std::{fs::File, io::BufReader, path::PathBuf};

    use crate::{DecodingError, ImageDecoder, PixelFormat, PngDecoder};

    #[test]
    fn image_decoding_tests() -> Result<(), DecodingError> {
        let assets_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("tests")
            .join("assets");

        let test_file = File::open(assets_path.clone().join("test.png")).unwrap();

        let _decoder = PngDecoder::<_, u8, {PixelFormat::TruecolorAlpha as u8}>::open(BufReader::new(test_file))?;

        Ok(())
    }
}
