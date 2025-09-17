use crate::{
    macro_utils::{
        gen_ident_range_just_idents,
        GenIdentRange,
    },
    macro_utils_shared::{
        expect_no_tokens,
        match_token,
        parse_count_and,
        parse_parentheses,
        parse_range_param,
        parse_unbounded_range_param,
        usize_tt,
        CountAnd,
        RangeB,
        RepeatTimes,
        Spans,
    },
    mmatches,
    try_,
    used_proc_macro::{
        token_stream::IntoIter,
        Delimiter,
        Group,
        TokenStream,
        TokenTree,
    },
};

use core::{
    iter::{
        Chain,
        Cycle,
        Peekable,
    },
    marker::PhantomData,
    ops::RangeFrom,
};

use alloc::{
    boxed::Box,
    format,
    string::ToString,
};
