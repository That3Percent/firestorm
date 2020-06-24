#![feature(const_type_name)]

use criterion::{criterion_group, criterion_main, Criterion};
use firestorm::*;

fn loop_100() {
    profile_fn!(loop_20());
    for _ in 0..100 {
        profile_section!(inner);
    }
}

fn e() {
    profile_fn!(e());
}

fn d() {
    profile_fn!(d());
    e();
    e();
}

fn c() {
    profile_fn!(c());
    d();
    e();
    d();
    e();
}

struct B;
impl B {
    fn b() {
        profile_method!(b());
        c();
        d();
        c();
        d();
    }
}

fn a() {
    profile_fn!(a());
    B::b();
    c();
    d();
    B::b();
    c();
    d();
}

fn outer() {
    profile_fn!(outer());
    loop_100();
    a();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("soak", |b| {
        b.iter(|| {
            outer();
            clear();
        })
    });
    outer();
    firestorm::to_svg(&mut std::fs::File::create("flame-graph.svg").unwrap()).unwrap();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
