use std::fs::DirEntry;

use criterion::{Criterion, criterion_group, criterion_main, BenchmarkGroup, measurement::Measurement, Throughput};
use forsith_decoding::{ImageDecoder, PngDecoder};

criterion_group!(benches, png_decode_benchmarks);
criterion_main!(benches);

fn png_decode_benchmarks(c: &mut Criterion) {
    let benches: Vec<DirEntry> = std::fs::read_dir("benches/assets").unwrap().filter_map(|entry| {
        let entry = entry.unwrap();

        match entry.path().extension() {
            Some(ext) if ext == "png" => {Some(entry)}
            _ => None
        }
    }).collect();

    let mut maxg = c.benchmark_group("decode_full");
    for entry in &benches {
        let data = std::fs::read(entry.path()).unwrap();
        benchmark_decoding::<false>(&mut maxg, entry.file_name().into_string().unwrap(), &data);
    }; maxg.finish();

    let mut ming = c.benchmark_group("decode_buffered");
    for entry in &benches {
        let data = std::fs::read(entry.path()).unwrap();
        benchmark_decoding::<true>(&mut ming, entry.file_name().into_string().unwrap(), &data);
    }; ming.finish();
}

fn benchmark_decoding<const BUFFERED: bool>(g: &mut BenchmarkGroup<impl Measurement>, filename: String, data: &[u8]) {
    let check_decoder = PngDecoder::<_, u8, {forsith_decoding::PixelFormat::TruecolorAlpha as u8}>::open(data).unwrap();

    let size = if BUFFERED {check_decoder.min_buf_size()} else {check_decoder.max_buf_size()};

    let mut buf = vec![0u8; size];

    g.throughput(Throughput::Bytes(check_decoder.max_buf_size() as u64));
    g.bench_function(filename, |b| b.iter(|| {
        let mut decoder = PngDecoder::<_, u8, {forsith_decoding::PixelFormat::TruecolorAlpha as u8}>::open(data).unwrap();
        while decoder.read(&mut buf).unwrap() > 0 {};
    }));
}
