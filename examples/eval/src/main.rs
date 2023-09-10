use chert::parse::nodes::Node;
use chert_derive::ChertStruct;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    #[clap(index = 1)]
    expression: String,
}

#[derive(Debug, ChertStruct)]
struct Variables {}

fn main() {
    let args = Arguments::parse();

    let tokens = chert::lex::lex(&args.expression).unwrap();
    let node = chert::parse::parse::<Variables>(tokens).unwrap();
    if let Node::Boolean(node) = node {
        println!("{node:?}");
        let mut engine = chert::compile::compile(Vec::from([node]));
        engine.load_variables(&Variables {});
        engine.eval();
        println!("{:?}", engine.heaps.boolean[engine.results[0]]);
    } else {
        panic!("expression must result in a boolean");
    }
}
