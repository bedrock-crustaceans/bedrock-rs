//! Manifest capabilities and the registry they are checked against.

use std::fmt;
use std::sync::Arc;

use bedrock_addon_core::pack_identity::PackType;
use indexmap::IndexMap;
use uuid::Uuid;

/// One `capabilities[]` entry: a feature the pack asks the host for.
///
/// Vanilla's registry knows five names; any other name is kept as written in
/// [`Capability::Other`] and fails the registry check.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Capability {
    /// `chemistry`: the Education Edition chemistry content.
    Chemistry,
    /// `editorExtension`: the pack extends the editor.
    EditorExtension,
    /// `script_eval`: `eval` and `Function` are defined in the pack's script context.
    ScriptEval,
    /// `raytraced`: ray-traced materials.
    Raytraced,
    /// `pbr`: physically based rendering (Vibrant Visuals) materials.
    Pbr,
    /// A name vanilla's registry does not know, as written.
    Other(String),
}

impl Capability {
    /// The capability spelled `name`, matched exactly.
    pub fn from_name(name: &str) -> Capability {
        match name {
            "chemistry" => Self::Chemistry,
            "editorExtension" => Self::EditorExtension,
            "script_eval" => Self::ScriptEval,
            "raytraced" => Self::Raytraced,
            "pbr" => Self::Pbr,
            other => Self::Other(other.to_owned()),
        }
    }

    /// The JSON spelling.
    pub fn name(&self) -> &str {
        match self {
            Self::Chemistry => "chemistry",
            Self::EditorExtension => "editorExtension",
            Self::ScriptEval => "script_eval",
            Self::Raytraced => "raytraced",
            Self::Pbr => "pbr",
            Self::Other(name) => name,
        }
    }
}

/// The JSON spelling.
impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// How far the host trusts a pack that holds a capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TrustLevel {
    /// Not trusted (vanilla's lowest trust level, 0).
    NotTrusted,
    /// Trusted (1).
    Trusted,
}

/// The outcome of checking one capability against a [`CapabilityRegistry`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CapabilityValidation {
    /// The host grants the capability, at this trust level.
    Succeeded {
        /// The trust level the capability carries.
        trust: TrustLevel,
    },
    /// The host knows the capability but refuses it.
    Failed,
    /// The host does not know the capability.
    NotFound,
}

/// What a capability validator sees: the pack asking for it.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct CapabilityCx<'a> {
    /// The capability asked for.
    pub capability: &'a Capability,
    /// The pack's header `uuid`.
    pub pack_uuid: Uuid,
    /// The pack's type (from its last module).
    pub pack_type: PackType,
}

/// A capability validator: decides one capability for one pack.
pub type CapabilityValidator = Arc<dyn Fn(&CapabilityCx<'_>) -> CapabilityValidation + Send + Sync>;

/// The capabilities the host knows, which a manifest's `capabilities` are checked against
/// (vanilla's `PackCapabilityRegistry`): a name and a validator per capability.
///
/// [`CapabilityRegistry::default`] is the server default of the five vanilla names:
/// `chemistry` fails (no education toggle), `editorExtension` is not found (no editor),
/// `script_eval`, `raytraced` and `pbr` succeed untrusted. A name the registry does not hold is
/// not found.
#[derive(Clone)]
pub struct CapabilityRegistry {
    validators: IndexMap<&'static str, CapabilityValidator>,
}

impl CapabilityRegistry {
    /// A registry that knows no capability.
    pub fn empty() -> CapabilityRegistry {
        Self {
            validators: IndexMap::new(),
        }
    }

    /// The server default with the education toggle set: `chemistry` succeeds trusted when
    /// `education` is on and fails otherwise; the other four as [`CapabilityRegistry::default`].
    pub fn server(education: bool) -> CapabilityRegistry {
        // Vanilla registers exactly these five capabilities, each with a validator; these are
        // the crate's own defaults for a server.
        let mut registry = Self::empty();
        registry.insert("chemistry", move |_| {
            if education {
                CapabilityValidation::Succeeded {
                    trust: TrustLevel::Trusted,
                }
            } else {
                CapabilityValidation::Failed
            }
        });
        registry.insert("editorExtension", |_| CapabilityValidation::NotFound);
        let untrusted = |_: &CapabilityCx<'_>| CapabilityValidation::Succeeded {
            trust: TrustLevel::NotTrusted,
        };
        registry.insert("script_eval", untrusted);
        registry.insert("raytraced", untrusted);
        registry.insert("pbr", untrusted);
        registry
    }

    /// Registers (or replaces in place) the validator of the capability spelled `name`.
    pub fn insert(
        &mut self,
        name: &'static str,
        validator: impl Fn(&CapabilityCx<'_>) -> CapabilityValidation + Send + Sync + 'static,
    ) {
        self.validators.insert(name, Arc::new(validator));
    }

    /// Removes the capability spelled `name`; `true` when it was registered.
    pub fn remove(&mut self, name: &str) -> bool {
        self.validators.shift_remove(name).is_some()
    }

    /// The registered names, in registration order.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.validators.keys().copied()
    }

    /// Checks the capability in `cx`: its validator's answer, or
    /// [`CapabilityValidation::NotFound`] when no validator is registered under its name.
    pub fn validate(&self, cx: &CapabilityCx<'_>) -> CapabilityValidation {
        self.validators
            .get(cx.capability.name())
            .map_or(CapabilityValidation::NotFound, |validator| validator(cx))
    }
}

/// The server default (education off): see [`CapabilityRegistry`].
impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::server(false)
    }
}

/// The registered names.
impl fmt::Debug for CapabilityRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.validators.keys()).finish()
    }
}
