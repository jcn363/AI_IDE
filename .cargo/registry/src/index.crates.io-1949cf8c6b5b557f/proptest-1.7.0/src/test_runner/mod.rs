//-
// Copyright 2017, 2018 The proptest developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! State and functions for running proptest tests.
//!
//! You do not normally need to access things in this module directly except
//! when implementing new low-level strategies.

mod config;
mod errors;
mod failure_persistence;
mod reason;
#[cfg(feature = "fork")]
mod replay;
mod result_cache;
mod rng;
mod runner;
mod scoped_panic_hook;

pub use self::{
    config::*,
    errors::*,
    failure_persistence::*,
    reason::*,
    result_cache::*,
    rng::*,
    runner::*,
};
