//! Very limited rust parser, taken from nanoserde
//!
//! https://doc.rust-lang.org/reference/expressions/struct-expr.html
//! https://docs.rs/syn/0.15.44/syn/enum.Type.html
//! https://ziglang.org/documentation/0.5.0/#toc-typeInfo

use core::iter::Peekable;
use std::collections::HashSet;
use std::num::IntErrorKind;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub tokens: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Visibility {
    Public,
    Crate,
    Restricted,
    Private,
}

#[derive(Debug, Clone)]
pub struct Lifetime {
    pub(crate) ident: String,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub attributes: Vec<Attribute>,
    pub vis: Visibility,
    pub field_name: Option<String>,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum ConstValType {
    Value(isize),
    Named(Box<Type>),
}

#[derive(Debug, Clone)]
pub enum FnType {
    Bare,
    Closure { reusable: bool, fn_mut: bool },
}

#[derive(Debug, Clone)]
pub enum Category {
    Never,
    None,
    Array {
        content_type: Box<Type>,
        len: Option<ConstValType>,
    },
    Tuple {
        contents: Vec<Type>,
    },
    Named {
        path: String,
    },
    Lifetime {
        path: String,
    },
    Fn {
        category: FnType,
        args: Option<Box<Type>>,
        return_type: Option<Box<Type>>,
    },
    Object {
        is_dyn: bool,
        trait_names: Vec<Box<Type>>,
    },
    Associated {
        base: Box<Type>,
        as_trait: Box<Type>,
        associated: Box<Type>,
    },
    AssociatedBound {
        associated: String,
        is: Box<Type>,
    },
    AnonymousStruct {
        contents: Struct,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Type {
    pub ident: Category,
    pub wraps: Option<Vec<Type>>,
    pub ref_type: Option<Option<Lifetime>>,
    pub as_other: Option<Box<Type>>,
}

#[derive(Debug, Clone)]
pub enum Generic {
    ConstGeneric {
        name: String,
        _type: Type,
        default: Option<ConstValType>,
    },
    Generic {
        name: String,
        default: Option<Type>,
        bounds: Vec<Type>,
    },
    Lifetime {
        name: String,
        bounds: Vec<Lifetime>,
    },
    WhereBounded {
        name: String,
        bounds: Vec<Type>,
    },
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: Option<String>,
    pub named: bool,
    pub fields: Vec<Field>,
    pub attributes: Vec<Attribute>,
    pub generics: Vec<Generic>,
}

#[derive(Debug)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<Field>,
    pub attributes: Vec<Attribute>,
    pub generics: Vec<Generic>,
}

#[allow(dead_code)]
pub enum Data {
    Struct(Struct),
    Enum(Enum),
    Union(()),
}

#[allow(dead_code)]
impl Data {
    pub fn name(&self) -> &str {
        match self {
            Data::Struct(Struct { name, .. }) => match name {
                Some(name) => name.as_str(),
                None => "",
            },
            Data::Enum(Enum { name, .. }) => name.as_str(),
            _ => unimplemented!(),
        }
    }

    pub fn attributes(&self) -> &[Attribute] {
        match self {
            Data::Struct(Struct { attributes, .. }) => &attributes[..],
            Data::Enum(Enum { attributes, .. }) => &attributes[..],
            _ => unimplemented!(),
        }
    }
}

impl Generic {
    pub fn full(&self) -> String {
        match &self {
            Generic::ConstGeneric { name, .. } => name.clone(),
            Generic::Generic { name, .. } => name.clone(),
            Generic::Lifetime { name, .. } => name.clone(),
            Generic::WhereBounded { name, .. } => name.clone(),
        }
    }

    fn lifetime_prefix(&self) -> &str {
        match &self {
            Generic::Lifetime { .. } => "\'",
            _ => "",
        }
    }

    fn const_prefix(&self) -> &str {
        match &self {
            Generic::ConstGeneric { .. } => "const ",
            _ => "",
        }
    }

    pub fn ident_only(&self) -> String {
        format!("{}{}", self.lifetime_prefix(), self.full())
    }

    pub fn full_with_const(&self, extra_bounds: &[&str], bounds: bool) -> String {
        let bounds = match (bounds, &self) {
            (true, Generic::Lifetime { .. }) => self.get_bounds().join(" + "),
            (true, _) => {
                let mut bounds = self.get_bounds().join(" + ");
                if !extra_bounds.is_empty() {
                    if bounds.is_empty() {
                        bounds = extra_bounds.join(" + ")
                    } else {
                        bounds = format!("{} + {}", bounds, extra_bounds.join(" + "));
                    }
                }
                bounds
            }
            (_, Generic::ConstGeneric { .. }) => self.get_bounds().join(" + "),
            (false, _) => String::new(),
        };
        match bounds.is_empty() {
            true => format!(
                "{}{}{}:",
                self.const_prefix(),
                self.lifetime_prefix(),
                self.full()
            ),
            false => format!(
                "{}{}{}: {}",
                self.const_prefix(),
                self.lifetime_prefix(),
                self.full(),
                bounds
            ),
        }
    }

    #[allow(unused)]
    pub fn full_with_const_and_default(&self, extra_bounds: &[&str], bounds: bool) -> String {
        let bounds = match (bounds, &self) {
            (true, Generic::Lifetime { .. }) => String::new(),
            (true, _) | (false, Generic::ConstGeneric { .. }) => {
                let mut bounds = self.get_bounds().join(" + ");
                if !extra_bounds.is_empty() {
                    if bounds.is_empty() {
                        bounds = extra_bounds.join(" + ")
                    } else {
                        bounds = vec![bounds, extra_bounds.join(" + ")].join(" + ")
                    }
                }
                bounds
            }
            (false, _) => String::new(),
        };
        match bounds.is_empty() {
            true => format!(
                "{}{}{}{}",
                self.const_prefix(),
                self.lifetime_prefix(),
                self.full(),
                self.get_default()
            ),
            false => format!(
                "{}{}{}: {}{}",
                self.const_prefix(),
                self.lifetime_prefix(),
                self.full(),
                bounds,
                self.get_default()
            ),
        }
    }

    fn get_bounds(&self) -> Vec<String> {
        match &self {
            Generic::ConstGeneric { _type, .. } => vec![format!("{}", _type.full())],
            Generic::Generic { bounds, .. } => bounds.iter().map(Type::full).collect(),
            Generic::Lifetime { bounds, .. } => {
                bounds.iter().map(|x| format!("'{}", x.ident)).collect()
            }
            Generic::WhereBounded { bounds, .. } => bounds.iter().map(Type::full).collect(),
        }
    }

    fn get_default(&self) -> String {
        match &self {
            Generic::ConstGeneric {
                default: Some(def), ..
            } => match def {
                ConstValType::Value(v) => format!("= {}", v),
                ConstValType::Named(v) => format!("= {}", v.full()),
            },
            Generic::Generic {
                default: Some(def), ..
            } => format!("= {}", def.full()),
            _ => String::new(),
        }
    }
}

impl Category {
    pub fn path(&self, parent: &Type, no_ref: bool) -> String {
        #[allow(unused)]
        let mut holder: Option<Type> = None;
        let parent = match no_ref {
            true => {
                holder = Some(parent.clone().set_ref_type(None));
                holder.as_ref().unwrap()
            }
            false => parent,
        };
        match self {
            Category::Array { content_type, len } => match len {
                Some(ConstValType::Value(val)) => format!("[{};{}]", content_type.full(), val),
                Some(ConstValType::Named(const_gen)) => match &const_gen.as_other {
                    Some(as_type) => format!(
                        "[{};{} as {}]",
                        content_type.full(),
                        const_gen.full(),
                        as_type.full()
                    ),
                    None => format!("[{};{}]", content_type.full(), const_gen.full()),
                },
                None => format!("[{}]", content_type.full()),
            },
            Category::Tuple { contents } => format!(
                "({})",
                contents
                    .iter()
                    .map(Type::full)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Category::Named { path } => path.clone(),
            Category::Object {
                is_dyn,
                trait_names,
            } => match is_dyn {
                true => format!(
                    "dyn {}",
                    trait_names
                        .iter()
                        .map(|x| x.full())
                        .collect::<Vec<_>>()
                        .join(" + ")
                ),
                false => format!(
                    "impl {}",
                    trait_names
                        .iter()
                        .map(|x| x.full())
                        .collect::<Vec<_>>()
                        .join(" + ")
                ),
            },
            Category::Associated {
                base,
                as_trait,
                associated,
            } => format!(
                "<{} as {}>::{}",
                base.full(),
                as_trait.full(),
                associated.full()
            ),
            Category::AssociatedBound { associated, is } => format!(
                "{}<{}= {}>",
                associated,
                parent.wraps.as_ref().unwrap()[0].full(),
                is.full()
            ),
            Category::Lifetime { path } => format!("\'{}", path),
            Category::Fn {
                category,
                args,
                return_type,
            } => {
                let arg_str = args.as_ref().map(|x| x.full()).unwrap_or_default();
                let return_str = return_type
                    .as_ref()
                    .map(|x| format!(" -> {}", x.full()))
                    .unwrap_or_default();
                match category {
                    FnType::Bare => format!("fn({}){}", arg_str, return_str),
                    FnType::Closure { reusable, fn_mut } => match fn_mut {
                        true => format!("FnMut({}){}", arg_str, return_str),
                        false => match reusable {
                            true => format!("Fn({}){}", arg_str, return_str),
                            false => format!("FnOnce({}){}", arg_str, return_str),
                        },
                    },
                }
            }
            Category::Never => String::from("!"),
            Category::None => String::new(),
            Category::AnonymousStruct {
                contents: Struct { name, fields, .. },
            } => {
                let mut l = name.as_ref().map_or(String::new(), |x| x.clone());
                l!(l, "{\n");
                for field in fields.iter() {
                    l!(
                        l,
                        "\t{}: {}\n",
                        field.field_name.as_ref().expect("field must have name"),
                        field.ty.full()
                    );
                }
                l!(l, "}\n");
                l
            }
        }
    }
}

impl Type {
    pub fn base(&self) -> String {
        let mut base = match &self.ref_type {
            Some(inner) => match inner {
                Some(ident) => format!("&\'{} ", ident.ident),
                None => String::from("& "),
            },
            None => String::default(),
        };
        base.push_str(&self.ident.path(&self, false));
        base
    }

    pub fn full(&self) -> String {
        let mut base = match &self.ref_type {
            Some(inner) => match inner {
                Some(ident) => format!("&\'{} ", ident.ident),
                None => String::from("& "),
            },
            None => String::default(),
        };
        base.push_str(&self.ident.path(&self, false));
        if let (Some(wrapped), Category::Named { .. }) = (&self.wraps, &self.ident) {
            base.push('<');
            base.push_str(
                wrapped
                    .into_iter()
                    .map(|x| x.full())
                    .collect::<Vec<_>>()
                    .join(",")
                    .as_str(),
            );
            base.push('>');
        }
        base
    }

    pub fn set_ref_type(mut self, ref_type: Option<Option<Lifetime>>) -> Self {
        self.ref_type = ref_type;
        self
    }
}

pub fn next_visibility_modifier(
    source: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Option<String> {
    if let Some(TokenTree::Ident(ident)) = source.peek() {
        if format!("{}", ident) == "pub" {
            source.next();

            // skip (crate) and alike
            if let Some(TokenTree::Group(group)) = source.peek() {
                if group.delimiter() == Delimiter::Parenthesis {
                    next_group(source);
                }
            }

            return Some("pub".to_string());
        }
    }

    return None;
}

pub fn next_punct(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<String> {
    if let Some(TokenTree::Punct(punct)) = source.peek() {
        let punct = format!("{}", punct);
        source.next();
        return Some(punct);
    }

    return None;
}

pub fn next_exact_punct(
    source: &mut Peekable<impl Iterator<Item = TokenTree>>,
    pattern: &str,
) -> Option<String> {
    if let Some(TokenTree::Punct(punct)) = source.peek() {
        let punct = format!("{}", punct);
        if punct == pattern {
            source.next();
            return Some(punct);
        }
    }

    return None;
}

pub fn next_literal(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<String> {
    if let Some(TokenTree::Literal(lit)) = source.peek() {
        let mut literal = lit.to_string();

        // the only way to check that literal is string :/
        if literal.starts_with("\"") {
            literal.remove(0);
            literal.remove(literal.len() - 1);
        }
        source.next();
        return Some(literal);
    }

    return None;
}

pub fn next_eof<T: Iterator>(source: &mut Peekable<T>) -> Option<()> {
    if source.peek().is_none() {
        Some(())
    } else {
        None
    }
}

pub fn next_ident(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<String> {
    if let Some(TokenTree::Ident(ident)) = source.peek() {
        let ident = format!("{}", ident);
        source.next();
        Some(ident)
    } else {
        None
    }
}

pub fn next_group(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<Group> {
    if let Some(TokenTree::Group(_)) = source.peek() {
        let group = match source.next().unwrap() {
            TokenTree::Group(group) => group,
            _ => unreachable!("just checked with peek()!"),
        };
        Some(group)
    } else {
        None
    }
}

pub fn _debug_current_token(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> String {
    format!("{:?}", source.peek())
}

pub fn next_lifetime<T: Iterator<Item = TokenTree>>(source: &mut Peekable<T>) -> Option<Lifetime> {
    let Some(TokenTree::Punct(punct)) = source.peek() else {
        return None;
    };
    let '\'' = punct.as_char() else {
        return None;
    };

    let _ = source.next();
    Some(Lifetime {
        ident: next_ident(source).expect("must be an identifier after a single quote"),
    })
}

fn next_type<T: Iterator<Item = TokenTree> + Clone>(mut source: &mut Peekable<T>) -> Option<Type> {
    fn as_associated_definition<T: Iterator<Item = TokenTree> + Clone>(
        source: &mut Peekable<T>,
    ) -> Option<Type> {
        if let Some(TokenTree::Punct(punct)) = source.peek() {
            if punct.as_char() == '=' {
                source.next();
                let ty = next_type(source).expect("Missing type after \"as\"");
                return Some(ty);
            }
        }
        None
    }
    fn as_other_type<T: Iterator<Item = TokenTree> + Clone>(
        source: &mut Peekable<T>,
    ) -> Option<Type> {
        if let Some(TokenTree::Ident(ident)) = source.peek() {
            if ident.to_string() == "as" {
                source.next();
                let ty = next_type(source).expect("Missing type after \"as\"");
                return Some(ty);
            }
        }
        None
    }
    pub fn next_array<T: Iterator<Item = TokenTree> + Clone>(
        mut source: &mut Peekable<T>,
    ) -> Option<Type> {
        let next = next_type(&mut source).expect("Must be type after array declaration");

        let Some(_) = next_exact_punct(&mut source, ";") else {
            // This is an unbounded array, legal at end for unsized types
            return Some(Type {
                ident: Category::Array {
                    content_type: Box::new(next.clone()),
                    len: None,
                },
                wraps: Some(vec![next]),
                ref_type: None,
                as_other: None,
            });
        };

        //need to cover both the const generic and literal case
        let len = source.peek().unwrap().to_string();
        match len.parse::<usize>() {
            Ok(val) => Some(Type {
                ident: Category::Array {
                    content_type: Box::new(next.clone()),
                    len: Some(ConstValType::Value(val as isize)),
                },
                wraps: Some(vec![next]),
                ref_type: None,
                as_other: None,
            }),
            Err(err) if err.kind() == &IntErrorKind::Zero => Some(Type {
                ident: Category::Array {
                    content_type: Box::new(next.clone()),
                    len: Some(ConstValType::Value(0)),
                },
                wraps: Some(vec![next]),
                ref_type: None,
                as_other: None,
            }),
            _ => Some(Type {
                ident: Category::Array {
                    content_type: Box::new(next.clone()),
                    len: Some(ConstValType::Named(Box::new(next_type(source).unwrap()))),
                },
                wraps: Some(vec![next]),
                ref_type: None,
                as_other: None,
            }),
        }
    }

    pub fn next_tuple<T: Iterator<Item = TokenTree> + Clone>(
        source: &mut Peekable<T>,
    ) -> Option<Type> {
        let mut wraps = vec![];
        let mut path = "(".to_owned();
        while let Some(next_ty) = next_type(source) {
            wraps.push(next_ty.clone());
            path.push_str(&format!("{}", next_ty.full()));
            if next_exact_punct(source, ",").is_none() {
                break;
            }
            path.push(',')
        }
        path.push(')');

        let tuple_type = Type {
            ident: Category::Tuple {
                contents: wraps.clone(),
            },
            wraps: Some(wraps),
            ref_type: None,
            as_other: None,
        };

        return Some(tuple_type);
    }

    pub fn next_function_like<T: Iterator<Item = TokenTree> + Clone>(
        source: &mut Peekable<T>,
    ) -> Option<Type> {
        pub fn next_return_type<T: Iterator<Item = TokenTree> + Clone>(
            source: &mut Peekable<T>,
        ) -> Option<Type> {
            let mut tmp = source.clone();
            let (Some(_), Some(_)) = (
                next_exact_punct(&mut tmp, "-"),
                next_exact_punct(&mut tmp, ">"),
            ) else {
                return None;
            };
            drop(tmp);
            let _ = (source.next(), source.next());
            Some(next_type(source).expect("Missing return type"))
        }

        pub fn next_closure<T: Iterator<Item = TokenTree> + Clone>(
            source: &mut Peekable<T>,
            reusable: bool,
            fn_mut: bool,
        ) -> Type {
            let args = next_group(source)
                .map(|group| {
                    next_type(&mut group.stream().into_iter().peekable()).expect("Missing args")
                })
                .map(Box::new);

            let ret = next_return_type(source).map(Box::new);

            let wraps = if args.as_ref().map(|x| x.wraps.as_ref()).is_some()
                || ret.as_ref().map(|x| x.wraps.as_ref()).is_some()
            {
                let mut base = ret
                    .iter()
                    .filter_map(|x| x.wraps.as_ref())
                    .cloned()
                    .flatten()
                    .collect::<Vec<Type>>();
                base.extend(
                    args.iter()
                        .filter_map(|x| x.wraps.as_ref())
                        .cloned()
                        .flatten(),
                );
                Some(base)
            } else {
                None
            };
            Type {
                ident: Category::Fn {
                    category: FnType::Closure { reusable, fn_mut },
                    args,
                    return_type: ret,
                },
                wraps,
                ref_type: None,
                as_other: None,
            }
        }

        let Some(TokenTree::Ident(ident)) = source.peek().clone() else {
            return None;
        };
        let true = matches!(ident.to_string().as_str(), "fn" | "FnOnce" | "FnMut" | "Fn") else {
            return None;
        };
        let tok_str = source.next().unwrap().to_string();

        match tok_str.as_str() {
            "fn" => {
                let args = next_type(
                    &mut next_group(source)
                        .expect("Missing args group")
                        .stream()
                        .into_iter()
                        .peekable(),
                )
                .map(Box::new)
                .expect("Missing args");
                let ret = next_return_type(source).map(Box::new);

                let wraps =
                    if args.wraps.is_some() || ret.as_ref().map(|x| x.wraps.as_ref()).is_some() {
                        let mut base = ret
                            .iter()
                            .filter_map(|x| x.wraps.as_ref())
                            .cloned()
                            .flatten()
                            .collect::<Vec<Type>>();
                        base.extend_from_slice(args.wraps.clone().unwrap_or_default().as_ref());
                        Some(base)
                    } else {
                        None
                    };
                Some(Type {
                    ident: Category::Fn {
                        category: FnType::Bare,
                        args: Some(args),
                        return_type: ret,
                    },
                    wraps,
                    ref_type: None,
                    as_other: None,
                })
            }
            "Fn" => Some(next_closure(source, true, false)),
            "FnMut" => Some(next_closure(source, true, true)),
            "FnOnce" => Some(next_closure(source, false, false)),
            _ => None,
        }
    }

    pub fn next_object<T: Iterator<Item = TokenTree> + Clone>(
        source: &mut Peekable<T>,
    ) -> Option<Type> {
        let Some(TokenTree::Ident(ident)) = source.peek() else {
            return None;
        };
        let true = matches!(ident.to_string().as_str(), "impl" | "dyn") else {
            return None;
        };
        match source.next().unwrap().to_string().as_str() {
            "impl" => {
                let mut ident_types = vec![Box::new(
                    next_type(source).expect("impl must be followed by trait"),
                )];
                while let Some(_) = next_exact_punct(source, "+") {
                    ident_types.push(Box::new(
                        next_type(source).expect("impl must be followed by trait"),
                    ))
                }
                let ref_type = ident_types[0].ref_type.clone();
                let as_other = ident_types[0].as_other.clone();
                let wraps = ident_types[0].wraps.clone();
                Some(Type {
                    ident: Category::Object {
                        is_dyn: false,
                        trait_names: ident_types,
                    },
                    wraps,
                    ref_type,
                    as_other,
                })
            }
            "dyn" => {
                let mut ident_types = vec![Box::new(
                    next_type(source).expect("impl must be followed by trait"),
                )];
                while let Some(_) = next_exact_punct(source, "+") {
                    ident_types.push(Box::new(
                        next_type(source).expect("impl must be followed by trait"),
                    ))
                }
                let ref_type = ident_types[0].ref_type.clone();
                let as_other = ident_types[0].as_other.clone();
                let wraps = ident_types[0].wraps.clone();
                Some(Type {
                    ident: Category::Object {
                        is_dyn: true,
                        trait_names: ident_types,
                    },
                    wraps,
                    ref_type,
                    as_other,
                })
            }
            _ => None,
        }
    }

    //
    //

    if let Some(_) = next_exact_punct(&mut source, ",") {
        return None;
    };

    if let Some(_) = next_exact_punct(&mut source, "!") {
        return Some(Type {
            ident: Category::Never,
            wraps: None,
            ref_type: None,
            as_other: None,
        });
    };

    let None = next_exact_punct(source, "\'") else {
        return Some(Type {
            ident: Category::Lifetime {
                path: next_ident(source).expect("Need lifetime name"),
            },
            wraps: None,
            ref_type: None,
            as_other: None,
        });
    };

    let ref_type = match next_exact_punct(&mut source, "&") {
        Some(_) => Some(next_lifetime(source)),
        None => None,
    };

    if let Some(group) = next_group(&mut source.clone()) {
        match group.delimiter() {
            Delimiter::Bracket => {
                let mut group_stream = next_group(&mut source)
                    .unwrap()
                    .stream()
                    .into_iter()
                    .peekable();
                return next_array(&mut group_stream).map(|x| x.set_ref_type(ref_type));
            }
            Delimiter::Parenthesis => {
                let mut group_stream = next_group(&mut source)
                    .unwrap()
                    .stream()
                    .into_iter()
                    .peekable();
                return next_tuple(&mut group_stream).map(|x| x.set_ref_type(ref_type));
            }
            Delimiter::Brace => {
                let anonymous_struct = next_struct(&mut source);
                let wraps = Some(
                    anonymous_struct
                        .fields
                        .iter()
                        .map(|x| x.ty.clone())
                        .collect(),
                );
                return Some(Type {
                    ident: Category::AnonymousStruct {
                        contents: anonymous_struct,
                    },
                    wraps,
                    ref_type,
                    as_other: None,
                });
            }

            _ => {
                let mut group_stream = group.stream().into_iter().peekable();
                _debug_current_token(&mut group_stream);
                unimplemented!(
                    "Unexpected token: {}",
                    _debug_current_token(&mut group_stream)
                )
            }
        }
    }

    if let Some(obj) = next_object(source) {
        return Some(obj.set_ref_type(ref_type));
    }

    if let Some(obj) = next_function_like(source) {
        return Some(obj.set_ref_type(ref_type));
    }

    // read a path like a::b::c::d
    let mut ty = next_ident(&mut source).unwrap_or_default();
    while let Some(TokenTree::Punct(_)) = source.peek() {
        let mut tmp = source.clone();
        let (Some(_), Some(_)) = (
            next_exact_punct(&mut tmp, ":"),
            next_exact_punct(&mut tmp, ":"),
        ) else {
            break;
        };
        drop(tmp);
        let _ = (source.next(), source.next()); //skip the colons

        let next_ident = next_ident(&mut source).expect("Expecting next path part after ::");
        ty.push_str(&format!("::{}", next_ident));
    }

    let angel_bracket = next_exact_punct(&mut source, "<");
    if angel_bracket.is_some() {
        if ty.is_empty() {
            let ty = next_type(source).expect("Need a base type before 'as'");

            assert!(
                matches!(ty.ident, Category::Named { .. }),
                "need a named type here"
            );

            //skip the close bracket and two colons that must follow to get an associated type
            assert_eq!(Some(">".to_owned()), next_exact_punct(source, ">"));
            assert_eq!(
                (Some(":".to_owned()), Some(":".to_owned())),
                (
                    next_exact_punct(&mut source, ":"),
                    next_exact_punct(&mut source, ":")
                )
            );
            let associated =
                next_type(source).expect("Must be an associated type name after the trait");

            let as_trait = ty
                .as_other
                .clone()
                .expect("Must be an as_other for an associated type");

            return Some(Type {
                ident: Category::Associated {
                    base: Box::new(ty.clone()),
                    as_trait,
                    associated: Box::new(associated),
                },
                wraps: ty.wraps,
                ref_type: ref_type,
                as_other: None,
            });
        }

        let mut generics =
            vec![next_type(source).expect("Expecting at least one generic argument")];
        while let Some(_comma) = next_exact_punct(&mut source, ",") {
            generics.push(next_type(source).expect("Expecting generic argument after comma"));
        }

        let as_other = as_other_type(source).map(Box::new);

        if let Some(assoc_def) = as_associated_definition(source) {
            let _closing_bracket =
                next_exact_punct(&mut source, ">").expect("Expecting closing generic bracket");
            return Some(Type {
                ident: Category::AssociatedBound {
                    associated: ty,
                    is: Box::new(assoc_def),
                },
                wraps: Some(generics),
                ref_type,
                as_other,
            });
        }

        let _closing_bracket =
            next_exact_punct(&mut source, ">").expect("Expecting closing generic bracket");

        Some(Type {
            ident: Category::Named { path: ty },
            wraps: Some(generics),
            ref_type,
            as_other,
        })
    } else {
        let as_other = as_other_type(source).map(Box::new);
        if ty.is_empty() {
            Some(Type {
                ident: Category::None,
                wraps: None,
                ref_type,
                as_other,
            })
        } else {
            Some(Type {
                ident: Category::Named { path: ty },
                wraps: None,
                ref_type,
                as_other,
            })
        }
    }
}

fn next_attribute<T: Iterator<Item = TokenTree>>(
    mut source: &mut Peekable<T>,
) -> Option<Option<Vec<Attribute>>> {
    // all attributes, even doc-comments, starts with "#"
    let next_attr_punct = next_punct(&mut source);
    let Some("#") = next_attr_punct.as_deref() else {
        return None;
    };

    let mut attr_group = next_group(&mut source)
        .expect("Expecting attribute body")
        .stream()
        .into_iter()
        .peekable();

    let name = next_ident(&mut attr_group).expect("Attributes should start with a name");

    if name != "nserde" {
        return Some(None);
    }

    let mut args_group = next_group(&mut attr_group)
        .expect("Expecting attribute body")
        .stream()
        .into_iter()
        .peekable();

    let mut attrs = vec![];
    let mut attr_tokens = vec![];

    loop {
        let attribute_name = next_ident(&mut args_group).expect("Expecting attribute name");
        attr_tokens.push(attribute_name);

        // single-word attribute, like #[structdiff(whatever)]
        match (
            next_eof(&mut args_group).is_some(),
            next_punct(&mut args_group).as_deref(),
        ) {
            (true, _) => {
                attrs.push(Attribute {
                    name: name.clone(),
                    tokens: std::mem::take(&mut attr_tokens),
                });
                break;
            }
            (false, Some(",")) => {
                attrs.push(Attribute {
                    name: name.clone(),
                    tokens: std::mem::take(&mut attr_tokens),
                });
                continue;
            }
            (false, Some("=")) => (), // continue and get next literal
            _ => (),
        }

        let value = next_literal(&mut args_group).expect("Expecting argument value");

        attr_tokens.push(value.clone());

        match (
            next_eof(&mut args_group).is_some(),
            next_punct(&mut args_group).as_deref() == Some(","),
        ) {
            (true, _) => {
                attrs.push(Attribute {
                    name: name.clone(),
                    tokens: std::mem::take(&mut attr_tokens),
                });
                break;
            }
            (false, true) => {
                attrs.push(Attribute {
                    name: name.clone(),
                    tokens: std::mem::take(&mut attr_tokens),
                });
            }
            _ => {}
        }
    }

    return Some(Some(attrs));
}

fn next_attributes_list(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Vec<Attribute> {
    let mut attributes = vec![];

    while let Some(attr) = next_attribute(source) {
        if let Some(structdiff_attr) = attr {
            attributes.extend(structdiff_attr.into_iter());
        }
    }

    attributes
}

fn next_fields<T: Iterator<Item = TokenTree> + Clone>(
    mut body: &mut Peekable<T>,
    named: bool,
) -> Vec<Field> {
    let mut fields = vec![];

    loop {
        if next_eof(&mut body).is_some() {
            break;
        }

        let attributes = next_attributes_list(&mut body);
        let _visibility = next_visibility_modifier(&mut body);

        let field_name = if named {
            let field_name = next_ident(&mut body).expect("Field name expected");

            let _ = next_exact_punct(&mut body, ":").expect("Delimeter after field name expected");
            Some(field_name)
        } else {
            None
        };

        let ty = next_type(&mut body).expect("Expected field type");
        let _punct = next_punct(&mut body);

        fields.push(Field {
            attributes,
            vis: Visibility::Public,
            field_name,
            ty,
        });
    }
    fields
}

fn next_struct<T: Iterator<Item = TokenTree> + Clone>(mut source: &mut Peekable<T>) -> Struct {
    let struct_name = next_ident(&mut source);
    let generics = get_all_bounds(source);
    let group = next_group(&mut source);
    // unit struct
    if group.is_none() {
        // skip ; at the end of struct like this: "struct Foo;"
        let _ = next_punct(&mut source);

        return Struct {
            name: struct_name,
            fields: vec![],
            attributes: vec![],
            named: false,
            generics,
        };
    };

    let group = group.unwrap();
    let delimiter = group.delimiter();
    let named = match delimiter {
        Delimiter::Parenthesis => false,
        Delimiter::Brace => true,

        _ => panic!("Struct with unsupported delimiter"),
    };

    let mut body = group.stream().into_iter().peekable();
    let fields = next_fields(&mut body, named);

    if named == false {
        next_exact_punct(&mut source, ";").expect("Expected ; on the end of tuple struct");
    }

    Struct {
        name: struct_name,
        named,
        fields,
        attributes: vec![],
        generics,
    }
}

fn next_enum<T: Iterator<Item = TokenTree> + Clone>(mut source: &mut Peekable<T>) -> Enum {
    let enum_name = next_ident(&mut source).expect("Unnamed enums are not supported");
    let generic_types = get_all_bounds(source);
    let group = next_group(&mut source);
    // unit enum
    if group.is_none() {
        return Enum {
            name: enum_name,
            variants: vec![],
            attributes: vec![],
            generics: vec![],
        };
    };
    let group = group.unwrap();
    let mut body = group.stream().into_iter().peekable();

    let mut variants = vec![];
    loop {
        if next_eof(&mut body).is_some() {
            break;
        }

        let attributes = next_attributes_list(&mut body);

        let variant_name = next_ident(&mut body).expect("Unnamed variants are not supported");
        let ty = next_type(&mut body);
        let Some(ty) = ty else {
            variants.push(Field {
                ty: Type {
                    ident: Category::None,
                    wraps: None,
                    ref_type: None,
                    as_other: None,
                },
                attributes,
                vis: Visibility::Public,
                field_name: Some(variant_name),
            });
            let _maybe_comma = next_exact_punct(&mut body, ",");
            continue;
        };

        {
            variants.push(Field {
                field_name: Some(variant_name),
                ty: ty,
                attributes,
                vis: Visibility::Public,
            });
        }
        let _maybe_semicolon = next_exact_punct(&mut body, ";");
        let _maybe_coma = next_exact_punct(&mut body, ",");
    }

    Enum {
        name: enum_name,
        variants,
        attributes: vec![],
        generics: generic_types,
    }
}

fn next_const_generic<T: Iterator<Item = TokenTree> + Clone>(
    source: &mut Peekable<T>,
) -> (String, Type, Option<ConstValType>) {
    let name = source
        .next()
        .expect("Missing generic parameter after 'const'")
        .to_string();
    assert_eq!(
        source.next().unwrap().to_string(),
        ":",
        "Colon should follow const generic typename"
    );
    let cg_type = next_type(source).expect("Missing const generic type after 'colon'");
    if let Some(_) = next_exact_punct(source, "=") {
        if let Ok(default_value) = source
            .peek()
            .expect("default should follow equal for const generic")
            .to_string()
            .parse::<isize>()
        {
            source.next();
            (name, cg_type, Some(ConstValType::Value(default_value)))
        } else {
            let def =
                next_type(source).expect("must have either a value or other const as default");
            (name, cg_type, Some(ConstValType::Named(Box::new(def))))
        }
    } else {
        (name, cg_type, None)
    }
}

fn next_generic<T: Iterator<Item = TokenTree> + Clone>(
    source: &mut Peekable<T>,
) -> Option<Generic> {
    let Some(tok) = source.peek() else {
        return None;
    };
    match tok {
        TokenTree::Group(g) => {
            if matches!(g.delimiter(), Delimiter::Brace) {
                return None;
            }
            let mut bounds = vec![];
            let _type = next_type(source).expect("must be a type in group");
            if let Some(_) = next_exact_punct(source, ":") {
                while let Some(bound) = next_type(source) {
                    bounds.push(bound);
                    if next_exact_punct(source, "+").is_none() {
                        break;
                    }
                }
            }

            Some(Generic::WhereBounded {
                name: _type.full(),
                bounds,
            })
        }
        TokenTree::Ident(c) if c.to_string() == "const" => {
            source.next();
            let (name, _type, default) = next_const_generic(source);
            Some(Generic::ConstGeneric {
                name,
                _type,
                default,
            })
        }
        TokenTree::Ident(_) => {
            let mut default = None;
            let ty = next_type(source).expect("Expected type name after \'const\' keyword");

            let mut bounds = vec![];

            if let Some(_) = next_exact_punct(source, ":") {
                loop {
                    if let Some(ty) = next_type(source) {
                        bounds.push(ty);
                    }
                    if next_exact_punct(source, "+").is_none() {
                        break;
                    }
                }
            }

            if let Some(_) = next_exact_punct(source, "=") {
                default = Some(next_type(source).expect("Must be a default after eq sign"));
            }
            Some(Generic::Generic {
                name: ty.full(),
                default,
                bounds,
            })
        }
        TokenTree::Punct(punct) => match punct.as_char() {
            '>' => None,
            '\'' => {
                let ty = next_lifetime(source).expect("must be lifetime after \' mark");
                let mut bounds = vec![];
                if let Some(_) = next_exact_punct(source, ":") {
                    while let Some(bound) = next_lifetime(source) {
                        bounds.push(bound);
                        if next_exact_punct(source, "+").is_none() {
                            break;
                        }
                    }
                }
                Some(Generic::Lifetime {
                    name: ty.ident,
                    bounds,
                })
            }
            _ => unimplemented!("unexpected character: {}", _debug_current_token(source)),
        },
        TokenTree::Literal(_) => unimplemented!("should not be literals here"),
    }
}

fn get_all_bounds<T: Iterator<Item = TokenTree> + Clone>(source: &mut Peekable<T>) -> Vec<Generic> {
    let mut ret = Vec::new();
    let mut already = HashSet::new();
    if source.peek().map_or(false, |x| x.to_string() == "<") {
        source.next();
    } else {
        return ret;
    }

    // Angle bracket generics + bounds
    while let Some(gen) = next_generic(source) {
        if already.insert(gen.full()) {
            ret.push(gen);
        } else {
            match (
                ret.iter_mut().find(|x| x.full() == gen.full()).unwrap(),
                gen,
            ) {
                (
                    Generic::Generic { bounds, .. },
                    Generic::Generic {
                        bounds: other_bounds,
                        ..
                    },
                ) => bounds.extend_from_slice(&other_bounds),
                (
                    Generic::Lifetime { bounds, .. },
                    Generic::Lifetime {
                        bounds: other_bounds,
                        ..
                    },
                ) => bounds.extend_from_slice(&other_bounds),
                (
                    Generic::WhereBounded { bounds, .. },
                    Generic::WhereBounded {
                        bounds: other_bounds,
                        ..
                    },
                ) => bounds.extend_from_slice(&other_bounds),
                _ => {
                    panic!("mismatched generic types")
                }
            }
        }
        let Some(_) = next_exact_punct(source, ",") else {
            break;
        };
    }

    let _ = next_exact_punct(source, ">").expect("Need closing generic bracket");

    // "where" generics + bounds
    if let Some(content) = source.peek() {
        if content.to_string() != "where" {
            return ret;
        } else {
            source.next();
        }

        while let Some(gen) = next_generic(source) {
            if already.insert(gen.full()) {
                let gen = match gen {
                    Generic::Generic { name, bounds, .. } => Generic::WhereBounded { name, bounds },
                    where_bounded @ Generic::WhereBounded { .. } => where_bounded,
                    unused => {
                        unimplemented!(
                            "Shouldn't have unused lifetime or const generic in where bound: {}",
                            unused.full()
                        )
                    }
                };
                ret.push(gen);
            } else {
                match (
                    ret.iter_mut().find(|x| x.full() == gen.full()).unwrap(),
                    gen,
                ) {
                    (
                        Generic::Generic { bounds, .. },
                        Generic::Generic {
                            bounds: other_bounds,
                            ..
                        },
                    ) => bounds.extend_from_slice(&other_bounds),
                    (
                        Generic::Lifetime { bounds, .. },
                        Generic::Lifetime {
                            bounds: other_bounds,
                            ..
                        },
                    ) => bounds.extend_from_slice(&other_bounds),
                    (
                        Generic::WhereBounded { bounds, .. },
                        Generic::WhereBounded {
                            bounds: other_bounds,
                            ..
                        },
                    ) => bounds.extend_from_slice(&other_bounds),
                    _ => {
                        panic!("mismatched generic types")
                    }
                }
            }
            let Some(_) = next_exact_punct(source, ",") else {
                break;
            };
            let None = next_exact_punct(source, "{") else {
                break;
            };
        }
    }

    ret
}

pub fn parse_data(input: TokenStream) -> Data {
    let mut source = input.into_iter().peekable();

    let attributes = next_attributes_list(&mut source);

    let pub_or_type = next_ident(&mut source).expect("Not an ident");

    let type_keyword = if pub_or_type == "pub" {
        next_ident(&mut source).expect("pub(whatever) is not supported yet")
    } else {
        pub_or_type
    };

    let res;

    match type_keyword.as_str() {
        "struct" => {
            let mut struct_ = next_struct(&mut source);
            struct_.attributes = attributes;
            res = Data::Struct(struct_);
        }
        "enum" => {
            let enum_ = next_enum(&mut source);
            res = Data::Enum(enum_);
        }
        "union" => unimplemented!("Unions are not supported"),
        unexpected => panic!("Unexpected keyword: {}", unexpected),
    }

    assert!(
        source.next().is_none(),
        "Unexpected data after end of the struct: {}",
        _debug_current_token(&mut source)
    );
    res
}
