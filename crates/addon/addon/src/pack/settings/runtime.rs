//! The runtime values of one pack (the game's pack settings): its definitions applied to its
//! saved entry.

use indexmap::IndexMap;
use uuid::Uuid;

use crate::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Phase, Severity,
};
use crate::document::Document;
use crate::manifest::{DropdownOption, Manifest, Setting};
use crate::pack::{PackSide, StackSet};

use super::{SavedSettings, SettingValue, Settings, SettingsFile};

/// The current values of one pack's settings, built from the manifest's definitions and the
/// pack's saved entry (the game's pack settings).
///
/// Built by [`new`](Self::new) or [`for_pack`](Self::for_pack) and read through its accessors;
/// changed only through [`set`](Self::set), which checks a value against its definition.
#[derive(Clone, Debug, PartialEq)]
pub struct PackSettings {
    pack_id: Uuid,
    values: IndexMap<String, SettingValueAndDefault>,
    last_used: u64,
    /// The definitions, for [`set`](Self::set).
    definitions: Vec<Setting>,
}

/// A setting's current value and the default its definition gives.
#[derive(Clone, Debug, PartialEq)]
pub struct SettingValueAndDefault {
    /// The current value.
    pub value: SettingValue,
    /// The definition's default.
    pub default: SettingValue,
}

/// Why [`PackSettings::set`] refused a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SetError {
    /// The pack defines no setting of this name.
    #[error("no setting of this name")]
    UnknownName,
    /// The value is not of the setting's kind (a boolean for a slider, …).
    #[error("the value does not match the setting's type")]
    TypeMismatch,
    /// A dropdown value or a multiselect entry that names no option.
    #[error("the value names no option of the setting")]
    UnknownOption,
    /// A slider value that is NaN or infinite.
    #[error("the value is not a finite number")]
    NotFinite,
}

impl SetError {
    /// The diagnostic code of the refusal: `SettingUnknownName`, `SettingTypeMismatch` or
    /// `SettingUnknownOption`; `InvalidValue` for a number that is not finite.
    pub fn code(self) -> DiagnosticCode {
        match self {
            Self::UnknownName => DiagnosticCode::SettingUnknownName,
            Self::TypeMismatch => DiagnosticCode::SettingTypeMismatch,
            Self::UnknownOption => DiagnosticCode::SettingUnknownOption,
            Self::NotFinite => DiagnosticCode::InvalidValue { text: None },
        }
    }
}

impl PackSettings {
    /// The values of the pack `pack_id`: its `definitions` (the manifest's `settings`) applied
    /// to its `saved` entry.
    ///
    /// A label has no value. A slider takes the saved number, clamped to `[min, max]` (not
    /// snapped to `step`); a toggle the saved boolean; a dropdown the saved string when it
    /// names an option; a multiselect the saved list when every entry names an option. A saved
    /// value of another kind, or one that names no option, is ignored and the default applies;
    /// a saved name without a definition is ignored. Each of these is reported to `d` at
    /// `values/<name>`: a type mismatch, a clamp, an unknown option and an unknown
    /// name all as informations (the game is silent about each).
    ///
    /// # Examples
    ///
    /// ```
    /// use bedrock_addon::diagnostic::Diagnostics;
    /// use bedrock_addon::manifest::Setting;
    /// use bedrock_addon::pack::settings::{PackSettings, SettingValue};
    /// use uuid::Uuid;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let definitions = [Setting::Toggle {
    ///     name: "demo:glow".to_owned(),
    ///     text: "Glow".to_owned(),
    ///     default: false,
    /// }];
    /// let mut findings = Diagnostics::default();
    /// let mut settings = PackSettings::new(Uuid::from_u128(1), &definitions, None, &mut findings);
    /// assert_eq!(settings.value("demo:glow"), Some(&SettingValue::Bool(false)));
    /// assert!(settings.set("demo:glow", true.into())?);
    /// assert_eq!(settings.value("demo:glow"), Some(&SettingValue::Bool(true)));
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        pack_id: Uuid,
        definitions: &[Setting],
        saved: Option<&Settings>,
        d: &mut Diagnostics,
    ) -> Self {
        let saved_values = saved.map(|entry| &entry.values);
        let mut values = IndexMap::new();
        for definition in definitions {
            let Some(name) = definition.name() else {
                continue;
            };
            // `emplace`: a repeated name keeps the first definition.
            if values.contains_key(name) {
                continue;
            }
            let Some(default) = default_of(definition) else {
                continue;
            };
            let saved_value = saved_values.and_then(|values| values.get(name));
            let value = match saved_value.map(|value| check(definition, value.clone())) {
                Some(Ok((value, clamped))) => {
                    if clamped {
                        report(d, DiagnosticCode::SettingClamped, Severity::Info, name);
                    }
                    value
                }
                Some(Err(refused)) => {
                    // The game uses the default in silence, for a value of another type and for
                    // an unknown option alike (the other refusals are `set`'s alone), so the
                    // crate reports an information.
                    let severity = Severity::Info;
                    report(d, refused.code(), severity, name);
                    clamp_default(definition, &default)
                }
                None => clamp_default(definition, &default),
            };
            values.insert(name.to_owned(), SettingValueAndDefault { value, default });
        }
        for name in saved_values.into_iter().flat_map(IndexMap::keys) {
            if !values.contains_key(name) {
                report(
                    d,
                    DiagnosticCode::SettingUnknownName,
                    // The game ignores the saved name in silence, so the crate reports an
                    // information.
                    Severity::Info,
                    name,
                );
            }
        }
        Self {
            pack_id,
            values,
            last_used: saved.and_then(|entry| entry.last_used).unwrap_or(0),
            definitions: definitions.to_vec(),
        }
    }

    /// The values of the pack `manifest` describes, from the file its pack type keeps them in:
    /// a resource pack's from `global`, a behaviour pack's from `world`. `None` when the
    /// manifest has no `settings`, for a behaviour pack without a world (`world` is `None`; a
    /// world without a file is `Some` of an empty document), and for every other pack type.
    /// The first entry with the manifest's header UUID is the pack's.
    pub fn for_pack(
        manifest: &Manifest,
        world: Option<&SavedSettings>,
        global: &SavedSettings,
        d: &mut Diagnostics,
    ) -> Option<Self> {
        if manifest.settings.is_empty() {
            return None;
        }
        let saved = match SettingsFile::for_pack_type(manifest.pack_type())? {
            SettingsFile::World => world?,
            SettingsFile::Global => global,
        };
        let pack_id = manifest.header.uuid;
        Some(Self::new(
            pack_id,
            &manifest.settings,
            saved.get(pack_id),
            d,
        ))
    }

    /// The pack's header UUID.
    pub fn pack_id(&self) -> Uuid {
        self.pack_id
    }

    /// Every named setting in manifest order, with its value and its default; a label has no
    /// entry, and a repeated name keeps its first definition.
    pub fn values(&self) -> &IndexMap<String, SettingValueAndDefault> {
        &self.values
    }

    /// The saved entry's `last_used`, 0 without one. Vanilla stamps the time the values were
    /// last requested; nothing here has a clock.
    pub fn last_used(&self) -> u64 {
        self.last_used
    }

    /// The current value of the setting `name`.
    pub fn value(&self, name: &str) -> Option<&SettingValue> {
        self.values.get(name).map(|entry| &entry.value)
    }

    /// Changes the setting `name` to `value`; `Ok(true)` when the value changed, `Ok(false)`
    /// when it already was that.
    ///
    /// The value must fit the definition: a number for a slider (clamped to `[min, max]`), a
    /// boolean for a toggle, an option's name for a dropdown, a list of option names for a
    /// multiselect. A slider value that is NaN or infinite is refused. Otherwise nothing
    /// changes and the error says why.
    ///
    /// # Errors
    ///
    /// [`SetError`] when the pack defines no setting `name` (`UnknownName`), the value is not of the
    /// setting's kind (`TypeMismatch`), names no option (`UnknownOption`), or is a slider value that
    /// is NaN or infinite (`NotFinite`).
    // The game replaces the value whatever its type and ignores an unknown name; this crate
    // refuses both, a deliberate difference.
    pub fn set(&mut self, name: &str, value: SettingValue) -> Result<bool, SetError> {
        let definition = self
            .definitions
            .iter()
            .find(|definition| definition.name() == Some(name))
            .ok_or(SetError::UnknownName)?;
        // A NaN would compare unequal to every bound and pass the clamp unchanged; no number
        // of a JSON file is one, and none can be written back.
        if let (Setting::Slider { .. }, SettingValue::Float(x)) = (definition, &value)
            && !x.is_finite()
        {
            return Err(SetError::NotFinite);
        }
        let (value, _) = check(definition, value)?;
        let entry = self.values.get_mut(name).ok_or(SetError::UnknownName)?;
        if entry.value == value {
            return Ok(false);
        }
        entry.value = value;
        Ok(true)
    }

    /// The values that differ from their defaults, sorted by name: what a save stores for the
    /// pack (vanilla keeps them in a map ordered by name).
    pub fn changed(&self) -> IndexMap<String, SettingValue> {
        let mut changed: IndexMap<String, SettingValue> = self
            .values
            .iter()
            .filter(|(_, entry)| entry.value != entry.default)
            .map(|(name, entry)| (name.clone(), entry.value.clone()))
            .collect();
        changed.sort_unstable_keys();
        changed
    }
}

/// The default a definition gives; a label has none.
fn default_of(definition: &Setting) -> Option<SettingValue> {
    match definition {
        Setting::Slider { default, .. } => Some(SettingValue::Float(*default)),
        Setting::Toggle { default, .. } => Some(SettingValue::Bool(*default)),
        Setting::Dropdown { default, .. } => Some(SettingValue::String(default.clone())),
        Setting::Multiselect { defaults, .. } => Some(SettingValue::StringList(defaults.clone())),
        // A label, and, since `Setting` is `#[non_exhaustive]` in `bedrock_addon_core`, a kind
        // this reader does not know: no default.
        _ => None,
    }
}

/// A slider's default goes through the same clamp as a saved value; other defaults apply as
/// they are.
fn clamp_default(definition: &Setting, default: &SettingValue) -> SettingValue {
    match (definition, default) {
        (Setting::Slider { min, max, .. }, SettingValue::Float(value)) => {
            SettingValue::Float(clamp(*value, *min, *max))
        }
        _ => default.clone(),
    }
}

/// Vanilla's clamp: `x < min ? min : (max < x ? max : x)`.
fn clamp(x: f32, min: f32, max: f32) -> f32 {
    if x < min {
        min
    } else if max < x {
        max
    } else {
        x
    }
}

/// `value` as the definition accepts it (and whether a slider value was clamped), or why not:
/// a value of another kind, or a dropdown value or multiselect entry that names no option.
fn check(definition: &Setting, value: SettingValue) -> Result<(SettingValue, bool), SetError> {
    let is_option =
        |options: &[DropdownOption], name: &String| options.iter().any(|o| o.name == *name);
    match (definition, value) {
        (Setting::Slider { min, max, .. }, SettingValue::Float(x)) => {
            let clamped = clamp(x, *min, *max);
            #[allow(clippy::float_cmp)] // the clamp returns `x` itself when it is in range
            let changed = clamped != x;
            Ok((SettingValue::Float(clamped), changed))
        }
        (Setting::Toggle { .. }, value @ SettingValue::Bool(_)) => Ok((value, false)),
        (Setting::Dropdown { options, .. }, SettingValue::String(name)) => {
            if is_option(options, &name) {
                Ok((SettingValue::String(name), false))
            } else {
                Err(SetError::UnknownOption)
            }
        }
        (Setting::Multiselect { options, .. }, SettingValue::StringList(names)) => {
            if names.iter().all(|name| is_option(options, name)) {
                Ok((SettingValue::StringList(names), false))
            } else {
                Err(SetError::UnknownOption)
            }
        }
        _ => Err(SetError::TypeMismatch),
    }
}

/// A finding about the saved value `name`, at `values/<name>` of the pack's entry.
fn report(d: &mut Diagnostics, code: DiagnosticCode, severity: Severity, name: &str) {
    let mut at = JsonPath::new();
    at.push("values");
    at.push(name);
    d.push(
        Diagnostic::error(code)
            .severity(severity)
            .at(at)
            .phase(Phase::Validate),
    );
}

/// The entries of a saved-settings file that name no pack of the world: one
/// `UnresolvedReference` information of the link phase for each entry of `saved` whose
/// `pack_id` is the header UUID of no pack on the stack the file belongs to (the world file's
/// entries are behaviour packs', the global file's resource packs'), at
/// `minecraft:pack_settings/settings/<index>/pack_id` of the file, with the pack as its
/// reference detail.
///
/// Such an entry is kept and applies to nothing; a save drops none of them. The global file
/// is shared by every world, so entries for packs this world does not use are usual there.
/// An entry with the nil UUID is not told: its `pack_id` was no UUID, which the load already
/// reported.
// A proposed rule: no folder rule; the game keeps the entry and logs nothing.
pub fn unmatched_entries(saved: &Document<SavedSettings>, set: &StackSet) -> Diagnostics {
    let origin = saved.origin();
    let on_stack: std::collections::HashSet<Uuid> = match origin.side {
        PackSide::Behavior => set.behavior.packs().map(|pack| pack.id().uuid).collect(),
        PackSide::Resource => set.resource.packs().map(|pack| pack.id().uuid).collect(),
    };
    let mut found = Diagnostics::default();
    for (index, entry) in saved.settings.iter().enumerate() {
        if entry.pack_id.is_nil() || on_stack.contains(&entry.pack_id) {
            continue;
        }
        let at = JsonPath::of_keys(["minecraft:pack_settings", "settings"])
            .index(index)
            .child("pack_id");
        let finding = Diagnostic::info(DiagnosticCode::UnresolvedReference { text: None })
            .at(at)
            .phase(Phase::Link)
            .in_file(origin)
            .detail(DiagnosticDetail::Reference {
                kind: "pack",
                id: entry.pack_id.hyphenated().to_string(),
                hint: None,
            });
        found.push(finding);
    }
    found
}
