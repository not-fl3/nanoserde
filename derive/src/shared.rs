macro_rules! l {
    ($target:ident, $line:expr) => {
        $target.push_str($line);
    };

    ($target:ident, $line:expr, $($param:expr),*) => {
        $target.push_str(&format!($line, $($param,)*));
    };
}

pub fn attrs_proxy(attributes: &[crate::parse::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 2 && attr.tokens[0].0 == "proxy" {
            Some(attr.tokens[1].0.clone())
        } else {
            None
        }
    })
}

pub fn attrs_rename(attributes: &[crate::parse::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 2 && attr.tokens[0].0 == "rename" {
            Some(attr.tokens[1].0.clone())
        } else {
            None
        }
    })
}

pub fn attrs_default(attributes: &[crate::parse::Attribute]) -> Option<Option<(String, bool)>> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 1 && attr.tokens[0].0 == "default" {
            Some(None)
        } else if attr.tokens.len() == 2 && attr.tokens[0].0 == "default" {
            Some(Some(attr.tokens[1].clone()))
        } else {
            None
        }
    })
}

pub fn attrs_default_with(attributes: &[crate::parse::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attr| {
        if attr.tokens.len() == 2 && attr.tokens[0].0 == "default_with" {
            Some(attr.tokens[1].0.clone())
        } else {
            None
        }
    })
}

pub fn attrs_transparent(attributes: &[crate::parse::Attribute]) -> bool {
    attributes
        .iter()
        .any(|attr| attr.tokens.len() == 1 && attr.tokens[0].0 == "transparent")
}

pub fn attrs_skip(attributes: &[crate::parse::Attribute]) -> bool {
    attributes
        .iter()
        .any(|attr| attr.tokens.len() == 1 && attr.tokens[0].0 == "skip")
}
