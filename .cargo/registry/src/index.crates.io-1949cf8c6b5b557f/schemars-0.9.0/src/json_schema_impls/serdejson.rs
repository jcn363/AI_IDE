use crate::{
    SchemaGenerator,
    _alloc_prelude::*,
    json_schema,
    JsonSchema,
    Schema,
};
use alloc::{
    borrow::Cow,
    collections::BTreeMap,
};
use serde_json::{
    Map,
    Number,
    Value,
};

impl JsonSchema for Value {
    inline_schema!();

    fn schema_name() -> Cow<'static, str> {
        "AnyValue".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        true.into()
    }
}

forward_impl!(Map<String, Value> => BTreeMap<String, Value>);

impl JsonSchema for Number {
    inline_schema!();

    fn schema_name() -> Cow<'static, str> {
        "Number".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "number"
        })
    }
}

#[cfg(feature = "raw_value")]
forward_impl!(serde_json::value::RawValue => Value);
