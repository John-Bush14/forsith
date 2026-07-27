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
    let check_decoder = PngDecoder::<_, u8, {forsith_decoding::PixelFormat::TruecolorAlpha as u8}>::open(data).unwrap();

    let size = if BUFFERED {check_decoder.min_buf_size()} else {check_decoder.max_buf_size()};

    let mut buf = vec![0u8; size];

    g.throughput(Throughput::Bytes(check_decoder.max_buf_size() as u64));
    g.bench_function(BenchmarkId::new(if BUFFERED {"buffered"} else {"full"}, filename), |b| b.iter(|| {
        let mut decoder = PngDecoder::<_, u8, {forsith_decoding::PixelFormat::TruecolorAlpha as u8}>::open(data).unwrap();
        while decoder.read(&mut buf).unwrap() > 0 {};
    }));
}
