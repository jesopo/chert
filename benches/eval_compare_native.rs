use cidr::{IpCidr, Ipv4Cidr};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::net::{IpAddr, Ipv4Addr};

use chert_derive::ChertStruct;

#[derive(ChertStruct, Clone, Debug)]
struct Foo0 {
    i: u64,
}

#[derive(ChertStruct, Clone, Debug)]
struct Foo1 {
    ip: IpAddr,
}

fn criterion_benchmark(c: &mut Criterion) {
    let tokens = chert::lex::lex("i + 1 == 3").unwrap();
    let ast = chert::parse::parse::<Foo0>(tokens).unwrap();
    let node = match ast {
        chert::parse::nodes::Node::Boolean(node) => node,
        _ => unreachable!(),
    };
    let engine = chert::compile::compile(Vec::from([(0, node)]));
    let variables = Foo0 { i: 2 };

    let mut group = c.benchmark_group("$i + 1 == 3");
    group.bench_function("chert", |b| b.iter(|| engine.eval(&variables)));
    group.bench_function("rust", |b| {
        b.iter(|| black_box(&variables.i) + black_box(1) == black_box(3))
    });
    group.finish();

    let tokens = chert::lex::lex("ip in 1.1.1.0/24").unwrap();
    let ast = chert::parse::parse::<Foo1>(tokens).unwrap();
    let node = match ast {
        chert::parse::nodes::Node::Boolean(node) => node,
        _ => unreachable!(),
    };
    let engine = chert::compile::compile(Vec::from([(0, node)]));
    let variables = Foo1 {
        ip: IpAddr::V4(Ipv4Addr::from(16843009)),
    };

    let rust_cidr = IpCidr::V4(Ipv4Cidr::new(16843008.into(), 24).unwrap());

    let mut group = c.benchmark_group("$ip in 1.1.1.0/24");
    group.bench_function("chert", |b| b.iter(|| engine.eval(&variables)));
    group.bench_function("rust", |b| {
        b.iter(|| rust_cidr.contains(black_box(&variables.ip)))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
