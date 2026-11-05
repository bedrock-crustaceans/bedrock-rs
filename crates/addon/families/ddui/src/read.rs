//! Reading a root, a composition or a style the way the client's `cereal` parse does under its
//! strict error mask: every unknown member, missing member, wrong type and failed constraint is
//! an error and the file is skipped. The reader goes through the whole file and reports every
//! one of them, each at its own place.

use facet_json::RawJson;
use indexmap::IndexMap;

use super::style::{
    BorderImageRepeat, BoxSize, FormLabelStyle, FormPanelStyle, FormTextFieldStyle, StyleColor,
    StyleComponents, StyleDescription, UiStyle, hex_bytes,
};
use super::{
    Binding, Catalogue, ChildSet, Component, CompositionAttribs, CompositionDescription, Control,
    Layout, RootAttribs, RootDescription, UiComposition, UiRoot,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Severity, Span, VanillaText,
};
use bedrock_addon_core::id::Id;
use bedrock_addon_core::id::syntax::has_namespace;
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::version::FormatVersion;

/// The client's text for an identifier that is not `namespace:name`.
const NAMESPACE_TEXT: &str = "string must be prefixed with a namespace (eg. namespace:value)";

/// The members of a JSON object, each a slice of the file.
type Members<'a> = IndexMap<String, RawJson<'a>>;

/// The first byte of `text` that is neither white space nor inside a comment.
fn first_token(text: &str) -> Option<u8> {
    let mut rest = text.trim_start();
    loop {
        if let Some(comment) = rest.strip_prefix("//") {
            rest = comment.split_once('\n').map_or("", |(_, after)| after);
        } else if let Some(comment) = rest.strip_prefix("/*") {
            rest = comment.split_once("*/").map_or("", |(_, after)| after);
        } else {
            return rest.bytes().next();
        }
        rest = rest.trim_start();
    }
}

/// Where `slice` is in the file.
fn span_of(slice: JsonSlice<'_>) -> Span {
    let narrow = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    Span {
        offset: narrow(slice.offset),
        len: narrow(slice.text.len()),
    }
}

/// The client's text for a number no double holds: it logs "Failed to parse the Minecraft
/// Document JSON: {}" with rapidjson's "number too big" message.
const NUMBER_TOO_BIG: &str =
    "Failed to parse the Minecraft Document JSON: Number too big to be stored in double.";

/// The byte ranges of the numbers of `text` whose positive exponent is past `308` plus the
/// digits of their fraction, the point where rapidjson's number reader stops with its "number
/// too big" error before it has a value to compare (strings and comments are skipped).
fn exponent_too_big(text: &str) -> Vec<std::ops::Range<usize>> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                i += 1;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
            }
            b'-' | b'0'..=b'9' => {
                let start = i;
                i += 1;
                while i < bytes.len()
                    && matches!(bytes[i], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
                {
                    i += 1;
                }
                let token = &text[start..i];
                let body = token.strip_prefix('-').unwrap_or(token);
                let (mantissa, exponent) = match body.find(['e', 'E']) {
                    Some(at) => (&body[..at], Some(&body[at + 1..])),
                    None => (body, None),
                };
                let fraction = mantissa
                    .split_once('.')
                    .map_or(0, |(_, digits)| digits.len());
                if let Some(exponent) = exponent
                    && !exponent.starts_with('-')
                {
                    let digits = exponent.strip_prefix('+').unwrap_or(exponent);
                    let exponent = digits
                        .bytes()
                        .take_while(u8::is_ascii_digit)
                        .fold(0_usize, |n, d| {
                            n.saturating_mul(10).saturating_add(usize::from(d - b'0'))
                        });
                    if exponent > 308 + fraction {
                        found.push(start..i);
                    }
                }
            }
            _ => i += 1,
        }
    }
    found
}

/// One file's read: the catalogue of the engine, where the reader is, and what it found.
struct Reader<'d> {
    catalogue: Catalogue,
    /// The path of the value being read.
    at: JsonPath,
    /// A root's `ExtensionPoint` controls are checked for their name; a composition's are
    /// never collected.
    root: bool,
    /// An error was reported: the file is refused.
    failed: bool,
    d: &'d mut Diagnostics,
}

impl Reader<'_> {
    /// A finding at the reader's place.
    fn report(&mut self, code: DiagnosticCode, severity: Severity, slice: JsonSlice<'_>) {
        self.failed |= severity == Severity::Error;
        self.d.push(
            Diagnostic::error(code)
                .severity(severity)
                .at(self.at.clone())
                .span(span_of(slice)),
        );
    }

    /// An error at the reader's place.
    fn error(&mut self, code: DiagnosticCode, slice: JsonSlice<'_>) {
        self.report(code, Severity::Error, slice);
    }

    /// An error at the member `key` of the reader's place; `slice` is what the span covers.
    fn error_at(&mut self, key: &str, code: DiagnosticCode, slice: JsonSlice<'_>) {
        self.at.push(key);
        self.error(code, slice);
        self.at.pop();
    }

    /// The member `key` of `object` is required and absent.
    fn missing(&mut self, key: &str, object: JsonSlice<'_>) {
        self.error_at(
            key,
            DiagnosticCode::MissingRequiredField { text: None },
            object,
        );
    }

    /// A typed read of `slice` whose first token is `token`; a wrong type otherwise.
    fn typed<'a, T: facet::Facet<'a>>(&mut self, slice: JsonSlice<'a>, token: u8) -> Option<T> {
        if first_token(slice.text) != Some(token) {
            self.error(DiagnosticCode::InvalidType { text: None }, slice);
            return None;
        }
        // The token says what the value is, so the reader itself is asked (the text of a pack
        // file, comments allowed): `json::parse_at` would look for an array given to a map on
        // every level again, which the token has already ruled out.
        match bedrock_addon_core::json::from_str_borrowed_jsonc::<T>(slice.text) {
            Ok(value) => Some(value),
            Err(failure) => {
                // The text is JSON (the envelope probe read it), so only a value the reader
                // has no form for gets here.
                let message = failure.kind.to_string();
                self.error(
                    DiagnosticCode::MalformedInput {
                        message,
                        text: None,
                    },
                    slice,
                );
                None
            }
        }
    }

    /// The members of the object `slice`, in file order.
    fn object<'a>(&mut self, slice: JsonSlice<'a>) -> Option<Members<'a>> {
        self.typed(slice, b'{')
    }

    /// The string `slice`.
    fn string(&mut self, slice: JsonSlice<'_>) -> Option<String> {
        self.typed(slice, b'"')
    }

    /// The string member `key`, held to `accept`; `text` is vanilla's for a refused value.
    fn string_member(
        &mut self,
        key: &str,
        slice: JsonSlice<'_>,
        accept: fn(&str) -> bool,
        text: Option<&'static str>,
    ) -> Option<String> {
        self.at.push(key);
        let value = self.string(slice).filter(|value| {
            let accepted = accept(value);
            if !accepted {
                self.error(
                    DiagnosticCode::InvalidValue {
                        text: text.map(VanillaText::Fixed),
                    },
                    slice,
                );
            }
            accepted
        });
        self.at.pop();
        value
    }

    /// An object that binds the one string member `key`: the member's value, when it is
    /// written and accepted. What the object gets wrong is reported and fails the file.
    fn single(
        &mut self,
        slice: JsonSlice<'_>,
        key: &str,
        required: bool,
        accept: fn(&str) -> bool,
        text: Option<&'static str>,
    ) -> Option<String> {
        let members = self.object(slice)?;
        let mut value = None;
        for (name, raw) in &members {
            let Some(member) = slice.nested(raw) else {
                continue;
            };
            if name == key {
                value = self.string_member(key, member, accept, text);
            } else {
                self.error_at(
                    name,
                    DiagnosticCode::UnrecognizedField { text: None },
                    member,
                );
            }
        }
        if required && !members.contains_key(key) {
            self.missing(key, slice);
        }
        value
    }

    /// A document's `description`: the identifier, under `NamespaceConstraint` without a
    /// disallowed namespace.
    fn description(&mut self, slice: JsonSlice<'_>) -> Option<String> {
        self.single(
            slice,
            "identifier",
            true,
            has_namespace,
            Some(NAMESPACE_TEXT),
        )
    }

    /// A `layout`: `markup`, at least one control of `set`.
    fn layout(&mut self, slice: JsonSlice<'_>, set: ChildSet) -> Option<Layout> {
        let members = self.object(slice)?;
        let mut markup = None;
        for (name, raw) in &members {
            let Some(member) = slice.nested(raw) else {
                continue;
            };
            if name == "markup" {
                self.at.push("markup");
                markup = self.controls(member, set);
                if markup.as_ref().is_some_and(Vec::is_empty) {
                    // `markup` must hold at least one control.
                    self.error(DiagnosticCode::InvalidValue { text: None }, member);
                    markup = None;
                }
                self.at.pop();
            } else {
                self.error_at(
                    name,
                    DiagnosticCode::UnrecognizedField { text: None },
                    member,
                );
            }
        }
        if !members.contains_key("markup") {
            self.missing("markup", slice);
        }
        markup.map(|markup| Layout { markup })
    }

    /// An array of controls of `set`; `None` when the array or one of its controls fails.
    fn controls(&mut self, slice: JsonSlice<'_>, set: ChildSet) -> Option<Vec<Control>> {
        let elements: Vec<RawJson<'_>> = self.typed(slice, b'[')?;
        let mut controls = Vec::with_capacity(elements.len());
        let mut complete = true;
        for (index, raw) in elements.iter().enumerate() {
            let Some(element) = slice.nested(raw) else {
                continue;
            };
            self.at.push_index(index);
            match self.control(element, set) {
                Some(control) => controls.push(control),
                None => complete = false,
            }
            self.at.pop();
        }
        complete.then_some(controls)
    }

    /// The `attribs` of a control: any JSON, held as its value. A number no double holds
    /// refuses the file: the game parses the document with rapidjson (comments and trailing
    /// commas allowed, no `NaN` or infinity), whose number reader reports "number too big" for
    /// a positive exponent past `308` plus the fraction's digits and for a value past
    /// `DBL_MAX`, and the loader logs "Failed to parse the Minecraft Document JSON: {}" as an
    /// error. Each such number is an error of its own, at its own place. An integer beyond 64
    /// bits is a number like any other here: the JSON reader hands it over as the double it
    /// spells.
    fn attribs(&mut self, slice: JsonSlice<'_>) -> Option<Value> {
        let mut value = self.typed::<Value>(slice, first_token(slice.text).unwrap_or(0))?;
        let mut too_big = false;
        // Core's walk: each infinity with its own path and span.
        for clamped in bedrock_addon_core::json::clamp_non_finite(&mut value, Some(slice)) {
            let mut at = self.at.clone();
            at.append(&clamped.path);
            self.failed = true;
            too_big = true;
            let finding = Diagnostic::error(DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed(NUMBER_TOO_BIG)),
            })
            .at(at);
            self.d.push(match clamped.span {
                Some(span) => finding.span(span),
                None => finding,
            });
        }
        // A finite value whose exponent rapidjson refuses all the same (`0e999`).
        for range in exponent_too_big(slice.text) {
            let token = JsonSlice {
                text: &slice.text[range.clone()],
                offset: slice.offset + range.start,
            };
            too_big = true;
            self.error(
                DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::Fixed(NUMBER_TOO_BIG)),
                },
                token,
            );
        }
        if too_big {
            return None;
        }
        Some(value)
    }

    /// The control type `slice` names, when the catalogue has it; a hint names the engine
    /// that has a type this one lacks: the first later one, else the last earlier one.
    fn component(&mut self, slice: JsonSlice<'_>) -> Option<Component> {
        let name = self.string(slice)?;
        let known = Component::from_name(&name);
        if let Some(component) = known.filter(|component| self.catalogue.has(*component)) {
            return Some(component);
        }
        // No alternative of any variant has this constant: the file is skipped.
        self.error(DiagnosticCode::InvalidValue { text: None }, slice);
        if let Some(later) = known.and_then(|component| self.catalogue.later_with(component)) {
            self.report(
                DiagnosticCode::FieldSinceEngine {
                    since: later.engine(),
                },
                Severity::Info,
                slice,
            );
        } else if let Some(earlier) =
            known.and_then(|component| self.catalogue.earlier_with(component))
        {
            // A type this engine has dropped (`Container.Item` from 1.26.45): the crate's own
            // hint.
            self.report(
                DiagnosticCode::FieldUntilEngine {
                    last: earlier.engine(),
                },
                Severity::Info,
                slice,
            );
        }
        None
    }

    /// One control standing where `set` is expected. The JSON nests at most as deep as the
    /// reader's depth limit lets a file through, which bounds this recursion.
    fn control(&mut self, slice: JsonSlice<'_>, set: ChildSet) -> Option<Control> {
        let members = self.object(slice)?;
        let Some(component) = members.get("component") else {
            self.missing("component", slice);
            return None;
        };
        self.at.push("component");
        let named = slice.nested(component);
        let component = named.and_then(|named| self.component(named));
        // A type of the catalogue that is no alternative of this place's variant.
        let placed = match (component, named) {
            (Some(component), Some(named)) if !self.catalogue.allows(set, component) => {
                self.error(DiagnosticCode::InvalidValue { text: None }, named);
                false
            }
            _ => true,
        };
        self.at.pop();
        let component = component?;
        let rule = self.catalogue.rule(component)?;
        let before = self.failed;
        self.failed = false;
        let mut attribs = None;
        let mut children = None;
        for (name, raw) in &members {
            let Some(member) = slice.nested(raw) else {
                continue;
            };
            match (name.as_str(), rule.children) {
                ("component", _) => {}
                ("attribs", _) if rule.attribs != Binding::Unbound => {
                    self.at.push("attribs");
                    // Any JSON is accepted; no key and no type is checked.
                    attribs = self.attribs(member);
                    self.at.pop();
                }
                ("children", Some(below)) => {
                    self.at.push("children");
                    children = self.controls(member, below);
                    self.at.pop();
                }
                _ => self.error_at(
                    name,
                    DiagnosticCode::UnrecognizedField { text: None },
                    member,
                ),
            }
        }
        if rule.attribs == Binding::Required && !members.contains_key("attribs") {
            self.missing("attribs", slice);
        }
        if rule.children.is_some() && !members.contains_key("children") {
            self.missing("children", slice);
        }
        let failed = std::mem::replace(&mut self.failed, before) || !placed;
        self.failed |= failed;
        if failed {
            return None;
        }
        let control = Control {
            component,
            attribs,
            children,
        };
        if self.root
            && component == Component::ExtensionPoint
            && control.extension_point_name().is_none()
        {
            // The client collects the extension point names of a root by reading
            // `attribs["name"]` as a string, and a missing key or another type throws out of
            // the whole load without a log line. The crate reports an error that refuses the
            // root instead.
            self.error(DiagnosticCode::InvalidValue { text: None }, slice);
        }
        Some(control)
    }
}

/// The client's text for a padding that is no two-value box size.
pub(crate) const BOX_SIZE_2_TEXT: &str = "Expected [top, right].";
/// The text for a padding that is no four-value box size.
pub(crate) const BOX_SIZE_4_TEXT: &str = "Expected [top, right, bottom, left].";
/// The client's text for an RGBA colour that is neither four 0-255 values nor a hex string.
pub(crate) const COLOR_RGBA_TEXT: &str = "Expected [r, g, b, a], with values in the 0-255 range, or a hex value with 8 characters (e.g. ARGB as #FFFFFFFF).";
/// The client's text for an RGB colour that is neither three 0-255 values nor a hex string.
pub(crate) const COLOR_RGB_TEXT: &str = "Expected [r, g, b], with values in the 0-255 range, or a hex value with 6 characters (e.g. RGB as #FFFFFF).";

/// Whether `text` is one JSON integer token: `-?(0|[1-9][0-9]*)`.
fn integer_token(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    !digits.is_empty()
        && digits.bytes().all(|byte| byte.is_ascii_digit())
        && (digits.len() == 1 || !digits.starts_with('0'))
}

/// How a key a style's object does not bind is reported.
#[derive(Clone, Copy)]
enum Unbound {
    /// An unknown member.
    Member,
    /// An unknown component, a key of `components`.
    Component,
}

impl Reader<'_> {
    /// The members of the object `slice`, each key `known` does not name an error (an unknown
    /// member or an unknown component, by `unbound`).
    fn bound<'a>(
        &mut self,
        slice: JsonSlice<'a>,
        known: &[&str],
        unbound: Unbound,
    ) -> Option<Members<'a>> {
        let members = self.object(slice)?;
        for (name, raw) in &members {
            if known.contains(&name.as_str()) {
                continue;
            }
            let Some(member) = slice.nested(raw) else {
                continue;
            };
            let code = match unbound {
                Unbound::Member => DiagnosticCode::UnrecognizedField { text: None },
                Unbound::Component => DiagnosticCode::UnrecognizedComponent { text: None },
            };
            self.error_at(name, code, member);
        }
        Some(members)
    }

    /// Runs `read` on the member `key` of `members` below it, when it is written.
    fn member<'a, T>(
        &mut self,
        members: &Members<'a>,
        object: JsonSlice<'a>,
        key: &str,
        read: impl FnOnce(&mut Self, JsonSlice<'a>) -> Option<T>,
    ) -> Option<T> {
        let member = object.nested(members.get(key)?)?;
        self.at.push(key);
        let value = read(self, member);
        self.at.pop();
        value
    }

    /// A required member: `read`'s value, or a `MissingRequiredField` error at it.
    fn required<'a, T>(
        &mut self,
        members: &Members<'a>,
        object: JsonSlice<'a>,
        key: &str,
        read: impl FnOnce(&mut Self, JsonSlice<'a>) -> Option<T>,
    ) -> Option<T> {
        if !members.contains_key(key) {
            self.missing(key, object);
            return None;
        }
        self.member(members, object, key, read)
    }

    /// A boolean.
    fn boolean(&mut self, slice: JsonSlice<'_>) -> Option<bool> {
        let token = first_token(slice.text);
        self.typed(
            slice,
            token.filter(|t| matches!(t, b't' | b'f')).unwrap_or(b't'),
        )
    }

    /// An array of integer tokens, each with its slice; a wrong type at the array or at the
    /// element otherwise.
    fn integers<'a>(&mut self, slice: JsonSlice<'a>) -> Option<Vec<(i64, JsonSlice<'a>)>> {
        let elements: Vec<RawJson<'_>> = self.typed(slice, b'[')?;
        let mut values = Vec::with_capacity(elements.len());
        let mut complete = true;
        for (index, raw) in elements.iter().enumerate() {
            let Some(element) = slice.nested(raw) else {
                complete = false;
                continue;
            };
            let text = element.text.trim();
            if let Some(value) = integer_token(text)
                .then(|| text.parse::<i64>().ok())
                .flatten()
            {
                values.push((value, element));
            } else {
                self.at.push_index(index);
                self.error(DiagnosticCode::InvalidType { text: None }, element);
                self.at.pop();
                complete = false;
            }
        }
        complete.then_some(values)
    }

    /// A conversion failure: `InvalidValue` with `text` at the reader's place.
    fn unconverted(&mut self, text: &'static str, slice: JsonSlice<'_>) {
        self.error(
            DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed(text)),
            },
            slice,
        );
    }

    /// A box size: two or four integers. Another count is the conversion's error
    /// (the two-value text up to two elements, the four-value text above).
    fn box_size(&mut self, slice: JsonSlice<'_>) -> Option<BoxSize> {
        let values = self.integers(slice)?;
        let ints: Option<Vec<i32>> = values
            .iter()
            .map(|(value, _)| i32::try_from(*value).ok())
            .collect();
        let size = match (ints.as_deref(), values.len()) {
            (Some(&[a, b]), _) => Some(BoxSize::Two([a, b])),
            (Some(&[a, b, c, d]), _) => Some(BoxSize::Four([a, b, c, d])),
            _ => None,
        };
        if size.is_none() {
            let text = if values.len() <= 2 {
                BOX_SIZE_2_TEXT
            } else {
                BOX_SIZE_4_TEXT
            };
            self.unconverted(text, slice);
        }
        size
    }

    /// A colour, RGBA tried first: four or three integers 0–255, or `#`
    /// with eight or six hex digits. What neither takes is the conversion's error, with the
    /// text of the alternative its shape is nearer (three elements or seven characters: RGB).
    fn color(&mut self, slice: JsonSlice<'_>) -> Option<StyleColor> {
        match first_token(slice.text) {
            Some(b'"') => {
                let text = self.string(slice)?;
                if hex_bytes::<4>(&text).is_some() {
                    return Some(StyleColor::HexArgb(text));
                }
                if hex_bytes::<3>(&text).is_some() {
                    return Some(StyleColor::HexRgb(text));
                }
                let near_rgb = text.chars().count() == 7;
                self.unconverted(
                    if near_rgb {
                        COLOR_RGB_TEXT
                    } else {
                        COLOR_RGBA_TEXT
                    },
                    slice,
                );
                None
            }
            Some(b'[') => {
                let values = self.integers(slice)?;
                let bytes: Option<Vec<u8>> = values
                    .iter()
                    .map(|(value, _)| u8::try_from(*value).ok())
                    .collect();
                let color = match bytes.as_deref() {
                    Some(&[r, g, b, a]) => Some(StyleColor::Rgba([r, g, b, a])),
                    Some(&[r, g, b]) => Some(StyleColor::Rgb([r, g, b])),
                    _ => None,
                };
                if color.is_none() {
                    let text = if values.len() == 3 {
                        COLOR_RGB_TEXT
                    } else {
                        COLOR_RGBA_TEXT
                    };
                    self.unconverted(text, slice);
                }
                color
            }
            _ => {
                self.error(DiagnosticCode::InvalidType { text: None }, slice);
                None
            }
        }
    }

    /// A border image repeat mode: `"stretch"` or `"repeat"`.
    fn repeat(&mut self, slice: JsonSlice<'_>) -> Option<BorderImageRepeat> {
        let name = self.string(slice)?;
        let value = BorderImageRepeat::from_name(&name);
        if value.is_none() {
            self.error(DiagnosticCode::InvalidValue { text: None }, slice);
        }
        value
    }

    /// An image reference: a string.
    fn image(&mut self, slice: JsonSlice<'_>) -> Option<String> {
        self.string(slice)
    }

    /// The `minecraft:form_panel` style component.
    fn form_panel(&mut self, slice: JsonSlice<'_>) -> Option<FormPanelStyle> {
        const KNOWN: [&str; 4] = [
            "padding",
            "image",
            "image_repeat_top_bottom",
            "image_repeat_right_left",
        ];
        let members = self.bound(slice, &KNOWN, Unbound::Member)?;
        Some(FormPanelStyle {
            padding: self.member(&members, slice, "padding", Self::box_size),
            image: self.member(&members, slice, "image", Self::image),
            image_repeat_top_bottom: self.member(
                &members,
                slice,
                "image_repeat_top_bottom",
                Self::repeat,
            ),
            image_repeat_right_left: self.member(
                &members,
                slice,
                "image_repeat_right_left",
                Self::repeat,
            ),
        })
    }

    /// The `minecraft:form_label` style component.
    fn form_label(&mut self, slice: JsonSlice<'_>) -> Option<FormLabelStyle> {
        let members = self.bound(slice, &["color", "shadow"], Unbound::Member)?;
        Some(FormLabelStyle {
            color: self.member(&members, slice, "color", Self::color),
            shadow: self.member(&members, slice, "shadow", Self::boolean),
        })
    }

    /// The `minecraft:form_textfield` style component: the four colours are required.
    fn form_textfield(&mut self, slice: JsonSlice<'_>) -> Option<FormTextFieldStyle> {
        const KNOWN: [&str; 9] = [
            "text_color",
            "caret_color",
            "selection_background_color",
            "text_color_disabled",
            "default_image",
            "hovered_image",
            "disabled_image",
            "focused_image",
            "panel_image",
        ];
        let members = self.bound(slice, &KNOWN, Unbound::Member)?;
        let text_color = self.required(&members, slice, "text_color", Self::color);
        let caret_color = self.required(&members, slice, "caret_color", Self::color);
        let selection_background_color =
            self.required(&members, slice, "selection_background_color", Self::color);
        let text_color_disabled =
            self.required(&members, slice, "text_color_disabled", Self::color);
        let mut style = FormTextFieldStyle::new(
            text_color?,
            caret_color?,
            selection_background_color?,
            text_color_disabled?,
        );
        style.default_image = self.member(&members, slice, "default_image", Self::image);
        style.hovered_image = self.member(&members, slice, "hovered_image", Self::image);
        style.disabled_image = self.member(&members, slice, "disabled_image", Self::image);
        style.focused_image = self.member(&members, slice, "focused_image", Self::image);
        style.panel_image = self.member(&members, slice, "panel_image", Self::image);
        Some(style)
    }

    /// The `components` of a style: at least one component.
    fn style_components(&mut self, slice: JsonSlice<'_>) -> Option<StyleComponents> {
        let [panel, label, textfield] = StyleComponents::KEYS;
        let members = self.bound(slice, &StyleComponents::KEYS, Unbound::Component)?;
        let components = StyleComponents {
            form_panel: self.member(&members, slice, panel, Self::form_panel),
            form_label: self.member(&members, slice, label, Self::form_label),
            form_textfield: self.member(&members, slice, textfield, Self::form_textfield),
        };
        if members.is_empty() {
            self.error(DiagnosticCode::InvalidValue { text: None }, slice);
            return None;
        }
        Some(components)
    }
}

/// Reads the payload of a form style: a `description` and its `components`.
pub(crate) fn style(payload: JsonSlice<'_>, at: &JsonPath, d: &mut Diagnostics) -> Option<UiStyle> {
    let mut reader = Reader {
        catalogue: Catalogue::V1_26_60,
        at: at.clone(),
        root: false,
        failed: false,
        d,
    };
    let members = reader.bound(payload, &["description", "components"], Unbound::Member)?;
    let identifier = reader.required(&members, payload, "description", Reader::description);
    let components = reader.required(&members, payload, "components", Reader::style_components);
    if reader.failed {
        return None;
    }
    Some(UiStyle {
        description: StyleDescription {
            identifier: Id::parse(identifier?),
        },
        components: components?,
    })
}

/// A reader for a payload at `at`, read by an engine of version `engine`.
fn reader<'d>(
    engine: FormatVersion,
    at: &JsonPath,
    root: bool,
    d: &'d mut Diagnostics,
) -> Reader<'d> {
    Reader {
        catalogue: Catalogue::for_engine(engine),
        at: at.clone(),
        root,
        failed: false,
        d,
    }
}

/// Reads the payload of a root: `description`, `attribs` and `layout`, all required.
pub(crate) fn root(
    payload: JsonSlice<'_>,
    engine: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<UiRoot> {
    let mut reader = reader(engine, at, true, d);
    let members = reader.object(payload)?;
    let mut identifier = None;
    let mut root_point = None;
    let mut layout = None;
    for (name, raw) in &members {
        let Some(member) = payload.nested(raw) else {
            continue;
        };
        reader.at.push(name);
        match name.as_str() {
            "description" => identifier = reader.description(member),
            // `attribs` binds `root_point` alone, which must not be empty.
            "attribs" => {
                root_point =
                    reader.single(member, "root_point", true, |text| !text.is_empty(), None);
            }
            "layout" => layout = reader.layout(member, ChildSet::RootControl),
            _ => reader.error(DiagnosticCode::UnrecognizedField { text: None }, member),
        }
        reader.at.pop();
    }
    for key in ["description", "attribs", "layout"] {
        if !members.contains_key(key) {
            reader.missing(key, payload);
        }
    }
    if reader.failed {
        return None;
    }
    Some(UiRoot {
        description: RootDescription {
            identifier: Id::parse(identifier?),
        },
        attribs: RootAttribs {
            root_point: root_point?,
        },
        layout: layout?,
    })
}

/// Reads the payload of a composition: `description`, `attribs` and `layout`, all required.
pub(crate) fn composition(
    payload: JsonSlice<'_>,
    engine: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<UiComposition> {
    let mut reader = reader(engine, at, false, d);
    let members = reader.object(payload)?;
    let mut identifier = None;
    let mut extension_point = None;
    let mut layout = None;
    for (name, raw) in &members {
        let Some(member) = payload.nested(raw) else {
            continue;
        };
        reader.at.push(name);
        match name.as_str() {
            "description" => identifier = reader.description(member),
            // `attribs` binds an optional `extension_point` alone, which must not be empty.
            "attribs" => {
                extension_point = reader.single(
                    member,
                    "extension_point",
                    false,
                    |text| !text.is_empty(),
                    None,
                );
            }
            "layout" => layout = reader.layout(member, ChildSet::PanelContent),
            _ => reader.error(DiagnosticCode::UnrecognizedField { text: None }, member),
        }
        reader.at.pop();
    }
    for key in ["description", "attribs", "layout"] {
        if !members.contains_key(key) {
            reader.missing(key, payload);
        }
    }
    if reader.failed {
        return None;
    }
    Some(UiComposition {
        description: CompositionDescription {
            identifier: Id::parse(identifier?),
        },
        attribs: CompositionAttribs { extension_point },
        layout: layout?,
    })
}

/// Where the file `document` writes the value at `path`; `None` when a step of the path is
/// not there. One read of each level on the way.
pub(crate) fn span_at(document: JsonSlice<'_>, path: &JsonPath) -> Option<Span> {
    let mut current = document;
    for step in path.segments() {
        current = match (first_token(current.text)?, step) {
            (b'{', JsonPathSegment::Key(key)) => {
                let members =
                    bedrock_addon_core::json::from_str_borrowed_jsonc::<Members<'_>>(current.text)
                        .ok()?;
                current.nested(members.get(key)?)?
            }
            (b'[', JsonPathSegment::Index(index)) => {
                let elements =
                    bedrock_addon_core::json::from_str_borrowed_jsonc::<Vec<RawJson<'_>>>(
                        current.text,
                    )
                    .ok()?;
                current.nested(elements.get(index)?)?
            }
            _ => return None,
        };
    }
    Some(span_of(current))
}
