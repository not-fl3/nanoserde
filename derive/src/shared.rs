use alloc::string::String;

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

pub fn attrs_rename(attributes: &[crate::parse::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 2 && attr.tokens[0] == "rename" {
            Some(attr.tokens[1].clone())
        } else {
            None
        }
    })
}

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

pub fn attrs_default_with(attributes: &[crate::parse::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 2 && attr.tokens[0] == "default_with" {
            Some(attr.tokens[1].clone())
        } else {
            None
        }
    })
}

pub fn attrs_transparent(attributes: &[crate::parse::Attribute]) -> bool {
    attributes
        .iter()
        .any(|attr| attr.tokens.len() == 1 && attr.tokens[0] == "transparent")
}

pub fn attrs_skip(attributes: &[crate::parse::Attribute]) -> bool {
    attributes
        .iter()
        .any(|attr| attr.tokens.len() == 1 && attr.tokens[0] == "skip")
}

pub(crate) fn struct_bounds_strings(struct_: &Struct, bound_name: &str) -> (String, String) {
    let generics: &Vec<_> = &struct_.generics;

    if generics.is_empty() {
        return ("".to_string(), "".to_string());
    }
    let mut generic_w_bounds = "<".to_string();
    for generic in generics.iter().filter(|g| g.ident_only() != "Self") {
        generic_w_bounds += generic
            .full_with_const(&[format!("nanoserde::{}", bound_name).as_str()], true)
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
    return (generic_w_bounds, generic_no_bounds);
}

pub(crate) fn enum_bounds_strings(enum_: &Enum, bound_name: &str) -> (String, String) {
    let generics: &Vec<_> = &enum_.generics;

    if generics.is_empty() {
        return ("".to_string(), "".to_string());
    }
    let mut generic_w_bounds = "<".to_string();
    for generic in generics.iter().filter(|g| g.ident_only() != "Self") {
        generic_w_bounds += generic
            .full_with_const(&[format!("nanoserde::{}", bound_name).as_str()], true)
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
    return (generic_w_bounds, generic_no_bounds);
}
