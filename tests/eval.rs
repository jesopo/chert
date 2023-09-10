use chert::parse::nodes::Node;
use chert_derive::ChertStruct;

#[derive(Debug, ChertStruct)]
struct Foo {
    a: u64,
    b: String,
}

#[test]
fn test_1() {
    match chert::lex::lex("(a + 1) == 3") {
        Ok(tokens) => {
            let node = chert::parse::parse::<Foo>(tokens).unwrap();
            if let Node::Boolean(node) = node {
                let mut engine = chert::compile::compile(Vec::from([(0, node)]));
                engine.load_variables(&Foo {
                    a: 2,
                    b: "meow".to_owned(),
                });
                let matched = engine.eval();
                assert_eq!(matched, Vec::from([&0]));
            }
        }
        Err(e) => unreachable!("{e:?}"),
    }
}
