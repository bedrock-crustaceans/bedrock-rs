//! Reading a root, a composition or a style the way the client's `cereal` parse does under its
//! strict error mask: every unknown member, missing member, wrong type and failed constraint is
//! an error and the file is skipped. The documents are `#[derive(Facet, Schema)]` types read
//! by the derived reader, which goes through the whole file and tells every one of them, each
//! at its own place; what stays here is the hand code of three shapes: the leaves of their own
//! syntax (a box size, a colour), the `read_with` fns of members (an identifier, `attribs`,
//! `markup`, the `children` of each child set) and the rules across members (the control type
//! `component` names, at least one style component, a root's extension point names).

use super::codes;
use super::component::Bound;
use super::style::{BoxSize, StyleColor, StyleComponents, UiStyle};
use super::{Catalogue, ChildSet, Component, Control, UiComposition, UiRoot};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, Diagnostics, JsonPath, JsonPathSegment, Severity, Span, builders,
};
use bedrock_addon_core::id::syntax::has_namespace;
use bedrock_addon_core::id::{Id, Kind};
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::schema::{FromJson, read_document, read_each};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::Reading;
use bedrock_addon_shared_shapes::color::hex_bytes;

/// Where `slice` is in the file.
fn span_of(slice: JsonSlice<'_>) -> Span {
    let narrow = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    Span {
        offset: narrow(slice.offset),
        len: narrow(slice.text.len()),
    }
}

/// How every level of a ddui document is read: the client's strict error mask, so an unknown
/// member, a missing member and a value of the wrong JSON type are each an error that skips the
/// file; every scalar of its own JSON type, and an unknown member told and not kept.
const READING: Reading = Reading::DOCUMENT.strict();

/// A box size: two or four integers. Another count is the conversion's error (the two-value
/// text up to two elements, the four-value text above).
impl FromJson for BoxSize {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let values = read_each(node, read, i64::from_json)?;
        let ints: Option<Vec<i32>> = values.iter().map(|v| i32::try_from(*v).ok()).collect();
        let size = match ints.as_deref() {
            Some(&[a, b]) => Some(BoxSize::Two([a, b])),
            Some(&[a, b, c, d]) => Some(BoxSize::Four([a, b, c, d])),
            _ => None,
        };
        if size.is_none() {
            let text = if values.len() <= 2 {
                &codes::BOX_SIZE_2
            } else {
                &codes::BOX_SIZE_4
            };
            read.refuse(node, Code::from(text));
        }
        size
    }
}

/// A colour, RGBA tried first: four or three integers 0–255, or `#` with eight or six hex
/// digits. What neither takes is the conversion's error, with the text of the alternative its
/// shape is nearer (three elements or seven characters: RGB).
impl FromJson for StyleColor {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let (color, near_rgb) = if node.value().is_array() {
            let values = read_each(node, read, i64::from_json)?;
            let bytes: Option<Vec<u8>> = values.iter().map(|v| u8::try_from(*v).ok()).collect();
            let color = match bytes.as_deref() {
                Some(&[r, g, b, a]) => Some(StyleColor::Rgba([r, g, b, a])),
                Some(&[r, g, b]) => Some(StyleColor::Rgb([r, g, b])),
                _ => None,
            };
            (color, values.len() == 3)
        } else {
            let text = String::from_json(node, read)?;
            let near_rgb = text.chars().count() == 7;
            let color = if hex_bytes::<4>(&text).is_some() {
                Some(StyleColor::HexArgb(text))
            } else if hex_bytes::<3>(&text).is_some() {
                Some(StyleColor::HexRgb(text))
            } else {
                None
            };
            (color, near_rgb)
        };
        if color.is_none() {
            let text = if near_rgb {
                &codes::COLOR_RGB
            } else {
                &codes::COLOR_RGBA
            };
            read.refuse(node, Code::from(text));
        }
        color
    }
}

bedrock_addon_core::impl_schema_leaf!(BoxSize, StyleColor);
bedrock_addon_core::impl_no_molang!(BoxSize, StyleColor);

/// A document's `identifier` (`schema::read_with`): a string under `NamespaceConstraint`
/// without a disallowed namespace.
pub(crate) fn identifier<K: Kind>(node: &Node<'_>, read: &mut Read<'_>) -> Option<Id<K>> {
    let text = String::from_json(node, read)?;
    if !has_namespace(&text) {
        read.refuse(node, Code::from(&codes::NOT_NAMESPACED));
        return None;
    }
    Some(Id::parse(text))
}

/// A style's `components` writes at least one (`schema::after_read` of [`StyleComponents`]).
pub(crate) fn some_component(_: &mut StyleComponents, node: &Node<'_>, read: &mut Read<'_>) {
    if node
        .value()
        .as_object()
        .is_some_and(facet_value::VObject::is_empty)
    {
        let mut finding =
            Diagnostic::error(Code::from(&core_codes::INVALID_VALUE)).at(node.path().clone());
        finding.location.span = node.span();
        read.push(finding);
    }
}

/// The `attribs` of a control (`schema::read_with`): any JSON, held as its value. A number no
/// double holds refuses the file: the game parses the document with rapidjson (comments and
/// trailing commas allowed, no `NaN` or infinity), whose number reader reports "number too big"
/// for a positive exponent past `308` plus the fraction's digits and for a value past
/// `DBL_MAX`, and the loader logs "Failed to parse the Minecraft Document JSON: {}" as an
/// error. Each such number is an error of its own, at its own place. An integer beyond 64 bits
/// is a number like any other here: the JSON reader hands it over as the double it spells.
pub(crate) fn attribs(node: &Node<'_>, read: &mut Read<'_>) -> Option<Value> {
    let slice = node.text()?;
    let mut value = node.value().clone();
    let mut too_big = false;
    // Core's walk: each infinity with its own path and span.
    for clamped in bedrock_addon_core::json::clamp_non_finite(&mut value, Some(slice)) {
        let mut at = node.path().clone();
        at.append(&clamped.path);
        too_big = true;
        let finding = Diagnostic::of(&codes::NUMBER_TOO_BIG).at(at);
        read.push(match clamped.span {
            Some(span) => finding.span(span),
            None => finding,
        });
    }
    // A finite value whose exponent rapidjson refuses all the same (`0e999`).
    for range in bedrock_addon_core::json::exponent_beyond_double(slice.text) {
        too_big = true;
        let finding = Diagnostic::of(&codes::NUMBER_TOO_BIG)
            .at(node.path().clone())
            .span(span_of(JsonSlice {
                text: &slice.text[range.clone()],
                offset: slice.offset + range.start,
            }));
        read.push(finding);
    }
    (!too_big).then_some(value)
}

/// The control type an object names (`schema::tag_with` of [`Bound`]): its `component`, a
/// string the catalogue of the read's engine has. Another name refuses the file; a hint names
/// the engine that has a type this one lacks: the first later one, else the last earlier one.
pub(crate) fn component(node: &Node<'_>, read: &mut Read<'_>) -> Option<&'static str> {
    let members = read.object(node)?;
    let Some(named) = members.get("component") else {
        read.missing_member(&members, "component");
        return None;
    };
    let name = String::from_json(&named, read)?;
    let catalogue = Catalogue::for_engine(read.gating_engine().unwrap_or_default());
    let known = Component::from_name(&name);
    if let Some(component) = known.filter(|component| catalogue.has(*component)) {
        return Some(component.name());
    }
    // No alternative of any variant has this constant: the file is skipped.
    let finding = |code: Code, severity: Severity| {
        let mut finding = Diagnostic::error(code)
            .severity(severity)
            .at(named.path().clone());
        finding.location.span = named.span();
        finding
    };
    read.push(finding(
        Code::from(&core_codes::INVALID_VALUE),
        Severity::Error,
    ));
    let later = known.and_then(|component| catalogue.later_with(component));
    let earlier = known.and_then(|component| catalogue.earlier_with(component));
    let hint = match (later, earlier) {
        (Some(later), _) => Some(core_codes::FIELD_SINCE_ENGINE.with([later.engine().to_string()])),
        // A type this engine has dropped (`Container.Item` from 1.26.45): the crate's own hint.
        (None, Some(earlier)) => {
            Some(core_codes::FIELD_UNTIL_ENGINE.with([earlier.engine().to_string()]))
        }
        (None, None) => None,
    };
    if let Some(hint) = hint {
        read.push(finding(hint, Severity::Info));
    }
    None
}

/// Whether `node` is in a root: the first step of its path is a root's payload key. A root's
/// markup is of root controls, a composition's of panel content, and only a root's
/// `ExtensionPoint` controls are collected.
fn in_root(node: &Node<'_>) -> bool {
    use bedrock_addon_core::versioning::Family;
    matches!(
        node.path().segments().next(),
        Some(JsonPathSegment::Key(key)) if key == crate::history::RootFamily::PAYLOAD_KEY
    )
}

/// A finding at `node`, spanning it.
fn at(node: &Node<'_>, code: Code, severity: Severity) -> Diagnostic {
    let mut finding = Diagnostic::error(code)
        .severity(severity)
        .at(node.path().clone());
    finding.location.span = node.span();
    finding
}

/// A string member that must not be empty (`schema::read_with`: a root's `root_point`, a
/// composition's `extension_point`).
pub(crate) fn non_empty(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let text = String::from_json(node, read)?;
    if text.is_empty() {
        read.refuse(node, Code::from(&core_codes::INVALID_VALUE));
        return None;
    }
    Some(text)
}

/// A `layout`'s `markup` (`schema::read_with`): at least one control, of root controls in a
/// root and of panel content in a composition.
pub(crate) fn markup(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Control>> {
    let set = if in_root(node) {
        ChildSet::RootControl
    } else {
        ChildSet::PanelContent
    };
    let markup = children(node, read, set)?;
    if markup.is_empty() {
        read.refuse(node, Code::from(&core_codes::INVALID_VALUE));
        return None;
    }
    Some(markup)
}

/// An array of controls standing where `set` is expected. A type of the catalogue that is no
/// alternative of the place is refused at its `component` before the control is read, and the
/// control is read all the same (what stands below it is told too). The JSON nests at most as
/// deep as the reader's depth limit lets a file through, which bounds this recursion.
pub(crate) fn children(
    node: &Node<'_>,
    read: &mut Read<'_>,
    set: ChildSet,
) -> Option<Vec<Control>> {
    read_each(node, read, |element, read| {
        let misplaced = misplaced(element, read, set);
        let control = Bound::from_json(element, read).map(Bound::into_control);
        control.filter(|_| !misplaced)
    })
}

/// Whether the control `element` is of a type the catalogue of the read's engine has and that
/// is no alternative of `set`; told at its `component`.
fn misplaced(element: &Node<'_>, read: &mut Read<'_>, set: ChildSet) -> bool {
    let Some(named) = element
        .value()
        .as_object()
        .and_then(|_| read.object(element))
        .and_then(|members| members.get("component"))
    else {
        return false;
    };
    let catalogue = Catalogue::for_engine(read.gating_engine().unwrap_or_default());
    let known = named
        .value()
        .as_string()
        .and_then(|name| Component::from_name(name.as_str()))
        .filter(|component| catalogue.has(*component));
    match known {
        Some(component) if !catalogue.allows(set, component) => {
            read.push(at(
                &named,
                Code::from(&core_codes::INVALID_VALUE),
                Severity::Error,
            ));
            true
        }
        _ => false,
    }
}

/// The client collects the extension point names of a root by reading `attribs["name"]` as a
/// string, and a missing key or another type throws out of the whole load without a log line.
/// The crate tells an error that refuses the root instead, at the control (`schema::after_read`
/// of [`Bound`]), when the control is otherwise read without one: no member it does not bind.
pub(crate) fn extension_point(bound: &mut Bound, node: &Node<'_>, read: &mut Read<'_>) {
    let Bound::ExtensionPoint(point) = bound else {
        return;
    };
    let bound_only = node.value().as_object().is_some_and(|members| {
        members
            .iter()
            .all(|(key, _)| matches!(key.as_str(), "component" | "attribs"))
    });
    let named = point
        .attribs
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|attribs| attribs.get("name"))
        .is_some_and(Value::is_string);
    if in_root(node) && bound_only && !named {
        read.push(at(
            node,
            Code::from(&core_codes::INVALID_VALUE),
            Severity::Error,
        ));
    }
}

/// Reads `payload` at `at` as `T`, by `profile`, on an engine of version `engine`: the
/// payload a [`Node`] with its text. A payload the untyped reader cannot read (not reached: the
/// envelope probe read the file) is the reader's `MalformedInput`, spanning it. Any error told
/// refuses the file.
fn document<T: FromJson>(
    payload: JsonSlice<'_>,
    engine: FormatVersion,
    profile: Profile,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<T> {
    let parsed = match bedrock_addon_core::json::parse::parse_slice(
        payload,
        bedrock_addon_core::json::parse::Options::default(),
    ) {
        Ok(parsed) => parsed,
        Err(failure) => {
            let finding = Diagnostic::of(builders::syntax(failure.kind.to_string()))
                .at(at.clone())
                .span(span_of(payload));
            d.push(finding);
            return None;
        }
    };
    let start = d.len();
    let value = {
        let mut read = Read::new(READING, d).engine(engine).profile(profile);
        read_document::<T>(&Node::parsed(&parsed, at.clone()), &mut read)
    };
    let failed = d.iter().skip(start).any(|f| f.severity == Severity::Error);
    value.filter(|_| !failed)
}

/// Reads the payload of a form style: a `description` and its `components`, the members of
/// each object in declaration order after its unknown ones.
pub(crate) fn style(payload: JsonSlice<'_>, at: &JsonPath, d: &mut Diagnostics) -> Option<UiStyle> {
    document(
        payload,
        FormatVersion::default(),
        Profile::DECLARED_ORDER,
        at,
        d,
    )
}

/// Reads the payload of a root: `description`, `attribs` and `layout`, all required, the
/// members of each object in file order.
pub(crate) fn root(
    payload: JsonSlice<'_>,
    engine: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<UiRoot> {
    document(payload, engine, Profile::EVERY_FAILURE, at, d)
}

/// Reads the payload of a composition: `description`, `attribs` and `layout`, all required.
pub(crate) fn composition(
    payload: JsonSlice<'_>,
    engine: FormatVersion,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<UiComposition> {
    document(payload, engine, Profile::EVERY_FAILURE, at, d)
}
