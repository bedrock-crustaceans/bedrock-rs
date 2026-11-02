//! `font/font_metadata.json`: reading the file, the checks of the game's reader, and the
//! game's reading of an entry.

use std::collections::HashSet;
use std::ops::RangeInclusive;

use super::fields::{Fields, Got, Written, real_value};
use super::{
    BUILT_IN_FONTS, FontAlias, FontAliasRef, FontEntry, FontFormat, FontMetadata, FontRange,
    PageSpec,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::json::Value;
use bedrock_addon_support::fixed_path::{
    Authored, array, as_int, as_real, finding, invalid, invalid_text, iterated, key,
};

// The reader's texts, as the game writes them.
/// A file that is not JSON.
pub(super) const NOT_JSON: &str = "Couldn't parse font_metadata.json";
/// A `version` that is not the integer 1.
pub(super) const VERSION: &str = "Unsupported version number.";
const NO_FORMAT: &str = "Font is missing font_format.";
const NO_NAME: &str = "Font is missing font_name.";
const NO_BITMAP_FILE: &str =
    "Bitmap font must have either an ascii_font_file or a unicode_file_prefix specified";
const NO_TTF_FILE: &str = "TTF is missing font_file.";
const NO_MSDF_FILE: &str = "MSDF is missing font_file.";
const PAGES_FORMAT: &str = "Invalid format for the generate_pages_on_start json object.";
const NO_ALIAS: &str = "Font Alias missing alias.";
const ALIAS_FONTS: &str = "Font Alias property fonts must be an array.";
const NO_REFERENCE: &str = "Font alias missing font_reference";
const LANGUAGE_CODE: &str = "Font alias font_language_code must be a string";
const RANGES: &str = "Font alias font_ranges must be an array";
const RANGE_BOUNDS: &str = "Font alias font_ranges first and last values must be integers.";

/// The longest path the game's fixed buffers hold.
const MAX_PATH: usize = 1023;

fn missing(text: &'static str, at: JsonPath) -> Diagnostic {
    let code = DiagnosticCode::MissingRequiredField {
        text: Some(VanillaText::Fixed(text)),
    };
    finding(code, Severity::Info, at)
}

fn wrong_type(text: &'static str, at: JsonPath) -> Diagnostic {
    let code = DiagnosticCode::InvalidType {
        text: Some(VanillaText::Fixed(text)),
    };
    finding(code, Severity::Info, at)
}

fn warn(at: JsonPath) -> Diagnostic {
    finding(invalid(), Severity::Info, at)
}

/// The game ignores a scale that is no JSON real (only a number written as a decimal counts)
/// and resolves an unknown `font_reference` to an invalid font without a content-log line (its
/// "Could not find font alias {}" goes to the developer display at run time), so the crate
/// reports these as informations.
fn silent(at: JsonPath) -> Diagnostic {
    finding(invalid(), Severity::Info, at)
}

/// The warning of a scale beyond a float, which is held as the largest finite float.
pub(super) fn clamp_warning(at: JsonPath) -> Diagnostic {
    finding(
        DiagnosticCode::ClampedValue { text: None },
        Severity::Info,
        at,
    )
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

    /// Reads one element of `fonts`; `None` for an entry the game skips.
    fn read(element: &Value, at: &JsonPath, diagnostics: &mut Diagnostics) -> Option<Self> {
        let mut fields = Fields::new(element, at.clone());
        let Some(font_format) = fields.string("font_format").into_typed() else {
            diagnostics.push(missing(NO_FORMAT, fields.path("font_format")));
            return None;
        };
        let Some(font_name) = fields.string("font_name").into_typed() else {
            diagnostics.push(missing(NO_NAME, fields.path("font_name")));
            return None;
        };
        let format = FontFormat::from_name(&font_format);
        if format.is_none() {
            diagnostics.push(warn(fields.path("font_format")));
        }
        let low = fields.get("lowPerformanceCompatible", Value::as_bool);
        if low.is_other(true) {
            diagnostics.push(warn(fields.path("lowPerformanceCompatible")));
        }
        let mut path = |fields: &mut Fields<'_>, name: &'static str| {
            let text = fields.string(name).into_typed()?;
            if text.len() > MAX_PATH {
                // The game leaves the path empty.
                diagnostics.push(warn(fields.path(name)));
                fields.keep(name);
                return None;
            }
            Some(text)
        };
        let ascii_font_file = path(&mut fields, "ascii_font_file");
        let unicode_file_prefix = path(&mut fields, "unicode_file_prefix");
        let font_file = path(&mut fields, "font_file");
        match format {
            Some(FontFormat::Bitmap)
                if ascii_font_file.is_none() && unicode_file_prefix.is_none() =>
            {
                diagnostics.push(missing(NO_BITMAP_FILE, at.clone()));
            }
            Some(FontFormat::Ttf | FontFormat::TtfMsdf) if font_file.is_none() => {
                diagnostics.push(missing(NO_TTF_FILE, fields.path("font_file")));
            }
            Some(FontFormat::Msdf) if font_file.is_none() => {
                diagnostics.push(missing(NO_MSDF_FILE, fields.path("font_file")));
            }
            _ => {}
        }
        let true_type = matches!(format, Some(FontFormat::Ttf | FontFormat::TtfMsdf));
        let version = fields.int("version");
        if true_type && version.is_other(true) {
            diagnostics.push(warn(fields.path("version")));
        }
        let size = fields.int("target_font_render_size");
        if format == Some(FontFormat::Ttf) {
            match &size {
                Got::Other(_) => diagnostics.push(warn(fields.path("target_font_render_size"))),
                Got::Typed(size) if !(1..=255).contains(size) => {
                    diagnostics.push(warn(fields.path("target_font_render_size")));
                }
                _ => {}
            }
        }
        let generate_pages_on_start = read_pages(&mut fields, diagnostics);
        Some(Self {
            font_format,
            font_name,
            low_performance_compatible: low.into_typed(),
            generate_pages_on_start,
            ascii_font_file,
            unicode_file_prefix,
            font_file,
            version: version.into_typed(),
            target_font_render_size: size.into_typed(),
            extra: fields.finish(diagnostics),
        })
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
                object.finish(&bedrock_addon_core::document::Extra::default())
            }
            Self::Other(written) => written.clone(),
        }
    }
}

/// The `generate_pages_on_start` member.
fn read_pages(fields: &mut Fields<'_>, diagnostics: &mut Diagnostics) -> Option<Vec<PageSpec>> {
    let at = fields.path("generate_pages_on_start");
    let pages = fields.get("generate_pages_on_start", |member| {
        member.as_array().map(|elements| {
            elements
                .iter()
                .enumerate()
                .map(|(i, element)| {
                    let at = bedrock_addon_support::fixed_path::index(&at, i);
                    read_page(element, at, diagnostics)
                })
                .collect::<Vec<_>>()
        })
    });
    if pages.is_other(true) {
        // The whole member is ignored.
        diagnostics.push(warn(at));
    }
    pages.into_typed()
}

fn read_page(element: &Value, at: JsonPath, diagnostics: &mut Diagnostics) -> PageSpec {
    if let Some(page) = as_int(element) {
        return PageSpec::Page(page);
    }
    if !element.is_object() {
        diagnostics.push(finding(invalid_text(PAGES_FORMAT), Severity::Info, at));
        return PageSpec::Other(element.clone());
    }
    let bound = |name: &str| element.as_object()?.get(name).and_then(as_int);
    let only_bounds = element.as_object().is_some_and(|object| object.len() == 2);
    match (bound("first_page"), bound("last_page")) {
        (Some(first_page), Some(last_page)) if only_bounds => {
            if first_page > last_page {
                // The range holds no page.
                diagnostics.push(warn(at));
            }
            PageSpec::Range {
                first_page,
                last_page,
            }
        }
        (Some(first_page), Some(last_page)) => {
            // A range with other members is kept as written; it reads as the range.
            if first_page > last_page {
                diagnostics.push(warn(at));
            }
            PageSpec::Other(element.clone())
        }
        _ => {
            // A bound that is no integer.
            diagnostics.push(warn(at));
            PageSpec::Other(element.clone())
        }
    }
}

impl FontAliasRef {
    /// Reads one element of an alias's `fonts`; `None` when the game drops the whole alias
    /// entry.
    fn read(element: &Value, at: &JsonPath, diagnostics: &mut Diagnostics) -> Option<Self> {
        let mut fields = Fields::new(element, at.clone());
        let Some(font_reference) = fields.string("font_reference").into_typed() else {
            diagnostics.push(missing(NO_REFERENCE, fields.path("font_reference")));
            return None;
        };
        let ranges_at = fields.path("font_ranges");
        let mut bad_bound = false;
        let ranges = fields.get("font_ranges", |member| {
            let elements = member.as_array()?;
            let mut ranges = Vec::with_capacity(elements.len());
            for (i, element) in elements.iter().enumerate() {
                let bound = |name: &str| element.as_object()?.get(name).and_then(as_int);
                if let (Some(first), Some(last)) = (bound("first"), bound("last")) {
                    ranges.push(FontRange { first, last });
                } else {
                    let at = bedrock_addon_support::fixed_path::index(&ranges_at, i);
                    diagnostics.push(wrong_type(RANGE_BOUNDS, at));
                    bad_bound = true;
                }
            }
            Some(ranges)
        });
        if bad_bound {
            return None;
        }
        if ranges.is_other(false) {
            // The font is kept without ranges.
            diagnostics.push(wrong_type(RANGES, ranges_at));
        }
        let code = fields.string("font_language_code");
        let coerced = code.is_other(false);
        if coerced {
            // The font is kept with the code "".
            diagnostics.push(wrong_type(LANGUAGE_CODE, fields.path("font_language_code")));
        }
        let (scale, clamped) = fields.real("font_scale");
        if clamped {
            // A number beyond a float is held at the float's width and told by a finding of its
            // own, which the information for a scale that is no decimal is not.
            diagnostics.push(clamp_warning(fields.path("font_scale")));
        } else if scale.is_other(false) {
            diagnostics.push(silent(fields.path("font_scale")));
        }
        Some(Self {
            font_reference,
            font_language_code: code.into_typed().or(coerced.then(String::new)),
            font_ranges: ranges.into_typed(),
            font_scale: scale.into_typed(),
            extra: fields.finish(diagnostics),
        })
    }

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
                    object.finish(&bedrock_addon_core::document::Extra::default())
                }))
            }),
        );
        object.finish(&self.extra)
    }
}

impl FontAlias {
    /// Reads one element of `font_aliases`; `None` for an alias the game drops.
    fn read(element: &Value, at: &JsonPath, diagnostics: &mut Diagnostics) -> Option<Self> {
        let mut fields = Fields::new(element, at.clone());
        let Some(alias) = fields.string("alias").into_typed() else {
            diagnostics.push(missing(NO_ALIAS, fields.path("alias")));
            return None;
        };
        let fonts_at = fields.path("fonts");
        let Some(elements) = fields.raw("fonts").and_then(Value::as_array) else {
            diagnostics.push(wrong_type(ALIAS_FONTS, fonts_at));
            return None;
        };
        if elements.is_empty() {
            diagnostics.push(warn(fonts_at));
            return None;
        }
        let mut fonts = Vec::with_capacity(elements.len());
        for (i, element) in elements.iter().enumerate() {
            let at = bedrock_addon_support::fixed_path::index(&fonts_at, i);
            fonts.push(FontAliasRef::read(element, &at, diagnostics)?);
        }
        let (scale, clamped) = fields.real("font_scale_factor_override");
        if clamped {
            diagnostics.push(clamp_warning(fields.path("font_scale_factor_override")));
        } else if scale.is_other(false) {
            diagnostics.push(silent(fields.path("font_scale_factor_override")));
        }
        Some(Self {
            alias,
            fonts,
            font_scale_factor_override: scale.into_typed(),
            extra: fields.finish(diagnostics),
        })
    }

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
        let mut fields = Fields::new(root, JsonPath::new());
        fields.raw("version");
        // Both lists are iterated without a type test: an object gives its member values.
        // Each entry held with its place in the file.
        let fonts = fields.raw("fonts").map(|member| {
            iterated(member, &key(&fields.at, "fonts"))
                .into_iter()
                .filter_map(|(at, element)| {
                    let font = FontEntry::read(element, &at, diagnostics)?;
                    Some((at, font))
                })
                .collect::<Vec<_>>()
        });
        let font_aliases = fields.raw("font_aliases").map(|member| {
            iterated(member, &key(&fields.at, "font_aliases"))
                .into_iter()
                .filter_map(|(at, element)| {
                    let alias = FontAlias::read(element, &at, diagnostics)?;
                    Some((at, alias))
                })
                .collect::<Vec<_>>()
        });
        let mut names: HashSet<String> = HashSet::new();
        for (at, font) in fonts.iter().flatten() {
            if !names.insert(font.font_name.clone()) {
                // A repeated font name, reported at the entry's place in the file.
                diagnostics.push(finding(
                    DiagnosticCode::FontNameOverridden,
                    Severity::Info,
                    key(at, "font_name"),
                ));
            }
        }
        let metadata = Self {
            fonts: fonts.map(|fonts| fonts.into_iter().map(|(_, font)| font).collect()),
            font_aliases: None,
            extra: fields.finish(diagnostics),
            written: None,
        };
        // The alias checks run in the order the game registers the aliases.
        let mut aliases = Vec::new();
        if let Some(read) = &font_aliases {
            for (at, alias) in read {
                for (i, font) in alias.fonts.iter().enumerate() {
                    let name = font.font_reference.as_str();
                    if !names.contains(name) && !BUILT_IN_FONTS.contains(&name) {
                        // The reference resolves to no font.
                        let at = bedrock_addon_support::fixed_path::index(&key(at, "fonts"), i);
                        diagnostics.push(silent(key(&at, "font_reference")));
                    }
                }
                if !names.insert(alias.alias.clone()) {
                    diagnostics.push(finding(
                        DiagnosticCode::FontNameOverridden,
                        Severity::Info,
                        key(at, "alias"),
                    ));
                }
            }
            aliases = read.iter().map(|(_, alias)| alias.clone()).collect();
        }
        let has_aliases = font_aliases.is_some();
        drop(names);
        Self {
            font_aliases: has_aliases.then_some(aliases),
            ..metadata
        }
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
    use bedrock_addon_core::test_cost::assert_linear_at;
    use bedrock_addon_support::jsoncpp::read;

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
        assert!(FontAliasRef::read(&root, &JsonPath::new(), &mut diagnostics).is_none());
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
    use bedrock_addon_support::fixed_path::testing::{assert_kept_root_written_linearly, list};

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
