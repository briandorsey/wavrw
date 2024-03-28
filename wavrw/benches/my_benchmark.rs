#![allow(missing_docs)]
use core::fmt::Write;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Curiosity hex encoding comparison.
fn hex_benchmark(c: &mut Criterion) {
    let input_int = 73968139221736414906607698143988937859_u128;

    c.bench_function("hex_fmt", |b| b.iter(|| hex_fmt(black_box(input_int))));

    c.bench_function("hex_encode", |b| {
        b.iter(|| hex_encode(black_box(input_int)));
    });

    c.bench_function("hex_iter", |b| b.iter(|| hex_iter(black_box(input_int))));
}

fn hex_fmt(input: u128) -> String {
    format!("0x{:X}", input)
}

fn hex_encode(input: u128) -> String {
    hex::encode(input.to_be_bytes())
}

fn hex_iter(input: u128) -> String {
    input
        .to_be_bytes()
        .iter()
        .fold(String::new(), |mut output, b| {
            let _ = write!(&mut output, "{:02X}", b);
            output
        })
}

criterion_group!(benches, hex_benchmark);
criterion_main!(benches);
