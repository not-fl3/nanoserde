# nanoserde

[![Github Actions](https://github.com/not-fl3/nanoserde/workflows/Cross-compile/badge.svg)](https://github.com/not-fl3/nanoserde/actions?query=workflow%3A)
[![Crates.io version](https://img.shields.io/crates/v/nanoserde.svg)](https://crates.io/crates/nanoserde)
[![Documentation](https://docs.rs/nanoserde/badge.svg)](https://docs.rs/nanoserde)
[![Discord chat](https://img.shields.io/discord/710177966440579103.svg?label=discord%20chat)](https://discord.gg/WfEp6ut)

Fork of https://crates.io/crates/makepad-tinyserde with all the dependencies removed.
No more syn, proc_macro2 or quote in the build tree!

```
> cargo tree
nanoserde v0.2.0 (/../nanoserde)
└── nanoserde-derive v0.2.0 (/../nanoserde/derive)
```

## Example:

```rust
use nanoserde::{DeJson, SerJson};

#[derive(Clone, Debug, Default, DeJson, SerJson)]
pub struct Property {
    pub name: String,
    #[nserde(default)]
    pub value: String,
    #[nserde(rename = "type")]
    pub ty: String,
}
```

For more examples take a look at [tests](/tests)

## Features support matrix:

| Feature                                                   | json   | bin   | ron    | toml  |
| ---------------------------------------------------       | ------ | ----- | ------ | ----- |
| serialization                                             |   •    |   •   |   •    | no    |
| deserialization                                           |   •    |   •   |   •    | no    |
| container: Struct                                         |   •    |   •   |   •    | no    |
| container: Tuple Struct                                   | no     |   •   |   •    | no    |
| container: Enum                                           |   •    |   •   |   •    | no    |
| field: `std::collections::HashMap`                        |   •    |   •   |   •    | no    |
| field: `std::vec::Vec`                                    |   •    |   •   |   •    | no    |
| field: `Option`                                           |   •    |   •   |   •    | no    |
| field: `i*`/`f*`/`String`/`T: De*/Ser*`                   |   •    |   •   |   •    | no    |
| field attribute: `#[nserde(default)]`                     |   •    | no    |   •    | no    |
| field attribute: `#[nserde(rename = "")]`                 |   •    |   •   |   •    | no    |
| field attribute: `#[nserde(proxy = "")]`                  | no     |   •   | no     | no    |
| field attribute: `#[nserde(serialize_none_as_null)]`      |   •    | no    | no     | no    |
| container attribute: `#[nserde(default)]`                 |   •    | no    |   •    | no    |
| container attribute: `#[nserde(default = "")]`            |   •    | no    |   •    | no    |
| container attribute: `#[nserde(default_with = "")]`       |   •    | no    |   •    | no    |
| container attribute: `#[nserde(skip)]` (implies `default`)|   •    | no    |   •    | no    |
| container attribute: `#[nserde(serialize_none_as_null)]`  |   •    | no    | no     | no    |
| container attribute: `#[nserde(rename = "")]`             |   •    |   •   |   •    | no    |
| container attribute: `#[nserde(proxy = "")]`              |   •    |   •   | no     | no    |
| container attribute: `#[nserde(transparent)]`             |   •    | no    | no     | no    |
| container attribute: `#[nserde(crate = "")]`              |   •    |   •   |   •    | no    |

• ≝ yes

## Crate features:

All features are enabled by default. To enable only specific formats, import nanoserde using
```toml
nanoserde = { version = "*", default-features = false, features = ["std", "{format feature name}"] }
```
in your `Cargo.toml` and add one or more of the following crate features:

| Format    | Feature Name   |
| ----------| -------------- |
| Binary    | `binary`       |
| JSON      | `json`         |
| RON       | `ron`          |
| TOML      | `toml`         |
