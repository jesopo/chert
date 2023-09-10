use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    #[clap(index = 1)]
    expression: String,
}

fn main() {
    let args = Arguments::parse();
    println!("{:#?}", chert::lex::lex(&args.expression));
}
