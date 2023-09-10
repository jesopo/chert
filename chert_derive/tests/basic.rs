use chert_accessor::{ChertField, ChertStruct};
use chert_derive::ChertStruct;

#[derive(Debug, ChertStruct)]
struct TestUint64 {
    test: u32,
}

#[test]
fn test_u32() {
    let fields = TestUint64::fields();
    if let Some((_, ChertField::Uint64(accessor))) = fields.get("test") {
        let test = TestUint64 { test: 123 };
        assert_eq!(accessor(&test), &123u32);
    } else {
        panic!("bad {fields:?}");
    };
}
