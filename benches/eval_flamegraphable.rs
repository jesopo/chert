use criterion::{black_box, criterion_group, criterion_main, Criterion};

use chert_derive::ChertStruct;

#[derive(Clone, ChertStruct, Debug)]
struct Variables {
    a: u64,
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut expressions = Vec::new();
    for i in 0..1000 {
        expressions.push((
            i,
            chert::parse::<Variables>(&format!("a + 1 == {i}")).unwrap(),
        ));
    }
    let engine = chert::compile::compile(expressions);
    let variables = Variables { a: 2 };

    c.bench_function("bench", |b| b.iter(|| engine.eval(black_box(&variables))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
