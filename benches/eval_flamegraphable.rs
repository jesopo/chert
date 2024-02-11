use chert::Variables;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[derive(Clone, Variables, Debug)]
struct Variables {
    a: u64,
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut expressions = Vec::new();
    for i in 0..1000 {
        expressions.push((i, chert::parse(&format!("a + 1 == {i}")).unwrap()));
    }
    let engine = chert::compile(expressions);
    let variables = Variables { a: 2 };

    c.bench_function("bench", |b| b.iter(|| engine.eval(black_box(&variables))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
