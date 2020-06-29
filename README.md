# nanoserde

[![Github Actions](https://github.com/not-fl3/nanoserde/workflows/Cross-compile/badge.svg)](https://github.com/not-fl3/nanoserde/actions?query=workflow%3A)
[![Crates.io version](https://img.shields.io/crates/v/nanoserde.svg)](https://crates.io/crates/nanoserde)
[![Documentation](https://docs.rs/nanoserde/badge.svg)](https://docs.rs/nanoserde)
[![Discord chat](https://img.shields.io/discord/710177966440579103.svg?label=discord%20chat)](https://discord.gg/WfEp6ut)

Fork of https://crates.io/crates/makepad-tinyserde with all the dependencies removed.
No more syn, proc_macro2 or quote in the build tree!

```
> cargo tree
nanoserde v0.1.0 (/../nanoserde)
└── nanoserde-derive v0.1.0 (/../nanoserde/derive)
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

For more examples take a look on [tests](/tests)

## Features support matrix:

| Feature                                        | json   | bin   | ron    | toml  |
| ---------------------------------------------- | ------ | ----- | ------ | ----- |
| serialization                                  | yes    | yes   | no     | no    |
| deserialization                                | yes    | yes   | yes    | no    |
| container: Struct                              | yes    | yes   | yes    | no    |
| container: Tuple Struct                        | no     | yes   | no     | no    |
| container: Enum                                | no     | no    | no     | no    |
| field: `std::collections::HashMap`             | yes    | yes   | yes    | no    |
| field: `std::vec::Vec`                         | yes    | yes   | yes    | no    |
| field: `Option`                                | yes    | yes   | yes    | no    |
| field: `i*`/`f*`/`String`/`T: De*/Ser*`        | yes    | yes   | yes    | no    |
| field attribute: `#[nserde(default)]`          | yes    | no    | yes    | no    |
| field attribute: `#[nserde(rename = "")]`      | yes    | yes   | yes    | no    |
| field attribute: `#[nserde(proxy = "")]`       | no     | yes   | no     | no    |
| container attribute: `#[nserde(default)]`      | yes    | no    | yes    | no    |
| container attribute: `#[nserde(rename = "")]`  | yes    | yes   | yes    | no    |
| container attribute: `#[nserde(proxy = "")]`   | yes    | yes   | no     | no    |

