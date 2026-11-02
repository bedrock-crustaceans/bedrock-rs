//! What the fixed-path files of the menu texts, the fonts and the UI images' side files share:
//! reading one copy as vanilla's JSON reader does, the `Json::Value` conversions the readers
//! lean on, the walk of one file at the pack root, and the document of an unversioned file.

use bedrock_addon_core::load::ListedFile;
use bedrock_addon_core::load::{AppDataPlace, ListDepth, StackPack, Walk, WalkOrder};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{Document, Extra, Origin, Rejected, VersionInfo};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle, write_json};
use bedrock_addon_core::version::FormatVersion;

/// The origin of a file read on its own: no pack.
#[must_use]
pub fn lone(path: &PackPath) -> Origin {
    origin(PackRef::app_data(), PackSide::Resource, path)
}

/// The origin of a file of `pack`.
#[must_use]
pub fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    Origin::new(pack, side, path.clone(), None, Vec::new())
}

/// A document of an unversioned file: no `format_version` is read on any path.
pub fn document<T>(value: T, origin: Origin, mut diagnostics: Diagnostics) -> Document<T> {
    locate(&mut diagnostics, &origin);
    let none = FormatVersion::new(0, 0, 0);
    Document::new(
        value,
        origin,
        VersionInfo::new(None, false, none, none, false),
        diagnostics,
    )
}

/// A rejection with one finding.
#[must_use]
pub fn reject(origin: Origin, finding: Diagnostic) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding);
    locate(&mut diagnostics, &origin);
    Rejected::new(origin, diagnostics)
}

/// Gives every finding the document's pack and file.
pub fn locate(diagnostics: &mut Diagnostics, origin: &Origin) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        diagnostics.push(finding);
    }
}

/// A finding at `at` inside a file.
#[must_use]
pub fn finding(code: DiagnosticCode, severity: Severity, at: JsonPath) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Parse,
        Location::new(None, None, at, None),
    )
}

/// A finding about the file `path` of `pack` as a whole.
#[must_use]
pub fn file_finding(
    code: DiagnosticCode,
    severity: Severity,
    pack: &PackRef,
    path: &PackPath,
) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Validate,
        Location::new(
            Some(pack.clone()),
            Some(path.clone()),
            JsonPath::new(),
            None,
        ),
    )
}

/// `InvalidValue` without a vanilla text (a finding of the crate's own).
#[must_use]
pub fn invalid() -> DiagnosticCode {
    DiagnosticCode::InvalidValue { text: None }
}

/// `InvalidValue` carrying vanilla's text.
#[must_use]
pub fn invalid_text(text: &'static str) -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: Some(VanillaText::Fixed(text)),
    }
}

/// A file that is not JSON: `MalformedInput` with the reader's message and, where vanilla
/// says something, its text.
pub fn malformed(message: String, text: Option<&'static str>) -> DiagnosticCode {
    DiagnosticCode::MalformedInput {
        message,
        text: text.map(VanillaText::Fixed),
    }
}

/// `at` with the member `key` appended.
#[must_use]
pub fn key(at: &JsonPath, key: &str) -> JsonPath {
    let mut path = at.clone();
    path.push(key);
    path
}

/// `at` with the element `index` appended.
#[must_use]
pub fn index(at: &JsonPath, index: usize) -> JsonPath {
    let mut path = at.clone();
    path.push_index(index);
    path
}

/// The member `key` of an object, a `null` included; `None` for a missing member and for
/// anything but an object.
#[must_use]
pub fn raw_member<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_object().and_then(|object| object.get(key))
}

/// `Json::Value::asString("")`: a string is its text, a boolean `true` / `false`, and null,
/// numbers, arrays and objects are empty.
#[must_use]
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
#[must_use]
pub fn iterated<'v>(value: &'v Value, at: &JsonPath) -> Vec<(JsonPath, &'v Value)> {
    if let Some(array) = value.as_array() {
        return array
            .iter()
            .enumerate()
            .map(|(i, element)| (index(at, i), element))
            .collect();
    }
    let Some(object) = value.as_object() else {
        return Vec::new();
    };
    let mut members: Vec<(&str, &Value)> = object.iter().map(|(k, v)| (k.as_str(), v)).collect();
    members.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    members
        .into_iter()
        .map(|(name, member)| (key(at, name), member))
        .collect()
}

/// jsoncpp's `isInt` for a member read into an `i32`: a number written without a fraction or
/// an exponent, in `i32`.
#[must_use]
pub fn as_int(value: &Value) -> Option<i32> {
    let number = value.as_number()?;
    if number.is_float() {
        return None;
    }
    number.to_i32()
}

/// jsoncpp's `isDouble`: a number written as a real.
#[must_use]
pub fn as_real(value: &Value) -> Option<f64> {
    let number = value.as_number()?;
    if number.is_float() {
        number.to_f64()
    } else {
        None
    }
}

/// Every member of an object that is none of `read`, in file order, and an `UnrecognizedField`
/// info for each.
pub fn others(value: &Value, read: &[&str], at: &JsonPath, diagnostics: &mut Diagnostics) -> Extra {
    let mut extra = Extra::default();
    if let Some(object) = value.as_object() {
        for (name, member) in object {
            if !read.contains(&name.as_str()) {
                diagnostics.push(finding(
                    DiagnosticCode::UnrecognizedField { text: None },
                    Severity::Info,
                    key(at, name.as_str()),
                ));
                extra.insert(name.as_str().to_owned(), member.clone());
            }
        }
    }
    extra
}

/// An array of values.
pub fn array(values: impl IntoIterator<Item = Value>) -> Value {
    let mut array = facet_value::VArray::new();
    for value in values {
        array.push(value);
    }
    array.into_value()
}

/// An object under construction, members in insertion order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Object(facet_value::VObject);

impl Object {
    /// Adds `key` when `value` is present.
    pub fn put(&mut self, key: &str, value: Option<Value>) {
        if let Some(value) = value {
            self.0.insert(key, value);
        }
    }

    /// Adds the unknown members.
    pub fn extra(&mut self, extra: &Extra) {
        for (key, value) in extra.iter() {
            self.0.insert(key, value.clone());
        }
    }

    /// The object.
    #[must_use]
    pub fn finish(self) -> Value {
        self.0.into_value()
    }
}

/// Writes a value as a file's JSON.
///
/// # Errors
///
/// `EncodeError::NotRepresentable` when `value` holds a non-finite number, or nests deeper than the
/// readers of the crate follow.
pub fn json_text(value: &Value, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
    write_json(value, out, style)
}

/// A value as a file's JSON text.
///
/// # Errors
///
/// `EncodeError::NotRepresentable` when `value` holds a non-finite number, or nests deeper than the
/// readers of the crate follow.
pub fn json_string(value: &Value) -> Result<String, EncodeError> {
    bedrock_addon_core::json::to_json(value)
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

/// One copy of a file as the stack walk hands it over: the file read, or why not.
pub type FileCopy<T> = Result<Document<T>, Rejected>;

/// A walk of the pack root for one fixed file name, lowest pack first, packs only.
pub const fn root_file(accept: fn(&PackPath) -> bool) -> Walk {
    Walk {
        directories: &[""],
        depth: ListDepth::Children,
        accept,
        order: WalkOrder::BottomUp,
        directory_major: false,
        app_data: &[],
        app_data_place: AppDataPlace::After,
        in_pack: None,
        shadowed: None,
        read_error: None,
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
#[doc(hidden)]
pub mod testing {
    use super::{Authored, Diagnostics};
    use crate::jsoncpp::read;
    use bedrock_addon_core::test_cost::assert_linear_at;

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
