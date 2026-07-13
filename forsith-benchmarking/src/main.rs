use std::time::Duration;

use forsith_decoding::{ImageDecoder, PngDecoder};

const REPETITION_COUNT: usize = 100;

fn main() {
    let rgba = include_bytes!("../assets/test-rgba.png");
    let rgb = include_bytes!("../assets/test-rgb.png");

    let mut buffer = [0u8; 1080 * 1920 * 3];


    println!(
        "Average time for decoding 1080p rgba PNG: {:?}",
        benchmark_decoding(rgba, &mut buffer).unwrap()
    );

    println!(
        "Average time for decoding 1080p rgb PNG: {:?}",
        benchmark_decoding(rgb, &mut buffer).unwrap()
    );
}

fn benchmark_decoding(data: &[u8], buffer: &mut [u8]) -> Result<Duration, forsith_decoding::DecodingError> {
    let total_time = std::time::Instant::now();
    for _ in 0..REPETITION_COUNT {
        let mut decoder = PngDecoder::<_, 8, {forsith_decoding::PixelFormat::Truecolor as u8}>::open(data).unwrap();
        while decoder.read(buffer).unwrap() > 0 {};
    }

    Ok(total_time.elapsed() / REPETITION_COUNT as u32)
}
