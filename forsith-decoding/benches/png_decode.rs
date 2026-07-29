use std::{fs::DirEntry, path::Path};

use criterion::{BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main, measurement::Measurement};
use forsith_decoding::{ImageDecoder, PngDecoder};

criterion_group!(benches, png_decode_benchmarks);
criterion_main!(benches);

fn png_decode_benchmarks(c: &mut Criterion) {
    let asset_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("assets");

    let benches: Vec<DirEntry> = std::fs::read_dir(asset_dir).unwrap().filter_map(|entry| {
        let entry = entry.unwrap();

        match entry.path().extension() {
            Some(ext) if ext == "png" => {Some(entry)}
            _ => None
        }
    }).collect();

    let mut g = c.benchmark_group("png_decode");
    for entry in &benches {
        let data = std::fs::read(entry.path()).unwrap();
        benchmark_decoding::<false>(&mut g, entry.file_name().into_string().unwrap(), &data);
        benchmark_decoding::<true>(&mut g, entry.file_name().into_string().unwrap(), &data);
    }; g.finish();
}

fn benchmark_decoding<const BUFFERED: bool>(g: &mut BenchmarkGroup<impl Measurement>, filename: String, data: &[u8]) {
    let info_decoder = PngDecoder::<_, u8, {forsith_decoding::PixelFormat::TruecolorAlpha as u8}>::open(data).unwrap();

    let size = if BUFFERED {info_decoder.min_buf_size()} else {info_decoder.max_buf_size()};

    let mut buf = vec![0u8; size];

    let dim = info_decoder.image_dimensions();
    g.throughput(Throughput::Bytes((dim.0 * dim.1 * info_decoder.source_pixel_format() as usize * info_decoder.source_bit_depth() as usize / 8) as u64));
    g.bench_function(BenchmarkId::new(if BUFFERED {"buffered"} else {"full"}, filename), |b| b.iter(|| {
        let mut decoder = PngDecoder::<_, u8, {forsith_decoding::PixelFormat::TruecolorAlpha as u8}>::open(data).unwrap();
        while decoder.read(&mut buf).unwrap() > 0 {};
    }));
}
