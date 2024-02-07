#[test]
fn test_serialize() {
    #[derive(chert::ChertStruct, Debug)]
    struct Variables {
        a: u64,
    }

    let ast = chert::parse::<Variables>("a == 1").unwrap();
    let ast = serde_json::to_string_pretty(&ast.get_root()).unwrap();

    let ast: chert::NodeBoolean = serde_json::from_str(&ast).unwrap();
    let engine = unsafe { chert::compile_unsafe::<Variables, _, _, _>(Vec::from([(0, ast)])) };
    engine.eval(&Variables { a: 1 });
}

#[test]
fn test_serialize_with_id() {
    #[derive(chert::ChertStruct, Debug)]
    struct Variables {
        a: u64,
    }

    let ast = chert::parse::<Variables>("a == 1").unwrap();
    let ast = serde_json::to_string_pretty(&Vec::from([(0, ast.get_root())])).unwrap();

    let asts: Vec<(i32, chert::NodeBoolean)> = serde_json::from_str(&ast).unwrap();
    let engine = unsafe { chert::compile_unsafe::<Variables, _, _, _>(asts) };
    engine.eval(&Variables { a: 1 });
}
