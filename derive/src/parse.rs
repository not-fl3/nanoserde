//! Very limited rust parser
//!
//! https://doc.rust-lang.org/reference/expressions/struct-expr.html
//! https://docs.rs/syn/0.15.44/syn/enum.Type.html
//! https://ziglang.org/documentation/0.5.0/#toc-typeInfo

use proc_macro::{Group, TokenStream, TokenTree};

use std::iter::Peekable;

pub struct Attribute {
    pub attr: String,
}

#[allow(dead_code)]
pub enum Visibility {
    Public,
    Crate,
    Restricted,
    Private,
}

pub struct Field {
    pub attributes: Vec<Attribute>,
    pub vis: Visibility,
    pub field_name: Option<String>,
    pub ty: Type,
}

#[allow(dead_code)]
pub enum Type {
    Tuple { is_option: bool, ty: Vec<Type> },
    Path { is_option: bool, path: String },
}

impl Type {
    pub fn is_option(&self) -> bool {
        match self {
            Type::Tuple { is_option, .. } => *is_option,
            Type::Path { is_option, .. } => *is_option,
        }
    }
}

pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[allow(dead_code)]
pub enum Data {
    Struct(Struct),
    Enum(()),
    Union(()),
}

pub fn parse_data(input: TokenStream) -> Data {
    let mut source = input.into_iter().peekable();

    fn maybe_visibility_modifier<T: Iterator<Item = TokenTree>>(
        source: &mut Peekable<T>,
    ) -> Option<String> {
        if let Some(TokenTree::Ident(ident)) = source.peek() {
            if format!("{}", ident) == "pub" {
                source.next();
                return Some("pub".to_string());
            }
        }

        return None;
    }

    fn maybe_punct<T: Iterator<Item = TokenTree>>(source: &mut Peekable<T>) -> Option<String> {
        if let Some(TokenTree::Punct(punct)) = source.peek() {
            let punct = format!("{}", punct);
            source.next();
            return Some(punct);
        }

        return None;
    }

    fn maybe_doc_comment<T: Iterator<Item = TokenTree>>(
        mut source: &mut Peekable<T>,
    ) -> Option<()> {
        // for some reason structs with doc comment are started with "#" character followed by a group with comments
        let maybe_doc_punct = maybe_punct(&mut source);
        if let Some("#") = maybe_doc_punct.as_deref() {
            let _doc_comment = next_group(&mut source);
            return Some(());
        }

        None
    }
    fn maybe_eof<T: Iterator>(source: &mut Peekable<T>) -> Option<()> {
        if source.peek().is_none() {
            Some(())
        } else {
            None
        }
    }

    fn next_ident(mut source: impl Iterator<Item = TokenTree>) -> Option<String> {
        if let TokenTree::Ident(ident) = source.next().unwrap() {
            Some(format!("{}", ident))
        } else {
            None
        }
    }

    fn next_punct(mut source: impl Iterator<Item = TokenTree>) -> Option<String> {
        if let TokenTree::Punct(punct) = source.next().unwrap() {
            Some(format!("{}", punct))
        } else {
            None
        }
    }

    fn next_type<T: Iterator<Item = TokenTree>>(mut source: &mut Peekable<T>) -> Option<Type> {
        let ty = next_ident(&mut source)?;

        if ty == "Option" {
            let _bracket =
                next_punct(&mut source).expect("Option without generic bound is not supported");
            let ty = next_ident(&mut source).expect("Option without type is not supported");
            let _bracket =
                next_punct(&mut source).expect("Option without generic bound is not supported");
            Some(Type::Path {
                path: ty,
                is_option: true,
            })
        } else {
            Some(Type::Path {
                path: ty,
                is_option: false,
            })
        }
    }

    fn next_group(mut source: impl Iterator<Item = TokenTree>) -> Option<Group> {
        if let TokenTree::Group(ident) = source.next().unwrap() {
            Some(ident)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn debug_current_token<T: Iterator<Item = TokenTree>>(source: &mut Peekable<T>) {
        println!("{:?}", source.peek());
    }

    while let Some(_doc_comment) = maybe_doc_comment(&mut source) {}


    let pub_or_struct = next_ident(&mut source).expect("Not an ident");

    let struct_keyword = if pub_or_struct == "pub" {
        next_ident(&mut source).expect("pub(whatever) is not supported yet")
    } else {
        pub_or_struct
    };

    assert_eq!(struct_keyword, "struct");

    let struct_name = next_ident(&mut source).expect("Unnamed structs are not supported");

    let group = next_group(&mut source).expect("Struct body expected");
    let mut body = group.stream().into_iter().peekable();

    let mut fields = vec![];

    loop {
        if maybe_eof(&mut body).is_some() {
            break;
        }

        while let Some(_doc_comment) = maybe_doc_comment(&mut body) {}

        let _visibility = maybe_visibility_modifier(&mut body);
        let field_name = next_ident(&mut body).expect("Field name expected");

        let punct = next_punct(&mut body).expect("Delimeter after field name expected");
        assert_eq!(punct, ":");
        let ty = next_type(&mut body).expect("Expected field type");

        let _punct = maybe_punct(&mut body);

        let _doc_comment = maybe_doc_comment(&mut source);

        fields.push(Field {
            attributes: vec![],
            vis: Visibility::Public,
            field_name: Some(field_name),
            ty,
        });
    }

    assert!(
        source.next().is_none(),
        "Unexpected data after end of the struct"
    );

    Data::Struct(Struct {
        name: struct_name,
        fields,
    })
}
