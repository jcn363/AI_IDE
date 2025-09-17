//-
// Copyright 2017, 2018, 2019 The proptest developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Re-exports the most commonly-needed APIs of proptest.
//!
//! This module is intended to be wildcard-imported, i.e.,
//! `use proptest::prelude::*;`. Note that it re-exports the whole crate itself
//! under the name `prop`, so you don't need a separate `use proptest;` line.
//!
//! In addition to Proptest's own APIs, this also reexports a small portion of
//! the `rand` crate sufficient to easily use `prop_perturb` and other
//! functionality that exposes random number generators. Please note that this
//! is will always be a direct reexport; using these in preference to using the
//! `rand` crate directly will not provide insulation from the upcoming
//! revision to the `rand` crate.

pub use crate::{
    arbitrary::{
        any,
        any_with,
        Arbitrary,
    },
    prop_assert,
    prop_assert_eq,
    prop_assert_ne,
    prop_assume,
    prop_compose,
    prop_oneof,
    proptest,
    strategy::{
        BoxedStrategy,
        Just,
        SBoxedStrategy,
        Strategy,
    },
    test_runner::{
        Config as ProptestConfig,
        TestCaseError,
    },
};

pub use rand::{
    Rng,
    RngCore,
};

/// Re-exports the entire public API of proptest so that an import of `prelude`
/// allows simply writing, for example, `prop::num::i32::ANY` rather than
/// `proptest::num::i32::ANY` plus a separate `use proptest;`.
pub mod prop {
    #[cfg(feature = "std")]
    pub use crate::string;
    pub use crate::{
        arbitrary,
        array,
        bits,
        bool,
        char,
        collection,
        num,
        option,
        result,
        sample,
        strategy,
        test_runner,
        tuple,
    };
}
