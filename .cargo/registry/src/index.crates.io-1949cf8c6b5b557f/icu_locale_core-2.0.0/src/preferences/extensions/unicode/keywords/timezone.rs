// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::{
    extensions::unicode::Value,
    preferences::extensions::unicode::{
        errors::PreferencesParseError,
        struct_keyword,
    },
    subtags::Subtag,
};

struct_keyword!(
    /// A Unicode Timezone Identifier defines a timezone.
    ///
    /// The valid values are listed in [LDML](https://unicode.org/reports/tr35/#UnicodeTimezoneIdentifier).
    [Copy]
    TimeZoneShortId,
    "tz",
    Subtag,
    |input: Value| {
        input
            .into_single_subtag()
            .map(Self)
            .ok_or(PreferencesParseError::InvalidKeywordValue)
    },
    |input: TimeZoneShortId| {
        crate::extensions::unicode::Value::from_subtag(Some(input.0))
    }
);
