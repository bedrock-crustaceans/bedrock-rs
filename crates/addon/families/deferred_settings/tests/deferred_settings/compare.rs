//! Two JSON texts compared as values, member for member: what the generated round trip holds a
//! written document against its input with (FAMILY-NOTES, wave 4: "write-back is checked as
//! authored").
//!
//! Both texts are read through the door the loaders read through (`bedrock_addon_core::json`), so a
//! number is the number a loader sees: numbers are compared as numbers, each with its kind (an
//! integer or a real), and a quoted string never equals a number. Each number keeps the token
//! it was written as: where two numbers are equal and of one kind, their tokens are compared
//! too, so that a number written in another spelling is seen and named.

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{JsonSlice, Value, span_at};

/// A number as a loader sees it, with the token it was written as.
#[derive(Clone, Debug)]
pub struct Number {
    /// The value when the token is an integer a 64-bit integer holds; any other number (a
    /// real token, an integer beyond 64 bits) is its double alone.
    pub int: Option<i128>,
    /// The value as a double.
    pub real: f64,
    /// The token of the text.
    pub token: String,
}

impl Number {
    /// Whether the token is an integer a 64-bit integer holds.
    pub fn is_int(&self) -> bool {
        self.int.is_some()
    }
}

/// Two numbers are equal when they are of one kind and one value; their tokens are not
/// compared (`-0.0` and `0.0` are one number).
impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self.int, other.int) {
            (Some(one), Some(other)) => one == other,
            (None, None) => self.real == other.real,
            _ => false,
        }
    }
}

/// A JSON value as it is compared: an object's members in the order of the text.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(Number),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// The value of `text`, which must be JSON. Of a member written twice it holds the last
    /// value, as every loader does.
    #[track_caller]
    pub fn of_text(text: &str) -> Self {
        let value: Value = bedrock_addon_core::json::from_str(text)
            .unwrap_or_else(|e| panic!("no JSON: {e}\n{text}"));
        Self::of(&value, text, &mut JsonPath::new())
    }

    fn of(value: &Value, text: &str, path: &mut JsonPath) -> Self {
        if let Some(object) = value.as_object() {
            Self::Obj(
                object
                    .iter()
                    .map(|(key, member)| {
                        path.push(key.as_str());
                        let member = Self::of(member, text, path);
                        path.pop();
                        (key.as_str().to_owned(), member)
                    })
                    .collect(),
            )
        } else if let Some(array) = value.as_array() {
            Self::Arr(
                array
                    .iter()
                    .enumerate()
                    .map(|(index, element)| {
                        path.push_index(index);
                        let element = Self::of(element, text, path);
                        path.pop();
                        element
                    })
                    .collect(),
            )
        } else if let Some(text) = value.as_string() {
            Self::Str(text.as_str().to_owned())
        } else if let Some(flag) = value.as_bool() {
            Self::Bool(flag)
        } else if let Some(number) = value.as_number() {
            let int = if number.is_integer() {
                number
                    .to_i64()
                    .map(i128::from)
                    .or_else(|| number.to_u64().map(i128::from))
            } else {
                None
            };
            let span = span_at(JsonSlice::new(text), path)
                .unwrap_or_else(|| panic!("no value at {path} in {text}"));
            let (from, len) = (span.offset as usize, span.len as usize);
            Self::Num(Number {
                int,
                real: number.to_f64_lossy(),
                token: text[from..from + len].to_owned(),
            })
        } else {
            Self::Null
        }
    }

    /// The member `key` of an object.
    pub fn get(&self, key: &str) -> Option<&Json> {
        self.members()?
            .iter()
            .find_map(|(name, value)| (name == key).then_some(value))
    }

    /// The members of an object.
    pub fn members(&self) -> Option<&Vec<(String, Json)>> {
        match self {
            Self::Obj(members) => Some(members),
            _ => None,
        }
    }

    /// The members of an object, to change.
    pub fn members_mut(&mut self) -> Option<&mut Vec<(String, Json)>> {
        match self {
            Self::Obj(members) => Some(members),
            _ => None,
        }
    }

    /// The value at the keys `path` below this one, to change.
    pub fn at_mut(&mut self, path: &[&str]) -> Option<&mut Json> {
        let Some((first, rest)) = path.split_first() else {
            return Some(self);
        };
        self.members_mut()?
            .iter_mut()
            .find_map(|(name, value)| (name == first).then_some(value))?
            .at_mut(rest)
    }

    /// Takes the member `key` out of an object.
    pub fn take(&mut self, key: &str) -> Option<Json> {
        let members = self.members_mut()?;
        let index = members.iter().position(|(name, _)| name == key)?;
        Some(members.remove(index).1)
    }
}

/// One step of a path into a document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    Key(String),
    Index(usize),
}

/// The path as the findings print theirs: steps joined by `/`.
pub fn shown(path: &[Step]) -> String {
    path.iter()
        .map(|step| match step {
            Step::Key(key) => key.clone(),
            Step::Index(index) => index.to_string(),
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// How the written text differs from the input at one place.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    /// A member of the input that the written text lacks.
    Removed(Json),
    /// A member of the written text that the input lacks.
    Added(Json),
    /// Another value: `(input, written)`. Two numbers of one value and two kinds are this.
    Replaced(Json, Json),
    /// One number of one kind in two spellings: `(input, written)`.
    Respelt(Number, Number),
    /// The members both objects have stand in another order.
    Reordered,
}

/// One difference: where, and what.
#[derive(Clone, Debug, PartialEq)]
pub struct Difference {
    pub path: Vec<Step>,
    pub change: Change,
}

/// Every difference of `written` from `input`: at each object the members only the input has,
/// then the order, then the written members in their order.
pub fn differences(input: &Json, written: &Json) -> Vec<Difference> {
    let mut walk = Walk {
        path: Vec::new(),
        found: Vec::new(),
    };
    walk.compare(input, written);
    walk.found
}

struct Walk {
    path: Vec<Step>,
    found: Vec<Difference>,
}

impl Walk {
    fn report(&mut self, change: Change) {
        self.found.push(Difference {
            path: self.path.clone(),
            change,
        });
    }

    fn compare(&mut self, input: &Json, written: &Json) {
        match (input, written) {
            (Json::Obj(before), Json::Obj(after)) => self.objects(before, after),
            (Json::Arr(before), Json::Arr(after)) if before.len() == after.len() => {
                for (index, (before, after)) in before.iter().zip(after).enumerate() {
                    self.path.push(Step::Index(index));
                    self.compare(before, after);
                    self.path.pop();
                }
            }
            (Json::Num(before), Json::Num(after)) if before == after => {
                if before.token != after.token {
                    self.report(Change::Respelt(before.clone(), after.clone()));
                }
            }
            _ if input == written => {}
            _ => self.report(Change::Replaced(input.clone(), written.clone())),
        }
    }

    fn objects(&mut self, before: &[(String, Json)], after: &[(String, Json)]) {
        for (key, value) in before {
            if !after.iter().any(|(name, _)| name == key) {
                self.path.push(Step::Key(key.clone()));
                self.report(Change::Removed(value.clone()));
                self.path.pop();
            }
        }
        let kept = |of: &[(String, Json)], other: &[(String, Json)]| -> Vec<String> {
            of.iter()
                .filter(|(key, _)| other.iter().any(|(name, _)| name == key))
                .map(|(key, _)| key.clone())
                .collect()
        };
        if kept(before, after) != kept(after, before) {
            self.report(Change::Reordered);
        }
        for (key, value) in after {
            self.path.push(Step::Key(key.clone()));
            match before.iter().find(|(name, _)| name == key) {
                Some((_, input)) => self.compare(input, value),
                None => self.report(Change::Added(value.clone())),
            }
            self.path.pop();
        }
    }
}
