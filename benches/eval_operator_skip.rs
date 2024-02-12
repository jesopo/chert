use chert::Variables;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[derive(Clone, Variables, Debug)]
struct Variables {
    nick: String,
    host: String,
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut expressions = Vec::new();
    for i in 0..10_000 {
        expressions.push((
            i,
            chert::parse(&format!("nick == 'meow' and host == 'meow'")).unwrap(),
        ));
    }
    let engine = chert::compile(expressions).unwrap();

    let variables = Variables {
        nick: String::from("meow"),
        host: String::from("meow"),
    };
    c.bench_function("can't skip", |b| {
        b.iter(|| engine.eval(black_box(&variables)))
    });

    let variables = Variables {
        nick: String::from("purr"),
        host: String::from("meow"),
    };
    c.bench_function("can skip", |b| {
        b.iter(|| engine.eval(black_box(&variables)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
