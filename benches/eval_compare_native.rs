use chert::Variables;
use cidr::{IpCidr, Ipv4Cidr};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::net::{IpAddr, Ipv4Addr};

fn criterion_1(c: &mut Criterion) {
    let mut group = c.benchmark_group("$i + 1 == 3");
    #[derive(Variables, Debug)]
    struct Variables {
        i: u64,
    }

    let ast = chert::parse("i + 1 == 3").unwrap();
    let engine = chert::compile(Vec::from([(0, ast)])).unwrap();
    let variables = Variables { i: 2 };

    group.bench_function("chert", |b| b.iter(|| engine.eval(&variables)));
    group.bench_function("rust", |b| {
        b.iter(|| black_box(&variables.i) + black_box(1) == black_box(3))
    });
}

fn criterion_2(c: &mut Criterion) {
    let mut group = c.benchmark_group("$ip in 1.1.1.0/24");
    #[derive(Variables, Debug)]
    struct Variables {
        ip: IpAddr,
    }

    let ast = chert::parse("ip in 1.1.1.0/24").unwrap();
    let engine = chert::compile(Vec::from([(0, ast)])).unwrap();
    let variables = Variables {
        ip: IpAddr::V4(Ipv4Addr::from(16843009)),
    };

    let rust_cidr = IpCidr::V4(Ipv4Cidr::new(16843008.into(), 24).unwrap());

    group.bench_function("chert", |b| b.iter(|| engine.eval(&variables)));
    group.bench_function("rust", |b| {
        b.iter(|| rust_cidr.contains(black_box(&variables.ip)))
    });
}

criterion_group!(benches, criterion_1, criterion_2);
criterion_main!(benches);
