use chert::parse::nodes::Node;

fn cidr(s: &'static str) -> cidr::IpCidr {
    use std::str::FromStr as _;
    cidr::IpCidr::from_str(s).unwrap()
}
fn ip(s: &'static str) -> std::net::IpAddr {
    use std::str::FromStr as _;
    std::net::IpAddr::from_str(s).unwrap()
}
fn re(s: &'static str) -> regex::Regex {
    regex::Regex::new(s).unwrap()
}

#[test]
fn test_boolean() {
    #[derive(chert::ChertStruct, Debug)]
    struct Variables {
        a: bool,
    }
    if let Ok(Node::Boolean(node)) = chert::parse("a && true") {
        let engine = chert::compile(Vec::from([(0, node)]));
        assert_eq!(engine.eval(&Variables { a: true }), &[&0]);
        assert_eq!(engine.eval(&Variables { a: false }), &[&0; 0]);
    } else {
        unreachable!();
    }
}

#[test]
fn test_cidr() {
    #[derive(chert::ChertStruct, Debug)]
    struct Variables {
        a: cidr::IpCidr,
    }
    if let Ok(Node::Boolean(node)) = chert::parse("1.1.1.1 in a") {
        let engine = chert::compile(Vec::from([(0, node)]));
        assert_eq!(
            engine.eval(&Variables {
                a: cidr("1.1.1.0/24")
            }),
            &[&0]
        );
        assert_eq!(
            engine.eval(&Variables {
                a: cidr("1.1.2.0/24")
            }),
            &[&0; 0]
        );
    } else {
        unreachable!();
    }
}

#[test]
fn test_int64() {
    #[derive(chert::ChertStruct, Debug)]
    struct Variables {
        a: i64,
    }
    if let Ok(Node::Boolean(node)) = chert::parse("a == -1") {
        let engine = chert::compile(Vec::from([(0, node)]));
        assert_eq!(engine.eval(&Variables { a: -1 }), &[&0]);
        assert_eq!(engine.eval(&Variables { a: -2 }), &[&0; 0]);
    } else {
        unreachable!();
    }
}

#[test]
fn test_ip() {
    #[derive(chert::ChertStruct, Debug)]
    struct Variables {
        a: std::net::IpAddr,
    }
    if let Ok(Node::Boolean(node)) = chert::parse("a == 1.1.1.1") {
        let engine = chert::compile(Vec::from([(0, node)]));
        assert_eq!(engine.eval(&Variables { a: ip("1.1.1.1") }), &[&0]);
        assert_eq!(engine.eval(&Variables { a: ip("1.1.1.2") }), &[&0; 0]);
    } else {
        unreachable!();
    }
}

#[test]
fn test_string() {
    #[derive(chert::ChertStruct, Debug)]
    struct Variables {
        a: String,
    }
    if let Ok(Node::Boolean(node)) = chert::parse("a == 'foo'") {
        let engine = chert::compile(Vec::from([(0, node)]));
        assert_eq!(
            engine.eval(&Variables {
                a: String::from("foo")
            }),
            &[&0]
        );
        assert_eq!(
            engine.eval(&Variables {
                a: String::from("bar")
            }),
            &[&0; 0]
        );
    } else {
        unreachable!();
    }
}

#[test]
fn test_uint64() {
    #[derive(chert::ChertStruct, Debug)]
    struct Variables {
        a: u64,
    }
    if let Ok(Node::Boolean(node)) = chert::parse("a == 1") {
        let engine = chert::compile(Vec::from([(0, node)]));
        assert_eq!(engine.eval(&Variables { a: 1 }), &[&0]);
        assert_eq!(engine.eval(&Variables { a: 2 }), &[&0; 0]);
    } else {
        unreachable!();
    }
}

#[test]
fn test_regex() {
    #[derive(chert::ChertStruct, Debug)]
    struct Variables {
        a: regex::Regex,
    }
    if let Ok(Node::Boolean(node)) = chert::parse("'foo' ~ a") {
        let engine = chert::compile(Vec::from([(0, node)]));
        assert_eq!(engine.eval(&Variables { a: re("f..") }), &[&0]);
        assert_eq!(engine.eval(&Variables { a: re("b..") }), &[&0; 0]);
    } else {
        unreachable!();
    }
}

#[test]
fn test_all_types() {
    #[derive(chert::ChertStruct, Debug)]
    struct Variables {
        a: bool,
        b: cidr::IpCidr,
        c: i64,
        d: std::net::IpAddr,
        e: String,
        f: u64,
        g: regex::Regex,
    }
    if let Ok(Node::Boolean(node)) = chert::parse(
        &Vec::from([
            "a == true",
            "1.1.1.1 in b",
            "c == -1",
            "d == 1.1.1.1",
            "e == 'foo'",
            "f == 1",
            "'foo' ~ g",
        ])
        .join(" and "),
    ) {
        let engine = chert::compile(Vec::from([(0, node)]));
        assert_eq!(
            engine.eval(&Variables {
                a: true,
                b: cidr("1.1.1.0/24"),
                c: -1,
                d: ip("1.1.1.1"),
                e: String::from("foo"),
                f: 1,
                g: re("f.."),
            }),
            &[&0]
        );
        assert_eq!(
            engine.eval(&Variables {
                a: false,
                b: cidr("1.1.2.0/24"),
                c: -2,
                d: ip("1.1.1.2"),
                e: String::from("bar"),
                f: 2,
                g: re("b.."),
            }),
            &[&0; 0]
        );
    } else {
        unreachable!();
    }
}
