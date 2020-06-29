# nanoserde
![Documentation](https://docs.rs/nanoserde/badge.svg)
Fork of https://crates.io/crates/makepad-tinyserde with all the dependencies removed.
No more syn, proc_macro2 or quote in the build tree!

```
> cargo tree
nanoserde v0.1.0 (/../nanoserde)
└── nanoserde-derive v0.1.0 (/../nanoserde/derive)
```

Features support matrix:
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

And this is going to be even more restricted and limited serialization/deserialization library than makepad-tinyserde. 
Generic bounds, lifetime bounds, where clauses and probably a lot more is not supported and probably will never be supported.

This is used in [macroquad](https://github.com/not-fl3/macroquad/) game engine and only features needed for macroquad's internal serialization needs are going to be well supported. 

