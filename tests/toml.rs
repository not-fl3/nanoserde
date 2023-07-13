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
