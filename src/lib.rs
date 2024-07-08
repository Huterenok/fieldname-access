use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields, FieldsNamed};

/// Derive macro for safe struct field access by their names in runtime
#[proc_macro_derive(FieldnameAccess)]
pub fn fieldname_accessor(inp: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(inp as DeriveInput);

    let structure = match inp.data {
        Data::Struct(ref s) => s,
        Data::Union(_) => {
            panic!("FieldnameAccess cannot be used with unions")
        }
        Data::Enum(_) => {
            panic!("FieldnameAccessor cannot be used with enums")
        }
    };

    let struct_ident = inp.ident;
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

            let variant_name = generate_variant_name(&field_type);
            let variant_ident = Ident::new(&variant_name, Span::call_site());

            (field_name, field_type, variant_ident)
        })
        .collect::<Vec<_>>();

    let value_enum_ident = Ident::new(&format!("{}Field", struct_ident), Span::call_site());
    let value_variants = generate_enum_variants(&field_map, false);

    let value_enum_ident_mut = Ident::new(&format!("{}FieldMut", struct_ident), Span::call_site());
    let value_variants_mut = generate_enum_variants(&field_map, true);

    let match_arms = generate_match_arms(&field_map, &value_enum_ident, false);
    let match_arms_mut = generate_match_arms(&field_map, &value_enum_ident_mut, true);

    let tokens = quote! {
        enum #value_enum_ident<'a> {
            #(#value_variants,)*
        }

        enum #value_enum_ident_mut<'a> {
            #(#value_variants_mut,)*
        }

        impl #struct_ident {
            /// Method for getting reference to struct field by its name
            fn field<'a>(&'a self, fieldname: &str) -> Option<#value_enum_ident<'a>> {
                match fieldname {
                    #(#match_arms,)*
                    _ => None
                }
            }

            /// Method for getting mut reference to struct field by its name
            fn field_mut<'a>(&'a mut self, fieldname: &str) -> Option<#value_enum_ident_mut<'a>> {
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
    let cleaned_str = type_str
        .replace(" ", "")
        .replace("::", "")
        .replace("<", "")
        .replace(">", "")
        .replace(",", "");

    let mut chars = cleaned_str.chars();
    if let Some(first) = chars.next() {
        first.to_uppercase().collect::<String>() + chars.as_str()
    } else {
        String::new()
    }
}

fn generate_enum_variants(
    field_map: &[(Ident, syn::Type, Ident)],
    is_mut: bool,
) -> Vec<proc_macro2::TokenStream> {
    field_map
        .iter()
        .unique_by(|(_, field_type, _)| field_type)
        .map(|(_, field_type, variant_ident)| {
            if is_mut {
                quote! {
                    #variant_ident(&'a mut #field_type)
                }
            } else {
                quote! {
                    #variant_ident(&'a #field_type)
                }
            }
        })
        .collect()
}

fn generate_match_arms(
    field_map: &[(Ident, syn::Type, Ident)],
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
