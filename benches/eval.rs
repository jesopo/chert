use cidr::{IpCidr, Ipv4Cidr};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::net::{IpAddr, Ipv4Addr};

use chert_derive::ChertStruct;

#[derive(ChertStruct, Debug)]
struct Foo0 {
    a: u64,
}

#[derive(ChertStruct, Debug)]
struct Foo1 {
    a: IpAddr,
}

fn criterion_benchmark(c: &mut Criterion) {
    let tokens = chert::lex::lex("a + 1 == 3").unwrap();
    let ast = chert::parse::parse::<Foo0>(tokens).unwrap();
    let node = match ast {
        chert::parse::nodes::Node::Boolean(node) => node,
        _ => unreachable!(),
    };
    let mut engine = chert::compile::compile::<Foo0>(Vec::from([node]));
    let variables = Foo0 { a: 2 };
    engine.load_variables(&variables);

    c.bench_function("basic-chert", |b| b.iter(|| engine.eval()));
    c.bench_function("basic-rust", |b| {
        b.iter(|| black_box(&variables.a) + black_box(1) == black_box(3))
    });

    let tokens = chert::lex::lex("a in 1.1.1.0/24").unwrap();
    let ast = chert::parse::parse::<Foo1>(tokens).unwrap();
    let node = match ast {
        chert::parse::nodes::Node::Boolean(node) => node,
        _ => unreachable!(),
    };
    let mut engine = chert::compile::compile::<Foo1>(Vec::from([node]));
    let variables = Foo1 {
        a: IpAddr::V4(Ipv4Addr::from(16843009)),
    };
    engine.load_variables(&variables);

    let rust_cidr = IpCidr::V4(Ipv4Cidr::new(16843008.into(), 24).unwrap());

    c.bench_function("basic-chert", |b| b.iter(|| engine.eval()));
    c.bench_function("basic-rust", |b| {
        b.iter(|| rust_cidr.contains(black_box(&variables.a)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
