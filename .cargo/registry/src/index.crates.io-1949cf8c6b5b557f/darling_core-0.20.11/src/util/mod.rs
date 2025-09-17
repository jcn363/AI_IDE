//! Utility types for attribute parsing.

mod callable;
mod flag;
mod ident_string;
mod ignored;
mod over_ride;
mod parse_attribute;
pub mod parse_expr;
mod path_list;
mod path_to_string;
mod shape;
mod spanned_value;
mod with_original;

pub use self::{
    callable::Callable,
    flag::Flag,
    ident_string::IdentString,
    ignored::Ignored,
    over_ride::Override,
    parse_attribute::parse_attribute_to_meta_list,
    path_list::PathList,
    path_to_string::path_to_string,
    shape::{
        AsShape,
        Shape,
        ShapeSet,
    },
    spanned_value::SpannedValue,
    with_original::WithOriginal,
};
