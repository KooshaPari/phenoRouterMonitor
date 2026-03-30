use std::collections::BTreeMap;

use serde_json::Value;

pub(crate) fn to_sorted(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> = map
                .into_iter()
                .map(|(key, value)| (key, to_sorted(value)))
                .collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(to_sorted).collect()),
        other => other,
    }
}

pub(crate) fn to_sorted_pretty(value: Value) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&to_sorted(value))
}

pub(crate) fn to_sorted_line(value: Value) -> Result<String, serde_json::Error> {
    serde_json::to_string(&to_sorted(value))
}
