use std::time::Instant;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn push_bench(c: &mut Criterion) {
    c.bench_function("push", |b| {
        b.iter_custom(|iters| {
            let mut vec = Vec::<u32>::new();

            let start = Instant::now();

            for i in 0..iters {
                vec.push(black_box(i as u32));
            }

            start.elapsed()
        });
    });
}

fn extend_many(c: &mut Criterion) {
    c.bench_function("extend 10k", |b| {
        b.iter_custom(|iters| {
            let to_add = (0..10000).collect::<Vec<u32>>();
            let mut vec = Vec::<u32>::new();

            let start = Instant::now();

            for _ in 0..iters {
                vec.extend(black_box(to_add.iter()));
            }

            start.elapsed()
        });
    });
}

fn extend_some(c: &mut Criterion) {
    c.bench_function("extend 100", |b| {
        b.iter_custom(|iters| {
            let to_add = (0..100).collect::<Vec<u32>>();
            let mut vec = Vec::<u32>::new();

            let start = Instant::now();

            for _ in 0..iters {
                vec.extend(black_box(to_add.iter()));
            }

            start.elapsed()
        });
    });
}

fn get_seq(c: &mut Criterion) {
    c.bench_function("get() seq.", |b| {
        b.iter_custom(|iters| {
            let vec = (0..iters as u32).collect::<Vec<u32>>();

            let start = Instant::now();

            for i in 0..iters {
                vec.get(i as usize);
            }

            start.elapsed()
        });
    });
}

fn get_rand(c: &mut Criterion) {
    c.bench_function("get() random", |b| {
        b.iter_custom(|iters| {
            let vec = (0..iters as u32).collect::<Vec<u32>>();

            let start = Instant::now();

            for i in 0..iters {
                vec.get(i as usize * 100 % vec.len());
            }

            start.elapsed()
        });
    });
}

fn pop(c: &mut Criterion) {
    c.bench_function("pop", |b| {
        b.iter_custom(|iters| {
            let mut vec = (0..iters as u32).collect::<Vec<u32>>();

            let start = Instant::now();

            for _ in 0..iters {
                vec.pop();
            }

            start.elapsed()
        });
    });
}

criterion_group!(
    benches,
    push_bench,
    extend_some,
    extend_many,
    pop,
    get_seq,
    get_rand
);

criterion_main!(benches);
