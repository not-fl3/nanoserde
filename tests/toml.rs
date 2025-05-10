#![cfg(feature = "toml")]

extern crate alloc;

use alloc::collections::BTreeMap;
use nanoserde::Toml;
use nanoserde::TomlParser;

#[test]
fn de_toml() {
    let toml_str = r#"
[[array]]
# a
name = "a" #aasdf

[[array]]
name = "b#asdf"
data = 123

[[array]]
name = "c"

[[other_array]]
hmm = "a"

[[other_array]]
hmm = "b"

"#;

    let toml = TomlParser::parse(toml_str).unwrap();

    assert_eq!(toml["array"].arr()[0]["name"].str(), "a");
    assert_eq!(toml["array"].arr()[1]["name"].str(), "b#asdf");
    assert_eq!(toml["array"].arr()[1]["data"].num(), 123.);

    assert_eq!(toml["other_array"].arr()[0]["hmm"].str(), "a");
    assert_eq!(toml["other_array"].arr()[1]["hmm"].str(), "b");
}

#[test]
fn comment() {
    let data = r#"
    # This is a full-line comment
    key = "value"  # This is a comment at the end of a line
    another = " # This is not a comment"
    "#;

    let toml = TomlParser::parse(data).unwrap();
    assert_eq!(toml["key"].str(), "value");
    assert_eq!(toml["another"].str(), " # This is not a comment");
}

#[test]
fn key_without_value() {
    let data = r#"
    key = # INVALID
    "#;

    assert!(TomlParser::parse(data).is_err());
}

#[test]
fn assert_specific_toml_types() {
    let data = r#"
    num = 3.14
    str = "quoth the raven"
    simple_arr = [1, 2, 3, 4]
    boolean = false
    date = 1979-05-27
    "#;
    assert_eq!(TomlParser::parse(data).unwrap()["num"].num(), 3.14);
    assert_eq!(
        TomlParser::parse(data).unwrap()["str"].str(),
        "quoth the raven"
    );
    assert_eq!(TomlParser::parse(data).unwrap()["boolean"].boolean(), false);
    assert_eq!(
        TomlParser::parse(data).unwrap()["date"].date(),
        "1979-05-27".to_string()
    );
    assert_eq!(
        TomlParser::parse(data).unwrap()["simple_arr"].simple_arr(),
        &vec![
            Toml::Num(1.0),
            Toml::Num(2.0),
            Toml::Num(3.0),
            Toml::Num(4.0)
        ]
    );
}

#[test]
fn toml_key_chars() {
    let toml_str = r#"
        [foo.bar.baz]
        123abc456def = "myval"
        -inf = 0
        2024-04-30 = 100
        ½ = 0.5
    "#;

    assert_eq!(
        TomlParser::parse(toml_str).unwrap(),
        BTreeMap::from([
            (
                "foo.bar.baz.123abc456def".to_string(),
                Toml::Str("myval".to_string())
            ),
            ("foo.bar.baz.-inf".to_string(), Toml::Num(0.0)),
            ("foo.bar.baz.2024-04-30".to_string(), Toml::Num(100.0)),
            ("foo.bar.baz.½".to_string(), Toml::Num(0.5))
        ])
    );
}

#[test]
fn carriage_return() {
    let toml_str = "foo = 1\r\nbar = false\r\n";
    let toml = TomlParser::parse(toml_str).unwrap();

    assert_eq!(toml["foo"].num(), 1.0);
    assert_eq!(toml["bar"].boolean(), false);
}
