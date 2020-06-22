use nanoserde::{DeBin, DeBinErr, SerBin};

#[test]
fn binary() {
    #[derive(DeBin, SerBin, PartialEq)]
    pub struct Test {
        pub a: i32,
        pub b: f32,
        c: Option<String>,
        d: Option<String>,
    }

    let test: Test = Test {
        a: 1,
        b: 2.,
        c: Some("asd".to_string()),
        d: None,
    };

    let bytes = SerBin::serialize_bin(&test);

    let test_deserialized = DeBin::deserialize_bin(&bytes).unwrap();

    assert!(test == test_deserialized);
}
