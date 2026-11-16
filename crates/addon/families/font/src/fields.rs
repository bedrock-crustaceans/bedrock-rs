//! Writing the objects of a font file: the typed members, then the members kept as written
//! whose key no typed member wrote.

use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::build::Object;

/// An object being written: the typed members, then the members kept as written whose key no
/// typed member wrote.
#[derive(Default)]
pub(super) struct Written {
    object: Object,
    keys: Vec<String>,
}

impl Written {
    /// Adds `name` when `value` is present.
    pub(super) fn put(&mut self, name: &str, value: Option<Value>) {
        if value.is_some() {
            self.keys.push(name.to_owned());
        }
        self.object.put(name, value);
    }

    /// The object, with the members of `extra` no typed member stands for.
    pub(super) fn finish(mut self, extra: &UnknownMembers) -> Value {
        for (name, value) in extra.iter() {
            if !self.keys.iter().any(|written| written == name) {
                self.object.put(name, Some(value.clone()));
            }
        }
        self.object.finish()
    }
}

/// A decimal number as JSON that reads back as a decimal: `2.0`, never `2`.
pub(super) fn real_value(real: f64) -> Value {
    Value::from(real)
}
