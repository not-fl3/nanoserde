use nanoserde::{DeBin, DeJson, SerBin, SerJson};

use std::collections::HashMap;

#[test]
fn ser_de() {
    #[derive(DeBin, SerBin, DeJson, SerJson, PartialEq)]
    pub struct Test {
        pub a: i32,
        pub b: f32,
        c: Option<String>,
        d: Option<String>,
        e: Option<HashMap<String, String>>,
        f: Option<([u32; 4], String)>,
    }

    let mut map = HashMap::new();
    map.insert("a".to_string(), "b".to_string());

    let test: Test = Test {
        a: 1,
        b: 2.,
        c: Some("asd".to_string()),
        d: None,
        e: Some(map),
        f: Some(([1, 2, 3, 4], "tuple".to_string())),
    };

    let bytes = SerBin::serialize_bin(&test);
    let test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();
    assert!(test == test_deserialized);

    let bytes = SerJson::serialize_json(&test);
    let test_deserialized = DeJson::deserialize_json(&bytes).unwrap();
    assert!(test == test_deserialized);
}
