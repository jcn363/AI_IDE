//! Check that these imports do not import overlapping items.
#![allow(unused_imports)]

pub use objc2::{
    runtime::*,
    *,
};
#[cfg(feature = "objc2-core-foundation")]
pub use objc2_core_foundation::*;
pub use objc2_foundation::*;
