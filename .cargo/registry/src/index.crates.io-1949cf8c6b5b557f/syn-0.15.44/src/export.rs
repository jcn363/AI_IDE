pub use std::{
    clone::Clone,
    cmp::{
        Eq,
        PartialEq,
    },
    convert::From,
    default::Default,
    fmt::{
        self,
        Debug,
        Formatter,
    },
    hash::{
        Hash,
        Hasher,
    },
    marker::Copy,
    option::Option::{
        None,
        Some,
    },
    result::Result::{
        Err,
        Ok,
    },
};

#[cfg(feature = "printing")]
pub extern crate quote;

pub use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};

pub use span::IntoSpans;

#[cfg(all(
    not(all(target_arch = "wasm32", any(target_os = "unknown", target_os = "wasi"))),
    feature = "proc-macro"
))]
pub use proc_macro::TokenStream;

#[cfg(feature = "printing")]
pub use quote::{
    ToTokens,
    TokenStreamExt,
};

#[allow(non_camel_case_types)]
pub type bool = help::Bool;
#[allow(non_camel_case_types)]
pub type str = help::Str;

mod help {
    pub type Bool = bool;
    pub type Str = str;
}
