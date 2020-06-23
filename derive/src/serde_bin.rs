use crate::parse::Struct;

use proc_macro::TokenStream;

pub fn derive_ser_bin_proxy(proxy_type: &str, type_: &str) -> TokenStream {
    format!(
        "impl SerBin for {} {{
            fn ser_bin(&self, s: &mut Vec<u8>) {{
                let proxy: {} = self.into();
                proxy.ser_bin(s);
            }}
        }}",
        type_, proxy_type
    )
    .parse()
    .unwrap()
}

pub fn derive_de_bin_proxy(proxy_type: &str, type_: &str) -> TokenStream {
    format!(
        "impl DeBin for {} {{
            fn de_bin(o:&mut usize, d:&[u8]) -> std::result::Result<Self, DeBinErr> {{
                let proxy: {} = DeBin::deserialize_bin(d)?;
                std::result::Result::Ok(Into::into(&proxy))
            }}
        }}",
        type_, proxy_type
    )
    .parse()
    .unwrap()
}

pub fn derive_ser_bin_struct(struct_: &Struct) -> TokenStream {
    let mut body = String::new();

    for field in &struct_.fields {
        if let Some(proxy) = crate::shared::attrs_proxy(&field.attributes) {
            l!(
                body,
                "let proxy: {} = Into::into(&self.{});",
                proxy,
                field.field_name
            );
            l!(body, "proxy.ser_bin(s);");
        } else {
            l!(body, "self.{}.ser_bin(s);", field.field_name);
        }
    }
    format!(
        "impl SerBin for {} {{
            fn ser_bin(&self, s: &mut Vec<u8>) {{
                {}
            }}
        }}",
        struct_.name, body
    )
    .parse()
    .unwrap()
}

// pub fn derive_ser_bin_struct_unnamed(input: &DeriveInput, fields:&FieldsUnnamed) -> TokenStream {
//     let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
//     let bound = parse_quote!(SerBin);
//     let bounded_where_clause = where_clause_with_bound(&input.generics, bound);
//     let ident = &input.ident;

//     let mut fieldname = Vec::new();
//     for (index, field) in fields.unnamed.iter().enumerate() {
//         fieldname.push(LitInt::new(&format!("{}", index), field.span()));
//     }
//     quote! {
//         impl #impl_generics SerBin for #ident #ty_generics #bounded_where_clause {
//             fn ser_bin(&self, s: &mut Vec<u8>) {
//                 #(
//                     self.#fieldname.ser_bin(s);
//                 ) *
//             }
//         }
//     }
// }

pub fn derive_de_bin_struct(struct_: &Struct) -> TokenStream {
    let mut body = String::new();

    for field in &struct_.fields {
        if let Some(proxy) = crate::shared::attrs_proxy(&field.attributes) {
            l!(body, "{}: {{", field.field_name);
            l!(body, "let proxy: {} = DeBin::de_bin(o, d)?;", proxy);
            l!(body, "Into::into(&proxy)");
            l!(body, "},")
        } else {
            l!(body, "{}: DeBin::de_bin(o, d)?,", field.field_name);
        }
    }

    format!(
        "impl DeBin for {} {{
            fn de_bin(o:&mut usize, d:&[u8]) -> std::result::Result<Self, DeBinErr> {{
                std::result::Result::Ok(Self {{
                    {}
                }})
            }}
        }}",
        struct_.name, body
    )
    .parse()
    .unwrap()
}

// pub fn derive_de_bin_struct_unnamed(input: &DeriveInput, fields:&FieldsUnnamed) -> TokenStream {
//     let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
//     let ident = &input.ident;
//     let bound = parse_quote!(DeBin);
//     let bounded_where_clause = where_clause_with_bound(&input.generics, bound);

//     let mut fieldname = Vec::new();
//     for (index, field) in fields.unnamed.iter().enumerate() {
//         fieldname.push(LitInt::new(&format!("{}", index), field.span()));
//     }

//     quote! {
//         impl #impl_generics DeBin for #ident #ty_generics #bounded_where_clause {
//             fn de_bin(o:&mut usize, d:&[u8]) -> std::result::Result<Self,DeBinErr> {
//                 std::result::Result::Ok(Self {
//                     #(
//                         #fieldname: DeBin::de_bin(o,d)?,
//                     ) *
//                 })
//             }
//         }
//     }
// }

// pub fn derive_ser_bin_enum(input: &DeriveInput, enumeration: &DataEnum) -> TokenStream {
//     let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
//     let bound = parse_quote!(SerBin);
//     let bounded_where_clause = where_clause_with_bound(&input.generics, bound);

//     let ident = &input.ident;

//     let mut match_item = Vec::new();

//     for (index, variant) in enumeration.variants.iter().enumerate() {
//         let lit = LitInt::new(&format!("{}u16", index), ident.span());
//         let ident = &variant.ident;
//         match &variant.fields {
//             Fields::Unit => {
//                 match_item.push(quote!{
//                     Self::#ident => #lit.ser_bin(s),
//                 })
//             },
//             Fields::Named(fields_named) => {
//                 let mut field_names = Vec::new();
//                 for field in &fields_named.named {
//                     if let Some(ident) = &field.ident {
//                         field_names.push(ident);
//                     }
//                 }
//                 match_item.push(quote!{
//                     Self::#ident {#(#field_names,) *} => {
//                         #lit.ser_bin(s);
//                         #(#field_names.ser_bin(s);) *
//                     }
//                 });
//             },
//             Fields::Unnamed(fields_unnamed) => {
//                 let mut field_names = Vec::new();
//                 for (index, field) in fields_unnamed.unnamed.iter().enumerate() {
//                     field_names.push(Ident::new(&format!("f{}", index), field.span()));
//                 }
//                 match_item.push(quote!{
//                     Self::#ident (#(#field_names,) *) => {
//                         #lit.ser_bin(s);
//                         #(#field_names.ser_bin(s);) *
//                     }
//                 });
//             },
//         }
//     }

//     quote! {
//         impl #impl_generics SerBin for #ident #ty_generics #bounded_where_clause {
//             fn ser_bin(&self, s: &mut Vec<u8>) {
//                 match self {
//                     #(
//                         #match_item
//                     ) *
//                 }
//             }
//         }
//     }
// }

// pub fn derive_de_bin_enum(input: &DeriveInput, enumeration: &DataEnum) -> TokenStream {

//     let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
//     let ident = &input.ident;
//     let bound = parse_quote!(DeBin);
//     let bounded_where_clause = where_clause_with_bound(&input.generics, bound);

//     let mut match_item = Vec::new();

//     for (index, variant) in enumeration.variants.iter().enumerate() {
//         let lit = LitInt::new(&format!("{}u16", index), ident.span());
//         let ident = &variant.ident;
//         match &variant.fields {
//             Fields::Unit => {
//                 match_item.push(quote!{
//                     #lit => Self::#ident,
//                 })
//             },
//             Fields::Named(fields_named) => {
//                 let mut field_names = Vec::new();
//                 for field in &fields_named.named {
//                     if let Some(ident) = &field.ident {
//                         field_names.push(quote!{#ident: DeBin::de_bin(o,d)?});
//                     }
//                 }
//                 match_item.push(quote!{
//                     #lit => Self::#ident {#(#field_names,) *},
//                 });
//             },
//             Fields::Unnamed(fields_unnamed) => {
//                 let mut field_names = Vec::new();
//                 for _ in &fields_unnamed.unnamed {
//                     field_names.push(quote! {DeBin::de_bin(o,d)?});
//                 }
//                 match_item.push(quote!{
//                     #lit => Self::#ident(#(#field_names,) *),
//                 });
//             },
//         }
//     }

//     quote! {
//         impl #impl_generics DeBin for #ident #ty_generics #bounded_where_clause {
//             fn de_bin(o:&mut usize, d:&[u8]) -> std::result::Result<Self, DeBinErr> {
//                 let id: u16 = DeBin::de_bin(o,d)?;
//                 Ok(match id {
//                     #(
//                         #match_item
//                     ) *
//                     _ => return std::result::Result::Err(DeBinErr{o:*o, l:0, s:d.len()})
//                 })
//             }
//         }
//     }
// }
