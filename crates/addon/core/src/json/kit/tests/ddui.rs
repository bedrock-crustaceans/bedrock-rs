//! ddui's form style reader in the kit's terms: `style` and the readers below it in
//! `families/ddui/src/read.rs` (`bound`, `member`, `required`, `single` for the description,
//! `style_components`, `form_panel`, `form_label`, `form_textfield`, `box_size`, `color`,
//! `integers`, `repeat`, `image`, `boolean`), each level a slice of the file.
//!
//! The reading is the strict document's, and the root node carries the payload's text, so
//! every finding has the span ddui gives it: the value for a wrong type or an unknown member,
//! the object for a missing one. What ddui words itself (a conversion's text, the namespace
//! rule, an unknown component) goes through [`Read::push`] at the node.

use facet_value::VNumber;

use super::*;
use crate::diagnostic::codes as core_codes;
use crate::id::syntax::has_namespace;

const NAMESPACE_TEXT: &str = "string must be prefixed with a namespace (eg. namespace:value)";
const BOX_SIZE_2_TEXT: &str = "Expected [top, right].";
const BOX_SIZE_4_TEXT: &str = "Expected [top, right, bottom, left].";
const COLOR_RGBA_TEXT: &str = "Expected [r, g, b, a], with values in the 0-255 range, or a hex value with 8 characters (e.g. ARGB as #FFFFFFFF).";
const COLOR_RGB_TEXT: &str = "Expected [r, g, b], with values in the 0-255 range, or a hex value with 6 characters (e.g. RGB as #FFFFFF).";
const COMPONENTS: [&str; 3] = [
    "minecraft:form_panel",
    "minecraft:form_label",
    "minecraft:form_textfield",
];

/// An error with `code` at `node`, spanning it.
fn error(read: &mut Read<'_>, node: &Node<'_>, code: Code) {
    let mut finding = Diagnostic::error(code).at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
}

/// `Reader::bound`: the members, each key outside `known` an unknown member.
fn bound<'v>(read: &mut Read<'_>, node: &Node<'v>, known: &[&str]) -> Option<Members<'v>> {
    let members = read.object(node)?;
    read.unknown(&members, known, Disposition::Keep);
    Some(members)
}

/// `Reader::single` for the description: the namespaced `identifier`, the other members
/// unknown, told in file order.
fn description(read: &mut Read<'_>, node: &Node<'_>) -> Option<String> {
    let members = read.object(node)?;
    let mut identifier = None;
    for (key, member) in members.iter() {
        if key == "identifier" {
            identifier = read.string(&member).filter(|text| {
                let accepted = has_namespace(text);
                if !accepted {
                    error(
                        read,
                        &member,
                        core_codes::INVALID_VALUE_WORDED.with([NAMESPACE_TEXT]),
                    );
                }
                accepted
            });
        } else {
            read.unknown_member(&member, Disposition::Keep);
        }
    }
    // Absent: told after the walk, spanning the object.
    if !members.contains("identifier") {
        read.required(&members, "identifier");
    }
    identifier.map(str::to_owned)
}

/// `Reader::integers`: integer tokens; a wrong type at an element that is none.
fn integers(read: &mut Read<'_>, node: &Node<'_>) -> Option<Vec<i64>> {
    let elements = read.array(node)?;
    let mut values = Vec::with_capacity(elements.len());
    let mut complete = true;
    for element in elements.iter() {
        let integer = element
            .value()
            .as_number()
            .filter(|number| !number.is_float())
            .and_then(VNumber::to_i64);
        if let Some(value) = integer {
            values.push(value);
        } else {
            read.wrong_type(&element);
            complete = false;
        }
    }
    complete.then_some(values)
}

/// `Reader::box_size`: two or four `i32`s; another count is the conversion's text.
fn box_size(read: &mut Read<'_>, node: &Node<'_>) -> Option<Vec<i32>> {
    let values = integers(read, node)?;
    let ints: Option<Vec<i32>> = values.iter().map(|v| i32::try_from(*v).ok()).collect();
    let size = ints.filter(|ints| matches!(ints.len(), 2 | 4));
    if size.is_none() {
        let text = if values.len() <= 2 {
            BOX_SIZE_2_TEXT
        } else {
            BOX_SIZE_4_TEXT
        };
        error(read, node, core_codes::INVALID_VALUE_WORDED.with([text]));
    }
    size
}

/// `style::hex_bytes`: `#` and `2 * n` hex digits.
fn hex(text: &str, n: usize) -> bool {
    text.strip_prefix('#').is_some_and(|digits| {
        digits.len() == 2 * n && digits.bytes().all(|b| b.is_ascii_hexdigit())
    })
}

/// `Reader::color`: a hex string or three or four bytes.
fn color(read: &mut Read<'_>, node: &Node<'_>) -> Option<String> {
    let value = node.value();
    if value.is_string() {
        let text = read.string(node)?;
        if hex(text, 4) || hex(text, 3) {
            return Some(text.to_owned());
        }
        let near_rgb = text.chars().count() == 7;
        let text = if near_rgb {
            COLOR_RGB_TEXT
        } else {
            COLOR_RGBA_TEXT
        };
        error(read, node, core_codes::INVALID_VALUE_WORDED.with([text]));
        return None;
    }
    if value.is_array() {
        let values = integers(read, node)?;
        let bytes: Option<Vec<u8>> = values.iter().map(|v| u8::try_from(*v).ok()).collect();
        let color = bytes.filter(|bytes| matches!(bytes.len(), 3 | 4));
        if color.is_none() {
            let text = if values.len() == 3 {
                COLOR_RGB_TEXT
            } else {
                COLOR_RGBA_TEXT
            };
            error(read, node, core_codes::INVALID_VALUE_WORDED.with([text]));
        }
        return color.map(|bytes| format!("{bytes:?}"));
    }
    read.wrong_type(node);
    None
}

/// `Reader::repeat`: `"stretch"` or `"repeat"`.
fn repeat(read: &mut Read<'_>, node: &Node<'_>) -> Option<String> {
    let name = read.string(node)?;
    if !matches!(name, "stretch" | "repeat") {
        error(read, node, Code::from(&core_codes::INVALID_VALUE));
        return None;
    }
    Some(name.to_owned())
}

/// `Reader::image`: a string.
fn image(read: &mut Read<'_>, node: &Node<'_>) -> Option<String> {
    read.string(node).map(str::to_owned)
}

/// `Reader::member`: the member read by `each`, when it is written.
fn member<'v, T>(
    read: &mut Read<'_>,
    members: &Members<'v>,
    key: &str,
    each: impl FnOnce(&mut Read<'_>, &Node<'v>) -> Option<T>,
) -> Option<T> {
    let node = read.opt(members, key)?;
    each(read, &node)
}

/// `Reader::required`: the member read by `each`, or missing.
fn required<'v, T>(
    read: &mut Read<'_>,
    members: &Members<'v>,
    key: &str,
    each: impl FnOnce(&mut Read<'_>, &Node<'v>) -> Option<T>,
) -> Option<T> {
    let node = read.required(members, key)?;
    each(read, &node)
}

/// `minecraft:form_panel`.
fn form_panel(read: &mut Read<'_>, node: &Node<'_>) -> Option<Vec<Option<String>>> {
    let known = [
        "padding",
        "image",
        "image_repeat_top_bottom",
        "image_repeat_right_left",
    ];
    let members = bound(read, node, &known)?;
    Some(vec![
        member(read, &members, "padding", box_size).map(|size| format!("{size:?}")),
        member(read, &members, "image", image),
        member(read, &members, "image_repeat_top_bottom", repeat),
        member(read, &members, "image_repeat_right_left", repeat),
    ])
}

/// `minecraft:form_label`.
fn form_label(read: &mut Read<'_>, node: &Node<'_>) -> Option<(Option<String>, Option<bool>)> {
    let members = bound(read, node, &["color", "shadow"])?;
    Some((
        member(read, &members, "color", color),
        member(read, &members, "shadow", |read, node| read.bool(node)),
    ))
}

/// `minecraft:form_textfield`: four colours required, five images.
fn form_textfield(read: &mut Read<'_>, node: &Node<'_>) -> Option<Vec<Option<String>>> {
    let known = [
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
    let members = bound(read, node, &known)?;
    let colors: Vec<Option<String>> = known[..4]
        .iter()
        .map(|key| required(read, &members, key, color))
        .collect();
    if colors.iter().any(Option::is_none) {
        return None;
    }
    let mut style = colors;
    for key in &known[4..] {
        style.push(member(read, &members, key, image));
    }
    Some(style)
}

/// `Reader::style_components`: at least one component; another key an unknown component.
fn style_components(read: &mut Read<'_>, node: &Node<'_>) -> Option<()> {
    let members = read.object(node)?;
    for (key, member) in members.iter() {
        if !COMPONENTS.contains(&key) {
            error(
                read,
                &member,
                Code::from(&core_codes::UNRECOGNIZED_COMPONENT),
            );
        }
    }
    let [panel, label, textfield] = COMPONENTS;
    member(read, &members, panel, form_panel);
    member(read, &members, label, form_label);
    member(read, &members, textfield, form_textfield);
    if members.is_empty() {
        error(read, node, Code::from(&core_codes::INVALID_VALUE));
        return None;
    }
    Some(())
}

/// `read::style`.
fn style(read: &mut Read<'_>, payload: &Node<'_>) -> Option<(String, ())> {
    let members = bound(read, payload, &["description", "components"])?;
    let identifier = required(read, &members, "description", description);
    let components = required(read, &members, "components", style_components);
    if read.failed() {
        return None;
    }
    Some((identifier?, components?))
}

#[test]
fn ddui_s_style_reader_reads_as_the_kit() {
    assert_reads_as(FIXTURES, EXPECTED, |text, found| {
        let json = value(text);
        let payload = Node::with_text(&json, at(&["minecraft:ui-style"]), JsonSlice::new(text));
        let mut read = Read::new(Reading::DOCUMENT, found);
        style(&mut read, &payload).is_some()
    });
}

/// Style payloads after `addon/tests/ddui/style.rs` (`s01`, `s04` to `s07` and the
/// conversions).
const FIXTURES: &[&str] = &[
    r#"{ "description": { "identifier": "demo:style" }, "components": { } }"#,
    r#"{ "description": { "identifier": "demo:style" }, "parts": { "minecraft:form_label": {} } }"#,
    r#"{ "description": { "identifier": "demo:style" }, "components": { "minecraft:form_label": {}, "minecraft:form_button": {} } }"#,
    r#"{ "description": { "identifier": "demo:style" }, "components": { "minecraft:form_panel": { "margin": [1, 2] } } }"#,
    r#"{ "description": { "identifier": "demo:style", "name": "x" }, "components": { "minecraft:form_label": {} } }"#,
    r#"{ "description": { "identifier": "demo:style" }, "components": { "minecraft:form_label": { "shadow": null } } }"#,
    r#"{ "description": { "identifier": "demo:style" }, "components": { "minecraft:form_label": { "color": null } } }"#,
    r#"{ "description": { "identifier": "demo:style" }, "components": { "minecraft:form_panel": { "image": null } } }"#,
    r#"{ "description": { "identifier": "demo:style" }, "components": { "minecraft:form_panel": null } }"#,
    r##"{ "description": { "identifier": "demo:style" }, "components": { "minecraft:form_panel": { "padding": [2, 4], "image": "textures/ui/form_bg", "image_repeat_top_bottom": "repeat", "image_repeat_right_left": "stretch" }, "minecraft:form_label": { "color": "#FF101010", "shadow": false } } }"##,
    r#"{ "description": { "identifier": "demo:style" }, "components": { "minecraft:form_panel": { "padding": [1, "x"], "image_repeat_top_bottom": "sideways", "image": 3 } } }"#,
    r#"{ "description": { "identifier": "demo:style" }, "components": { "minecraft:form_panel": { "padding": [1, 2, 3] } } }"#,
    r##"{ "description": { "identifier": "style" }, "components": { "minecraft:form_label": { "color": "#12", "shadow": 1 } } }"##,
    r#"{ "description": { "identifier": "demo:style" }, "components": { "minecraft:form_label": { "color": 5 } } }"#,
    r#"{ "description": { "identifier": "demo:style" }, "components": { "minecraft:form_label": { "color": [1, 2, 300] } } }"#,
    r"[]",
    r#"{ "zz": 1 }"#,
    r#"{ "description": "demo:style", "components": [] }"#,
    r#"{ "description": { }, "components": { "minecraft:form_textfield": { "text_color": [1, 2, 3], "panel_image": "p", "caret": 1 } } }"#,
    r##"{ "description": { "identifier": "demo:style" }, "components": { "minecraft:form_textfield": { "text_color": [255, 255, 255, 255], "caret_color": [255, 255, 255], "selection_background_color": "#3366FF", "text_color_disabled": "#80FFFFFF", "default_image": "textures/ui/tf" } } }"##,
];

/// What `read::style` tells on each of `FIXTURES`, and whether it reads it.
const EXPECTED: &[(bool, &[&str])] = &[
    // { "description": { "identifier": "demo:style" }, "components": { } }
    (
        false,
        &[r"Error|minecraft:ui-style/components|InvalidValue { text: None }|Some((63, 3))|false"],
    ),
    // { "description": { "identifier": "demo:style" }, "parts": { "minecraft:form_label": {} } }
    (
        false,
        &[
            r"Error|minecraft:ui-style/parts|UnrecognizedField { text: None }|Some((58, 30))|false",
            r"Error|minecraft:ui-style/components|MissingRequiredField { text: None }|Some((0, 90))|false",
        ],
    ),
    // { "description": { "identifier": "demo:style" }, "components": { "minecraft:form_label": {}, "minecr
    (
        false,
        &[
            r"Error|minecraft:ui-style/components/minecraft:form_button|UnrecognizedComponent { text: None }|Some((118, 2))|false",
        ],
    ),
    // { "description": { "identifier": "demo:style" }, "components": { "minecraft:form_panel": { "margin":
    (
        false,
        &[
            r"Error|minecraft:ui-style/components/minecraft:form_panel/margin|UnrecognizedField { text: None }|Some((101, 6))|false",
        ],
    ),
    // { "description": { "identifier": "demo:style", "name": "x" }, "components": { "minecraft:form_label"
    (
        false,
        &[
            r"Error|minecraft:ui-style/description/name|UnrecognizedField { text: None }|Some((55, 3))|false",
        ],
    ),
    // { "description": { "identifier": "demo:style" }, "components": { "minecraft:form_label": { "shadow":
    (
        false,
        &[
            r"Error|minecraft:ui-style/components/minecraft:form_label/shadow|InvalidType { text: None }|Some((101, 4))|false",
        ],
    ),
    // { "description": { "identifier": "demo:style" }, "components": { "minecraft:form_label": { "color":
    (
        false,
        &[
            r"Error|minecraft:ui-style/components/minecraft:form_label/color|InvalidType { text: None }|Some((100, 4))|false",
        ],
    ),
    // { "description": { "identifier": "demo:style" }, "components": { "minecraft:form_panel": { "image":
    (
        false,
        &[
            r"Error|minecraft:ui-style/components/minecraft:form_panel/image|InvalidType { text: None }|Some((100, 4))|false",
        ],
    ),
    // { "description": { "identifier": "demo:style" }, "components": { "minecraft:form_panel": null } }
    (
        false,
        &[
            r"Error|minecraft:ui-style/components/minecraft:form_panel|InvalidType { text: None }|Some((89, 4))|false",
        ],
    ),
    // { "description": { "identifier": "demo:style" }, "components": { "minecraft:form_panel": { "padding"
    (true, &[]),
    // { "description": { "identifier": "demo:style" }, "components": { "minecraft:form_panel": { "padding"
    (
        false,
        &[
            r"Error|minecraft:ui-style/components/minecraft:form_panel/padding/1|InvalidType { text: None }|Some((106, 3))|false",
            r"Error|minecraft:ui-style/components/minecraft:form_panel/image|InvalidType { text: None }|Some((160, 1))|false",
            r"Error|minecraft:ui-style/components/minecraft:form_panel/image_repeat_top_bottom|InvalidValue { text: None }|Some((139, 10))|false",
        ],
    ),
    // { "description": { "identifier": "demo:style" }, "components": { "minecraft:form_panel": { "padding"
    (
        false,
        &[
            r#"Error|minecraft:ui-style/components/minecraft:form_panel/padding|InvalidValue { text: Some(Fixed("Expected [top, right, bottom, left].")) }|Some((102, 9))|false"#,
        ],
    ),
    // { "description": { "identifier": "style" }, "components": { "minecraft:form_label": { "color": "#12"
    (
        false,
        &[
            r#"Error|minecraft:ui-style/description/identifier|InvalidValue { text: Some(Fixed("string must be prefixed with a namespace (eg. namespace:value)")) }|Some((33, 7))|false"#,
            r#"Error|minecraft:ui-style/components/minecraft:form_label/color|InvalidValue { text: Some(Fixed("Expected [r, g, b, a], with values in the 0-255 range, or a hex value with 8 characters (e.g. ARGB as #FFFFFFFF).")) }|Some((95, 5))|false"#,
            r"Error|minecraft:ui-style/components/minecraft:form_label/shadow|InvalidType { text: None }|Some((112, 1))|false",
        ],
    ),
    // { "description": { "identifier": "demo:style" }, "components": { "minecraft:form_label": { "color":
    (
        false,
        &[
            r"Error|minecraft:ui-style/components/minecraft:form_label/color|InvalidType { text: None }|Some((100, 1))|false",
        ],
    ),
    // { "description": { "identifier": "demo:style" }, "components": { "minecraft:form_label": { "color":
    (
        false,
        &[
            r#"Error|minecraft:ui-style/components/minecraft:form_label/color|InvalidValue { text: Some(Fixed("Expected [r, g, b], with values in the 0-255 range, or a hex value with 6 characters (e.g. RGB as #FFFFFF).")) }|Some((100, 11))|false"#,
        ],
    ),
    // []
    (
        false,
        &[r"Error|minecraft:ui-style|InvalidType { text: None }|Some((0, 2))|false"],
    ),
    // { "zz": 1 }
    (
        false,
        &[
            r"Error|minecraft:ui-style/zz|UnrecognizedField { text: None }|Some((8, 1))|false",
            r"Error|minecraft:ui-style/description|MissingRequiredField { text: None }|Some((0, 11))|false",
            r"Error|minecraft:ui-style/components|MissingRequiredField { text: None }|Some((0, 11))|false",
        ],
    ),
    // { "description": "demo:style", "components": [] }
    (
        false,
        &[
            r"Error|minecraft:ui-style/description|InvalidType { text: None }|Some((17, 12))|false",
            r"Error|minecraft:ui-style/components|InvalidType { text: None }|Some((45, 2))|false",
        ],
    ),
    // { "description": { }, "components": { "minecraft:form_textfield": { "text_color": [1, 2, 3], "panel_
    (
        false,
        &[
            r"Error|minecraft:ui-style/description/identifier|MissingRequiredField { text: None }|Some((17, 3))|false",
            r"Error|minecraft:ui-style/components/minecraft:form_textfield/caret|UnrecognizedField { text: None }|Some((122, 1))|false",
            r"Error|minecraft:ui-style/components/minecraft:form_textfield/caret_color|MissingRequiredField { text: None }|Some((66, 59))|false",
            r"Error|minecraft:ui-style/components/minecraft:form_textfield/selection_background_color|MissingRequiredField { text: None }|Some((66, 59))|false",
            r"Error|minecraft:ui-style/components/minecraft:form_textfield/text_color_disabled|MissingRequiredField { text: None }|Some((66, 59))|false",
        ],
    ),
    // { "description": { "identifier": "demo:style" }, "components": { "minecraft:form_textfield": { "text
    (true, &[]),
];
