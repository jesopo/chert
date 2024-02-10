use chert::Variables;
use cidr::IpCidr;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    #[clap(index = 1)]
    expression: String,
}

#[derive(Debug, Variables)]
struct Variables {
    a: u64,
    b: IpCidr,
}

fn main() {
    let args = Arguments::parse();

    let tokens = chert::lex::lex(&args.expression).unwrap();
    println!("{:#?}", chert::parse::parse::<Variables>(tokens));
}
