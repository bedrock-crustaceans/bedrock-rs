//! Form styles (`ddui/styling`, `minecraft:ui-style`, 1.26.60): the
//! beta-only kind, its schema, defaults, conversion texts, the strict mask, the namespace rule
//! and its place in the stack. There is no vanilla style file; every document is
//! hand-written from the game's schema.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Content, Document, Placed, Rejected};
use bedrock_addon::families::ddui::{
    BorderImageRepeat, BoxSize, FormLabelStyle, FormPanelStyle, FormTextFieldStyle, StyleColor,
    StyleComponents, UiStyle, UiStyleId,
};
use bedrock_addon::json::{EncodeError, Value};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

use super::common::*;

const NO_PARSER: &str = "unsupported version: no parser available for version ";
const BOX_2: &str = "Expected [top, right].";
const BOX_4: &str = "Expected [top, right, bottom, left].";
const RGBA: &str = "Expected [r, g, b, a], with values in the 0-255 range, or a hex value with 8 characters (e.g. ARGB as #FFFFFFFF).";
const RGB: &str = "Expected [r, g, b], with values in the 0-255 range, or a hex value with 6 characters (e.g. RGB as #FFFFFF).";

/// A style document named `identifier` with `components` (the members of the JSON object),
/// at `version` (a JSON value).
fn style_json_at(version: &str, identifier: &str, components: &str) -> String {
    format!(
        r#"{{ "format_version": {version}, "minecraft:ui-style": {{
  "description": {{ "identifier": "{identifier}" }},
  "components": {{ {components} }} }} }}"#
    )
}

/// A beta style `demo:style` with `components`.
fn style_json(components: &str) -> String {
    style_json_at(r#""beta""#, "demo:style", components)
}

/// Loads `json` as a style file with `cx`.
fn style_in(json: &str, cx: &LoadContext) -> Result<Document<UiStyle>, Rejected> {
    UiStyle::parse(
        &PackPath::from_static("ddui/styling/test.json"),
        json.as_bytes(),
        cx,
    )
}

/// Loads `json` as a style file of an add-on pack on engine 1.26.60.
fn style(json: &str) -> Result<Document<UiStyle>, Rejected> {
    style_in(json, &cx(E60, false))
}

fn json(text: &str) -> Value {
    bedrock_addon::json::from_str_jsonc::<Value>(text)
        .unwrap_or_else(|error| panic!("{text}: {error}"))
}

/// `text` loads, writes the JSON it was read from, reads back equal and writes again the same.
#[track_caller]
fn round_trips(text: &str) -> Document<UiStyle> {
    let document = loads(style(text));
    let written = document.to_json().expect("writes");
    assert_eq!(json(&written), json(text), "as authored");
    let again = loads(style(&written));
    assert_eq!(again.value(), document.value());
    assert_eq!(again.to_json().expect("writes again"), written);
    document
}

/// The findings of a refused style, without the loader's closing line.
#[track_caller]
fn refused(text: &str) -> Vec<Found> {
    let mut found = found(&rejects(style(text)));
    assert_eq!(found.pop(), Some(failed_to_load()), "the closing line");
    found
}

const PANEL: &str = r#""minecraft:form_panel": { "padding": [2, 4], "image": "textures/ui/form_bg", "image_repeat_top_bottom": "repeat", "image_repeat_right_left": "stretch" }"#;
const LABEL: &str = r##""minecraft:form_label": { "color": "#FF101010", "shadow": false }"##;
const TEXTFIELD: &str = r##""minecraft:form_textfield": { "text_color": [255, 255, 255, 255], "caret_color": [255, 255, 255], "selection_background_color": "#3366FF", "text_color_disabled": "#80FFFFFF", "default_image": "textures/ui/tf", "hovered_image": "textures/ui/tf_hover", "disabled_image": "textures/ui/tf_disabled", "focused_image": "textures/ui/tf_focus", "panel_image": "textures/ui/tf_panel" }"##;

/// s01: every member of the three components, held as written; the golden value.
#[test]
fn s01_a_style_with_every_member_loads_and_writes_back_as_authored() {
    let text = style_json(&format!("{PANEL}, {LABEL}, {TEXTFIELD}"));
    let document = round_trips(&text);
    assert!(document.diagnostics().is_empty());
    assert_eq!(document.version().written, None);
    let mut textfield = FormTextFieldStyle::new(
        StyleColor::Rgba([255, 255, 255, 255]),
        StyleColor::Rgb([255, 255, 255]),
        StyleColor::HexRgb("#3366FF".to_owned()),
        StyleColor::HexArgb("#80FFFFFF".to_owned()),
    );
    textfield.default_image = Some("textures/ui/tf".to_owned());
    textfield.hovered_image = Some("textures/ui/tf_hover".to_owned());
    textfield.disabled_image = Some("textures/ui/tf_disabled".to_owned());
    textfield.focused_image = Some("textures/ui/tf_focus".to_owned());
    textfield.panel_image = Some("textures/ui/tf_panel".to_owned());
    let expected = UiStyle::new(
        UiStyleId::parse("demo:style"),
        StyleComponents {
            form_panel: Some(FormPanelStyle {
                padding: Some(BoxSize::Two([2, 4])),
                image: Some("textures/ui/form_bg".to_owned()),
                image_repeat_top_bottom: Some(BorderImageRepeat::Repeat),
                image_repeat_right_left: Some(BorderImageRepeat::Stretch),
            }),
            form_label: Some(FormLabelStyle {
                color: Some(StyleColor::HexArgb("#FF101010".to_owned())),
                shadow: Some(false),
            }),
            form_textfield: Some(textfield),
        },
    );
    assert_eq!(*document.value(), expected);
    assert_eq!(document.components.len(), 3);
}

/// Defaults: a label that writes neither member has opaque black text with a
/// shadow; the write-back leaves both out.
#[test]
fn s02_a_label_without_members_is_opaque_black_with_a_shadow() {
    let text = style_json(r#""minecraft:form_label": {}"#);
    let document = round_trips(&text);
    let label = document.components.form_label.as_ref().unwrap();
    assert_eq!(label.color, None);
    assert_eq!(label.shadow, None);
    assert_eq!(label.color(), StyleColor::Rgba([0, 0, 0, 255]));
    assert_eq!(label.color(), FormLabelStyle::DEFAULT_COLOR);
    assert!(label.shadow());
    assert_eq!(label.color().rgba(), Some([0, 0, 0, 255]));
    // A written value wins over the default.
    let text = style_json(r#""minecraft:form_label": { "color": [1, 2, 3], "shadow": false }"#);
    let label = loads(style(&text)).components.form_label.clone().unwrap();
    assert_eq!(label.color(), StyleColor::Rgb([1, 2, 3]));
    assert!(!label.shadow());
    // A panel without members is every member left out.
    let document = round_trips(&style_json(r#""minecraft:form_panel": {}"#));
    assert_eq!(
        document.components.form_panel,
        Some(FormPanelStyle::default())
    );
}

/// The beta slice alone, permitted for every pack without the toggle; any other
/// version has no parser.
#[test]
fn s03_only_beta_loads_and_every_pack_may_load_it() {
    let components = r#""minecraft:form_label": {}"#;
    for version in ["1.21.130", "1.26.59", "1.26.60", "1.26.61"] {
        let text = style_json_at(&format!(r#""{version}""#), "demo:style", components);
        assert_eq!(
            refused(&text),
            [error(
                "UnsupportedVersion",
                "format_version",
                &format!("{NO_PARSER}{version}")
            )],
            "{version}"
        );
    }
    let text = style_json(components);
    let mut no_toggle = cx(E60, false);
    no_toggle.beta_apis = false;
    let document = loads(style_in(&text, &no_toggle));
    assert_eq!(
        document.version().parsed_as,
        bedrock_addon::versioning::BETA_VERSION
    );
    assert!(document.diagnostics().is_empty());
    let shipped = text.replace("demo:style", "minecraft:style");
    assert!(style_in(&shipped, &cx(E60, true)).is_ok());
    // No `format_version` is "no valid version", a refused file.
    let text = text.replace(r#""format_version": "beta", "#, r#""other": "beta", "#);
    assert_eq!(
        refused(&text),
        [error(
            "UnsupportedVersion",
            "format_version",
            "no valid version passed by the caller or found in the input"
        )]
    );
}

/// The game's minimum size of 1: `components` holds at least one.
#[test]
fn s04_a_style_without_a_component_is_refused() {
    assert_eq!(
        refused(&style_json("")),
        [error(
            "InvalidValue",
            "minecraft:ui-style/components",
            "invalid value"
        )]
    );
    // Both members of `Style` are required.
    let text = style_json(r#""minecraft:form_label": {}"#).replace(r#""components""#, r#""parts""#);
    assert_eq!(
        refused(&text),
        [
            error(
                "UnrecognizedField",
                "minecraft:ui-style/parts",
                "unrecognized field"
            ),
            error(
                "MissingRequiredField",
                "minecraft:ui-style/components",
                "missing required field"
            ),
        ]
    );
}

/// The strict mask 0xfffc: an unknown component (`0x400`) and an unknown member (`0x4`)
/// are errors that refuse the file.
#[test]
fn s05_an_unknown_component_or_member_refuses_the_file() {
    let text = style_json(r#""minecraft:form_label": {}, "minecraft:form_button": {}"#);
    assert_eq!(
        refused(&text),
        [error(
            "UnrecognizedComponent",
            "minecraft:ui-style/components/minecraft:form_button",
            "unrecognized component"
        )]
    );
    let text = style_json(r#""minecraft:form_panel": { "margin": [1, 2] }"#);
    assert_eq!(
        refused(&text),
        [error(
            "UnrecognizedField",
            "minecraft:ui-style/components/minecraft:form_panel/margin",
            "unrecognized field"
        )]
    );
    let text = style_json(r#""minecraft:form_label": {}"#).replace(
        r#""identifier": "demo:style""#,
        r#""identifier": "demo:style", "name": "x""#,
    );
    assert_eq!(
        refused(&text),
        [error(
            "UnrecognizedField",
            "minecraft:ui-style/description/name",
            "unrecognized field"
        )]
    );
}

/// Rule N2: a member written `null` is present and fails its type.
#[test]
fn s06_a_null_member_is_a_wrong_type() {
    for (member, at) in [
        (
            r#""minecraft:form_label": { "shadow": null }"#,
            "minecraft:ui-style/components/minecraft:form_label/shadow",
        ),
        (
            r#""minecraft:form_label": { "color": null }"#,
            "minecraft:ui-style/components/minecraft:form_label/color",
        ),
        (
            r#""minecraft:form_panel": { "image": null }"#,
            "minecraft:ui-style/components/minecraft:form_panel/image",
        ),
        (
            r#""minecraft:form_panel": null"#,
            "minecraft:ui-style/components/minecraft:form_panel",
        ),
    ] {
        let got = refused(&style_json(member));
        assert_eq!(got.len(), 1, "{member}: {got:?}");
        assert_eq!(
            (got[0].0, got[0].1, got[0].2.as_str()),
            (Severity::Error, "InvalidType", at)
        );
    }
}

/// `BoxSize2D | BoxSize4D`: two or four integers; another count is the conversion's text;
/// an element that is no integer is a wrong type.
#[test]
fn s07_a_padding_is_two_or_four_integers() {
    let padding = |value: &str| {
        style_json(&format!(
            r#""minecraft:form_panel": {{ "padding": {value} }}"#
        ))
    };
    let at = "minecraft:ui-style/components/minecraft:form_panel/padding";
    let read = |value: &str| {
        loads(style(&padding(value)))
            .components
            .form_panel
            .clone()
            .unwrap()
            .padding
    };
    assert_eq!(read("[1, -2]"), Some(BoxSize::Two([1, -2])));
    assert_eq!(read("[1, 2, 3, 4]"), Some(BoxSize::Four([1, 2, 3, 4])));
    assert_eq!(BoxSize::Two([1, 2]).sides(), [1, 2, 1, 2]);
    round_trips(&padding("[0, 2147483647, -2147483648, 3]"));
    for (value, text) in [
        ("[]", BOX_2),
        ("[1]", BOX_2),
        ("[1, 2, 3]", BOX_4),
        ("[1, 2, 3, 4, 5]", BOX_4),
        ("[2147483648, 0]", BOX_2),
        ("[-2147483649, 0]", BOX_2),
    ] {
        assert_eq!(
            refused(&padding(value)),
            [error("InvalidValue", at, text)],
            "{value}"
        );
    }
    assert_eq!(
        refused(&padding("[1.5, 2]")),
        [error("InvalidType", &format!("{at}/0"), "wrong JSON type")]
    );
    assert_eq!(
        refused(&padding("[1, \"2\"]")),
        [error("InvalidType", &format!("{at}/1"), "wrong JSON type")]
    );
    assert_eq!(
        refused(&padding("4")),
        [error("InvalidType", at, "wrong JSON type")]
    );
}

/// `Color255RGBA | Color255RGB` (the game's colour type), RGBA tried first.
#[test]
fn s08_a_colour_is_rgba_or_rgb_as_integers_or_hex() {
    let label = |value: &str| {
        style_json(&format!(
            r#""minecraft:form_label": {{ "color": {value} }}"#
        ))
    };
    let at = "minecraft:ui-style/components/minecraft:form_label/color";
    let read = |value: &str| {
        loads(style(&label(value)))
            .components
            .form_label
            .clone()
            .unwrap()
            .color
            .unwrap()
    };
    assert_eq!(
        read("[0, 128, 255, 64]"),
        StyleColor::Rgba([0, 128, 255, 64])
    );
    assert_eq!(read("[0, 128, 255]"), StyleColor::Rgb([0, 128, 255]));
    let argb = read(r##""#40aBcDeF""##);
    assert_eq!(argb, StyleColor::HexArgb("#40aBcDeF".to_owned()));
    assert_eq!(argb.rgba(), Some([0xab, 0xcd, 0xef, 0x40]));
    let rgb = read(r##""#aBcDeF""##);
    assert_eq!(rgb, StyleColor::HexRgb("#aBcDeF".to_owned()));
    assert_eq!(rgb.rgba(), Some([0xab, 0xcd, 0xef, 0xff]));
    round_trips(&label(r##""#40aBcDeF""##));
    for (value, text) in [
        ("[256, 0, 0, 0]", RGBA),
        ("[-1, 0, 0, 0]", RGBA),
        ("[0, 0]", RGBA),
        ("[0, 0, 0, 0, 0]", RGBA),
        ("[256, 0, 0]", RGB),
        (r##""#12345""##, RGBA),
        (r##""#GGGGGG""##, RGB),
        (r#""FFFFFFFF""#, RGBA),
        (r#""""#, RGBA),
    ] {
        assert_eq!(
            refused(&label(value)),
            [error("InvalidValue", at, text)],
            "{value}"
        );
    }
    assert_eq!(
        refused(&label("[1, 2, 3.0]")),
        [error("InvalidType", &format!("{at}/2"), "wrong JSON type")]
    );
    assert_eq!(
        refused(&label("{}")),
        [error("InvalidType", at, "wrong JSON type")]
    );
}

/// The four colours of a text field are required.
#[test]
fn s09_a_text_field_needs_its_four_colours() {
    let text = style_json(
        r#""minecraft:form_textfield": { "text_color": [0, 0, 0], "caret_color": [0, 0, 0], "text_color_disabled": [0, 0, 0] }"#,
    );
    assert_eq!(
        refused(&text),
        [error(
            "MissingRequiredField",
            "minecraft:ui-style/components/minecraft:form_textfield/selection_background_color",
            "missing required field"
        )]
    );
    let text = style_json(
        r#""minecraft:form_textfield": { "text_color": [0, 0, 0], "caret_color": [0, 0, 0], "selection_background_color": [0, 0, 0], "text_color_disabled": [0, 0, 0] }"#,
    );
    let field = round_trips(&text)
        .components
        .form_textfield
        .clone()
        .unwrap();
    assert_eq!(field.panel_image, None);
}

/// `BorderImageRepeatType`: `stretch` (0) and `repeat` (1), compared with case.
#[test]
fn s10_an_image_repeat_is_stretch_or_repeat() {
    let panel = |value: &str| {
        style_json(&format!(
            r#""minecraft:form_panel": {{ "image_repeat_top_bottom": {value} }}"#
        ))
    };
    assert_eq!(
        BorderImageRepeat::from_name("stretch"),
        Some(BorderImageRepeat::Stretch)
    );
    assert_eq!(BorderImageRepeat::Repeat.name(), "repeat");
    round_trips(&panel(r#""repeat""#));
    let at = "minecraft:ui-style/components/minecraft:form_panel/image_repeat_top_bottom";
    for value in [r#""Stretch""#, r#""tile""#, r#""""#] {
        assert_eq!(
            refused(&panel(value)),
            [error("InvalidValue", at, "invalid value")],
            "{value}"
        );
    }
    assert_eq!(
        refused(&panel("0")),
        [error("InvalidType", at, "wrong JSON type")]
    );
}

/// The identifier's namespace constraint: the rule of roots
/// and compositions.
#[test]
fn s11_the_namespace_rules_of_roots_hold_for_styles() {
    let components = r#""minecraft:form_label": {}"#;
    let at = "minecraft:ui-style/description/identifier";
    let got = refused(&style_json_at(r#""beta""#, "nonamespace", components));
    assert_eq!(
        got,
        [error(
            "InvalidValue",
            at,
            "string must be prefixed with a namespace (eg. namespace:value)"
        )]
    );
    let vanilla = style_json_at(r#""beta""#, "minecraft:style", components);
    let got = refused(&vanilla);
    assert_eq!(
        (got[0].0, got[0].1, got[0].2.as_str()),
        (Severity::Error, "NamespaceRule", at)
    );
    assert!(
        got[0].3.starts_with(
            "external packs must not use the 'minecraft' namespace for DataDrivenUI documents"
        ),
        "{got:?}"
    );
    let got = found(&rejects(style_in(&style_json(components), &cx(E60, true))));
    assert_eq!(
        (got[0].0, got[0].1, got[0].2.as_str()),
        (Severity::Error, "NamespaceRule", at)
    );
    assert!(
        got[0].3.starts_with(
            "builtin packs must use the 'minecraft' namespace for DataDrivenUI documents"
        ),
        "{got:?}"
    );
}

/// The writer refuses what the reader refuses, with its path.
#[test]
fn s12_the_writer_refuses_what_the_reader_refuses() {
    let refused_at = |value: &UiStyle| match value.to_json() {
        Ok(text) => panic!("written: {text}"),
        Err(EncodeError::NotRepresentable { at, .. }) => at.to_string(),
        Err(other) => panic!("{other}"),
    };
    let empty = UiStyle::new(UiStyleId::parse("demo:s"), StyleComponents::default());
    assert_eq!(refused_at(&empty), "components");
    let mut bad = empty.clone();
    bad.components.form_label = Some(FormLabelStyle {
        color: Some(StyleColor::HexArgb("#123".to_owned())),
        shadow: None,
    });
    assert_eq!(refused_at(&bad), "components/minecraft:form_label/color");
    let mut nameless = bad.clone();
    nameless.description.identifier = UiStyleId::parse("x");
    assert_eq!(refused_at(&nameless), "description/identifier");
    // A built style writes text that loads equal.
    let mut built = empty;
    built.components.form_panel = Some(FormPanelStyle {
        padding: Some(BoxSize::Four([1, 2, 3, 4])),
        ..FormPanelStyle::default()
    });
    let written = built.to_json().unwrap();
    assert_eq!(*loads(style(&written)).value(), built);
}

#[test]
fn s13_the_default_path_is_below_ddui_styling() {
    let style = UiStyle::new(
        UiStyleId::parse("demo:forms/dark"),
        StyleComponents::default(),
    );
    assert_eq!(
        style.default_path().as_str(),
        "ddui/styling/demo/forms/dark.json"
    );
    let style = UiStyle::new(
        UiStyleId::parse("minecraft:dark"),
        StyleComponents::default(),
    );
    assert_eq!(style.default_path().as_str(), "ddui/styling/dark.json");
}

/// The group reads `ddui/styling` beside the other two directories; a style is stored by its
/// identifier, the lowest pack's winning as for roots; a file that is not `.json` is ignored.
#[test]
fn s14_the_stack_stores_styles_by_identifier() {
    let label = |color: &str| {
        style_json(&format!(
            r#""minecraft:form_label": {{ "color": {color} }}"#
        ))
    };
    let stack = stack(vec![
        external(
            1,
            "pack_1",
            vec![
                ("ddui/styling/s.json", label("[1, 1, 1]")),
                ("ddui/styling/notes.txt", "x".to_owned()),
            ],
        ),
        external(
            2,
            "pack_0",
            vec![("ddui/styling/other.json", label("[2, 2, 2]"))],
        ),
    ]);
    let ui = stack.data_driven_ui();
    assert_eq!(ui.styles.len(), 1);
    let (id, document) = ui.styles.iter().next().unwrap();
    assert_eq!(id.as_str(), "demo:style");
    assert_eq!(document.origin().pack.name(), "pack_0");
    assert_eq!(
        document.components.form_label.as_ref().unwrap().color,
        Some(StyleColor::Rgb([2, 2, 2]))
    );
    let codes: Vec<&str> = ui.diagnostics().map(|f| f.code.name()).collect();
    assert_eq!(codes, ["ShadowedDefinition", "IgnoredFile"]);
    assert!(ui.roots.is_empty() && ui.compositions.is_empty());
    // A refused style is no style, and its findings are the group's.
    let stack = super::common::stack(vec![external(
        1,
        "pack",
        vec![("ddui/styling/s.json", style_json(""))],
    )]);
    let ui = stack.data_driven_ui();
    assert!(ui.styles.is_empty());
    assert_eq!(found(ui.diagnostics()).last(), Some(&failed_to_load()));
}

/// `parse_as_written` gives the beta struct.
#[test]
fn s15_parse_as_written_gives_the_beta_struct() {
    use bedrock_addon::families::ddui::history::StyleAny;
    let text = style_json(r#""minecraft:form_label": {}"#);
    let path = PackPath::from_static("ddui/styling/test.json");
    let written = UiStyle::parse_as_written(&path, text.as_bytes(), &cx(E60, false)).unwrap();
    let StyleAny::Beta(value) = written.value();
    assert_eq!(value, loads(style(&text)).value());
}
