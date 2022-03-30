// TODO: Move to another crate

extern crate core_affinity;
use criterion::{criterion_group, criterion_main, Criterion};
use firestorm::*;
use thread_priority::{set_current_thread_priority, ThreadPriority};

fn loop_100() {
    profile_fn!(loop_20());
    for _ in 0..100 {
        profile_section!(inner);
        drop(inner);
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
    let core_ids = core_affinity::get_core_ids().unwrap();
    core_affinity::set_for_current(core_ids[0]);
    assert!(set_current_thread_priority(ThreadPriority::Max).is_ok());
    c.bench_function("soak", |b| {
        b.iter(|| {
            outer();
            clear();
        })
    });
    outer();
    save("flame-graph").unwrap();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
