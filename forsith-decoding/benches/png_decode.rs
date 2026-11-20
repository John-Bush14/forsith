use std::{fs::DirEntry, path::Path};

use forsith_bench::Bencher;
use forsith_decoding::image::{ChannelLayout, ImageDecoder, PixelFormat, PngDecoder};

fn main() {
    let mut b = Bencher::new();

    let asset_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("assets");

    let benches: Vec<DirEntry> = std::fs::read_dir(asset_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();

            match entry.path().extension() {
                Some(ext) if ext == "png" => Some(entry),
                _ => None,
            }
        })
        .collect();

    for entry in &benches {
        let data = std::fs::read(entry.path()).unwrap();
        benchmark_decoding::<false>(&mut b, entry.file_name().into_string().unwrap(), &data);
        benchmark_decoding::<true>(&mut b, entry.file_name().into_string().unwrap(), &data);
    }
}

fn benchmark_decoding<const BUFFERED: bool>(b: &mut Bencher, filename: String, data: &[u8]) {
    let info_decoder = PngDecoder::open(data, PixelFormat::default()).unwrap();

    let size = if BUFFERED {
        info_decoder.min_buf_size()
    } else {
        info_decoder.max_buf_size()
    };

    let mut buf = vec![0u8; size];

    let dim = info_decoder.image_dimensions();
    // g.throughput(Throughput::Bytes(
    //     (dim.0 * dim.1 * info_decoder.pixel_format().bpp() as usize / 8) as u64,
    // ));
    b.bench(
        &format!(
            "{} ({}x{}) [{}] decoding",
            filename,
            dim.0,
            dim.1,
            if BUFFERED { "buffered" } else { "full" }
        ),
        || {
            let mut decoder = PngDecoder::open(data, PixelFormat::default()).unwrap();
            while decoder.read(&mut buf).unwrap() > 0 {}
        },
    );
}
