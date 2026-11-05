//! Reading the members of one object of a font file: each known member as its typed reading,
//! or kept as written when the typed field cannot say it, and every other member kept and
//! reported.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Disposition, Node};
use bedrock_addon_support::fixed_path::{Object, as_int, as_real, raw_member, reader};

/// What a member turned out to be.
#[derive(Debug)]
pub(super) enum Got<T> {
    /// No such member.
    Absent,
    /// The member, as the typed field holds it.
    Typed(T),
    /// A member of another type; it is kept in `extra` as written.
    Other(Value),
}

impl<T> Got<T> {
    /// The typed reading.
    pub(super) fn into_typed(self) -> Option<T> {
        match self {
            Self::Typed(value) => Some(value),
            _ => None,
        }
    }

    /// Whether the member is there with another type; a `null` counts when `null_too`.
    pub(super) fn is_other(&self, null_too: bool) -> bool {
        matches!(self, Self::Other(value) if null_too || !value.is_null())
    }
}

/// The members of one object, read one by one.
pub(super) struct Fields<'v> {
    object: &'v Value,
    /// The object's place in the file.
    pub(super) at: JsonPath,
    known: Vec<&'static str>,
    extra: UnknownMembers,
}

impl<'v> Fields<'v> {
    /// The members of `object` (none when it is no object).
    pub(super) fn new(object: &'v Value, at: JsonPath) -> Self {
        Self {
            object,
            at,
            known: Vec::new(),
            extra: UnknownMembers::default(),
        }
    }

    /// The member `name` as written; the caller holds it whole.
    pub(super) fn raw(&mut self, name: &'static str) -> Option<&'v Value> {
        self.known.push(name);
        raw_member(self.object, name)
    }

    /// The member `name` through `read`; a member `read` refuses is kept as written.
    pub(super) fn get<T>(
        &mut self,
        name: &'static str,
        read: impl FnOnce(&Value) -> Option<T>,
    ) -> Got<T> {
        let Some(member) = self.raw(name) else {
            return Got::Absent;
        };
        if let Some(value) = read(member) {
            Got::Typed(value)
        } else {
            self.extra.insert(name.to_owned(), member.clone());
            Got::Other(member.clone())
        }
    }

    /// A string member.
    pub(super) fn string(&mut self, name: &'static str) -> Got<String> {
        self.get(name, |v| v.as_string().map(|s| s.as_str().to_owned()))
    }

    /// A member the game tests with `isInt`.
    pub(super) fn int(&mut self, name: &'static str) -> Got<i32> {
        self.get(name, as_int)
    }

    /// A member the game tests with `isDouble`: a number written as a decimal. The flag says
    /// the number is beyond what a float holds and reads as the largest finite float of its
    /// sign, so that no infinity is held (and none fails the writer).
    // What the game makes of such a scale is not observable, so it is clamped the same way as
    // every other number beyond a float.
    pub(super) fn real(&mut self, name: &'static str) -> (Got<f64>, bool) {
        let mut clamped = false;
        let got = self.get(name, |v| {
            let real = as_real(v)?;
            let limit = f64::from(f32::MAX);
            clamped = !(-limit..=limit).contains(&real);
            Some(if clamped { limit.copysign(real) } else { real })
        });
        (got, clamped)
    }

    /// Keeps `name` as written although it was read (a member the game then drops).
    pub(super) fn keep(&mut self, name: &'static str) {
        if let Some(member) = raw_member(self.object, name) {
            self.extra.insert(name.to_owned(), member.clone());
        }
    }

    /// The path of the member `name`.
    pub(super) fn path(&self, name: &str) -> JsonPath {
        self.at.child(name)
    }

    /// The members kept as written, then every member not read, each of those with an
    /// `UnrecognizedField` info.
    pub(super) fn finish(mut self, diagnostics: &mut Diagnostics) -> UnknownMembers {
        let mut read = reader(diagnostics);
        let node = Node::new(self.object, self.at.clone());
        if let Some(members) = self
            .object
            .is_object()
            .then(|| read.object(&node))
            .flatten()
        {
            let unknown = read.unknown(&members, &self.known, Disposition::Keep);
            self.extra.extend(
                unknown
                    .into_iter()
                    .map(|(key, value)| (key.to_owned(), value.clone())),
            );
        }
        self.extra
    }
}

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
