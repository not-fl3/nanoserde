use nanoserde::{DeBin, DeJson, DeRon, SerBin, SerJson, SerRon};

// https://github.com/not-fl3/nanoserde/issues/83
#[test]
fn test_trailing_comma() {
    #[rustfmt::skip]
    #[derive(Debug, DeBin, SerBin, DeJson, SerJson, DeRon, SerRon)]
    enum TestEnum {
        A
    }
}
