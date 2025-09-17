#![recursion_limit = "256"]
#![cfg_attr(feature = "diagnostics", feature(proc_macro_diagnostic))]
#![warn(rust_2018_idioms)]

#[cfg(feature = "diagnostics")]
extern crate proc_macro;

#[macro_use]
mod macros_private;
#[macro_use]
mod macros_public;

pub mod ast;
pub(crate) mod codegen;
pub mod derive;
pub mod error;
mod from_attributes;
mod from_derive_input;
mod from_field;
mod from_generic_param;
mod from_generics;
mod from_meta;
mod from_type_param;
mod from_variant;
pub(crate) mod options;
pub mod usage;
pub mod util;

pub use self::{
    error::{
        Error,
        Result,
    },
    from_attributes::FromAttributes,
    from_derive_input::FromDeriveInput,
    from_field::FromField,
    from_generic_param::FromGenericParam,
    from_generics::FromGenerics,
    from_meta::FromMeta,
    from_type_param::FromTypeParam,
    from_variant::FromVariant,
};

// Re-exports
#[doc(hidden)]
pub use quote::ToTokens;
#[doc(hidden)]
pub use syn;
