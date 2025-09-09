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
#[proc_macro_derive(Clone)]
pub fn clone_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let expanded = quote! {
        impl Clone for #name {
            fn clone(&self) -> Self {
                // Simple clone implementation - assumes all fields have clone
                // In practice, you'd derive this properly based on struct fields
                unimplemented!("Clone derived but not implemented")
            }
        }
    };

    TokenStream::from(expanded)
}
