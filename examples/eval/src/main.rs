use chert::Variables;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    #[clap(index = 1)]
    expression: String,
}

#[derive(Clone, Debug, Variables)]
struct Variables {
    _a: i64,
}

fn main() {
    let args = Arguments::parse();

    if let Ok(ast) = chert::parse(&args.expression) {
        println!("{ast:?}");
        let engine = chert::compile::compile(Vec::from([(0, ast)]));
        let results = engine.eval(&Variables { _a: 0 });
        println!("{results:?}");
    } else {
        panic!("expression must result in a boolean");
    }
}
