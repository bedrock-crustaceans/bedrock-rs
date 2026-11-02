//! The checks the client runs when a screen builds a control, run here over every resolved
//! definition and the children it writes. Every finding is an information:
//! vanilla logs these only when a screen instantiates the control, never at pack load, and a
//! finding with the game's text carries [`CREATION_TIME`] as its note; the crate's own
//! (`UnusedProperty`, the enum misses without a text) are informations too, because the game
//! says nothing about them at all.
//!
//! A definition is checked for the members it writes itself, with the type it has or
//! inherits; what it inherits is checked where it is written. The few inherited members a
//! check reads (`type`, `anim_type`, `from`, `to`) are looked up by key and remembered per
//! base ([`Lookups`]): no definition is flattened, so the checks of a table cost its size.

use std::collections::HashSet;

use super::chain::{Inherited, Lookups};
use super::control::{AnimationType, ControlType};
use super::expr::{Issue, tokenize};
use super::layout::{Term, Unit, parse_axis};
use super::props::{
    self, ANIMATED, DEAD_ANIMATION_KEYS, DEAD_KEYS, EnumRule, Kind, Lookup, Miss, RENDERERS,
};
use super::resolve::base_of;
use super::value::{first_key, member, number, string_or_empty};
use super::{Control, DefKey, DefRef, UiDefinitions};
use bedrock_addon_core::diagnostic::{DiagnosticCode, DiagnosticDetail, VanillaText};
use bedrock_addon_core::json::Value;

/// The note of a finding with the game's text: when the game would log it.
pub(crate) const CREATION_TIME: &str = "the game logs this only when a screen creates the control";

/// Whether `code` carries a text of the game (the creation-time texts), not one of the crate's own.
fn game_text(code: &DiagnosticCode) -> bool {
    matches!(
        code,
        DiagnosticCode::InvalidType { text: Some(_) }
            | DiagnosticCode::InvalidValue { text: Some(_) }
            | DiagnosticCode::InvalidEnumValue { text: Some(_) }
            | DiagnosticCode::MissingRequiredField { text: Some(_) }
            | DiagnosticCode::UnresolvedReference { text: Some(_) }
    )
}

/// One finding: the listed file, the breadcrumb inside it and the code.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Finding {
    pub file: String,
    pub path: Vec<String>,
    pub code: DiagnosticCode,
    pub detail: Option<DiagnosticDetail>,
}

/// Checks every definition of `definitions`. `unresolved` names the definitions whose base
/// chain did not resolve.
pub(crate) fn lint(
    definitions: &UiDefinitions,
    unresolved: &HashSet<(String, String)>,
) -> Vec<Finding> {
    let mut found = Vec::new();
    let mut lookups = Lookups::default();
    for control in definitions.controls() {
        let mut cx = Cx {
            definitions,
            control,
            found: &mut found,
            lookups: &mut lookups,
        };
        let base = Base {
            broken: unresolved.contains(&(control.namespace.clone(), control.key.name.clone())),
            namespace: base_of(
                &control.key,
                &control.raw,
                &control.namespace,
                &definitions.global_variables,
            )
            .map(|(namespace, _)| namespace),
        };
        let mut path = vec![control.key.to_string()];
        let effective = Effective {
            own: &control.raw,
            base: None,
            inherited: Some(&control.inherited),
        };
        cx.node(&control.key, effective, &base, &mut path, None);
    }
    found
}

/// A value the game resolves when it builds the control: a `$variable`, a `#binding`, an
/// expression, or a `{"__string": true, "value": …}` wrapper. Nothing is known of it here.
fn dynamic(value: &Value) -> bool {
    match value.as_string() {
        Some(text) => text.as_str().starts_with(['$', '#', '(']),
        None => member(value, "__string").is_some(),
    }
}

#[allow(clippy::unnecessary_wraps, reason = "fills the `text` field of a code")]
fn fixed(text: &'static str) -> Option<VanillaText> {
    Some(VanillaText::Fixed(text))
}

#[allow(clippy::unnecessary_wraps, reason = "fills the `text` field of a code")]
fn formatted(template: &'static str, args: &[&str]) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    })
}

/// The type-mismatch texts of the game's typed property getters.
const EXPECTED_STRING: &str = "JSON UI parse failure - expected a string.";
const EXPECTED_BOOL: &str = "JSON UI parse failure - expected a bool.";
const EXPECTED_NUMBER: &str = "JSON UI parse failure - expected a number.";
const EXPECTED_ARRAY: &str = "JSON UI parse failure - expected an array.";
const EXPECTED_ARRAY_OR_STRING: &str = "JSON UI parse failure - expected an array or a string.";
const EXPECTED_ARRAY_OR_NUMBER: &str = "JSON UI parse failure - expected an array or a number.";
const EXPECTED_OBJECT: &str = "JSON UI parse failure - expected an object.";

/// What is known of the base of the control being checked.
struct Base {
    /// The base did not resolve.
    broken: bool,
    /// The namespace the base was found in.
    namespace: Option<String>,
}

/// A control as the game builds it: what is written here over what it inherits.
#[derive(Clone, Copy)]
struct Effective<'d> {
    /// What is written here.
    own: &'d Value,
    /// The definition a child names as its base: what that one writes comes next.
    base: Option<&'d Control>,
    /// What the definition, or the child's base, inherits.
    inherited: Option<&'d Inherited>,
}

impl<'d> Effective<'d> {
    /// A value that inherits nothing: an inline animation.
    fn alone(own: &'d Value) -> Self {
        Self {
            own,
            base: None,
            inherited: None,
        }
    }
}

struct Cx<'a, 'd> {
    definitions: &'d UiDefinitions,
    control: &'d Control,
    found: &'a mut Vec<Finding>,
    lookups: &'a mut Lookups<'d>,
}

impl<'d> Cx<'_, 'd> {
    /// The member `key` of the flattened control, unless it is absent or `null`: the nearest
    /// definition's that writes the key. A `null` written here, or by the child's base,
    /// hides what is inherited, as it does in a flattened definition.
    fn effective(&mut self, of: Effective<'d>, key: &'static str) -> Option<&'d Value> {
        let written = |value: &'d Value| value.as_object().and_then(|object| object.get(key));
        match written(of.own).or_else(|| written(&of.base?.raw)) {
            Some(value) => Some(value).filter(|value| !value.is_null()),
            None => self.lookups.get(of.inherited?, key),
        }
    }

    fn raise(&mut self, path: &[String], code: DiagnosticCode) {
        let detail = game_text(&code).then_some(DiagnosticDetail::Note(CREATION_TIME));
        self.found.push(Finding {
            file: self.control.source_file.clone(),
            path: path.to_vec(),
            code,
            detail,
        });
    }

    fn raise_at(&mut self, path: &mut Vec<String>, key: &str, code: DiagnosticCode) {
        path.push(key.to_owned());
        self.raise(path, code);
        path.pop();
    }

    fn wrong_type(&mut self, path: &mut Vec<String>, key: &str, text: &'static str) {
        self.raise_at(path, key, DiagnosticCode::InvalidType { text: fixed(text) });
    }

    /// The definition a reference string names (`ns.name`, `name`, or either after an `@`).
    fn names_definition(&self, reference: &str) -> bool {
        let reference = DefRef::parse(reference.strip_prefix('@').unwrap_or(reference));
        self.definitions
            .resolve(&self.control.namespace, &reference)
            .is_some()
    }

    /// One control: a top-level definition or a child written in a `controls` array.
    /// `effective` is what is written here over the base chain, `base.broken` says the base
    /// did not resolve, and `parent` names the control that writes a child.
    fn node(
        &mut self,
        key: &DefKey,
        effective: Effective<'d>,
        base: &Base,
        path: &mut Vec<String>,
        parent: Option<&str>,
    ) {
        let own = effective.own;
        if !own.is_object() {
            return;
        }
        let broken = base.broken;
        let name = key.name.as_str();
        let animation_type = self.effective(effective, "anim_type");
        let written_type = self.effective(effective, "type");
        let mut control_type = None;
        let mut failed = false;
        match written_type {
            // Often a missing `@base`. A definition without a base is one others inherit
            // from, and is never built by itself.
            None if broken && animation_type.is_none() => {
                failed = true;
                self.raise(
                    path,
                    DiagnosticCode::MissingRequiredField {
                        text: formatted(
                            "Type not specified (or @-base not found) for control: {}",
                            &[name],
                        ),
                    },
                );
            }
            None => {}
            Some(written) if dynamic(written) => {}
            Some(written) => match written.as_string() {
                None => {
                    if member(own, "type").is_some() {
                        self.wrong_type(path, "type", EXPECTED_STRING);
                    }
                }
                Some(text) => {
                    control_type = ControlType::from_name(text.as_str());
                    // A type the game does not know is reported where the type is written.
                    if control_type.is_none() && member(own, "type").is_some() {
                        failed = true;
                        self.raise_at(
                            path,
                            "type",
                            DiagnosticCode::InvalidEnumValue {
                                text: formatted(
                                    "Type '{}' not recognized for control: {}",
                                    &[text.as_str(), name],
                                ),
                            },
                        );
                    }
                }
            },
        }
        if let (true, Some(parent)) = (failed, parent) {
            // A child that fails to be created fails its parent's creation too.
            let namespace = self.control.namespace.as_str();
            self.raise(
                path,
                DiagnosticCode::InvalidValue {
                    text: formatted(
                        "UIControlFactory parent control \"{}.{}\" failed to create child control \"{}.{}\", did you add a new type? Did you add it to the string-to-type function?",
                        &[namespace, parent, namespace, name],
                    ),
                },
            );
        }
        let is_animation = written_type.is_none() && animation_type.is_some();
        if is_animation {
            self.animation(name, effective, path);
        } else {
            for (key, value) in own.as_object().into_iter().flat_map(|object| object.iter()) {
                if !value.is_null() {
                    self.property(name, key.as_str(), value, control_type, path);
                }
            }
            self.children(name, own, path, base.namespace.as_deref());
        }
    }

    /// The children written in `own`'s `controls`.
    // An unqualified `@name` of a child is looked up in the file's namespace and then in the
    // namespace of the definition its parent inherits from: vanilla's own command block
    // screen writes `default@close_button_panel` under `minimize@common.close_button`, and
    // `close_button_panel` is `common`'s, so an unqualified base falls back to the namespace
    // of the parent's base.
    fn children(
        &mut self,
        parent: &str,
        own: &'d Value,
        path: &mut Vec<String>,
        parent_base_namespace: Option<&str>,
    ) {
        let Some(children) = member(own, "controls").and_then(Value::as_array) else {
            return;
        };
        for (index, child) in children.as_slice().iter().enumerate() {
            let Some(key_text) = first_key(child) else {
                continue;
            };
            let Some(body) = child.as_object().and_then(|object| object.get(key_text)) else {
                continue;
            };
            let key = DefKey::parse(key_text);
            let mut effective = Effective::alone(body);
            let mut base = Base {
                broken: false,
                namespace: None,
            };
            let unqualified = key
                .base
                .as_ref()
                .is_some_and(|base| base.namespace.is_none());
            // A reference with a variable this cannot resolve is the game's to resolve.
            if let Some((namespace, name)) = base_of(
                &key,
                body,
                &self.control.namespace,
                &self.definitions.global_variables,
            ) {
                let found = self.definitions.get(&namespace, &name).or_else(|| {
                    let fallback = parent_base_namespace.filter(|_| unqualified)?;
                    self.definitions.get(fallback, &name)
                });
                match found {
                    Some(found) => {
                        effective.base = Some(found);
                        effective.inherited = Some(&found.inherited);
                        base.namespace = Some(found.namespace.clone());
                    }
                    None => base.broken = true,
                }
            }
            path.extend([
                "controls".to_owned(),
                index.to_string(),
                key_text.to_owned(),
            ]);
            self.node(&key, effective, &base, path, Some(parent));
            path.truncate(path.len() - 3);
        }
    }

    /// One member a control writes.
    fn property(
        &mut self,
        name: &str,
        key: &str,
        value: &'d Value,
        control_type: Option<ControlType>,
        path: &mut Vec<String>,
    ) {
        // Variables and their defaults, and the merge's `texture:<n>` members.
        if key.starts_with('$') || key.contains('|') || key.starts_with("texture:") {
            return;
        }
        if DEAD_KEYS.contains(&key) {
            return self.raise_at(path, key, DiagnosticCode::UnusedProperty);
        }
        let kind = match props::lookup(control_type, key) {
            Lookup::Read(kind) => kind,
            // Only a type the game knows has components that can lack the key.
            Lookup::Unused => return self.raise_at(path, key, DiagnosticCode::UnusedProperty),
            Lookup::Unknown => return,
        };
        if dynamic(value) {
            return;
        }
        if ANIMATED.contains(&key) {
            // A string names an animation; an object with `anim_type` is one.
            if member(value, "anim_type").is_some() {
                path.push(key.to_owned());
                self.animation(name, Effective::alone(value), path);
                path.pop();
                return;
            }
            if value.is_string() {
                return;
            }
        }
        self.typed(key, value, kind, path);
        self.special(name, key, value, path);
    }

    /// The getter's type test and, for an enum, its string table.
    fn typed(&mut self, key: &str, value: &Value, kind: Kind, path: &mut Vec<String>) {
        let expected = match kind {
            Kind::Any => None,
            Kind::Bool => (!value.is_bool() && !value.is_number()).then_some(EXPECTED_BOOL),
            Kind::Number => (!value.is_number()).then_some(EXPECTED_NUMBER),
            Kind::Str => (!value.is_string()).then_some(EXPECTED_STRING),
            Kind::Array | Kind::Axes => (!value.is_array()).then_some(EXPECTED_ARRAY),
            Kind::Object => (!value.is_object()).then_some(EXPECTED_OBJECT),
            Kind::ArrayOrString => {
                (!value.is_array() && !value.is_string()).then_some(EXPECTED_ARRAY_OR_STRING)
            }
            Kind::ArrayOrNumber => {
                (!value.is_array() && !value.is_number()).then_some(EXPECTED_ARRAY_OR_NUMBER)
            }
            Kind::Enum(rule) => match value.as_string() {
                None => Some(EXPECTED_STRING),
                Some(text) => {
                    self.enumerated(key, text.as_str(), rule, path);
                    None
                }
            },
        };
        if let Some(expected) = expected {
            return self.wrong_type(path, key, expected);
        }
        if let (Kind::Axes, Some(axes)) = (kind, value.as_array()) {
            path.push(key.to_owned());
            for (index, axis) in axes.as_slice().iter().take(2).enumerate() {
                if dynamic(axis) {
                    continue;
                }
                if let Err(error) = parse_axis(axis) {
                    let (template, args) = error.text();
                    self.raise_at(
                        path,
                        &index.to_string(),
                        DiagnosticCode::InvalidValue {
                            text: Some(VanillaText::Formatted { template, args }),
                        },
                    );
                }
            }
            path.pop();
        }
    }

    /// A string against its enum.
    fn enumerated(&mut self, key: &str, text: &str, rule: &EnumRule, path: &mut Vec<String>) {
        if rule.values.contains(&text) {
            return;
        }
        let text = match rule.miss {
            Miss::Silent => return,
            Miss::CrateOwn => None,
            Miss::Fixed(fixed_text) => fixed(fixed_text),
            Miss::Format(template) => formatted(template, &[text]),
        };
        self.raise_at(path, key, DiagnosticCode::InvalidEnumValue { text });
    }

    /// The keys with a code path of their own.
    fn special(&mut self, name: &str, key: &str, value: &Value, path: &mut Vec<String>) {
        match key {
            "ignored" => {
                // `ignored` is a bool, an integer or a binding.
                let integral = number(value).is_some_and(|number| number.fract() == 0.0);
                if !value.is_bool() && !integral {
                    self.wrong_type(
                        path,
                        key,
                        "An invalid value was provided for \"ignored\" field.",
                    );
                }
            }
            "anims" => self.anims(value, path),
            "grid_item_template" => {
                // `grid_item_template` names a definition.
                if let Some(reference) = value.as_string()
                    && !self.names_definition(reference.as_str())
                {
                    self.unresolved(
                        path,
                        key,
                        formatted(
                            "gridItemTemplate control tree could not be created! Control: {}",
                            &[name],
                        ),
                        reference.as_str(),
                    );
                }
            }
            "bindings" => self.bindings(value, path),
            "variables" => self.variables(value, path),
            "factory" => self.factory(value, path),
            "focus_mapping" => self.focus_mapping(value, path),
            "button_mappings" => self.button_mappings(value, path),
            "sounds" => self.sounds(value, path),
            "renderer" => {
                if let Some(renderer) = value.as_string()
                    && !RENDERERS.contains(&renderer.as_str())
                {
                    self.raise_at(
                        path,
                        key,
                        DiagnosticCode::InvalidEnumValue {
                            text: formatted("Unrecognized renderer: '{}'", &[renderer.as_str()]),
                        },
                    );
                }
            }
            "state_controls" if !value.is_array() => self.wrong_type(
                path,
                key,
                "JSON UI parse failure: Must define state_controls as an array!",
            ),
            "images" if !value.is_array() => self.wrong_type(
                path,
                key,
                "JSON UI parse failure: Must define 'images' as an array!",
            ),
            "font_type" if !value.is_string() => self.wrong_type(
                path,
                key,
                "JSON UI parse failure: Expected a string for 'font_type'.",
            ),
            "backup_font_type" if !value.is_string() => self.wrong_type(
                path,
                key,
                "JSON UI parse failure: Expected a string for 'backup_font_type'.",
            ),
            "tiled" => {
                // Silent in vanilla: anything but a bool and the four strings is "none".
                let known = value.is_bool()
                    || matches!(string_or_empty(Some(value)), "x" | "y" | "xy" | "yx");
                if !known {
                    self.raise_at(path, key, DiagnosticCode::InvalidEnumValue { text: None });
                }
            }
            _ => {}
        }
    }

    fn unresolved(
        &mut self,
        path: &mut Vec<String>,
        key: &str,
        text: Option<VanillaText>,
        reference: &str,
    ) {
        path.push(key.to_owned());
        self.found.push(Finding {
            file: self.control.source_file.clone(),
            path: path.clone(),
            code: DiagnosticCode::UnresolvedReference { text },
            detail: Some(DiagnosticDetail::Reference {
                kind: "ui definition",
                id: reference.to_owned(),
                hint: None,
            }),
        });
        path.pop();
    }

    /// Every entry of `anims` names an animation definition.
    fn anims(&mut self, value: &Value, path: &mut Vec<String>) {
        let Some(entries) = value.as_array() else {
            return;
        };
        path.push("anims".to_owned());
        for (index, entry) in entries.as_slice().iter().enumerate() {
            if dynamic(entry) {
                continue;
            }
            let reference = string_or_empty(Some(entry));
            if !self.names_definition(reference) {
                self.unresolved(
                    path,
                    &index.to_string(),
                    fixed("JSON UI parse failure: Invalid anim reference!"),
                    reference,
                );
            }
        }
        path.pop();
    }

    /// The tokenizer's findings for an expression (a string starting with `(`).
    fn expression(&mut self, value: Option<&Value>, key: &str, path: &mut Vec<String>) {
        let text = string_or_empty(value);
        if !text.starts_with('(') {
            return;
        }
        for issue in tokenize(text).1 {
            let code = match issue {
                // A quote that is never closed.
                Issue::UnclosedQuote => DiagnosticCode::InvalidValue {
                    text: fixed("You didn't close your quotes >:( \n"),
                },
                Issue::BareHash => DiagnosticCode::InvalidValue {
                    text: fixed("The # sign cannot be used without providing quotes around it!"),
                },
                // `<=`, `>=` and `%` are not operators.
                Issue::UnsupportedOperator(_) => DiagnosticCode::UnsupportedOperator,
            };
            self.raise_at(path, key, code);
        }
    }

    /// The `requires` expressions of `variables`.
    fn variables(&mut self, value: &Value, path: &mut Vec<String>) {
        path.push("variables".to_owned());
        match value.as_array() {
            Some(entries) => {
                for (index, entry) in entries.as_slice().iter().enumerate() {
                    path.push(index.to_string());
                    self.expression(member(entry, "requires"), "requires", path);
                    path.pop();
                }
            }
            None => self.expression(member(value, "requires"), "requires", path),
        }
        path.pop();
    }

    /// The `bindings` array: each entry's type, condition and names.
    fn bindings(&mut self, value: &Value, path: &mut Vec<String>) {
        let Some(entries) = value.as_array() else {
            return self.wrong_type(
                path,
                "bindings",
                "JSON UI parse failure: Must define bindings as an array.",
            );
        };
        path.push("bindings".to_owned());
        for (index, entry) in entries.as_slice().iter().enumerate() {
            // Entries that are not objects, and ignored ones, are skipped silently.
            if !entry.is_object() || member(entry, "ignored").and_then(Value::as_bool) == Some(true)
            {
                continue;
            }
            path.push(index.to_string());
            self.binding(entry, path);
            path.pop();
        }
        path.pop();
    }

    /// An enum member of a binding: its string, or the fallback after a message. `None` when
    /// the game resolves the value itself.
    fn binding_enum<'v>(
        &mut self,
        entry: &'v Value,
        key: &str,
        values: &[&str],
        unknown: &'static str,
        fallback: &'static str,
        path: &mut Vec<String>,
    ) -> Option<&'v str> {
        let Some(written) = member(entry, key) else {
            return Some(fallback);
        };
        if dynamic(written) {
            return None;
        }
        match written.as_string() {
            None => {
                self.wrong_type(path, key, EXPECTED_STRING);
                Some(fallback)
            }
            Some(text) if values.contains(&text.as_str()) => Some(text.as_str()),
            Some(text) => {
                self.raise_at(
                    path,
                    key,
                    DiagnosticCode::InvalidEnumValue {
                        text: formatted(unknown, &[text.as_str()]),
                    },
                );
                Some(fallback)
            }
        }
    }

    fn binding(&mut self, entry: &Value, path: &mut Vec<String>) {
        let binding_type = self.binding_enum(
            entry,
            "binding_type",
            &["none", "global", "collection", "collection_details", "view"],
            "Unknown binding type value: {}",
            "global",
            path,
        );
        self.binding_enum(
            entry,
            "binding_condition",
            &[
                "none",
                "once",
                "always",
                "always_when_visible",
                "visible",
                "visibility_changed",
            ],
            "Unknown binding condition value: {}",
            "none",
            path,
        );
        self.expression(member(entry, "binding_name"), "binding_name", path);
        self.expression(
            member(entry, "source_property_name"),
            "source_property_name",
            path,
        );
        let missing = |key: &str| string_or_empty(member(entry, key)).is_empty();
        let mut require = |cx: &mut Self, key: &str, text: &'static str| {
            if missing(key) {
                cx.raise_at(
                    path,
                    key,
                    DiagnosticCode::MissingRequiredField { text: fixed(text) },
                );
            }
        };
        match binding_type {
            Some("global" | "collection") => {
                require(
                    self,
                    "binding_name",
                    "JSON UI parse failure: Must define a binding name!",
                );
                if binding_type == Some("collection") {
                    require(
                        self,
                        "binding_collection_name",
                        "JSON UI parse failure: Must define a collection name for the binding!",
                    );
                }
            }
            Some("view") => {
                require(
                    self,
                    "source_property_name",
                    "JSON UI parse failure: Must define a source property name in the binding!",
                );
                require(
                    self,
                    "target_property_name",
                    "JSON UI parse failure: Must define a target property name in the binding!",
                );
                let set = |key: &str| member(entry, key).and_then(Value::as_bool) == Some(true);
                if set("resolve_ancestor_scope") && set("resolve_sibling_scope") {
                    self.raise(
                        path,
                        DiagnosticCode::InvalidValue {
                            text: fixed(
                                "JSON UI parse failure: resolve_ancestor_scope and resolve_sibling_scope cannot both be set!",
                            ),
                        },
                    );
                }
            }
            // `none` adds no binding; `collection_details` requires nothing.
            _ => {}
        }
    }

    /// The `factory` component's keys.
    fn factory(&mut self, value: &Value, path: &mut Vec<String>) {
        if !value.is_object() {
            return;
        }
        path.push("factory".to_owned());
        if string_or_empty(member(value, "name")).is_empty() {
            self.raise_at(
                path,
                "name",
                DiagnosticCode::MissingRequiredField {
                    text: fixed("JSON UI parse failure: The factory name should not be empty!"),
                },
            );
        }
        if let Some(variables) = member(value, "factory_variables") {
            let all_variables = variables.as_array().is_some_and(|variables| {
                variables
                    .as_slice()
                    .iter()
                    .all(|variable| string_or_empty(Some(variable)).starts_with('$'))
            });
            if !all_variables {
                self.raise_at(
                    path,
                    "factory_variables",
                    DiagnosticCode::InvalidValue {
                        text: fixed(
                            "JSON UI parse failure: factory_variables should all be strings of json variables created higher up the tree.",
                        ),
                    },
                );
            }
        }
        if let Some(location) = member(value, "insert_location")
            && !dynamic(location)
        {
            let location = string_or_empty(Some(location));
            self.enumerated("insert_location", location, &props::INSERT_LOCATION, path);
        }
        path.pop();
    }

    /// The focus component's keys.
    fn focus_mapping(&mut self, value: &Value, path: &mut Vec<String>) {
        const DIRECTIONS: [(&str, &str); 4] = [
            (
                "focus_change_up",
                "JSON UI parse failure: Up Override set to blank string. This is redundant as blank strings imply no override.",
            ),
            (
                "focus_change_down",
                "JSON UI parse failure: Down Override set to blank string. This is redundant as blank strings imply no override.",
            ),
            (
                "focus_change_left",
                "JSON UI parse failure: Left Override set to blank string. This is redundant as blank strings imply no override.",
            ),
            (
                "focus_change_right",
                "JSON UI parse failure: Right Override set to blank string. This is redundant as blank strings imply no override.",
            ),
        ];
        let Some(entries) = value.as_array() else {
            return self.wrong_type(
                path,
                "focus_mapping",
                "JSON UI parse failure: Expected an array for property 'focus_mapping'.",
            );
        };
        path.push("focus_mapping".to_owned());
        for (index, entry) in entries.as_slice().iter().enumerate() {
            let index = index.to_string();
            if !entry.is_object() {
                self.wrong_type(
                    path,
                    &index,
                    "JSON UI parse failure: Expected an object inside array for property 'focus_mapping'.",
                );
                continue;
            }
            path.push(index);
            if member(entry, "focus_identifier").is_none() {
                self.raise_at(
                    path,
                    "focus_identifier",
                    DiagnosticCode::MissingRequiredField {
                        text: fixed("JSON UI parse failure: Expected a focus identifier."),
                    },
                );
            }
            if DIRECTIONS
                .iter()
                .all(|(direction, _)| member(entry, direction).is_none())
            {
                self.raise(
                    path,
                    DiagnosticCode::MissingRequiredField {
                        text: fixed("JSON UI parse failure: Expected a focus direction override."),
                    },
                );
            }
            for (direction, blank) in DIRECTIONS {
                let written = member(entry, direction).and_then(Value::as_string);
                if written.is_some_and(facet_value::VString::is_empty) {
                    self.raise_at(
                        path,
                        direction,
                        DiagnosticCode::InvalidValue { text: fixed(blank) },
                    );
                }
            }
            path.pop();
        }
        path.pop();
    }

    /// The input component's keys, including each `button_mappings` entry.
    fn button_mappings(&mut self, value: &Value, path: &mut Vec<String>) {
        let Some(entries) = value.as_array() else {
            return;
        };
        path.push("button_mappings".to_owned());
        for (index, entry) in entries.as_slice().iter().enumerate() {
            if !entry.is_object() {
                continue;
            }
            path.push(index.to_string());
            let mapping_type = member(entry, "mapping_type");
            if !mapping_type.is_some_and(dynamic) {
                // An absent or empty type gets the message too; the mapping is `pressed`.
                let written = string_or_empty(mapping_type);
                if !matches!(written, "global" | "pressed" | "focused" | "double_pressed") {
                    self.raise_at(
                        path,
                        "mapping_type",
                        DiagnosticCode::InvalidEnumValue {
                            text: formatted("Unrecognized mapping type String: {}", &[written]),
                        },
                    );
                }
            }
            for (key, rule) in [
                ("scope", &props::MAPPING_SCOPE),
                ("input_mode_condition", &props::INPUT_MODE),
            ] {
                if let Some(written) = member(entry, key)
                    && !dynamic(written)
                {
                    self.enumerated(key, string_or_empty(Some(written)), rule, path);
                }
            }
            path.pop();
        }
        path.pop();
    }

    /// The sound component's keys.
    fn sounds(&mut self, value: &Value, path: &mut Vec<String>) {
        let Some(entries) = value.as_array() else {
            return;
        };
        path.push("sounds".to_owned());
        for (index, entry) in entries.as_slice().iter().enumerate() {
            if let Some(event) = member(entry, "event_type")
                && !dynamic(event)
                && string_or_empty(Some(event)) != "button_event"
            {
                path.push(index.to_string());
                self.raise_at(
                    path,
                    "event_type",
                    DiagnosticCode::InvalidEnumValue {
                        text: fixed("Unrecognized sound event type.\n"),
                    },
                );
                path.pop();
            }
        }
        path.pop();
    }

    /// An animation: a definition with `anim_type`, or an inline one.
    fn animation(&mut self, name: &str, effective: Effective<'d>, path: &mut Vec<String>) {
        let own = effective.own;
        let written = self.effective(effective, "anim_type");
        let mut animation = None;
        if let Some(written) = written.filter(|written| !dynamic(written)) {
            match written.as_string() {
                None => {
                    if member(own, "anim_type").is_some() {
                        self.wrong_type(path, "anim_type", EXPECTED_STRING);
                    }
                }
                Some(text) => {
                    animation = AnimationType::from_name(text.as_str());
                    // Silent in vanilla: no animation is created.
                    if animation.is_none() && member(own, "anim_type").is_some() {
                        self.raise_at(
                            path,
                            "anim_type",
                            DiagnosticCode::InvalidEnumValue { text: None },
                        );
                    }
                }
            }
        }
        for (key, value) in own.as_object().into_iter().flat_map(|object| object.iter()) {
            let key = key.as_str();
            if value.is_null() {
                continue;
            }
            let unread = DEAD_ANIMATION_KEYS.contains(&key)
                || animation.is_some_and(|animation| !props::animation_reads(animation, key));
            if unread {
                self.raise_at(path, key, DiagnosticCode::UnusedProperty);
            }
        }
        match animation {
            Some(AnimationType::FlipBook) => {
                if member(own, "frame_count").and_then(number) == Some(0.0) {
                    self.raise_at(
                        path,
                        "frame_count",
                        DiagnosticCode::InvalidValue {
                            text: formatted(
                                "FrameCount must be non-zero! Problem found in UIControl{}",
                                &[name],
                            ),
                        },
                    );
                }
                if let Some(orientation) = member(own, "orientation").filter(|o| !dynamic(o)) {
                    self.typed(
                        "orientation",
                        orientation,
                        Kind::Enum(&props::ORIENTATION),
                        path,
                    );
                }
            }
            Some(AnimationType::Offset | AnimationType::Size) => {
                let touches = ["anim_type", "from", "to"]
                    .iter()
                    .any(|key| member(own, key).is_some());
                if touches {
                    self.layout_animation(effective, path);
                }
            }
            _ => {}
        }
    }

    /// The endpoint rules of an `offset` / `size` animation.
    fn layout_animation(&mut self, effective: Effective<'d>, path: &[String]) {
        let endpoints = (
            self.effective(effective, "from"),
            self.effective(effective, "to"),
        );
        let (Some(from), Some(to)) = endpoints else {
            return self.raise(
                path,
                DiagnosticCode::MissingRequiredField {
                    text: fixed("The animation needs to have a specified axis offset."),
                },
            );
        };
        if dynamic(from) || dynamic(to) {
            return;
        }
        let axes = |value: &Value| -> Option<[Option<Vec<Term>>; 2]> {
            let pair = value.as_array()?.as_slice();
            let axis = |index: usize| {
                let axis = pair.get(index)?;
                if dynamic(axis) {
                    None
                } else {
                    parse_axis(axis).ok()
                }
            };
            Some([axis(0), axis(1)])
        };
        let (Some(from), Some(to)) = (axes(from), axes(to)) else {
            return;
        };
        let counts = [
            "The animation needs to have the same number x values.",
            "The animation needs to have the same number y values.",
        ];
        for ((from, to), count) in from.iter().zip(&to).zip(counts) {
            let (Some(from), Some(to)) = (from, to) else {
                continue;
            };
            let text = if from.len() != to.len() {
                count
            } else if from.iter().zip(to).any(|(a, b)| a.negative != b.negative) {
                "All value's operations need to match with each other."
            } else if from
                .iter()
                .zip(to)
                .any(|(a, b)| !same_type(&a.unit, &b.unit))
            {
                "All value's types need to match with each other."
            } else {
                continue;
            };
            self.raise(path, DiagnosticCode::InvalidValue { text: fixed(text) });
        }
    }
}

/// Whether two terms of an animation's endpoints have one type. A JSON number pairs with any
/// unit: vanilla's own screen animations run from `0` to `"100%"`.
fn same_type(from: &Unit, to: &Unit) -> bool {
    from == to || *from == Unit::Number || *to == Unit::Number
}
