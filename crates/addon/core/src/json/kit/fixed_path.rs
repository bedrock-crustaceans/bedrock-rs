//! What the fixed-path files of the menu texts, the fonts and the UI images' side files share
//! beyond the reader kit: the read the kit reads them under ([`reader`]), the `Json::Value`
//! conversions the readers lean on, the kept root ([`Authored`]) and the files of a pack below
//! a directory. Their loaders are core's (`load::merged`), their documents core's
//! `document::unversioned`; their findings' builders are `diagnostic::builders`, their writers'
//! objects `json::build`.

use super::{Disposition, Members, Read};
use crate::diagnostic::{Diagnostics, JsonPath, Severity};
use crate::document::UnknownMembers;
use crate::json::{Coerce, Coercing, IsIntRule, Value, WideInt};
use crate::load::{ListedFile, StackPack};
use crate::versioning::{MemberFailure, Reading};

/// The member `key` of an object, a `null` included; `None` for a missing member and for
/// anything but an object.
pub fn raw_member<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_object().and_then(|object| object.get(key))
}

/// `Json::Value::asString("")`: a string is its text, a boolean `true` / `false`, and null,
/// numbers, arrays and objects are empty.
pub fn to_text(value: &Value) -> String {
    if let Some(text) = value.as_string() {
        text.as_str().to_owned()
    } else if let Some(flag) = value.as_bool() {
        flag.to_string()
    } else {
        String::new()
    }
}

/// What `begin()` / `end()` iterate of a value, each with its path below `at`: the elements of
/// an array, the member values of an object in key order (jsoncpp's objects are a `std::map`),
/// nothing for a scalar.
pub fn iterated<'v>(value: &'v Value, at: &JsonPath) -> Vec<(JsonPath, &'v Value)> {
    if let Some(array) = value.as_array() {
        return array
            .iter()
            .enumerate()
            .map(|(i, element)| (at.index(i), element))
            .collect();
    }
    let Some(object) = value.as_object() else {
        return Vec::new();
    };
    let mut members: Vec<(&str, &Value)> = object.iter().map(|(k, v)| (k.as_str(), v)).collect();
    members.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    members
        .into_iter()
        .map(|(name, member)| (at.child(name), member))
        .collect()
}

/// jsoncpp's `isInt` for a member read into an `i32`: a number written without a fraction or
/// an exponent, in `i32` ([`IsIntRule::Token`]).
pub fn as_int(value: &Value) -> Option<i32> {
    value.is_int(Coercing::new(IsIntRule::Token, WideInt::LowWord))
}

/// jsoncpp's `isDouble`: a number written as a real.
pub fn as_real(value: &Value) -> Option<f64> {
    let number = value.as_number()?;
    if number.is_float() {
        number.to_f64()
    } else {
        None
    }
}

/// The read of a fixed-path file's root: the game's reader of these files runs its checks
/// without a content-log line, so every finding of the reader kit (a wrong type, a missing
/// member, an unknown member) is an information, and none fails what is read.
pub fn reader(findings: &mut Diagnostics) -> Read<'_> {
    Read::new(
        Reading::OWN.with_on_member_failure(MemberFailure::DefaultAndTell),
        findings,
    )
    .unknown_severity(Severity::Info)
}

/// Every member of `members` that is none of `known`, in file order, each told as an
/// `UnrecognizedField` information by `read` and kept beside the fields.
pub fn kept_unknown(read: &mut Read<'_>, members: &Members<'_>, known: &[&str]) -> UnknownMembers {
    read.unknown(members, known, Disposition::Keep)
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.clone()))
        .collect()
}

/// A file's value that keeps its root as the author wrote it beside its fields, wherever the
/// fields cannot say it: the fields are the game's reading of the file, and write-back is the
/// file itself, so nothing the author wrote is lost or normalised in silence.
pub trait Authored: Sized + PartialEq {
    /// The game's reading of a root, with nothing kept beside it.
    fn read(root: &Value, diagnostics: &mut Diagnostics) -> Self;

    /// What the fields alone are written as.
    fn fields_value(&self) -> Value;

    /// The root kept as written.
    fn written_mut(&mut self) -> &mut Option<Value>;

    /// Applies to a root about to be kept the rewrites the load reported (a number
    /// clamped, for example); none unless a type says so.
    fn recorded_rewrites(_root: &mut Value) {}

    /// Reads a root, and keeps it as written when the fields would not write it back.
    fn from_value(root: &Value, diagnostics: &mut Diagnostics) -> Self {
        let mut value = Self::read(root, diagnostics);
        let fields = value.fields_value();
        if fields != *root {
            let mut kept = root.clone();
            Self::recorded_rewrites(&mut kept);
            if kept != fields {
                *value.written_mut() = Some(kept);
            }
        }
        value
    }

    /// The file as it is written: the root as the author wrote it while the fields are still
    /// the game's reading of it, else the fields.
    fn to_value(&self) -> Value
    where
        Self: Clone,
    {
        let mut fields = self.clone();
        if let Some(written) = fields.written_mut().take()
            && Self::from_value(&written, &mut Diagnostics::default()) == *self
        {
            return written;
        }
        self.fields_value()
    }
}

/// The files of `pack` below the directory `directory`.
pub fn below<'a>(pack: &'a StackPack, directory: &'a str) -> impl Iterator<Item = &'a ListedFile> {
    let files = pack.files();
    // Keys below the directory are the ones from `<directory>/` on: a sibling `<directory>.x`
    // sorts before them.
    let first = [directory.as_bytes(), b"/"].concat();
    let start = files.partition_point(|file| file.key.as_str().as_bytes() < first.as_slice());
    files[start..].iter().take_while(move |file| {
        file.key
            .as_str()
            .strip_prefix(directory)
            .is_some_and(|rest| rest.starts_with('/'))
    })
}

/// What a family's test needs to measure that writing a value that keeps its root is linear.
/// It measures with [`crate::test_cost`], so it is there under the same `test-cost` feature.
// hidden: test support: what a family's test measures a linear write with
#[cfg(any(test, feature = "test-cost"))]
#[doc(hidden)]
pub mod testing {
    use super::{Authored, Diagnostics};
    use crate::json::jsoncpp::read;
    use crate::test_cost::assert_linear_at;

    /// A value read from a root that its fields do not write back: the root is kept beside
    /// them, which is when a write reads it again.
    fn kept<T: Authored + Clone>(text: &str) -> T {
        let root = read(text.as_bytes()).expect("JSON");
        let value = T::from_value(&root, &mut Diagnostics::default());
        assert!(
            value.clone().written_mut().is_some(),
            "the root is not kept beside the fields"
        );
        value
    }

    /// `entries` elements from `element`, joined by commas.
    pub fn list(entries: usize, element: impl Fn(usize) -> String) -> String {
        (0..entries).map(element).collect::<Vec<_>>().join(",")
    }

    /// A write of a value that keeps its root clones the value, reads the kept root again and
    /// compares the two: three passes over the file, each linear in its entries. The re-read
    /// is what keeps "written as authored while the fields are still the game's reading" true
    /// without a dirty flag on types with public fields. One measurement per type, each on a
    /// root with one entry the game skips.
    pub fn assert_kept_root_written_linearly<T: Authored + Clone>(
        what: &str,
        n: usize,
        text: impl Fn(usize) -> String,
    ) {
        assert_linear_at(
            what,
            n,
            |entries| kept::<T>(&text(entries)),
            |value| {
                let root = value.to_value();
                assert_eq!(Some(&root), value.clone().written_mut().as_ref());
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::codes as core_codes;
    use crate::diagnostic::{Diagnostic, DiagnosticCode, Location, Phase};
    use crate::json::kit::Node;

    fn json(text: &str) -> Value {
        crate::json::from_str_jsonc::<Value>(text).unwrap()
    }

    /// The reader of the fixed-path files tells every finding of the kit as an information, at
    /// the member, and fails nothing.
    #[test]
    fn the_fixed_path_reader_tells_information() {
        let at = || JsonPath::of_keys(["a", "b"]);
        let json = json(r#"{"a": 1, "b": [2]}"#);
        let mut found = Diagnostics::default();
        let mut read = reader(&mut found);
        let members = read.object(&Node::new(&json, at())).unwrap();
        let kept = kept_unknown(&mut read, &members, &["a"]);
        let a = members.get("a").unwrap();
        assert_eq!(read.string(&a), None);
        let missing = read.required(&members, "c");
        assert!(missing.is_none());
        assert!(!read.failed());
        assert_eq!(kept.get("b"), Some(&json.as_object().unwrap()["b"]));
        let info = |code, at: JsonPath| {
            Diagnostic::new(
                code,
                Severity::Info,
                Phase::Parse,
                Location::new(None, None, at, None),
            )
        };
        let found: Vec<Diagnostic> = found.iter().cloned().collect();
        let child = |key: &str| {
            let mut at = at();
            at.push(key);
            at
        };
        assert_eq!(
            found,
            [
                info(
                    DiagnosticCode::from(&core_codes::UNRECOGNIZED_FIELD),
                    child("b")
                ),
                info(DiagnosticCode::from(&core_codes::INVALID_TYPE), child("a")),
                info(
                    DiagnosticCode::from(&core_codes::MISSING_REQUIRED_FIELD),
                    child("c")
                ),
            ]
        );
    }

    #[test]
    fn the_fixed_path_is_int_is_jsoncpp_s_token_rule() {
        let read = |text: &str| as_int(&json(text));
        assert_eq!(read("7"), Some(7));
        assert_eq!(read("-2147483648"), Some(i32::MIN));
        assert_eq!(read("2147483648"), None);
        assert_eq!(read("2.0"), None, "a real token is never an int");
        assert_eq!(read("1e2"), None);
        assert_eq!(read("true"), None);
        assert_eq!(read("\"7\""), None);
        assert_eq!(read("null"), None);
    }

    /// `below` and `StackPack::files_below` list the same run for a directory, and differ for
    /// `""`: `below` lists nothing there, `files_below` every file. Kept apart (a font root
    /// computed as `""` would read differently).
    #[test]
    fn below_is_files_below_but_for_the_empty_directory() {
        use crate::load::LoadContext;
        use crate::pack_identity::{PackOrigin, PackPath, PackRef};
        let listed = |key: &'static str| ListedFile {
            path: PackPath::from_static(key),
            key: PackPath::from_static(key),
        };
        let pack = StackPack::new(
            PackRef::app_data(),
            PackOrigin::Dev,
            LoadContext::detached(),
        )
        .with_files(
            [
                "font",
                "font.x",
                "font/a.json",
                "font/b/c.png",
                "fonts/d",
                "z",
            ]
            .map(listed),
        );
        for directory in ["font", "font/b", "fonts", "none"] {
            let below: Vec<&ListedFile> = below(&pack, directory).collect();
            let files_below: Vec<&ListedFile> = pack.files_below(directory).iter().collect();
            assert_eq!(below, files_below, "{directory}");
        }
        assert_eq!(below(&pack, "font").count(), 2);
        assert_eq!(below(&pack, "").count(), 0);
        assert_eq!(pack.files_below("").len(), 6);
    }
}
