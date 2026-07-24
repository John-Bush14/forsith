use criterion::{criterion_group, Criterion};
use forsith_decoding::{ImageDecoder, PngDecoder};
use gungraun::prelude::*;

macro_rules! benchmarks {
    ($group:ident: $max_size:expr => {$(
        $bench:ident => $file:literal, $size:expr
    ),+}) => {
        $(
            fn $bench(buf: &mut [u8]) {
                benchmark_decoding(include_bytes!($file), buf).unwrap()
            }

            #[library_benchmark]
            fn $bench() {super::$bench(&mut [0u8; $size])}
        )+

        fn $group(c: &mut Criterion) {
            let mut buf = [0u8; $max_size];

            $(
                c.bench_function("benchmark 1080p rgba image", |b| b.iter(|| $bench(&mut buf)));
            )+
        }

        criterion_group!(benches, $group);

        library_benchmark_group!(name = $group, benchmarks = [$($bench,)+]);
    };
}

benchmarks!(png_benchmarks: 1080*1920*4 => {
    decode_1080p_8rgba_image => "assets/test-rgba.png", 1080*1920*4,
    decode_1080p_8rgb_image => "assets/test-rgb.png", 1080*1920*3
});

fn benchmark_decoding(data: &[u8], buffer: &mut [u8]) -> Result<(), forsith_decoding::DecodingError> {
    let mut decoder = PngDecoder::<_, u8, {forsith_decoding::PixelFormat::TruecolorAlpha as u8}>::open(data).unwrap();
    while decoder.read(buffer).unwrap() > 0 {}; Ok(())
}
