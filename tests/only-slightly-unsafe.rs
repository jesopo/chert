#[test]
fn test_regex() {
    #[derive(chert::ChertStruct, Debug)]
    struct Variables {
        a: u64,
    }

    let ast = chert::parse::<Variables>("a == 1").unwrap();
    let ast = serde_json::to_string_pretty(&ast.root).unwrap();

    let ast: chert::NodeBoolean = serde_json::from_str(&ast).unwrap();
    let engine = unsafe { chert::compile_unsafe::<Variables, _>(Vec::from([(0, ast)])) };
    engine.eval(&Variables { a: 1 });
}
