use forsith_decoding::{ImageDecoder, PngDecoder};
use std::path::PathBuf;

const REPETITION_COUNT: usize = 100;

fn main() {
    let test_file = include_bytes!("../assets/test.png");

    let mut buffer = [0u8; 1080 * 1920 * 3];

    let total_time = std::time::Instant::now();
    for _ in 0..REPETITION_COUNT {
        let mut decoder = PngDecoder::<_, 8, {forsith_decoding::PixelFormat::Truecolor as u8}>::open(test_file.as_slice()).unwrap();
        while decoder.read(&mut buffer).unwrap() > 0 {};
    }

    println!(
        "Average time for decoding PNG: {:?}",
        total_time.elapsed() / REPETITION_COUNT as u32
    );
}
