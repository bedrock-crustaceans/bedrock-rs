//! The generated round trip: a few thousand documents over every member's forms (right,
//! wrong-typed, missing, unknown, misplaced), on both catalogues and both pack kinds. Each
//! one that loads writes text that equals its input as JSON values, member for member (but
//! for a number of an `attribs` the read replaced with a warning), and
//! loads again to an equal value; each one that does not load is refused with an error.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Content;
use bedrock_addon::families::ddui::{Catalogue, Component};

use super::common::*;
use bedrock_addon_test_support::json;

/// A small deterministic generator (an LCG; the low bits are dropped), and what the document
/// being generated is for: most of its choices are ones that reader takes.
struct Rng {
    state: u64,
    catalogue: Catalogue,
    builtin: bool,
}

impl Rng {
    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state >> 33
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).expect("a remainder below `n`, a `usize`")
    }

    fn one_in(&mut self, n: usize) -> bool {
        self.below(n) == 0
    }

    fn pick<'a>(&mut self, pool: &[&'a str]) -> &'a str {
        pool[self.below(pool.len())]
    }
}

/// What an `attribs` may hold: every JSON type, numbers at and past the edges of the float
/// and integer types (an integer beyond 64 bits is the double it spells), strings
/// with escapes.
const VALUES: [&str; 30] = [
    "{}",
    "[]",
    "null",
    "true",
    "false",
    "0",
    "-0",
    "1",
    "-1",
    "1.5",
    "0.1",
    "1e2",
    "1E+2",
    "-1e-7",
    "3.4028235e38",
    "1e39",
    "1.7976931348623157e308",
    "5e-324",
    "9007199254740993",
    "18446744073709551615",
    "-9223372036854775808",
    "18446744073709551616",
    "123456789012345678901234567890",
    "340282366920938463463374607431768211456",
    r#"[[], "12345678901234567890", -9223372036854775809]"#,
    r#""text""#,
    r#""a\"b\\c\n""#,
    r#""é ☃""#,
    r#"{ "$path": "$.minecraft.custom_form_data_{instanceId}", "n": [1, { "k": null }, [[]]] }"#,
    r#"{ "name": "demo:slot" }"#,
];

/// Numbers no double holds, alone and inside a value: an `attribs` holding one refuses the
/// file (the game's JSON parser rejects a number too big).
const REWRITTEN: [&str; 3] = [
    "1e999",
    "-1e999",
    r#"{ "list": [0, { "n": -1.8e308 }], "s": "x" }"#,
];

const IDENTIFIERS: [&str; 9] = [
    r#""demo:screen""#,
    r#""demo:screen""#,
    r#""minecraft:screen""#,
    r#""demo:minecraft_x""#,
    r#""a:b:c""#,
    r#""screen""#,
    r#""demo:""#,
    "5",
    "null",
];

const ROOT_POINTS: [&str; 10] = [
    r#""minecraft:screen""#,
    r#""minecraft:screen""#,
    r#""minecraft:screen""#,
    r#""minecraft:screen""#,
    r#""minecraft:screen""#,
    r#""minecraft:screen""#,
    r#""demo:hud""#,
    r#""""#,
    "7",
    "[]",
];

const EXTENSION_POINTS: [&str; 5] = [r#""demo:slot""#, r#""x""#, r#""""#, "null", "{}"];

/// The members of an object in a random order, some dropped, sometimes with an unknown one.
fn object(mut members: Vec<(String, String)>, drop_one_in: usize, rng: &mut Rng) -> String {
    if rng.one_in(drop_one_in) && !members.is_empty() {
        members.remove(rng.below(members.len()));
    }
    if rng.one_in(40) {
        let unknown = rng.pick(&["extra", "Component", "markup2", "", "attrib"]);
        members.push((unknown.to_owned(), rng.pick(&VALUES).to_owned()));
    }
    for index in (1..members.len()).rev() {
        members.swap(index, rng.below(index + 1));
    }
    let written: Vec<String> = members
        .iter()
        .map(|(key, value)| format!(r#""{key}": {value}"#))
        .collect();
    format!("{{ {} }}", written.join(", "))
}

/// Whether `name` is a control that mostly has no children.
fn is_leaf(name: &str) -> bool {
    matches!(
        name,
        "Panel.Text"
            | "Panel.Decoration"
            | "Panel.Spacing"
            | "Panel.CloseButton"
            | "ExtensionPoint"
            | "Form.Button"
            | "Form.Divider"
            | "Form.Dropdown"
            | "Form.Slider"
            | "Form.Switch"
            | "Form.TextField"
            | "Form.Image"
            | "Container.Item"
            | "Container.Slot"
            | "Container.TextField"
    )
}

/// The names a child of `name` is mostly drawn from.
fn controls_below(name: &str) -> &'static [&'static str] {
    match name {
        "Context" | "Container.Layout" => &["Panel", "Context", "Container.Layout"],
        "Context.List" | "ScrollableGridLayout" => &[
            "Container.Item",
            "Container.Slot",
            "Form.Button",
            "Panel.Text",
            "Context.List",
            "Visibility",
            "Form.Image",
        ],
        "Container.FixedGridLayout" => &["Container.Item", "Container.Slot", "Context.List"],
        _ => &[
            "Form.Button",
            "Form.Divider",
            "Panel.Text",
            "Panel.Spacing",
            "ExtensionPoint",
            "Visibility",
            "Form.ScrollView",
            "Context.List",
            "Container.FixedGridLayout",
            "ScrollableGridLayout",
            "Form.Image",
            "Panel.CloseButton",
        ],
    }
}

/// A control tree: mostly what `likely` (the names that fit the place) offers, sometimes any
/// name, a name of no catalogue, or no object at all.
fn control(rng: &mut Rng, likely: &[&str], depth: usize) -> String {
    if rng.one_in(60) {
        return rng.pick(&VALUES).to_owned();
    }
    let name = if rng.one_in(12) {
        if rng.one_in(3) {
            "Form.Label"
        } else {
            Component::ALL[rng.below(Component::ALL.len())].name()
        }
    } else {
        // A name of the place that the catalogue has, when a few draws find one.
        let mut name = rng.pick(likely);
        for _ in 0..8 {
            if Component::from_name(name).is_some_and(|c| rng.catalogue.has(c)) {
                break;
            }
            name = rng.pick(likely);
        }
        name
    };
    let mut members = vec![(
        "component".to_owned(),
        if rng.one_in(50) {
            "5".to_owned()
        } else {
            format!(r#""{name}""#)
        },
    )];
    let leaf = is_leaf(name);
    let no_attribs = matches!(
        name,
        "Panel.Decoration" | "Panel.Spacing" | "Form.Divider" | "Container.Layout"
    );
    let attribs = if name == "Panel.Text" {
        !rng.one_in(15)
    } else if no_attribs {
        rng.one_in(15)
    } else {
        rng.one_in(2)
    };
    if attribs {
        let value = if rng.one_in(12) {
            rng.pick(&REWRITTEN)
        } else {
            rng.pick(&VALUES)
        };
        members.push(("attribs".to_owned(), value.to_owned()));
    }
    let children = if leaf {
        rng.one_in(20)
    } else {
        !rng.one_in(20)
    };
    if children {
        let below = controls_below(name);
        let value = if rng.one_in(40) {
            rng.pick(&VALUES).to_owned()
        } else {
            let count = if depth >= 4 { 0 } else { rng.below(4) };
            let list: Vec<String> = (0..count).map(|_| control(rng, below, depth + 1)).collect();
            format!("[{}]", list.join(", "))
        };
        members.push(("children".to_owned(), value));
    }
    object(members, 80, rng)
}

/// One document: `(text, is a root)`.
fn document(rng: &mut Rng) -> (String, bool) {
    let root = !rng.one_in(3);
    let top: &[&str] = if root {
        &["Panel", "Context", "Container.Layout"]
    } else {
        &[
            "Form.Button",
            "Form.Divider",
            "Visibility",
            "ExtensionPoint",
            "Context.List",
        ]
    };
    let count = if rng.one_in(25) { 0 } else { 1 + rng.below(3) };
    let markup: Vec<String> = (0..count).map(|_| control(rng, top, 0)).collect();
    let layout = if rng.one_in(60) {
        rng.pick(&VALUES).to_owned()
    } else {
        object(
            vec![("markup".to_owned(), format!("[{}]", markup.join(", ")))],
            60,
            rng,
        )
    };
    let identifier = match (rng.one_in(6), rng.builtin) {
        (true, _) => rng.pick(&IDENTIFIERS),
        (false, true) => r#""minecraft:screen""#,
        (false, false) => r#""demo:screen""#,
    };
    let description = object(
        vec![("identifier".to_owned(), identifier.to_owned())],
        40,
        rng,
    );
    let attribs = if root {
        object(
            vec![("root_point".to_owned(), rng.pick(&ROOT_POINTS).to_owned())],
            40,
            rng,
        )
    } else if rng.one_in(3) {
        object(
            vec![(
                "extension_point".to_owned(),
                rng.pick(&EXTENSION_POINTS).to_owned(),
            )],
            1000,
            rng,
        )
    } else {
        "{}".to_owned()
    };
    let payload = object(
        vec![
            ("description".to_owned(), description),
            ("attribs".to_owned(), attribs),
            ("layout".to_owned(), layout),
        ],
        40,
        rng,
    );
    let key = if root {
        "minecraft:ui-root"
    } else {
        "minecraft:ui-composition"
    };
    let version = if rng.one_in(50) {
        rng.pick(&[r#""1.21.120""#, r#""beta""#, "1", r#""abc""#])
    } else {
        r#""1.21.130""#
    };
    let text = object(
        vec![
            ("format_version".to_owned(), version.to_owned()),
            (key.to_owned(), payload),
        ],
        80,
        rng,
    );
    (text, root)
}

/// One document through the property: `None` when it was refused, otherwise whether the read
/// replaced a number of an `attribs` (never now: such a file is refused, which
/// `Some(true)` now counts as the refusal for the number).
fn check<T: Content + PartialEq + std::fmt::Debug>(
    text: &str,
    cx: &bedrock_addon::load::LoadContext,
    path: &'static str,
) -> Option<bool> {
    let path = bedrock_addon::storage::PackPath::from_static(path);
    match T::parse(&path, text.as_bytes(), cx) {
        Ok(document) => {
            assert!(!document.diagnostics().has_errors(), "{text}");
            let written = document
                .to_json()
                .unwrap_or_else(|error| panic!("loaded and not written: {error}\n{text}"));
            // A document that loads is written as authored: a number no double holds refuses
            // the file instead of being replaced.
            let rewritten = false;
            assert_eq!(json(&written), json(text), "as authored\n{text}\n{written}");
            let again = T::parse(&path, written.as_bytes(), cx)
                .unwrap_or_else(|rejected| panic!("{written}\n{:#?}", rejected.diagnostics));
            assert_eq!(again.value(), document.value(), "{text}");
            assert_eq!(again.to_json().expect("writes again"), written);
            // What is written is read without the rewrite finding.
            assert!(
                !again.diagnostics().iter().any(|f| f
                    .location
                    .json_path
                    .to_string()
                    .contains("/attribs")),
                "{written}"
            );
            Some(rewritten)
        }
        Err(rejected) => {
            assert!(
                rejected
                    .diagnostics
                    .iter()
                    .any(|f| f.severity == Severity::Error),
                "refused without an error\n{text}"
            );
            let too_big = rejected.diagnostics.iter().any(|f| {
                f.severity == Severity::Error
                    && f.code.to_string()
                        == "Failed to parse the Minecraft Document JSON: Number too big to be stored in double."
            });
            if too_big { Some(true) } else { None }
        }
    }
}

#[test]
fn every_generated_document_that_loads_writes_back_as_authored() {
    use bedrock_addon::families::ddui::{UiComposition, UiRoot};
    let mut rng = Rng {
        state: 0x00dd_0001,
        catalogue: Catalogue::V1_26_20,
        builtin: false,
    };
    let (mut loaded, mut refused, mut rewritten) = (0, 0, 0_usize);
    for round in 0..3000 {
        let engine = if round % 2 == 0 { E20 } else { E45 };
        rng.catalogue = Catalogue::for_engine(engine);
        rng.builtin = round % 3 == 0;
        let (text, root) = document(&mut rng);
        let cx = cx(engine, rng.builtin);
        let ok = if root {
            check::<UiRoot>(&text, &cx, "ddui/root/generated.json")
        } else {
            check::<UiComposition>(&text, &cx, "ddui/composition/generated.json")
        };
        match ok {
            Some(false) => loaded += 1,
            Some(true) => {
                refused += 1;
                rewritten += 1;
            }
            None => refused += 1,
        }
    }
    assert!(
        rewritten >= 20,
        "{rewritten} refused for a number no double holds, {loaded} loaded"
    );
    assert!(loaded >= 300, "{rewritten} of {loaded}");
    // Both sides of the property are exercised: the generator is tuned so that a good part
    // of its documents load.
    eprintln!("{loaded} loaded, {refused} refused ({rewritten} for a number no double holds)");
    // A root's nameless `ExtensionPoint` refuses the root: fewer load than
    // before.
    assert!(loaded >= 350, "{loaded} loaded, {refused} refused");
    assert!(refused >= 400, "{loaded} loaded, {refused} refused");
}

/// Numbers in `attribs` are held and written as their author wrote their value: the forms a
/// float type would round or overflow among them.
#[test]
fn every_number_form_in_attribs_writes_back_as_its_value() {
    for value in VALUES {
        let text = panel(&format!(
            r#"{{ "component": "Form.Slider", "attribs": {{ "value": {value}, "list": [{value}, {value}] }} }}"#
        ));
        assert_eq!(
            check::<bedrock_addon::families::ddui::UiRoot>(
                &text,
                &cx(E20, false),
                "ddui/root/number.json"
            ),
            Some(false),
            "{value}"
        );
    }
}
