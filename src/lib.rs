use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Expr, ExprLit, Fields,
    FieldsNamed, Lit, Meta, Type,
};

/// # Description
///
/// Derive macro for safe struct field access by their names in runtime
///
///### Container attributes
///* `#fieldname_enum(name = "NewName")` - Name of generated enum of possible values
///```rust
///use fieldname_access::FieldnameAccess;
///
///#[derive(FieldnameAccess, Default)]
///#[fieldname_enum(name = "NewName")]
///struct NamedFieldname {
///  name: String,
///  age: i64,
///}
///
///let mut instance = NamedFieldname::default();
///match instance.field("name").unwrap() {
///    NewName::String(val) => {}
///    NewName::I64(val) => {},
///}
///match instance.field_mut("name").unwrap() {
///    NewNameMut::String(val) => {}
///    NewNameMut::I64(val) => {},
///}
///```
///
///* `#fieldname_enum(derive = [Debug, Clone], derive_mut = [Debug])` - Derive macroses for generated enums.
///`derive` only for enum with immutable references, `derive_mut` only for enum with mutable references.
///It can be helpful when you want to derive `Clone` but only for immutable references as mutable are not clonable
///```rust
///use fieldname_access::FieldnameAccess;
///
///#[derive(FieldnameAccess)]
///#[fieldname_enum(derive = [Debug, Clone], derive_mut = [Debug])]
///struct NamedFieldname {
///  name: String,
///  age: i64,
///}
///```
///
///* `#fieldname_enum(derive_all = [Debug])` - Derive macroses for immutable and mutable generated enums
///```rust
///use fieldname_access::FieldnameAccess;
///
///#[derive(FieldnameAccess)]
///#[fieldname_enum(derive_all = [Debug])]
///struct NamedFieldname {
///  name: String,
///  age: i64,
///}
///```
///
///### Field attributes
///
///* `#fieldname = "AmazingAge"` - Name of variant for field in generated enum.
///It can be helpfull when you want to 'mark' field with specific variant name
///```rust
///use fieldname_access::FieldnameAccess;
///
///#[derive(FieldnameAccess, Default)]
///struct NamedFieldname {
///  name: String,
///  #[fieldname = "MyAge"]
///  age: i64,
///  dog_age: i64
///}
///let mut instance = NamedFieldname::default();
///match instance.field("name").unwrap() {
///    NamedFieldnameField::String(val) => {}
///    NamedFieldnameField::MyAge(val) => {}
///    NamedFieldnameField::I64(val) => {}
///}
///match instance.field_mut("name").unwrap() {
///    NamedFieldnameFieldMut::String(val) => {}
///    NamedFieldnameFieldMut::MyAge(val) => {}
///    NamedFieldnameFieldMut::I64(val) => {}
///}  
///```
#[proc_macro_derive(FieldnameAccess, attributes(fieldname_enum, fieldname))]
pub fn fieldname_accessor(inp: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(inp as DeriveInput);
    let structure = match inp.data {
        Data::Struct(ref s) => s,
        Data::Union(_) => {
            panic!("FieldnameAccess cannot be used with unions")
        }
        Data::Enum(_) => {
            panic!("FieldnameAccess cannot be used with enums")
        }
    };
    let struct_ident = inp.ident;
    let visibility = inp.vis;
    let field_lifetime: syn::GenericParam = parse_quote!('field);
    let generics = inp.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let mut enum_generics = generics.clone();
    enum_generics.params.push(field_lifetime.clone());

    let fields = match &structure.fields {
        Fields::Named(FieldsNamed { named: x, .. }) => x.to_owned(),
        Fields::Unnamed(_) | Fields::Unit => {
            panic!("Nameless fields are not supported")
        }
    };
    let field_map = fields
        .into_iter()
        .map(|field| {
            let field_type = field.ty;
            let field_name = field.ident.expect("Nameless fields are not supported");
            let variant_ident = if let Some(name) = retrieve_fieldname(&field.attrs) {
                name
            } else {
                let type_str = generate_variant_name(&field_type);
                Ident::new(&type_str, Span::call_site())
            };
            (field_name, field_type, variant_ident)
        })
        .collect::<Vec<_>>();

    let (derive, derive_mut) = if let Some(derives) = retrieve_derives(&inp.attrs, "derive_all") {
        (Some(derives.clone()), Some(derives))
    } else {
        let derive = retrieve_derives(&inp.attrs, "derive");
        let derive_mut = retrieve_derives(&inp.attrs, "derive_mut");
        (derive, derive_mut)
    };

    let value_enum_ident = retrieve_enum_name(&inp.attrs).unwrap_or(Ident::new(
        &format!("{}Field", struct_ident),
        Span::call_site(),
    ));
    let value_variants = generate_enum_variants(&field_map, false);
    let value_enum_ident_mut = Ident::new(&format!("{}Mut", value_enum_ident), Span::call_site());
    let value_variants_mut = generate_enum_variants(&field_map, true);

    let match_arms = generate_match_arms(&field_map, &value_enum_ident, false);
    let match_arms_mut = generate_match_arms(&field_map, &value_enum_ident_mut, true);

    let tokens = quote! {
        /// Enum with reference to possible field
        #derive
        #visibility enum #value_enum_ident #enum_generics #where_clause {
            #(#value_variants,)*
        }

        /// Enum with mutable reference to possible field
        #derive_mut
        #visibility enum #value_enum_ident_mut #enum_generics #where_clause {
            #(#value_variants_mut,)*
        }
        impl #impl_generics #struct_ident #ty_generics #where_clause {
            /// Method for getting reference to struct field by its name
            #visibility fn field<#field_lifetime>(&#field_lifetime self, fieldname: &str) -> Option<#value_enum_ident #enum_generics> {
                match fieldname {
                    #(#match_arms,)*
                    _ => None
                }
            }
            /// Method for getting mutable reference to struct field by its name
            #visibility fn field_mut<#field_lifetime>(&#field_lifetime mut self, fieldname: &str) -> Option<#value_enum_ident_mut #enum_generics> {
                match fieldname {
                    #(#match_arms_mut,)*
                    _ => None
                }
            }
        }
    };
    tokens.into()
}

fn generate_variant_name(ty: &syn::Type) -> String {
    let type_str = ty.to_token_stream().to_string();
    shorten_type(type_str)
}

fn shorten_type(type_str: String) -> String {
    let mut short_type = type_str
        .chars()
        .skip_while(|c| !c.is_uppercase())
        .peekable();
    if short_type.peek().is_some() {
        let mut complex_type_str = String::new();
        while let Some(c) = short_type.next() {
            if c.is_ascii_alphanumeric() {
                complex_type_str.push(c);
            }
            if c == '<' {
                complex_type_str += &shorten_type(short_type.collect());
                break;
            }
        }
        complex_type_str
    } else {
        let cleaned_str = type_str
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>();
        cleaned_str[0..1].to_uppercase() + &cleaned_str[1..]
    }
}

fn generate_enum_variants(
    field_map: &[(Ident, syn::Type, Ident)],
    is_mut: bool,
) -> Vec<proc_macro2::TokenStream> {
    field_map
        .iter()
        .unique_by(|(_, _, variant_ident)| variant_ident)
        .map(|(_, field_type, variant_ident)| {
            if is_mut {
                quote! {
                    #variant_ident(&'field mut #field_type)
                }
            } else {
                quote! {
                    #variant_ident(&'field #field_type)
                }
            }
        })
        .collect()
}

fn generate_match_arms(
    field_map: &[(Ident, Type, Ident)],
    value_enum_ident: &Ident,
    is_mut: bool,
) -> Vec<proc_macro2::TokenStream> {
    field_map
        .iter()
        .map(|(field_name, _, variant_ident)| {
            let field_name_str = field_name.to_string();
            if is_mut {
                quote! {
                    #field_name_str => Some(#value_enum_ident::#variant_ident(&mut self.#field_name))
                }
            } else {
                quote! {
                    #field_name_str => Some(#value_enum_ident::#variant_ident(&self.#field_name))
                }
            }
        })
        .collect()
}

fn retrieve_enum_name(attrs: &[Attribute]) -> Option<Ident> {
    if let Some(TokenTree::Literal(lit)) = get_fieldname_enum_val(attrs, "name") {
        let lit = lit.to_string();
        Some(Ident::new(&lit[1..lit.len() - 1], Span::call_site()))
    } else {
        None
    }
}

fn retrieve_derives(attrs: &[Attribute], derive_group: &str) -> Option<proc_macro2::TokenStream> {
    if let Some(TokenTree::Group(group)) = get_fieldname_enum_val(attrs, derive_group) {
        let token_stream = group.stream();
        Some(quote!(#[derive(#token_stream)]))
    } else {
        None
    }
}

fn retrieve_fieldname(attrs: &[Attribute]) -> Option<Ident> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(meta_name_value) => {
            let fieldname_enum_attr = meta_name_value.path.segments.first()?;
            if fieldname_enum_attr.ident != "fieldname" {
                return None;
            }
            if let Expr::Lit(ExprLit {
                lit: Lit::Str(ref str),
                ..
            }) = meta_name_value.value
            {
                Some(Ident::new(&str.value(), Span::call_site()))
            } else {
                None
            }
        }

        _ => None,
    })
}

fn get_fieldname_enum_val(attrs: &[Attribute], attr_name: &str) -> Option<TokenTree> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::List(meta_list) => {
            let fieldname_enum_attr = meta_list.path.segments.first()?;
            if fieldname_enum_attr.ident != "fieldname_enum" {
                return None;
            }
            meta_list
                .tokens
                .clone()
                .into_iter()
                .skip_while(|token| token.to_string() != attr_name)
                .nth(2)
        }
        _ => None,
    })
}
