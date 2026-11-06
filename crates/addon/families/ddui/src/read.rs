//! Reading a root, a composition or a style the way the client's `cereal` parse does under its
//! strict error mask: every unknown member, missing member, wrong type and failed constraint is
//! an error and the file is skipped. The reader goes through the whole file and reports every
//! one of them, each at its own place.

use super::style::{
    BorderImageRepeat, BoxSize, FormLabelStyle, FormPanelStyle, FormTextFieldStyle, StyleColor,
    StyleComponents, StyleDescription, UiStyle,
};
use super::{
    Binding, Catalogue, ChildSet, Component, CompositionAttribs, CompositionDescription, Control,
    Layout, RootAttribs, RootDescription, UiComposition, UiRoot,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, Span, VanillaText,
};
use bedrock_addon_core::id::Id;
use bedrock_addon_core::id::syntax::has_namespace;
use bedrock_addon_core::json::kit::{Disposition, Elements, Members, Node, Read};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::Reading;
use bedrock_addon_shared_shapes::color::hex_bytes;

/// The client's text for an identifier that is not `namespace:name`.
const NAMESPACE_TEXT: &str = "string must be prefixed with a namespace (eg. namespace:value)";

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

/// How every level of a ddui document is read: the client's strict error mask, so an unknown
/// member, a missing member and a value of the wrong JSON type are each an error that skips the
/// file.
const READING: Reading = Reading::DOCUMENT;

/// One file's read: the catalogue of the engine, and where the findings go. Each value is a
/// [`Node`] of the payload with its text, so every finding has its path and span.
struct Reader<'d> {
    catalogue: Catalogue,
    /// A root's `ExtensionPoint` controls are checked for their name; a composition's are
    /// never collected.
    root: bool,
    /// How many findings `d` held before the read: an error told after it refuses the file.
    start: usize,
    d: &'d mut Diagnostics,
}

impl Reader<'_> {
    /// The kit's read over the findings.
    fn read(&mut self) -> Read<'_> {
        Read::new(READING, self.d)
    }

    /// Whether an error was told since the findings held `mark`.
    fn failed_since(&self, mark: usize) -> bool {
        self.d
            .iter()
            .skip(mark)
            .any(|finding| finding.severity == Severity::Error)
    }

    /// Whether the reader told an error: the file is refused.
    fn failed(&self) -> bool {
        self.failed_since(self.start)
    }

    /// A finding at `node`, spanning it.
    fn report(&mut self, node: &Node<'_>, code: DiagnosticCode, severity: Severity) {
        let mut finding = Diagnostic::error(code)
            .severity(severity)
            .at(node.path().clone());
        finding.location.span = node.span();
        self.read().push(finding);
    }

    /// An error at `node`, spanning it.
    fn error(&mut self, node: &Node<'_>, code: DiagnosticCode) {
        self.report(node, code, Severity::Error);
    }

    /// The members of the object `node`, in file order; a wrong type otherwise.
    fn object<'v>(&mut self, node: &Node<'v>) -> Option<Members<'v>> {
        self.read().object(node)
    }

    /// The elements of the array `node`; a wrong type otherwise.
    fn array<'v>(&mut self, node: &Node<'v>) -> Option<Elements<'v>> {
        self.read().array(node)
    }

    /// The string `node`: its text is a string token (the untyped value is a string for an
    /// integer too wide too, which the client reads as a number); a wrong type otherwise.
    fn string(&mut self, node: &Node<'_>) -> Option<String> {
        let text = (token(node) == Some(b'"'))
            .then(|| node.value().as_string())
            .flatten();
        if text.is_none() {
            self.read().wrong_type(node);
        }
        text.map(|text| text.as_str().to_owned())
    }

    /// The string member `node`, held to `accept`; `text` is vanilla's for a refused value.
    fn string_member(
        &mut self,
        node: &Node<'_>,
        accept: fn(&str) -> bool,
        text: Option<&'static str>,
    ) -> Option<String> {
        self.string(node).filter(|value| {
            let accepted = accept(value);
            if !accepted {
                self.error(
                    node,
                    DiagnosticCode::InvalidValue {
                        text: text.map(VanillaText::Fixed),
                    },
                );
            }
            accepted
        })
    }

    /// An object that binds the one string member `key`: the member's value, when it is
    /// written and accepted. What the object gets wrong is reported and fails the file.
    fn single(
        &mut self,
        node: &Node<'_>,
        key: &str,
        required: bool,
        accept: fn(&str) -> bool,
        text: Option<&'static str>,
    ) -> Option<String> {
        let members = self.object(node)?;
        let mut value = None;
        for (name, member) in members.iter() {
            if name == key {
                value = self.string_member(&member, accept, text);
            } else {
                self.read().unknown_member(&member, Disposition::Keep);
            }
        }
        if required {
            self.read().required(&members, key);
        }
        value
    }

    /// A document's `description`: the identifier, under `NamespaceConstraint` without a
    /// disallowed namespace.
    fn description(&mut self, node: &Node<'_>) -> Option<String> {
        self.single(
            node,
            "identifier",
            true,
            has_namespace,
            Some(NAMESPACE_TEXT),
        )
    }

    /// A `layout`: `markup`, at least one control of `set`.
    fn layout(&mut self, node: &Node<'_>, set: ChildSet) -> Option<Layout> {
        let members = self.object(node)?;
        let mut markup = None;
        for (name, member) in members.iter() {
            if name == "markup" {
                markup = self.controls(&member, set);
                if markup.as_ref().is_some_and(Vec::is_empty) {
                    // `markup` must hold at least one control.
                    self.error(&member, DiagnosticCode::InvalidValue { text: None });
                    markup = None;
                }
            } else {
                self.read().unknown_member(&member, Disposition::Keep);
            }
        }
        self.read().required(&members, "markup");
        markup.map(|markup| Layout { markup })
    }

    /// An array of controls of `set`; `None` when the array or one of its controls fails.
    fn controls(&mut self, node: &Node<'_>, set: ChildSet) -> Option<Vec<Control>> {
        let elements = self.array(node)?;
        let mut controls = Vec::with_capacity(elements.len());
        let mut complete = true;
        for element in elements.iter() {
            match self.control(&element, set) {
                Some(control) => controls.push(control),
                None => complete = false,
            }
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
    fn attribs(&mut self, node: &Node<'_>) -> Option<Value> {
        let slice = node.text()?;
        let mut value = node.value().clone();
        let mut too_big = false;
        // Core's walk: each infinity with its own path and span.
        for clamped in bedrock_addon_core::json::clamp_non_finite(&mut value, Some(slice)) {
            let mut at = node.path().clone();
            at.append(&clamped.path);
            too_big = true;
            let finding = Diagnostic::error(DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed(NUMBER_TOO_BIG)),
            })
            .at(at);
            self.read().push(match clamped.span {
                Some(span) => finding.span(span),
                None => finding,
            });
        }
        // A finite value whose exponent rapidjson refuses all the same (`0e999`).
        for range in exponent_too_big(slice.text) {
            too_big = true;
            let finding = Diagnostic::error(DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed(NUMBER_TOO_BIG)),
            })
            .at(node.path().clone())
            .span(span_of(JsonSlice {
                text: &slice.text[range.clone()],
                offset: slice.offset + range.start,
            }));
            self.read().push(finding);
        }
        if too_big {
            return None;
        }
        Some(value)
    }

    /// The control type `node` names, when the catalogue has it; a hint names the engine
    /// that has a type this one lacks: the first later one, else the last earlier one.
    fn component(&mut self, node: &Node<'_>) -> Option<Component> {
        let name = self.string(node)?;
        let known = Component::from_name(&name);
        if let Some(component) = known.filter(|component| self.catalogue.has(*component)) {
            return Some(component);
        }
        // No alternative of any variant has this constant: the file is skipped.
        self.error(node, DiagnosticCode::InvalidValue { text: None });
        if let Some(later) = known.and_then(|component| self.catalogue.later_with(component)) {
            self.report(
                node,
                DiagnosticCode::FieldSinceEngine {
                    since: later.engine(),
                },
                Severity::Info,
            );
        } else if let Some(earlier) =
            known.and_then(|component| self.catalogue.earlier_with(component))
        {
            // A type this engine has dropped (`Container.Item` from 1.26.45): the crate's own
            // hint.
            self.report(
                node,
                DiagnosticCode::FieldUntilEngine {
                    last: earlier.engine(),
                },
                Severity::Info,
            );
        }
        None
    }

    /// One control standing where `set` is expected. The JSON nests at most as deep as the
    /// reader's depth limit lets a file through, which bounds this recursion.
    fn control(&mut self, node: &Node<'_>, set: ChildSet) -> Option<Control> {
        let members = self.object(node)?;
        let named = self.read().required(&members, "component")?;
        let component = self.component(&named);
        // A type of the catalogue that is no alternative of this place's variant.
        let placed = match component {
            Some(component) if !self.catalogue.allows(set, component) => {
                self.error(&named, DiagnosticCode::InvalidValue { text: None });
                false
            }
            _ => true,
        };
        let component = component?;
        let rule = self.catalogue.rule(component)?;
        let mark = self.d.len();
        let mut attribs = None;
        let mut children = None;
        for (name, member) in members.iter() {
            match (name, rule.children) {
                ("component", _) => {}
                // Any JSON is accepted; no key and no type is checked.
                ("attribs", _) if rule.attribs != Binding::Unbound => {
                    attribs = self.attribs(&member);
                }
                ("children", Some(below)) => children = self.controls(&member, below),
                _ => self.read().unknown_member(&member, Disposition::Keep),
            }
        }
        if rule.attribs == Binding::Required {
            self.read().required(&members, "attribs");
        }
        if rule.children.is_some() {
            self.read().required(&members, "children");
        }
        if self.failed_since(mark) || !placed {
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
            self.error(node, DiagnosticCode::InvalidValue { text: None });
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

/// The first token of `node`'s text.
fn token(node: &Node<'_>) -> Option<u8> {
    node.text().and_then(|slice| slice.first_byte())
}

impl Reader<'_> {
    /// The members of the object `node`, each key `known` does not name an unknown member.
    fn bound<'v>(&mut self, node: &Node<'v>, known: &[&str]) -> Option<Members<'v>> {
        let members = self.object(node)?;
        self.read().unknown(&members, known, Disposition::Keep);
        Some(members)
    }

    /// Runs `read` on the member `key` of `members`, when it is written.
    fn member<'v, T>(
        &mut self,
        members: &Members<'v>,
        key: &str,
        read: impl FnOnce(&mut Self, &Node<'v>) -> Option<T>,
    ) -> Option<T> {
        let member = members.get(key)?;
        read(self, &member)
    }

    /// A required member: `read`'s value, or a `MissingRequiredField` error at it.
    fn required<'v, T>(
        &mut self,
        members: &Members<'v>,
        key: &str,
        read: impl FnOnce(&mut Self, &Node<'v>) -> Option<T>,
    ) -> Option<T> {
        let member = self.read().required(members, key)?;
        read(self, &member)
    }

    /// A boolean.
    fn boolean(&mut self, node: &Node<'_>) -> Option<bool> {
        self.read().bool(node)
    }

    /// An array of integer tokens, each with its node; a wrong type at the array or at the
    /// element otherwise.
    fn integers(&mut self, node: &Node<'_>) -> Option<Vec<i64>> {
        let elements = self.array(node)?;
        let mut values = Vec::with_capacity(elements.len());
        let mut complete = true;
        for element in elements.iter() {
            let Some(slice) = element.text() else {
                complete = false;
                continue;
            };
            let text = slice.text.trim();
            if let Some(value) = integer_token(text)
                .then(|| text.parse::<i64>().ok())
                .flatten()
            {
                values.push(value);
            } else {
                self.read().wrong_type(&element);
                complete = false;
            }
        }
        complete.then_some(values)
    }

    /// A conversion failure: `InvalidValue` with `text` at `node`.
    fn unconverted(&mut self, text: &'static str, node: &Node<'_>) {
        self.error(
            node,
            DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed(text)),
            },
        );
    }

    /// A box size: two or four integers. Another count is the conversion's error
    /// (the two-value text up to two elements, the four-value text above).
    fn box_size(&mut self, node: &Node<'_>) -> Option<BoxSize> {
        let values = self.integers(node)?;
        let ints: Option<Vec<i32>> = values
            .iter()
            .map(|value| i32::try_from(*value).ok())
            .collect();
        let size = match ints.as_deref() {
            Some(&[a, b]) => Some(BoxSize::Two([a, b])),
            Some(&[a, b, c, d]) => Some(BoxSize::Four([a, b, c, d])),
            _ => None,
        };
        if size.is_none() {
            let text = if values.len() <= 2 {
                BOX_SIZE_2_TEXT
            } else {
                BOX_SIZE_4_TEXT
            };
            self.unconverted(text, node);
        }
        size
    }

    /// A colour, RGBA tried first: four or three integers 0–255, or `#`
    /// with eight or six hex digits. What neither takes is the conversion's error, with the
    /// text of the alternative its shape is nearer (three elements or seven characters: RGB).
    fn color(&mut self, node: &Node<'_>) -> Option<StyleColor> {
        match token(node) {
            Some(b'"') => {
                let text = self.string(node)?;
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
                    node,
                );
                None
            }
            Some(b'[') => {
                let values = self.integers(node)?;
                let bytes: Option<Vec<u8>> = values
                    .iter()
                    .map(|value| u8::try_from(*value).ok())
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
                    self.unconverted(text, node);
                }
                color
            }
            _ => {
                self.read().wrong_type(node);
                None
            }
        }
    }

    /// A border image repeat mode: `"stretch"` or `"repeat"`.
    fn repeat(&mut self, node: &Node<'_>) -> Option<BorderImageRepeat> {
        let name = self.string(node)?;
        let value = BorderImageRepeat::from_name(&name);
        if value.is_none() {
            self.error(node, DiagnosticCode::InvalidValue { text: None });
        }
        value
    }

    /// An image reference: a string.
    fn image(&mut self, node: &Node<'_>) -> Option<String> {
        self.string(node)
    }

    /// The `minecraft:form_panel` style component.
    fn form_panel(&mut self, node: &Node<'_>) -> Option<FormPanelStyle> {
        const KNOWN: [&str; 4] = [
            "padding",
            "image",
            "image_repeat_top_bottom",
            "image_repeat_right_left",
        ];
        let members = self.bound(node, &KNOWN)?;
        Some(FormPanelStyle {
            padding: self.member(&members, "padding", Self::box_size),
            image: self.member(&members, "image", Self::image),
            image_repeat_top_bottom: self.member(&members, "image_repeat_top_bottom", Self::repeat),
            image_repeat_right_left: self.member(&members, "image_repeat_right_left", Self::repeat),
        })
    }

    /// The `minecraft:form_label` style component.
    fn form_label(&mut self, node: &Node<'_>) -> Option<FormLabelStyle> {
        let members = self.bound(node, &["color", "shadow"])?;
        Some(FormLabelStyle {
            color: self.member(&members, "color", Self::color),
            shadow: self.member(&members, "shadow", Self::boolean),
        })
    }

    /// The `minecraft:form_textfield` style component: the four colours are required.
    fn form_textfield(&mut self, node: &Node<'_>) -> Option<FormTextFieldStyle> {
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
        let members = self.bound(node, &KNOWN)?;
        let text_color = self.required(&members, "text_color", Self::color);
        let caret_color = self.required(&members, "caret_color", Self::color);
        let selection_background_color =
            self.required(&members, "selection_background_color", Self::color);
        let text_color_disabled = self.required(&members, "text_color_disabled", Self::color);
        let mut style = FormTextFieldStyle::new(
            text_color?,
            caret_color?,
            selection_background_color?,
            text_color_disabled?,
        );
        style.default_image = self.member(&members, "default_image", Self::image);
        style.hovered_image = self.member(&members, "hovered_image", Self::image);
        style.disabled_image = self.member(&members, "disabled_image", Self::image);
        style.focused_image = self.member(&members, "focused_image", Self::image);
        style.panel_image = self.member(&members, "panel_image", Self::image);
        Some(style)
    }

    /// The `components` of a style: at least one component; another key an unknown
    /// component.
    fn style_components(&mut self, node: &Node<'_>) -> Option<StyleComponents> {
        let [panel, label, textfield] = StyleComponents::KEYS;
        let members = self.object(node)?;
        for (name, member) in members.iter() {
            if !StyleComponents::KEYS.contains(&name) {
                self.error(
                    &member,
                    DiagnosticCode::UnrecognizedComponent { text: None },
                );
            }
        }
        let components = StyleComponents {
            form_panel: self.member(&members, panel, Self::form_panel),
            form_label: self.member(&members, label, Self::form_label),
            form_textfield: self.member(&members, textfield, Self::form_textfield),
        };
        if members.is_empty() {
            self.error(node, DiagnosticCode::InvalidValue { text: None });
            return None;
        }
        Some(components)
    }
}

/// Reads `payload` at `at` with `read`, the payload a [`Node`] with its text. A payload the
/// untyped reader cannot read (not reached: the envelope probe read the file) is the reader's
/// `MalformedInput`, spanning it.
fn with_payload<T>(
    payload: JsonSlice<'_>,
    at: &JsonPath,
    reader: &mut Reader<'_>,
    read: impl FnOnce(&mut Reader<'_>, &Node<'_>) -> Option<T>,
) -> Option<T> {
    let json = match bedrock_addon_core::json::from_str_borrowed_jsonc::<Value>(payload.text) {
        Ok(json) => json,
        Err(failure) => {
            let finding = Diagnostic::error(DiagnosticCode::MalformedInput {
                message: failure.kind.to_string(),
                text: None,
            })
            .at(at.clone())
            .span(span_of(payload));
            reader.read().push(finding);
            return None;
        }
    };
    read(reader, &Node::with_text(&json, at.clone(), payload))
}

/// Reads the payload of a form style: a `description` and its `components`.
pub(crate) fn style(payload: JsonSlice<'_>, at: &JsonPath, d: &mut Diagnostics) -> Option<UiStyle> {
    let mut reader = Reader {
        catalogue: Catalogue::V1_26_60,
        root: false,
        start: d.len(),
        d,
    };
    with_payload(payload, at, &mut reader, |reader, payload| {
        let members = reader.bound(payload, &["description", "components"])?;
        let identifier = reader.required(&members, "description", Reader::description);
        let components = reader.required(&members, "components", Reader::style_components);
        if reader.failed() {
            return None;
        }
        Some(UiStyle {
            description: StyleDescription {
                identifier: Id::parse(identifier?),
            },
            components: components?,
        })
    })
}

/// A reader for a payload read by an engine of version `engine`.
fn reader(engine: FormatVersion, root: bool, d: &mut Diagnostics) -> Reader<'_> {
    Reader {
        catalogue: Catalogue::for_engine(engine),
        root,
        start: d.len(),
        d,
    }
}

/// The members a root and a composition both require.
const DOCUMENT_MEMBERS: [&str; 3] = ["description", "attribs", "layout"];

/// Reads the payload of a root: `description`, `attribs` and `layout`, all required.
pub(crate) fn root(
    payload: JsonSlice<'_>,
    engine: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<UiRoot> {
    let mut reader = reader(engine, true, d);
    with_payload(payload, at, &mut reader, |reader, payload| {
        let members = reader.object(payload)?;
        let mut identifier = None;
        let mut root_point = None;
        let mut layout = None;
        for (name, member) in members.iter() {
            match name {
                "description" => identifier = reader.description(&member),
                // `attribs` binds `root_point` alone, which must not be empty.
                "attribs" => {
                    root_point =
                        reader.single(&member, "root_point", true, |text| !text.is_empty(), None);
                }
                "layout" => layout = reader.layout(&member, ChildSet::RootControl),
                _ => reader.read().unknown_member(&member, Disposition::Keep),
            }
        }
        for key in DOCUMENT_MEMBERS {
            reader.read().required(&members, key);
        }
        if reader.failed() {
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
    })
}

/// Reads the payload of a composition: `description`, `attribs` and `layout`, all required.
pub(crate) fn composition(
    payload: JsonSlice<'_>,
    engine: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<UiComposition> {
    let mut reader = reader(engine, false, d);
    with_payload(payload, at, &mut reader, |reader, payload| {
        let members = reader.object(payload)?;
        let mut identifier = None;
        let mut extension_point = None;
        let mut layout = None;
        for (name, member) in members.iter() {
            match name {
                "description" => identifier = reader.description(&member),
                // `attribs` binds an optional `extension_point` alone, which must not be empty.
                "attribs" => {
                    extension_point = reader.single(
                        &member,
                        "extension_point",
                        false,
                        |text| !text.is_empty(),
                        None,
                    );
                }
                "layout" => layout = reader.layout(&member, ChildSet::PanelContent),
                _ => reader.read().unknown_member(&member, Disposition::Keep),
            }
        }
        for key in DOCUMENT_MEMBERS {
            reader.read().required(&members, key);
        }
        if reader.failed() {
            return None;
        }
        Some(UiComposition {
            description: CompositionDescription {
                identifier: Id::parse(identifier?),
            },
            attribs: CompositionAttribs { extension_point },
            layout: layout?,
        })
    })
}
