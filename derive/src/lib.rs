#![cfg_attr(feature = "no_std", no_std)]

extern crate alloc;
extern crate proc_macro;

#[macro_use]
mod shared;

#[cfg(feature = "binary")]
mod serde_bin;
#[cfg(feature = "binary")]
use crate::serde_bin::*;

#[cfg(feature = "ron")]
mod serde_ron;
#[cfg(feature = "ron")]
use crate::serde_ron::*;

#[cfg(feature = "json")]
mod serde_json;
#[cfg(feature = "json")]
use crate::serde_json::*;

mod parse;

#[cfg(feature = "binary")]
#[proc_macro_derive(SerBin, attributes(nserde))]
pub fn derive_ser_bin(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    if let Some(proxy) = shared::attrs_proxy(&input.attributes()) {
        return derive_ser_bin_proxy(&proxy, &input.name());
    }

    // ok we have an ident, its either a struct or a enum
    let ts = match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_ser_bin_struct(struct_),
        parse::Data::Struct(struct_) => derive_ser_bin_struct_unnamed(struct_),
        parse::Data::Enum(enum_) => derive_ser_bin_enum(enum_),
        _ => unimplemented!("Only structs and enums are supported"),
    };

    ts
}

#[cfg(feature = "binary")]
#[proc_macro_derive(DeBin, attributes(nserde))]
pub fn derive_de_bin(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    if let Some(proxy) = shared::attrs_proxy(&input.attributes()) {
        return derive_de_bin_proxy(&proxy, &input.name());
    }

    // ok we have an ident, its either a struct or a enum
    let ts = match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_de_bin_struct(struct_),
        parse::Data::Struct(struct_) => derive_de_bin_struct_unnamed(struct_),
        parse::Data::Enum(enum_) => derive_de_bin_enum(enum_),

        _ => unimplemented!("Only structs and enums are supported"),
    };

    ts
}

#[cfg(feature = "ron")]
#[proc_macro_derive(SerRon, attributes(nserde))]
pub fn derive_ser_ron(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    if let Some(proxy) = shared::attrs_proxy(&input.attributes()) {
        return derive_ser_ron_proxy(&proxy, &input.name());
    }

    // ok we have an ident, its either a struct or a enum
    let ts = match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_ser_ron_struct(struct_),
        parse::Data::Struct(struct_) => derive_ser_ron_struct_unnamed(struct_),
        parse::Data::Enum(enum_) => derive_ser_ron_enum(enum_),
        _ => unimplemented!("Only structs and enums are supported"),
    };

    ts
}

#[cfg(feature = "ron")]
#[proc_macro_derive(DeRon, attributes(nserde))]
pub fn derive_de_ron(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    if let Some(proxy) = shared::attrs_proxy(&input.attributes()) {
        return derive_de_ron_proxy(&proxy, &input.name());
    }

    // ok we have an ident, its either a struct or a enum
    let ts = match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_de_ron_struct(struct_),
        parse::Data::Struct(struct_) => derive_de_ron_struct_unnamed(struct_),
        parse::Data::Enum(enum_) => derive_de_ron_enum(enum_),
        _ => unimplemented!("Only structs and enums are supported"),
    };

    ts
}

#[cfg(feature = "json")]
#[proc_macro_derive(SerJson, attributes(nserde))]
pub fn derive_ser_json(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    if let Some(proxy) = shared::attrs_proxy(&input.attributes()) {
        return derive_ser_json_proxy(&proxy, &input.name());
    }

    // ok we have an ident, its either a struct or a enum
    let ts = match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_ser_json_struct(struct_),
        parse::Data::Struct(struct_) => derive_ser_json_struct_unnamed(struct_),
        parse::Data::Enum(enum_) => derive_ser_json_enum(enum_),
        _ => unimplemented!(""),
    };

    ts
}

#[cfg(feature = "json")]
#[proc_macro_derive(DeJson, attributes(nserde))]
pub fn derive_de_json(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse::parse_data(input);

    if let Some(proxy) = shared::attrs_proxy(&input.attributes()) {
        return derive_de_json_proxy(&proxy, &input.name());
    }

    // ok we have an ident, its either a struct or a enum
    let ts = match &input {
        parse::Data::Struct(struct_) if struct_.named => derive_de_json_struct(struct_),
        parse::Data::Struct(struct_) => derive_de_json_struct_unnamed(struct_),
        parse::Data::Enum(enum_) => derive_de_json_enum(enum_),
        parse::Data::Union(_) => unimplemented!("Unions are not supported"),
    };

    ts
}
