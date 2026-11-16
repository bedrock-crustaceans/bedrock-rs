//! `font/font_metadata.json`: reading the file, the checks of the game's reader, and the
//! game's reading of an entry.

use std::collections::HashSet;
use std::ops::RangeInclusive;

use super::codes;
use super::fields::{Written, real_value};
use super::{
    BUILT_IN_FONTS, FontAlias, FontAliasRef, FontEntry, FontFormat, FontMetadata, FontRange,
    PageSpec,
};
use bedrock_addon_core::diagnostic::builders::invalid;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{CodeDef, Diagnostic, Diagnostics, JsonPath};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::build::array;
use bedrock_addon_core::json::kit::fixed_path::{Authored, as_int, as_real, strict_reader};
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::schema::{
    ElementFailure, FromJson, read_document, read_each, read_iterated_silently, read_list,
};

/// The longest path the game's fixed buffers hold.
const MAX_PATH: usize = 1023;

/// The game ignores a scale that is no JSON real (only a number written as a decimal counts)
/// and resolves an unknown `font_reference` to an invalid font without a content-log line (its
/// "Could not find font alias {}" goes to the developer display at run time), so the crate
/// reports these as informations.
fn silent(at: JsonPath) -> Diagnostic {
    Diagnostic::info(invalid()).at(at)
}

/// The warning of a scale beyond a float, which is held as the largest finite float.
pub(super) fn clamp_warning(at: JsonPath) -> Diagnostic {
    Diagnostic::of(&core_codes::CLAMPED_VALUE).at(at)
}

/// An element of one of the lists, read as `T` when it is an object; any other value is an
/// object without members to the game, so its first member, `first`, is missing (`code`).
fn element<T: FromJson>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    code: &'static CodeDef<0>,
    first: &str,
) -> Option<T> {
    if node.value().is_object() {
        return T::from_json(node, read);
    }
    read.push(Diagnostic::of(code).at(node.path().child(first)));
    None
}

/// `fonts`: iterated without a type test, an entry the game skips left out.
pub(super) fn fonts(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<FontEntry>> {
    read_iterated_silently(node, read, ElementFailure::Drop, |entry, read| {
        element(entry, read, &codes::NO_FORMAT, "font_format")
    })
}

/// `font_aliases`: iterated without a type test, an alias the game drops left out.
pub(super) fn aliases(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<FontAlias>> {
    read_iterated_silently(node, read, ElementFailure::Drop, |alias, read| {
        element(alias, read, &codes::NO_ALIAS, "alias")
    })
}

/// An alias's `fonts`: a non-empty array, given up at its first font the game cannot take.
pub(super) fn alias_fonts(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<FontAliasRef>> {
    let Some(elements) = node.value().as_array() else {
        read.push(Diagnostic::of(&codes::ALIAS_FONTS).at(node.path().clone()));
        return None;
    };
    if elements.is_empty() {
        read.push(Diagnostic::info(invalid()).at(node.path().clone()));
        return None;
    }
    read_list(node, read, ElementFailure::Stop, |font, read| {
        element(font, read, &codes::NO_REFERENCE, "font_reference")
    })
}

/// A path that fits the game's buffer; a longer one is left empty (told by the entry's rule).
pub(super) fn path(node: &Node<'_>, _: &mut Read<'_>) -> Option<String> {
    let text = node.value().as_string()?.as_str();
    (text.len() <= MAX_PATH).then(|| text.to_owned())
}

/// A member the game tests with `isDouble`: a number written as a decimal (a `null` is no value
/// and told nothing). A number beyond a float reads as the largest finite float of its sign, so
/// that no infinity is held (and none fails the writer), told by a finding of its own.
// What the game makes of such a scale is not observable, so it is clamped the same way as
// every other number beyond a float.
pub(super) fn real(node: &Node<'_>, read: &mut Read<'_>) -> Option<f64> {
    if node.value().is_null() {
        return None;
    }
    let Some(number) = as_real(node.value()) else {
        read.push(silent(node.path().clone()));
        return None;
    };
    let limit = f64::from(f32::MAX);
    if (-limit..=limit).contains(&number) {
        return Some(number);
    }
    read.push(clamp_warning(node.path().clone()));
    Some(limit.copysign(number))
}

/// `font_language_code`: a string; another value but `null` is told (and read as the empty
/// code by the font's rule).
pub(super) fn language_code(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    if let Some(code) = node.value().as_string() {
        return Some(code.as_str().to_owned());
    }
    if !node.value().is_null() {
        read.push(Diagnostic::of(&codes::LANGUAGE_CODE).at(node.path().clone()));
    }
    None
}

/// `font_ranges`: an array whose every range has integer bounds (each other one told, and the
/// alias then dropped); a value that is no array is told unless it is `null`, and the font kept
/// without ranges.
pub(super) fn ranges(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<FontRange>> {
    if node.value().is_array() {
        return read_each(node, read, FontRange::from_json);
    }
    if !node.value().is_null() {
        read.push(Diagnostic::of(&codes::RANGES).at(node.path().clone()));
    }
    None
}

/// One range: `first` and `last` integers; its other members are not read.
impl FromJson for FontRange {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let bound = |name: &str| node.value().as_object()?.get(name).and_then(as_int);
        if let (Some(first), Some(last)) = (bound("first"), bound("last")) {
            return Some(Self { first, last });
        }
        read.push(Diagnostic::of(&codes::RANGE_BOUNDS).at(node.path().clone()));
        None
    }
}

bedrock_addon_core::impl_schema_leaf!(FontRange, PageSpec);
bedrock_addon_core::impl_no_molang!(FontRange, PageSpec);

/// One element of `generate_pages_on_start`: a page number, a range of exactly its two bounds,
/// or anything else kept as written (the game skips it, or reads a range with other members
/// beside its bounds as the range).
impl FromJson for PageSpec {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let element = node.value();
        let at = node.path();
        if let Some(page) = as_int(element) {
            return Some(Self::Page(page));
        }
        let Some(object) = element.as_object() else {
            read.push(Diagnostic::of(&codes::PAGES_FORMAT).at(at.clone()));
            return Some(Self::Other(element.clone()));
        };
        let bound = |name: &str| object.get(name).and_then(as_int);
        let (Some(first_page), Some(last_page)) = (bound("first_page"), bound("last_page")) else {
            // A bound that is no integer.
            read.push(Diagnostic::info(invalid()).at(at.clone()));
            return Some(Self::Other(element.clone()));
        };
        if first_page > last_page {
            // The range holds no page.
            read.push(Diagnostic::info(invalid()).at(at.clone()));
        }
        Some(if object.len() == 2 {
            Self::Range {
                first_page,
                last_page,
            }
        } else {
            // A range with other members is kept as written; it reads as the range.
            Self::Other(element.clone())
        })
    }
}

/// The checks the game runs as it binds an entry's members, which the typed or kept members
/// alone do not say, in its order; then `generate_pages_on_start`, read here after them
/// (`schema::after_members` of [`FontEntry`]).
pub(super) fn entry_rules(entry: &mut FontEntry, node: &Node<'_>, read: &mut Read<'_>) {
    let at = node.path();
    let format = entry.format();
    if format.is_none() {
        read.push(Diagnostic::info(invalid()).at(at.child("font_format")));
    }
    // A member kept as written is one the typed field cannot say.
    let kept = |name: &str| entry.extra.get(name);
    if kept("lowPerformanceCompatible").is_some() {
        read.push(Diagnostic::info(invalid()).at(at.child("lowPerformanceCompatible")));
    }
    for name in ["ascii_font_file", "unicode_file_prefix", "font_file"] {
        // A text kept is too long for the game's buffer: the game leaves the path empty.
        if kept(name).is_some_and(Value::is_string) {
            read.push(Diagnostic::info(invalid()).at(at.child(name)));
        }
    }
    match format {
        Some(FontFormat::Bitmap)
            if entry.ascii_font_file.is_none() && entry.unicode_file_prefix.is_none() =>
        {
            read.push(Diagnostic::of(&codes::NO_BITMAP_FILE).at(at.clone()));
        }
        Some(FontFormat::Ttf | FontFormat::TtfMsdf) if entry.font_file.is_none() => {
            read.push(Diagnostic::of(&codes::NO_TTF_FILE).at(at.child("font_file")));
        }
        Some(FontFormat::Msdf) if entry.font_file.is_none() => {
            read.push(Diagnostic::of(&codes::NO_MSDF_FILE).at(at.child("font_file")));
        }
        _ => {}
    }
    let true_type = matches!(format, Some(FontFormat::Ttf | FontFormat::TtfMsdf));
    if true_type && kept("version").is_some() {
        read.push(Diagnostic::info(invalid()).at(at.child("version")));
    }
    if format == Some(FontFormat::Ttf)
        && (kept("target_font_render_size").is_some()
            || entry
                .target_font_render_size
                .is_some_and(|size| !(1..=255).contains(&size)))
    {
        read.push(Diagnostic::info(invalid()).at(at.child("target_font_render_size")));
    }
    let pages = read
        .object(node)
        .and_then(|members| members.get("generate_pages_on_start"));
    if let Some(pages) = pages {
        if pages.value().is_array() {
            entry.generate_pages_on_start = read_each(&pages, read, PageSpec::from_json);
        } else {
            // The whole member is ignored.
            entry.generate_pages_on_start = None;
            read.push(Diagnostic::info(invalid()).at(pages.path().clone()));
            let kept = pages.value().clone();
            entry
                .extra
                .insert("generate_pages_on_start".to_owned(), kept);
        }
    }
}

/// A code of another type than a string (`null` aside) is the empty code to the game
/// (`schema::after_read` of [`FontAliasRef`]).
pub(super) fn reference_rules(font: &mut FontAliasRef, _: &Node<'_>, _: &mut Read<'_>) {
    let coerced = font
        .extra
        .get("font_language_code")
        .is_some_and(|code| !code.is_null());
    if font.font_language_code.is_none() && coerced {
        font.font_language_code = Some(String::new());
    }
}

/// A font name written twice, told at the later entry, after both lists are read and before the
/// unknown members of the file (`schema::after_members` of [`FontMetadata`]).
pub(super) fn font_names(metadata: &mut FontMetadata, node: &Node<'_>, read: &mut Read<'_>) {
    let fonts = node.path().child("fonts");
    let mut names: HashSet<&str> = HashSet::new();
    for (i, font) in metadata.fonts.iter().flatten().enumerate() {
        if !names.insert(&font.font_name) {
            // Named at the entry's place in the file.
            let at = fonts.index(i).child("font_name");
            read.push(Diagnostic::of(&core_codes::FONT_NAME_OVERRIDDEN).at(at));
        }
    }
}

/// The alias checks, in the order the game registers the aliases, after the unknown members of
/// the file; `version` is the file's, no member the value keeps (`schema::after_read` of
/// [`FontMetadata`]).
pub(super) fn alias_names(metadata: &mut FontMetadata, node: &Node<'_>, read: &mut Read<'_>) {
    metadata.extra.remove("version");
    let aliases_at = node.path().child("font_aliases");
    let mut names: HashSet<&str> = metadata
        .fonts
        .iter()
        .flatten()
        .map(|font| font.font_name.as_str())
        .collect();
    for (i, alias) in metadata.font_aliases.iter().flatten().enumerate() {
        let at = aliases_at.index(i);
        for (j, font) in alias.fonts.iter().enumerate() {
            let name = font.font_reference.as_str();
            if !names.contains(name) && !BUILT_IN_FONTS.contains(&name) {
                // The reference resolves to no font.
                read.push(silent(at.child("fonts").index(j).child("font_reference")));
            }
        }
        if !names.insert(&alias.alias) {
            read.push(Diagnostic::of(&core_codes::FONT_NAME_OVERRIDDEN).at(at.child("alias")));
        }
    }
}

impl FontFormat {
    /// The format a `font_format` names, compared byte for byte.
    // The game tries `bitmap`, `ttf` and `ttfmsdf` in turn, then an exact `msdf`.
    #[must_use]
    pub fn from_name(font_format: &str) -> Option<Self> {
        match font_format {
            "bitmap" => Some(Self::Bitmap),
            "ttf" => Some(Self::Ttf),
            "ttfmsdf" => Some(Self::TtfMsdf),
            "msdf" => Some(Self::Msdf),
            _ => None,
        }
    }

    /// The `font_format` text of the format; [`FontFormat::from_name`] reads it back.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bitmap => "bitmap",
            Self::Ttf => "ttf",
            Self::TtfMsdf => "ttfmsdf",
            Self::Msdf => "msdf",
        }
    }

    /// The extensions the game tries for a TrueType `font_file`, in order; none for the other
    /// formats.
    #[must_use]
    pub fn file_extensions(self) -> &'static [&'static str] {
        match self {
            Self::Ttf => &[".ttf", ".otf"],
            Self::TtfMsdf => &[".ttf", ".otf", ".ttc", ".otc"],
            Self::Bitmap | Self::Msdf => &[],
        }
    }
}

/// A member that is no boolean read with a default of `true`: a number is true unless 0, a
/// string, array or object unless empty, `null` is true.
fn as_bool(value: &Value) -> bool {
    if let Some(number) = value.as_number() {
        number.to_f64_lossy() != 0.0
    } else if let Some(text) = value.as_string() {
        !text.as_str().is_empty()
    } else if let Some(array) = value.as_array() {
        !array.is_empty()
    } else if let Some(object) = value.as_object() {
        !object.is_empty()
    } else {
        true
    }
}

impl FontEntry {
    /// The format `font_format` names; `None` for a text the game knows no format by.
    pub fn format(&self) -> Option<FontFormat> {
        FontFormat::from_name(&self.font_format)
    }

    /// Whether the font is used on a low-performance device (JSON `lowPerformanceCompatible`,
    /// true when absent). A value that is no boolean is converted as the game converts it: a
    /// number is true unless 0, a string unless empty, so `"false"` is true.
    pub fn is_low_performance_compatible(&self) -> bool {
        self.low_performance_compatible.unwrap_or_else(|| {
            self.extra
                .get("lowPerformanceCompatible")
                .is_none_or(as_bool)
        })
    }

    /// The TrueType format version the game uses: `version`, or 1.
    pub fn ttf_version(&self) -> i32 {
        self.version.unwrap_or(1)
    }

    /// The size the game renders a `ttf` font at: `target_font_render_size`, or 64, narrowed
    /// to a byte as the game narrows it (256 is 0, 300 is 44).
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the game's own narrowing"
    )]
    pub fn render_size(&self) -> u8 {
        self.target_font_render_size.unwrap_or(64) as u8
    }

    /// The pages drawn at start, as ranges: one per page number and per range whose first
    /// page is not above its last.
    pub fn start_pages(&self) -> Vec<RangeInclusive<i32>> {
        self.generate_pages_on_start
            .iter()
            .flatten()
            .filter_map(|spec| {
                let (first, last) = match spec {
                    PageSpec::Page(page) => (*page, *page),
                    PageSpec::Range {
                        first_page,
                        last_page,
                    } => (*first_page, *last_page),
                    // A range written with other members beside its bounds.
                    PageSpec::Other(written) => {
                        let bound = |name: &str| written.as_object()?.get(name).and_then(as_int);
                        (bound("first_page")?, bound("last_page")?)
                    }
                };
                (first <= last).then_some(first..=last)
            })
            .collect()
    }

    fn to_value(&self) -> Value {
        let text = |text: &Option<String>| text.as_deref().map(Value::from);
        let mut object = Written::default();
        object.put("font_format", Some(Value::from(self.font_format.as_str())));
        object.put("font_name", Some(Value::from(self.font_name.as_str())));
        object.put("version", self.version.map(Value::from));
        object.put("font_file", text(&self.font_file));
        object.put("ascii_font_file", text(&self.ascii_font_file));
        object.put("unicode_file_prefix", text(&self.unicode_file_prefix));
        object.put(
            "target_font_render_size",
            self.target_font_render_size.map(Value::from),
        );
        object.put(
            "lowPerformanceCompatible",
            self.low_performance_compatible.map(Value::from),
        );
        object.put(
            "generate_pages_on_start",
            self.generate_pages_on_start
                .as_ref()
                .map(|pages| array(pages.iter().map(PageSpec::to_value))),
        );
        object.finish(&self.extra)
    }
}

impl PageSpec {
    fn to_value(&self) -> Value {
        match self {
            Self::Page(page) => Value::from(*page),
            Self::Range {
                first_page,
                last_page,
            } => {
                let mut object = Written::default();
                object.put("first_page", Some(Value::from(*first_page)));
                object.put("last_page", Some(Value::from(*last_page)));
                object.finish(&bedrock_addon_core::document::UnknownMembers::default())
            }
            Self::Other(written) => written.clone(),
        }
    }
}

impl FontAliasRef {
    fn to_value(&self) -> Value {
        let mut object = Written::default();
        object.put(
            "font_reference",
            Some(Value::from(self.font_reference.as_str())),
        );
        // A code the game read as "" from another type stays as written.
        let coerced = self.font_language_code.as_deref() == Some("")
            && self.extra.get("font_language_code").is_some();
        if !coerced {
            object.put(
                "font_language_code",
                self.font_language_code.as_deref().map(Value::from),
            );
        }
        object.put("font_scale", self.font_scale.map(real_value));
        object.put(
            "font_ranges",
            self.font_ranges.as_ref().map(|ranges| {
                array(ranges.iter().map(|range| {
                    let mut object = Written::default();
                    object.put("first", Some(Value::from(range.first)));
                    object.put("last", Some(Value::from(range.last)));
                    object.finish(&bedrock_addon_core::document::UnknownMembers::default())
                }))
            }),
        );
        object.finish(&self.extra)
    }
}

impl FontAlias {
    fn to_value(&self) -> Value {
        let mut object = Written::default();
        object.put("alias", Some(Value::from(self.alias.as_str())));
        object.put(
            "font_scale_factor_override",
            self.font_scale_factor_override.map(real_value),
        );
        object.put(
            "fonts",
            Some(array(self.fonts.iter().map(FontAliasRef::to_value))),
        );
        object.finish(&self.extra)
    }
}

impl FontMetadata {
    /// Whether the game reads a file with this root: only when `version` is the integer 1.
    pub(crate) fn version_is_supported(root: &Value) -> bool {
        root.as_object()
            .and_then(|object| object.get("version"))
            .and_then(as_int)
            == Some(1)
    }
}

impl Authored for FontMetadata {
    /// Reads the root of a file whose version is supported.
    fn read(root: &Value, diagnostics: &mut Diagnostics) -> Self {
        let mut read = strict_reader(diagnostics);
        read_document(&Node::new(root, JsonPath::new()), &mut read).unwrap_or_default()
    }

    /// The lists as the JSON object they are written as, with `"version": 1`.
    fn fields_value(&self) -> Value {
        let mut object = Written::default();
        object.put("version", Some(Value::from(1)));
        object.put(
            "fonts",
            self.fonts
                .as_ref()
                .map(|fonts| array(fonts.iter().map(FontEntry::to_value))),
        );
        object.put(
            "font_aliases",
            self.font_aliases
                .as_ref()
                .map(|aliases| array(aliases.iter().map(FontAlias::to_value))),
        );
        object.finish(&self.extra)
    }

    fn written_mut(&mut self) -> &mut Option<Value> {
        &mut self.written
    }

    /// A scale beyond a float is held, and written, as the largest float of its sign.
    fn recorded_rewrites(root: &mut Value) {
        fn clamp(object: &mut Value, name: &str) {
            let Some(member) = object.as_object_mut().and_then(|o| o.get_mut(name)) else {
                return;
            };
            let limit = f64::from(f32::MAX);
            if let Some(real) = as_real(member).filter(|real| !(-limit..=limit).contains(real)) {
                *member = real_value(limit.copysign(real));
            }
        }
        /// What the game iterates of a list: an array's elements, an object's member values.
        fn each(list: &mut Value, mut visit: impl FnMut(&mut Value)) {
            if let Some(array) = list.as_array_mut() {
                array.iter_mut().for_each(&mut visit);
            } else if let Some(object) = list.as_object_mut() {
                object.values_mut().for_each(&mut visit);
            }
        }
        let Some(aliases) = root.as_object_mut().and_then(|o| o.get_mut("font_aliases")) else {
            return;
        };
        each(aliases, |alias| {
            clamp(alias, "font_scale_factor_override");
            if let Some(fonts) = alias.as_object_mut().and_then(|o| o.get_mut("fonts")) {
                // Only an array of fonts is read.
                if fonts.is_array() {
                    each(fonts, |font| clamp(font, "font_scale"));
                }
            }
        });
    }
}

impl FontMetadata {
    /// The font the name `name` ends up with among `fonts`: the last entry of that name.
    pub fn font(&self, name: &str) -> Option<&FontEntry> {
        self.fonts
            .iter()
            .flatten()
            .rfind(|font| font.font_name == name)
    }

    /// The alias the name `name` ends up with: the last entry of that name.
    pub fn alias(&self, name: &str) -> Option<&FontAlias> {
        self.font_aliases
            .iter()
            .flatten()
            .rfind(|alias| alias.alias == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::json::jsoncpp::read;
    use bedrock_addon_core::test_cost::assert_linear_at;

    /// Every format reads back from its own `font_format` text, and nothing else names one.
    #[test]
    fn a_font_format_reads_back_from_its_text() {
        for format in [
            FontFormat::Bitmap,
            FontFormat::Ttf,
            FontFormat::TtfMsdf,
            FontFormat::Msdf,
        ] {
            assert_eq!(FontFormat::from_name(format.as_str()), Some(format));
        }
        assert_eq!(FontFormat::from_name("TTF"), None);
    }

    /// A `font_reference` that is missing drops the whole alias entry.
    #[test]
    fn an_alias_font_without_a_reference_is_dropped() {
        let root = read(br#"{ "font_scale": 1.0 }"#).expect("JSON");
        let mut diagnostics = Diagnostics::default();
        let mut reader = strict_reader(&mut diagnostics);
        let node = Node::new(&root, JsonPath::new());
        assert!(read_document::<FontAliasRef>(&node, &mut reader).is_none());
        drop(reader);
        assert_eq!(diagnostics.len(), 1);
    }

    /// The names of a font list are resolved in time linear in its entries: every alias
    /// names a font, an earlier alias and a name nobody has.
    #[test]
    fn font_aliases_are_resolved_in_linear_time() {
        let build = |entries: usize| {
            let fonts: Vec<String> = (0..entries / 2)
                .map(|i| {
                    format!(r#"{{ "font_format": "ttf", "font_name": "t{i}", "font_file": "font/t{i}" }}"#)
                })
                .collect();
            let aliases: Vec<String> = (0..entries / 2)
                .map(|i| {
                    format!(
                        r#"{{ "alias": "a{i}", "fonts": [{{ "font_reference": "t{i}" }}, {{ "font_reference": "a{}" }}, {{ "font_reference": "x{i}" }}] }}"#,
                        i.saturating_sub(1)
                    )
                })
                .collect();
            let text = format!(
                r#"{{ "version": 1, "fonts": [{}], "font_aliases": [{}] }}"#,
                fonts.join(","),
                aliases.join(",")
            );
            read(text.as_bytes()).expect("JSON")
        };
        assert_linear_at("FontMetadata::from_value", 5_000, build, |root| {
            let mut diagnostics = Diagnostics::default();
            let list = FontMetadata::from_value(root, &mut diagnostics);
            let aliases = list.font_aliases.as_ref().map_or(0, Vec::len);
            assert_eq!(list.fonts.as_ref().map(Vec::len), Some(aliases));
            // One finding for each name nobody has, and one for the first alias naming itself.
            assert_eq!(diagnostics.len(), aliases + 1);
        });
    }
}

#[cfg(test)]
mod kept_root_tests {
    use super::*;
    use bedrock_addon_core::json::kit::fixed_path::testing::{
        assert_kept_root_written_linearly, list,
    };

    /// The root kept beside the fields is left out of equality: a loaded value that kept one
    /// equals the value built by hand with the same fields, and is still written as authored.
    #[test]
    fn a_kept_root_is_left_out_of_equality() {
        let root = bedrock_addon_core::json::jsoncpp::read(
            br#"{"version": 1, "fonts": [{"font_format": "ttf", "font_name": "t", "font_file": "font/t"}, 5]}"#,
        )
        .expect("JSON");
        let loaded = FontMetadata::from_value(&root, &mut Diagnostics::default());
        assert_eq!(*loaded.written, Some(root.clone()));
        let built = FontMetadata {
            written: bedrock_addon_core::json::kit::Written::default(),
            ..loaded.clone()
        };
        assert_eq!(loaded, built);
        assert_eq!(loaded.to_value(), root);
        assert_ne!(built.to_value(), root);
    }

    /// A write of a value that keeps its root is linear in its entries, with one entry the
    /// game skips.
    #[test]
    fn a_kept_root_is_written_in_linear_time() {
        assert_kept_root_written_linearly::<FontMetadata>(
            "FontMetadata::to_value",
            2000,
            |entries| {
                let fonts = list(entries, |i| {
                    format!(
                        r#"{{ "font_format": "ttf", "font_name": "t{i}", "font_file": "font/t{i}" }}"#
                    )
                });
                format!(r#"{{ "version": 1, "fonts": [{fonts}, 5] }}"#)
            },
        );
    }
}
