//! Very limited rust parser
//!
//! https://doc.rust-lang.org/reference/expressions/struct-expr.html
//! https://docs.rs/syn/0.15.44/syn/enum.Type.html
//! https://ziglang.org/documentation/0.5.0/#toc-typeInfo

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

use std::iter::Peekable;

#[derive(Debug)]
pub struct Attribute {
    pub name: String,
    pub tokens: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Visibility {
    Public,
    Crate,
    Restricted,
    Private,
}

#[derive(Debug)]
pub struct Field {
    pub attributes: Vec<Attribute>,
    pub vis: Visibility,
    pub field_name: Option<String>,
    pub ty: Type,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Type {
    pub is_option: bool,
    pub path: String,
}

#[derive(Debug)]
pub struct Struct {
    pub name: String,
    pub named: bool,
    pub fields: Vec<Field>,
    pub attributes: Vec<Attribute>,
}

#[allow(dead_code)]
pub enum Data {
    Struct(Struct),
    Enum(()),
    Union(()),
}

impl Data {
    pub fn name(&self) -> &str {
        match self {
            Data::Struct(Struct { name, .. }) => name.as_str(),
            _ => unimplemented!(),
        }
    }

    pub fn attributes(&self) -> &[Attribute] {
        match self {
            Data::Struct(Struct { attributes, .. }) => &attributes[..],
            _ => unimplemented!(),
        }
    }
}

pub fn parse_data(input: TokenStream) -> Data {
    let mut source = input.into_iter().peekable();

    fn maybe_visibility_modifier<T: Iterator<Item = TokenTree>>(
        source: &mut Peekable<T>,
    ) -> Option<String> {
        if let Some(TokenTree::Ident(ident)) = source.peek() {
            if format!("{}", ident) == "pub" {
                source.next();
                maybe_group(source);
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

    fn maybe_exact_punct<T: Iterator<Item = TokenTree>>(
        source: &mut Peekable<T>,
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

    fn next_literal<T: Iterator<Item = TokenTree>>(source: &mut Peekable<T>) -> Option<String> {
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

    fn maybe_attribute<T: Iterator<Item = TokenTree>>(
        mut source: &mut Peekable<T>,
    ) -> Option<Option<Attribute>> {
        // all attributes, even doc-comments, starts with "#"
        let maybe_attr_punct = maybe_punct(&mut source);
        if let Some("#") = maybe_attr_punct.as_deref() {
            let mut attr_group = maybe_group(&mut source)
                .expect("Expecting attribute body")
                .stream()
                .into_iter()
                .peekable();

            let name = next_ident(&mut attr_group).expect("Attributes should start with a name");

            if name != "nserde" {
                return Some(None);
            }

            let mut args_group = maybe_group(&mut attr_group)
                .expect("Expecting attribute body")
                .stream()
                .into_iter()
                .peekable();

            let mut attr_tokens = vec![];

            loop {
                let attribute_name = next_ident(&mut args_group).expect("Expecting attribute name");
                attr_tokens.push(attribute_name);

                // single-word attribute, like #[nserde(whatever)]
                if maybe_eof(&mut args_group).is_some() {
                    break;
                }
                let _ = maybe_exact_punct(&mut args_group, "=")
                    .expect("Expecting = after attribute argument name");
                let value = next_literal(&mut args_group).expect("Expecting argument value");

                attr_tokens.push(value);

                if maybe_eof(&mut args_group).is_some() {
                    break;
                }
            }

            return Some(Some(Attribute {
                name,
                tokens: attr_tokens,
            }));
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

    fn next_type<T: Iterator<Item = TokenTree>>(mut source: &mut Peekable<T>) -> Option<Type> {
        let mut ty = next_ident(&mut source)?;

        while let Some(_) = maybe_exact_punct(&mut source, ":") {
            let _second_colon = maybe_exact_punct(&mut source, ":").expect("Expecting second :");

            let next_ident = next_ident(&mut source).expect("Expecting next path part after ::");
            ty.push_str(&format!("::{}", next_ident));
        }

        let angel_bracket = maybe_exact_punct(&mut source, "<");

        if angel_bracket.is_some() {
            let mut generic_type = next_type(source).expect("Expecting generic argument");
            while let Some(_comma) = maybe_exact_punct(&mut source, ",") {
                let next_ty = next_type(source).expect("Expecting generic argument");
                generic_type.path.push_str(&format!(", {}", next_ty.path));
            }

            let _closing_bracket =
                maybe_exact_punct(&mut source, ">").expect("Expecting closing generic bracket");

            if ty == "Option" {
                Some(Type {
                    path: generic_type.path,
                    is_option: true,
                })
            } else {
                Some(Type {
                    path: format!("{}<{}>", ty, generic_type.path),
                    is_option: false,
                })
            }
        } else {
            Some(Type {
                path: ty,
                is_option: false,
            })
        }
    }

    fn maybe_group<T: Iterator<Item = TokenTree>>(source: &mut Peekable<T>) -> Option<Group> {
        if let TokenTree::Group(_) = source.peek().unwrap() {
            let group = match source.next().unwrap() {
                TokenTree::Group(group) => group,
                _ => unreachable!("just checked with peek()!"),
            };
            Some(group)
        } else {
            None
        }
    }

    fn maybe_attributes_list<T: Iterator<Item = TokenTree>>(
        source: &mut Peekable<T>,
    ) -> Vec<Attribute> {
        let mut attributes = vec![];

        while let Some(attr) = maybe_attribute(source) {
            if let Some(nserde_attr) = attr {
                attributes.push(nserde_attr);
            }
        }

        attributes
    }

    #[allow(dead_code)]
    fn debug_current_token<T: Iterator<Item = TokenTree>>(source: &mut Peekable<T>) {
        println!("{:?}", source.peek());
    }

    let attributes = maybe_attributes_list(&mut source);

    let pub_or_struct = next_ident(&mut source).expect("Not an ident");

    let struct_keyword = if pub_or_struct == "pub" {
        next_ident(&mut source).expect("pub(whatever) is not supported yet")
    } else {
        pub_or_struct
    };

    assert_eq!(struct_keyword, "struct");

    let struct_name = next_ident(&mut source).expect("Unnamed structs are not supported");

    let group = maybe_group(&mut source).expect("Struct body expected");
    let delimiter = group.delimiter();

    let named = match delimiter {
        Delimiter::Parenthesis => false,
        Delimiter::Brace => true,
        _ => panic!("Struct with unsupported delimiter"),
    };
    let mut body = group.stream().into_iter().peekable();

    let mut fields = vec![];

    loop {
        if maybe_eof(&mut body).is_some() {
            break;
        }

        let attributes = maybe_attributes_list(&mut body);

        let _visibility = maybe_visibility_modifier(&mut body);
        let field_name = if named {
            let field_name = next_ident(&mut body).expect("Field name expected");

            let _ = maybe_exact_punct(&mut body, ":").expect("Delimeter after field name expected");
            Some(field_name)
        } else {
            None
        };
        let ty = next_type(&mut body).expect("Expected field type");
        let _punct = maybe_punct(&mut body);

        fields.push(Field {
            attributes,
            vis: Visibility::Public,
            field_name: field_name,
            ty,
        });
    }

    if named == false {
        maybe_exact_punct(&mut source, ";").expect("Expected ; on the end of tuple struct");
    }

    assert!(
        source.next().is_none(),
        "Unexpected data after end of the struct"
    );

    Data::Struct(Struct {
        name: struct_name,
        named,
        fields,
        attributes,
    })
}
