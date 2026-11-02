//! JSON UI: the files a resource pack lists in `ui/_ui_defs.json`, merged across the stack the
//! way the client merges them, and the checks the client runs when it
//! builds a control, run over every resolved definition (`UiDefinitions`, `UiFile`, `Control`,
//! `GlobalVariables`). The family is unversioned: no file carries a `format_version` and
//! nothing is gated on an engine version.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub use control::{
    AnimationType, Binding, BindingCondition, BindingType, ButtonMapping, ButtonMappingType,
    Control, ControlKind, ControlType, FontSize, InputModeCondition, MappingScope, Orientation,
    TextAlignment, TileDirection, Variables,
};
pub use globals::{COLOR_VARIABLES, ENGINE_VARIABLES, GlobalVariables};
pub use key::{DefKey, DefRef};

mod chain;
mod control;
mod expr;
mod globals;
mod key;
mod layout;
mod lint;
mod load;
mod merge;
mod props;
mod resolve;
mod value;
mod write;

use bedrock_addon_core::pack_identity::PackRef;
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;

#[doc(hidden)]
pub use load::load;

/// The path of the list of UI files, in every pack.
pub const UI_DEFS_PATH: &str = "ui/_ui_defs.json";
/// The path of the global variables, in every pack.
pub const GLOBAL_VARIABLES_PATH: &str = "ui/_global_variables.json";
/// The namespace of a file that names none.
pub const ROOT_NAMESPACE: &str = "_root";

/// The JSON UI of a resource stack: every listed file merged across the packs, the resolved
/// definitions by namespace and name, the global variables and what each pack got wrong.
///
/// A pack's `ui/_ui_defs.json` adds files to the list; each listed file is merged from the
/// lowest pack up; a definition's `@base` chain is flattened into it; of two definitions with
/// one namespace and name the first wins.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
#[doc(alias = "_ui_defs.json")]
#[doc(alias = "UIDefRepository")]
pub struct UiDefinitions {
    /// The listed files, sorted by path, each as merged from every pack's copy. A listed file
    /// no pack has, or whose lowest copy is not JSON, is absent.
    pub files: IndexMap<String, UiFile>,
    /// The resolved definitions: namespace, then name.
    pub namespaces: IndexMap<String, IndexMap<String, Control>>,
    /// The merged `ui/_global_variables.json`.
    pub global_variables: GlobalVariables,
    /// What the merge reported against each pack (the client's pack report), top pack first;
    /// only packs with a report are listed.
    pub reports: Vec<UiPackReport>,
}

impl UiDefinitions {
    /// The definition `name` of `namespace`.
    #[must_use]
    pub fn get(&self, namespace: &str, name: &str) -> Option<&Control> {
        self.namespaces.get(namespace)?.get(name)
    }

    /// Every resolved definition, namespace by namespace.
    pub fn controls(&self) -> impl Iterator<Item = &Control> + '_ {
        self.namespaces.values().flat_map(IndexMap::values)
    }

    /// The definition a reference names, read from a file of `namespace`.
    #[must_use]
    pub fn resolve(&self, namespace: &str, reference: &DefRef) -> Option<&Control> {
        self.get(
            reference.namespace.as_deref().unwrap_or(namespace),
            &reference.name,
        )
    }
}

/// One UI file: an optional namespace and its definitions.
///
/// JSON: `{"namespace": "hud", "name@namespace.base": {…}, …}`. Every member but `namespace`
/// is a definition, a control (it has a `type`, or inherits one) or an animation (it has an
/// `anim_type`), held as the JSON it was written as.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiFile {
    /// The namespace of the file's definitions (JSON `namespace`); without one they are in
    /// `_root`.
    pub namespace: Option<String>,
    /// The definitions in file order, by key. A `null` member is kept and defines nothing.
    pub defs: IndexMap<DefKey, Value>,
}

impl UiFile {
    /// The namespace the file's definitions are in: its own, or `_root` when it has none or
    /// an empty one.
    // The game asserts "Namespace should not be empty at this point.", so an empty namespace
    // is read as `_root`, the same as an absent one.
    #[must_use]
    pub fn namespace_or_root(&self) -> &str {
        match self.namespace.as_deref() {
            None | Some("") => ROOT_NAMESPACE,
            Some(namespace) => namespace,
        }
    }

    /// The members that define something (every member that is not `null`), in the order the
    /// game reads them: by key.
    pub fn definitions(&self) -> impl Iterator<Item = (&DefKey, &Value)> + '_ {
        let mut defs: Vec<(String, &DefKey, &Value)> = self
            .defs
            .iter()
            .filter(|(_, body)| !body.is_null())
            // A `namespace` member that is no string is kept for write-back and defines
            // nothing, as a string one defines nothing.
            .filter(|(key, _)| key.base.is_some() || key.name != "namespace")
            .map(|(key, body)| (key.to_string(), key, body))
            .collect();
        defs.sort_by(|a, b| a.0.cmp(&b.0));
        defs.into_iter().map(|(_, key, body)| (key, body))
    }
}

/// The list of UI files a pack adds (`ui/_ui_defs.json`).
///
/// JSON: `{"ui_defs": ["ui/hud_screen.json", …]}`. The lists of every pack are united; a pack
/// can add files and never remove one.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "_ui_defs.json")]
pub struct UiDefsList {
    /// The pack-relative paths (JSON `ui_defs`). An element that is not a string is read as
    /// `""`.
    pub ui_defs: Vec<String>,
    /// The other members of the file; the game reads none. Also `ui_defs` as the file wrote
    /// it, when the typed list cannot say it (no array, an element that is no string, an
    /// empty array): it is written back while the typed list still is its reading.
    pub extra: Extra,
}

/// What the merge reported against one pack.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct UiPackReport {
    /// The pack.
    pub pack: PackRef,
    /// Its `uiPackError`s, in the order raised.
    pub errors: Vec<UiPackError>,
}

/// One `uiPackError` of the client's pack report.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct UiPackError {
    /// Which error.
    pub kind: UiPackErrorKind,
    /// The arguments of its text: the file name first.
    pub args: Vec<String>,
    /// An error rolls the pack's copy of the file back; a warning does not.
    pub severity: Severity,
}

/// The kinds of `uiPackError`, in the order of the game's numbering (1 to 9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum UiPackErrorKind {
    /// `uiPackError.invalidChildNames`: an overlay introduced empty or repeated child names.
    InvalidChildNames,
    /// `uiPackError.parseError`: an overlay copy is not JSON.
    ParseError,
    /// `uiPackError.missingControl`: an overlay key or path names no control.
    MissingControl,
    /// `uiPackError.missingControlTarget`: a name condition matched no element.
    MissingControlTarget,
    /// `uiPackError.missingArrayName`: neither `array_name` nor `control_name`.
    MissingArrayName,
    /// `uiPackError.missingCondition`: an operation without the condition it needs.
    MissingCondition,
    /// `uiPackError.missingValue`: an insert or replace without `value`.
    MissingValue,
    /// `uiPackError.missingOperation`: no `operation`.
    MissingOperation,
    /// `uiPackError.invalidOperationName`: an unknown `operation`.
    InvalidOperationName,
}

impl UiPackErrorKind {
    /// The localisation key (`uiPackError.parseError`).
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::InvalidChildNames => "uiPackError.invalidChildNames",
            Self::ParseError => "uiPackError.parseError",
            Self::MissingControl => "uiPackError.missingControl",
            Self::MissingControlTarget => "uiPackError.missingControlTarget",
            Self::MissingArrayName => "uiPackError.missingArrayName",
            Self::MissingCondition => "uiPackError.missingCondition",
            Self::MissingValue => "uiPackError.missingValue",
            Self::MissingOperation => "uiPackError.missingOperation",
            Self::InvalidOperationName => "uiPackError.invalidOperationName",
        }
    }

    /// The English text of the vanilla resource pack, with `{}` for each argument.
    // The vanilla resource pack's `texts/en_US.lang` has these lines; the `missingArrayName`
    // line ends with a space there.
    #[must_use]
    pub fn template(self) -> &'static str {
        match self {
            Self::InvalidChildNames => {
                "{}: Child controls must have unique names: {}In the future this will be an error and this file won't load."
            }
            Self::ParseError => "{}: Unable to parse UI JSON file with stack: {}",
            Self::MissingControl => "{}: Can't find the control: {}",
            Self::MissingControlTarget => {
                "{} ({}): Can't find the control '{}' during operation '{}'"
            }
            Self::MissingArrayName => {
                "{} ({}): Missing either the field 'array_name' or 'control_name'. "
            }
            Self::MissingCondition => "{} ({}): Missing condition on operation '{}'.",
            Self::MissingValue => "{} ({}): Missing the value for the operation '{}'.",
            Self::MissingOperation => "{} ({}): Missing operation.",
            Self::InvalidOperationName => "{} ({}): Invalid operation '{}'.",
        }
    }
}

/// The English text, as the pack report shows it.
impl std::fmt::Display for UiPackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut args = self.args.iter();
        let mut parts = self.kind.template().split("{}");
        if let Some(first) = parts.next() {
            f.write_str(first)?;
        }
        for part in parts {
            f.write_str(args.next().map_or("", String::as_str))?;
            f.write_str(part)?;
        }
        Ok(())
    }
}

impl std::error::Error for UiPackError {}
