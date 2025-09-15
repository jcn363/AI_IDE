//! Derive macros for common patterns in the Rust AI IDE codebase
//!
//! This crate provides procedural macros to reduce boilerplate code
//! and ensure consistency across derivative implementations.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for implementing Default with new()
#[proc_macro_derive(DefaultFromNew)]
pub fn default_from_new_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let expanded = quote! {
        impl Default for #name {
            fn default() -> Self {
                Self::new()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for Clone implementation
#[proc_macro_derive(DeriveClone)]
pub fn derive_clone_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let clone_impl = match &ast.data {
        syn::Data::Struct(data_struct) => {
            let fields = &data_struct.fields;

            match fields {
                syn::Fields::Named(named_fields) => {
                    let field_clones = named_fields.named.iter().map(|field| {
                        let field_name = &field.ident;
                        quote! {
                            #field_name: self.#field_name.clone(),
                        }
                    });

                    quote! {
                        #name {
                            #(#field_clones)*
                        }
                    }
                }
                syn::Fields::Unnamed(unnamed_fields) => {
                    let field_clones = unnamed_fields.unnamed.iter().enumerate().map(|(i, _)| {
                        let index = syn::Index::from(i);
                        quote! {
                            self.#index.clone(),
                        }
                    });

                    quote! {
                        #name (
                            #(#field_clones)*
                        )
                    }
                }
                syn::Fields::Unit => {
                    quote! {
                        #name
                    }
                }
            }
        }
        syn::Data::Enum(data_enum) => {
            let variants = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    syn::Fields::Named(named_fields) => {
                        let field_patterns = named_fields.named.iter().map(|field| {
                            let field_name = &field.ident;
                            quote! { #field_name }
                        });
                        let field_clones = named_fields.named.iter().map(|field| {
                            let field_name = &field.ident;
                            quote! { #field_name: #field_name.clone() }
                        });

                        quote! {
                            #name::#variant_name { #(#field_patterns),* } => {
                                #name::#variant_name { #(#field_clones),* }
                            }
                        }
                    }
                    syn::Fields::Unnamed(unnamed_fields) => {
                        let field_patterns = unnamed_fields.unnamed.iter().enumerate().map(|(i, _)| {
                            let binding_name = syn::Ident::new(&format!("field_{}", i), proc_macro2::Span::call_site());
                            quote! { #binding_name }
                        });
                        let field_clones = unnamed_fields.unnamed.iter().enumerate().map(|(i, _)| {
                            let binding_name = syn::Ident::new(&format!("field_{}", i), proc_macro2::Span::call_site());
                            quote! { #binding_name.clone() }
                        });

                        quote! {
                            #name::#variant_name ( #(#field_patterns),* ) => {
                                #name::#variant_name ( #(#field_clones),* )
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! {
                            #name::#variant_name => #name::#variant_name
                        }
                    }
                }
            });

            quote! {
                match self {
                    #(#variants),*
                }
            }
        }
        syn::Data::Union(_) => {
            return syn::Error::new_spanned(ast, "Clone cannot be derived for unions")
                .to_compile_error()
                .into();
        }
    };

    let expanded = quote! {
        impl Clone for #name {
            fn clone(&self) -> Self {
                #clone_impl
            }
        }
    };

    TokenStream::from(expanded)
}
