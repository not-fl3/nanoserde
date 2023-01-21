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
    for (generic, extra_bounds) in generics.iter() {
        let mut bounds = extra_bounds.join("+");
        if !bounds.is_empty() {
            bounds += " + ";
        }
        bounds += &format!("nanoserde::{}", bound_name);

        generic_w_bounds += &format!("{}: {}, ", generic, bounds);
    }
    generic_w_bounds += ">";

    let mut generic_no_bounds = "<".to_string();
    for (generic, _bounds) in generics.iter() {
        generic_no_bounds += &format!("{}, ", generic);
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
    for (generic, bounds) in generics.iter() {
        let mut bounds = bounds.join("+");
        if !bounds.is_empty() {
            bounds += " + ";
        }
        bounds += &format!("nanoserde::{}", bound_name);
        generic_w_bounds += &format!("{}: {}, ", generic, bounds);
    }
    generic_w_bounds += ">";

    let mut generic_no_bounds = "<".to_string();
    for (generic, _bounds) in generics.iter() {
        generic_no_bounds += &format!("{}, ", generic);
    }
    generic_no_bounds += ">";
    return (generic_w_bounds, generic_no_bounds);
}
