use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Fields, Path, Token};

pub fn derive_from(input: DeriveInput) -> TokenStream {
    let type_name = input.ident;
    let variants = match input.data {
        Data::Enum(e) => e.variants,
        _ => panic!("derive(From) is only valid for enums"),
    };
    let generics = input.generics;

    let impls = variants.into_iter().filter_map(|v| {
        let name = v.ident;
        let (attrs, field) = match &v.fields {
            Fields::Unnamed(f) => {
                let field = f.unnamed.first()?;
                (&field.attrs, &field.ty)
            }
            _ => return None,
        };
        let mut nested = vec![];
        for a in attrs.iter().filter(|a| a.path().is_ident("nested")) {
            if let Ok(paths) = a.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated) {
                for path in paths {
                    nested.push(quote! {
                        impl #generics From<#path> for #type_name #generics {
                            fn from(m: #path) -> Self {
                                Self::#name(m.into())
                            }
                        }
                    });
                }
            }
        }
        Some(quote! {
            #(#nested)*
            impl #generics From<#field> for #type_name #generics {
                fn from(f: #field) -> Self {
                    Self::#name(f)
                }
            }
        })
    });

    quote! { #(#impls)* }
}
