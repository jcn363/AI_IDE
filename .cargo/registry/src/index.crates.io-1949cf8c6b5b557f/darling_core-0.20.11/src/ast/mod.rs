//! Utility types for working with the AST.

mod data;
mod generics;

pub use self::{
    data::*,
    generics::{
        GenericParam,
        GenericParamExt,
        Generics,
    },
};
