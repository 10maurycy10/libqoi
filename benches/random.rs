use criterion::{black_box, criterion_group, criterion_main, Criterion};
use libqoi;
use rand;
use rand::Rng;

fn random_image(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let pixels = (0..size);
    let img: Vec<u8> = pixels.map(|x| vec![rng.gen(), rng.gen(), rng.gen(), 0]).flatten().collect();
    img
}

fn criterion_benchmark(c: &mut Criterion) {
    let img_vec = random_image(2048 * 2048);
    let img = &img_vec[..];
    let compressed = libqoi::encode_qoi(img, 2048, 2048, 4, 1).unwrap();
    c.bench_function("Compress synthetic", |b| b.iter(|| libqoi::encode_qoi(black_box(img), 2048, 2048,4,1)));
    c.bench_function("Decompress synthetic", |b| b.iter(|| libqoi::decode_qoi(black_box(&compressed))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
