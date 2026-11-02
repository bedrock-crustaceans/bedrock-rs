//! Pack setting definitions (`settings[]`) and the checks vanilla's comprehensive validator runs
//! on them.

use std::collections::HashSet;

use crate::diagnostic::{DiagnosticCode as Code, Phase, Severity, VanillaText};
use crate::json::Value;
use crate::version::{FormatVersion, MinEngineVersion, SemVersion};

use super::read::{Sink, child, item, member, text};

/// One `settings[]` entry: a pack setting's definition (the saved values are `pack::settings`).
///
/// Format 3 manifests hold every kind; a legacy (format 1 or 2) manifest keeps only labels.
/// Vanilla picks the kind by the members an entry has, trying label, slider, toggle, dropdown
/// and multiselect in that order; `type` does not choose it.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Setting {
    /// `"type": "label"`: a line of text.
    Label {
        /// JSON `text`: the text or a localisation key.
        text: String,
    },
    /// `"type": "slider"`: a number in `[min, max]`.
    Slider {
        /// JSON `name`: `namespace:name`, not in `minecraft`.
        name: String,
        /// JSON `text`.
        text: String,
        /// JSON `min`.
        min: f32,
        /// JSON `max`: above `min`.
        max: f32,
        /// JSON `step`: above 0 and at most `max - min`.
        step: f32,
        /// JSON `default`: in `[min, max]`.
        default: f32,
    },
    /// `"type": "toggle"`: on or off.
    Toggle {
        /// JSON `name`: `namespace:name`, not in `minecraft`.
        name: String,
        /// JSON `text`.
        text: String,
        /// JSON `default`.
        default: bool,
    },
    /// `"type": "dropdown"`: one of 2 to 20 options.
    Dropdown {
        /// JSON `name`: `namespace:name`, not in `minecraft`.
        name: String,
        /// JSON `text`.
        text: String,
        /// JSON `options`: 2 to 20.
        options: Vec<DropdownOption>,
        /// JSON `default`: the `name` of an option.
        default: String,
    },
    /// `"type": "multiselect"` (1.26.50): any number of 2 to 20 options.
    Multiselect {
        /// JSON `name`: `namespace:name`, not in `minecraft`.
        name: String,
        /// JSON `text`.
        text: String,
        /// JSON `options`: 2 to 20.
        options: Vec<DropdownOption>,
        /// JSON `defaults`: at most 20 option names.
        defaults: Vec<String>,
        /// JSON `selection_text` (optional): the text shown for the selection. Left out, the
        /// game holds its member's initial value (an empty text); `None` writes nothing.
        selection_text: Option<String>,
    },
}

impl Setting {
    /// The setting's `name`; a label has none.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Label { .. } => None,
            Self::Slider { name, .. }
            | Self::Toggle { name, .. }
            | Self::Dropdown { name, .. }
            | Self::Multiselect { name, .. } => Some(name),
        }
    }

    /// The JSON `type` of this kind: `label`, `slider`, `toggle`, `dropdown`, `multiselect`.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Label { .. } => "label",
            Self::Slider { .. } => "slider",
            Self::Toggle { .. } => "toggle",
            Self::Dropdown { .. } => "dropdown",
            Self::Multiselect { .. } => "multiselect",
        }
    }
}

/// One option of a dropdown or multiselect setting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropdownOption {
    /// JSON `name`: what a saved value names; not empty.
    pub name: String,
    /// JSON `text`: what the option shows; not empty.
    pub text: String,
}

// ---- reading and checking (format 3) ----

/// The most settings a manifest may declare.
const MAX_SETTINGS: usize = 50;
/// The fewest and most options of a dropdown or multiselect.
const OPTIONS: std::ops::RangeInclusive<usize> = 2..=20;
/// The most defaults of a multiselect.
const MAX_DEFAULTS: usize = 20;
/// The engine version that introduced multiselect settings.
const MULTISELECT_SINCE: FormatVersion = FormatVersion::new(1, 26, 50);

/// The kinds in vanilla's dispatch order, with their required members and their optional
/// ones (no others).
const KINDS: [(&str, &[&str], &[&str]); 5] = [
    ("label", &["type", "text"], &[]),
    (
        "slider",
        &["type", "name", "text", "min", "max", "step", "default"],
        &[],
    ),
    ("toggle", &["type", "name", "text", "default"], &[]),
    (
        "dropdown",
        &["type", "name", "text", "options", "default"],
        &[],
    ),
    (
        "multiselect",
        &["type", "name", "text", "options", "defaults"],
        &["selection_text"],
    ),
];

/// Reads a format-3 `settings` array: 1 to 50 entries, each dispatched by shape, then the
/// semantic checks of the game's comprehensive-manifest check.
pub(super) fn read(
    sink: &mut Sink<'_>,
    root: &Value,
    min_engine: Option<&MinEngineVersion>,
) -> Vec<Setting> {
    const AT: &str = "settings";
    let Some(value) = member(root, AT) else {
        return Vec::new();
    };
    let Some(elements) = value.as_array().map(facet_value::VArray::as_slice) else {
        sink.error(Code::InvalidType { text: None }, AT);
        return Vec::new();
    };
    // An empty list is refused with "expected array with at least 1 elements", and more than
    // 50 settings are refused before any element is read. The crate reports both as
    // `InvalidValue`.
    if elements.is_empty() || elements.len() > MAX_SETTINGS {
        sink.error(Code::InvalidValue { text: None }, AT);
        return Vec::new();
    }
    let mut settings = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        if let Some(setting) = setting(sink, element, &item(AT, index)) {
            settings.push((index, setting));
        }
    }
    check(sink, &settings, min_engine);
    settings.into_iter().map(|(_, setting)| setting).collect()
}

/// One entry: the first kind (label, slider, toggle, dropdown, multiselect) whose members the
/// object has exactly, with the right JSON types; then the member constraints.
fn setting(sink: &mut Sink<'_>, element: &Value, at: &str) -> Option<Setting> {
    if element.as_object().is_none() {
        sink.error(Code::InvalidType { text: None }, at);
        return None;
    }
    let type_value = member(element, "type").and_then(text);
    // The variant is not tagged: the first alternative that loads without error wins, so the
    // shape is matched first and the constraints are checked after.
    let Some(setting) = KINDS
        .iter()
        .find_map(|(kind, required, optional)| fit(element, kind, required, optional))
    else {
        report_misfit(sink, element, at, type_value);
        return None;
    };
    if type_value != Some(setting.type_name()) {
        // `type` does not choose the kind. Vanilla is silent about a mismatch; the crate
        // reports an information.
        sink.note(Code::SettingTypeMismatch, &child(at, "type"));
    }
    constraints(sink, &setting, at);
    Some(setting)
}

/// The setting of kind `kind` when `element` has every `required` member and no member outside
/// `required` and `optional`, each with the right type.
fn fit(element: &Value, kind: &str, required: &[&str], optional: &[&str]) -> Option<Setting> {
    let object = element.as_object()?;
    if !required.iter().all(|key| object.contains_key(key))
        || !object
            .keys()
            .all(|key| required.contains(&key.as_str()) || optional.contains(&key.as_str()))
    {
        return None;
    }
    // `type` binds as a `SettingType`: one of the five names, whatever the kind.
    let type_name = member(element, "type").and_then(text)?;
    if !KINDS.iter().any(|(name, _, _)| *name == type_name) {
        return None;
    }
    let string = |key: &str| member(element, key).and_then(text).map(str::to_owned);
    let number = |key: &str| {
        #[allow(clippy::cast_possible_truncation, reason = "the members are `f32`")]
        member(element, key)
            .and_then(Value::as_number)
            .map(|n| n.to_f64_lossy() as f32)
    };
    let options = || -> Option<Vec<DropdownOption>> {
        member(element, "options")?
            .as_array()?
            .as_slice()
            .iter()
            .map(|option| {
                if option.as_object()?.len() != 2 {
                    return None;
                }
                Some(DropdownOption {
                    name: member(option, "name").and_then(text)?.to_owned(),
                    text: member(option, "text").and_then(text)?.to_owned(),
                })
            })
            .collect()
    };
    Some(match kind {
        "label" => Setting::Label {
            text: string("text")?,
        },
        "slider" => Setting::Slider {
            name: string("name")?,
            text: string("text")?,
            min: number("min")?,
            max: number("max")?,
            step: number("step")?,
            default: number("default")?,
        },
        "toggle" => Setting::Toggle {
            name: string("name")?,
            text: string("text")?,
            default: member(element, "default")?.as_bool()?,
        },
        "dropdown" => Setting::Dropdown {
            name: string("name")?,
            text: string("text")?,
            options: options()?,
            default: string("default")?,
        },
        _ => Setting::Multiselect {
            name: string("name")?,
            text: string("text")?,
            options: options()?,
            defaults: member(element, "defaults")?
                .as_array()?
                .as_slice()
                .iter()
                .map(|v| text(v).map(str::to_owned))
                .collect::<Option<_>>()?,
            // An optional member present with another type fails the alternative.
            selection_text: match member(element, "selection_text") {
                Some(value) => Some(text(value)?.to_owned()),
                None => None,
            },
        },
    })
}

/// An entry that fits no kind: the findings of the kind its `type` names: missing
/// members, unknown members, else a member of the wrong type. A `type` that names no kind is
/// reported itself.
fn report_misfit(sink: &mut Sink<'_>, element: &Value, at: &str, type_value: Option<&str>) {
    let named = type_value.and_then(|t| KINDS.iter().find(|(name, _, _)| *name == t));
    let Some((_, required, optional)) = named else {
        let type_at = child(at, "type");
        match member(element, "type") {
            None => sink.error(Code::MissingRequiredField { text: None }, &type_at),
            Some(value) if text(value).is_some() => {
                sink.error(Code::InvalidEnumValue { text: None }, &type_at);
            }
            Some(_) => sink.error(Code::InvalidType { text: None }, &type_at),
        }
        return;
    };
    let before = sink.errors();
    for key in *required {
        if member(element, key).is_none() {
            sink.error(Code::MissingRequiredField { text: None }, &child(at, key));
        }
    }
    let allowed: Vec<&str> = required.iter().chain(optional.iter()).copied().collect();
    super::comprehensive::unknown_members(sink, element, at, &allowed);
    if sink.errors() == before {
        // Every member is there and no other: one has the wrong type.
        sink.error(Code::InvalidType { text: None }, at);
    }
}

/// Member constraints of the kind: `InvalidValue` errors naming the member.
fn constraints(sink: &mut Sink<'_>, setting: &Setting, at: &str) {
    let invalid = |sink: &mut Sink<'_>, key: &str| {
        sink.error(Code::InvalidValue { text: None }, &child(at, key));
    };
    if setting.name().is_some_and(|name| !namespaced(name)) {
        // "string has invalid namespace (minecraft)" / "string must be prefixed with a
        // namespace (eg. namespace:value)".
        invalid(sink, "name");
    }
    match setting {
        Setting::Slider { step, .. } => {
            // The step must be above 0.0 (an exclusive minimum); a NaN fails it.
            if step.is_nan() || *step <= 0.0 {
                invalid(sink, "step");
            }
        }
        Setting::Dropdown {
            options, default, ..
        } => {
            option_constraints(sink, options, at);
            if default.is_empty() {
                invalid(sink, "default");
            }
        }
        Setting::Multiselect {
            options, defaults, ..
        } => {
            option_constraints(sink, options, at);
            if defaults.len() > MAX_DEFAULTS {
                invalid(sink, "defaults");
            }
        }
        Setting::Label { .. } | Setting::Toggle { .. } => {}
    }
}

/// 2 to 20 options, none with an empty name or text.
fn option_constraints(sink: &mut Sink<'_>, options: &[DropdownOption], at: &str) {
    let options_at = child(at, "options");
    if !OPTIONS.contains(&options.len()) {
        sink.error(Code::InvalidValue { text: None }, &options_at);
    }
    for (index, option) in options.iter().enumerate() {
        let entry_at = item(&options_at, index);
        if option.name.is_empty() {
            sink.error(Code::InvalidValue { text: None }, &child(&entry_at, "name"));
        }
        if option.text.is_empty() {
            sink.error(Code::InvalidValue { text: None }, &child(&entry_at, "text"));
        }
    }
}

/// The game's namespace constraint that forbids `minecraft`: `^(?:(?!^(minecraft):).)+:(?:.)+$`,
/// case-insensitive: a namespace, a colon and a name on one line, the namespace not
/// `minecraft` in any letter case.
fn namespaced(name: &str) -> bool {
    let line_break = |c: char| matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}');
    if name.chars().any(line_break) {
        return false;
    }
    if name
        .get(..10)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("minecraft:"))
    {
        return false;
    }
    let chars: Vec<char> = name.chars().collect();
    chars
        .iter()
        .enumerate()
        .any(|(i, c)| *c == ':' && i > 0 && i + 1 < chars.len())
}

/// The checks of the game's comprehensive-manifest check and the multiselect engine gate,
/// in vanilla's order per setting: the duplicate name, then the kind's own checks.
fn check(
    sink: &mut Sink<'_>,
    settings: &[(usize, Setting)],
    min_engine: Option<&MinEngineVersion>,
) {
    let mut names = HashSet::new();
    for (index, setting) in settings {
        let at = item("settings", *index);
        let fail = |sink: &mut Sink<'_>, text: VanillaText| {
            sink.push(
                Code::InvalidValue { text: Some(text) },
                Severity::Error,
                Phase::Validate,
                &at,
            );
        };
        if let Some(name) = setting.name()
            && !names.insert(name.to_owned())
        {
            fail(
                sink,
                VanillaText::DuplicatePackSettingName {
                    name: name.to_owned(),
                },
            );
        }
        match setting {
            Setting::Slider {
                name,
                min,
                max,
                step,
                default,
                ..
            } => {
                for text in slider_texts(name, *min, *max, *step, *default) {
                    fail(sink, text);
                }
            }
            Setting::Dropdown {
                name,
                options,
                default,
                ..
            } => {
                if !options.iter().any(|option| option.name == *default) {
                    fail(
                        sink,
                        VanillaText::DropdownDefaultNotInOptions {
                            name: name.clone(),
                            default: default.clone(),
                        },
                    );
                }
            }
            Setting::Multiselect {
                name,
                options,
                defaults,
                ..
            } => {
                for default in defaults {
                    if !options.iter().any(|option| option.name == *default) {
                        fail(
                            sink,
                            VanillaText::MultiselectDefaultNotInOptions {
                                name: name.clone(),
                                default: default.clone(),
                            },
                        );
                    }
                }
                // A multiselect needs engine 1.26.50. The game checks no engine version here,
                // so an older `min_engine_version` gets the crate's own information.
                if min_engine
                    .is_none_or(|engine| engine.version < SemVersion::from(MULTISELECT_SINCE))
                {
                    sink.push(
                        Code::FieldSinceEngine {
                            since: MULTISELECT_SINCE,
                        },
                        Severity::Info,
                        Phase::Validate,
                        &at,
                    );
                }
            }
            Setting::Label { .. } | Setting::Toggle { .. } => {}
        }
    }
}

/// The three slider checks, in vanilla's order. Exact comparisons (`max - min` in `f32`); a NaN
/// fails each check it takes part in.
fn slider_texts(name: &str, min: f32, max: f32, step: f32, default: f32) -> Vec<VanillaText> {
    use std::cmp::Ordering::{Equal, Less};
    let mut texts = Vec::new();
    if min.partial_cmp(&max) != Some(Less) {
        texts.push(VanillaText::SliderMaxNotAboveMin {
            name: name.to_owned(),
        });
    }
    if !matches!(step.partial_cmp(&(max - min)), Some(Less | Equal)) {
        texts.push(VanillaText::SliderStepTooLarge {
            name: name.to_owned(),
            step: step.to_string(),
        });
    }
    if default < min || !matches!(default.partial_cmp(&max), Some(Less | Equal)) {
        texts.push(VanillaText::SliderDefaultOutOfRange {
            name: name.to_owned(),
            default: default.to_string(),
        });
    }
    texts
}
