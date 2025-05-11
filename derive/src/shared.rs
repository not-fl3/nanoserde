#![cfg(any(feature = "json", feature = "ron", feature = "binary"))]

#[cfg(any(feature = "json", feature = "binary"))]
use alloc::{format, string::ToString, vec::Vec};

use alloc::string::String;

#[cfg(any(feature = "binary", feature = "json"))]
use crate::parse::{Enum, Struct};

macro_rules! l {
    ($target:ident, $line:expr) => {
        $target.push_str($line)
    };

    ($target:ident, $line:expr, $($param:expr),*) => {
        $target.push_str(&::alloc::format!($line, $($param,)*))
    };
}

pub fn attrs_proxy(attributes: &[crate::parse::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 2 && attr.tokens[0] == "proxy" {
            Some(attr.tokens[1].clone())
        } else {
            None
        }
    })
}

#[cfg(any(feature = "ron", feature = "json"))]
pub fn attrs_rename(attributes: &[crate::parse::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 2 && attr.tokens[0] == "rename" {
            Some(attr.tokens[1].clone())
        } else {
            None
        }
    })
}

#[cfg(any(feature = "ron", feature = "json"))]
pub fn attrs_default(attributes: &[crate::parse::Attribute]) -> Option<Option<String>> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 1 && attr.tokens[0] == "default" {
            Some(None)
        } else if attr.tokens.len() == 2 && attr.tokens[0] == "default" {
            Some(Some(attr.tokens[1].clone()))
        } else {
            None
        }
    })
}

#[cfg(any(feature = "ron", feature = "json"))]
pub fn attrs_default_with(attributes: &[crate::parse::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 2 && attr.tokens[0] == "default_with" {
            Some(attr.tokens[1].clone())
        } else {
            None
        }
    })
}

#[cfg(feature = "json")]
pub fn attrs_transparent(attributes: &[crate::parse::Attribute]) -> bool {
    attributes
        .iter()
        .any(|attr| attr.tokens.len() == 1 && attr.tokens[0] == "transparent")
}

#[cfg(any(feature = "json", feature = "ron", feature = "binary"))]
pub fn attrs_skip(attributes: &[crate::parse::Attribute]) -> bool {
    attributes
        .iter()
        .any(|attr| attr.tokens.len() == 1 && attr.tokens[0] == "skip")
}

#[cfg(feature = "json")]
pub fn attrs_serialize_none_as_null(attributes: &[crate::parse::Attribute]) -> bool {
    attributes
        .iter()
        .any(|attr| attr.tokens.len() == 1 && attr.tokens[0] == "serialize_none_as_null")
}

pub fn attrs_crate(attributes: &[crate::parse::Attribute]) -> Option<&str> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 2 && attr.tokens[0] == "crate" {
            Some(attr.tokens[1].as_str())
        } else {
            None
        }
    })
}

#[cfg(any(feature = "binary", feature = "json"))]
pub(crate) fn struct_bounds_strings(
    struct_: &Struct,
    bound_name: &str,
    crate_name: &str,
) -> (String, String) {
    let generics: &Vec<_> = &struct_.generics;

    if generics.is_empty() {
        return ("".to_string(), "".to_string());
    }
    let mut generic_w_bounds = "<".to_string();
    for generic in generics.iter().filter(|g| g.ident_only() != "Self") {
        generic_w_bounds += generic
            .full_with_const(&[format!("{}::{}", crate_name, bound_name).as_str()], true)
            .as_str();
        generic_w_bounds += ", ";
    }
    generic_w_bounds += ">";

    let mut generic_no_bounds = "<".to_string();
    for generic in generics.iter().filter(|g| g.ident_only() != "Self") {
        generic_no_bounds += generic.ident_only().as_str();
        generic_no_bounds += ", ";
    }
    generic_no_bounds += ">";
    (generic_w_bounds, generic_no_bounds)
}

#[cfg(any(feature = "binary", feature = "json"))]
pub(crate) fn enum_bounds_strings(
    enum_: &Enum,
    bound_name: &str,
    crate_name: &str,
) -> (String, String) {
    let generics: &Vec<_> = &enum_.generics;

    if generics.is_empty() {
        return ("".to_string(), "".to_string());
    }
    let mut generic_w_bounds = "<".to_string();
    for generic in generics.iter().filter(|g| g.ident_only() != "Self") {
        generic_w_bounds += generic
            .full_with_const(&[format!("{}::{}", crate_name, bound_name).as_str()], true)
            .as_str();
        generic_w_bounds += ", ";
    }
    generic_w_bounds += ">";

    let mut generic_no_bounds = "<".to_string();
    for generic in generics.iter().filter(|g| g.ident_only() != "Self") {
        generic_no_bounds += generic.ident_only().as_str();
        generic_no_bounds += ", ";
    }
    generic_no_bounds += ">";
    (generic_w_bounds, generic_no_bounds)
}
