//! The per-pack load context: what the game's loaders know about the pack a document is in.

use std::fmt;
use std::sync::Arc;

use indexmap::IndexMap;
use uuid::Uuid;

use super::{BuiltIns, Experiment, Experiments, LoadOptions};
use crate::id::{
    AtmosphereSettingsId, BiomeId, ColorGradingSettingsId, LightingSettingsId, WaterSettingsId,
};
use crate::molang::{ExperimentMask, RawVersion};
use crate::pack_identity::PackType;
use crate::version::{
    BaseGameVersion, ENGINE_VERSION, FormatVersion, MinEngineVersion, SemVersion,
};

/// The per-pack inputs a document load depends on (the game's per-pack load context): the pack's
/// `min_engine_version`, Molang version, experiments, whether it is a base-game pack, the
/// engine version.
///
/// Built per pack by the stack loaders; [`detached`](Self::detached) gives one for a file
/// outside any stack, whose fields can then be set (`cx.beta_apis = true`).
#[derive(Clone)]
#[non_exhaustive]
#[allow(clippy::struct_excessive_bools)] // vanilla's flags, one field each
pub struct LoadContext {
    /// The pack manifest's `min_engine_version`; `None` when absent or invalid.
    pub min_engine_version: Option<MinEngineVersion>,
    /// The pack's Molang version, mapped once from `min_engine_version`: the version of every
    /// Molang field whose version comes from the pack. Invalid (−1) without a
    /// `min_engine_version`.
    pub molang_version: RawVersion,
    /// [`experiments`](Self::experiments) as molang's experiment mask, for compiling.
    pub molang_experiments: ExperimentMask,
    /// The pack is a base-game pack: one the game ships (`PackOrigin::Package`).
    pub is_base_game_pack: bool,
    /// The pack is trusted (vanilla's flag of the same name; nothing here sets it).
    pub is_trusted_pack: bool,
    /// The pack's header UUID.
    pub pack_uuid: Uuid,
    /// The pack's type.
    pub pack_type: PackType,
    /// The pack's manifest has a script module its side's script host builds a plugin from
    /// (the game's manifest checks for plugins and client scripts), in a language and with an
    /// entry the host does not drop before it looks at a file. Set by `bedrock_addon`'s stack
    /// loaders; `false` on a context nothing set it on. A walk's
    /// [`in_pack`](super::Walk::in_pack) rule reads it to leave the script files of every
    /// other pack unread.
    pub runs_scripts: bool,
    /// The world's enabled experiments.
    pub experiments: Experiments,
    /// The Beta APIs toggle: allows `"format_version": "beta"` documents.
    pub beta_apis: bool,
    /// The Upcoming Creator Features toggle.
    pub upcoming_creator_features: bool,
    /// The engine version the crate tracks ([`ENGINE_VERSION`]).
    pub engine_version: FormatVersion,
    /// The world's base-game version (client biomes skip base-game `biomes_client.json` from
    /// 1.21.40, smithing-trim recipes need 1.20).
    pub world_base_game_version: BaseGameVersion,
    /// The render-configuration identifiers per biome that client-biome upgrades insert;
    /// `None` gives every biome vanilla's defaults.
    pub client_render_identifiers: Option<Arc<IndexMap<BiomeId, BiomeRenderIdentifiers>>>,
    /// The Molang compile check, shared from `LoadOptions::molang`.
    #[cfg(feature = "molang")]
    #[cfg_attr(docsrs, doc(cfg(feature = "molang")))]
    pub molang: Option<Arc<crate::molang::MolangCheck>>,
    /// The null rule of the document being read: set by the versioning load from the slice's
    /// `Family::reading`, read by `versioning::parse_payload` and `parse_component`; `None`
    /// (each read takes `null` as it does) outside a versioned load and for a family of the
    /// first run's input kind.
    pub null_rule: Option<crate::json::NullRule>,
    /// How the scalars of the document being read are read: set by the versioning load from
    /// the slice's `Family::reading` (`Reading::scalar_rules`), read by
    /// `versioning::parse_payload` and `parse_component` (the scalar pre-pass,
    /// `json::parse_with_rules`); [`ScalarRules::PLAIN`](crate::json::ScalarRules::PLAIN)
    /// outside a versioned load.
    pub scalar_rules: crate::json::ScalarRules,
    /// Whether the unknown members of the document's structs are kept in its table where a
    /// struct does not say otherwise: set by the versioning load from the slice's
    /// `Family::reading` (`Reading::keeps_unknown`), read by `versioning::parse_payload` and
    /// `parse_component`; `false` outside a versioned load.
    pub keeps_unknown: bool,
    /// The severity an unknown member a struct rejects or ignores (`schema::unknown =
    /// "reject"`, `"ignore"`) is told at: an error where the slice's unknown members are
    /// (`UnknownMask::Error`), else a warning. Set by the versioning load from
    /// `Reading::unknown`; a warning outside it. (An unknown member told as unrecognized is a
    /// warning the strict mask makes an error after the parse.)
    pub unknown_severity: crate::diagnostic::Severity,
    /// Whether the document is read by a cereal slice of a Minecraft Document family (the
    /// slice's null rule is N2, whatever the source): `parse_component` then rejects the
    /// document for a component whose typed read fails (a type error) or whose required member
    /// is missing, under both of the game's log masks. Set by the versioning
    /// load from `Reading::components_fatal`; `false` outside it, for `JsonCpp` slices and for a family's hand-written slices.
    pub strict_components: bool,
    /// What the document's bytes were before the loader read them: JSON text, or a
    /// binary `\x7FMCB` document the stack turned into text. The game reads an MCB as a
    /// binary input: no null rule, no JSON size check, and the item loader's MCB path meets the
    /// versioned loaders' beta gate (`Family::beta_permission`).
    pub source: DocumentSource,
    /// The JSON document size limit (`load::MAX_JSON_DOCUMENT_BYTES`); the warning comes from
    /// three quarters of it.
    /// Lowered only to test the refusal without a 128 MiB text.
    pub max_json_document_bytes: usize,
    /// The host's provider, shared from `LoadOptions::built_ins`
    /// ([`built_ins`](Self::built_ins), [`set_built_ins`](Self::set_built_ins)).
    built_ins: Option<Arc<dyn BuiltIns>>,
}

/// Prints the providers (`built_ins`, `molang`) as `present` or `absent`, as
/// [`LoadOptions`] does, so neither has to be `Debug`.
impl fmt::Debug for LoadContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let present = |present: bool| if present { "present" } else { "absent" };
        let mut debug = f.debug_struct("LoadContext");
        debug
            .field("min_engine_version", &self.min_engine_version)
            .field("molang_version", &self.molang_version)
            .field("molang_experiments", &self.molang_experiments)
            .field("is_base_game_pack", &self.is_base_game_pack)
            .field("is_trusted_pack", &self.is_trusted_pack)
            .field("pack_uuid", &self.pack_uuid)
            .field("pack_type", &self.pack_type)
            .field("runs_scripts", &self.runs_scripts)
            .field("experiments", &self.experiments)
            .field("beta_apis", &self.beta_apis)
            .field("upcoming_creator_features", &self.upcoming_creator_features)
            .field("engine_version", &self.engine_version)
            .field("world_base_game_version", &self.world_base_game_version)
            .field("client_render_identifiers", &self.client_render_identifiers);
        #[cfg(feature = "molang")]
        debug.field("molang", &present(self.molang.is_some()));
        debug
            .field("null_rule", &self.null_rule)
            .field("scalar_rules", &self.scalar_rules)
            .field("keeps_unknown", &self.keeps_unknown)
            .field("unknown_severity", &self.unknown_severity)
            .field("strict_components", &self.strict_components)
            .field("source", &self.source)
            .field("max_json_document_bytes", &self.max_json_document_bytes)
            .field("built_ins", &present(self.built_ins.is_some()))
            .finish()
    }
}

/// What a document's bytes were before the loader read them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DocumentSource {
    /// JSON text, as written.
    #[default]
    Json,
    /// A binary `\x7FMCB` document, turned into JSON text by the stack.
    Mcb,
}

impl LoadContext {
    /// A context for a file outside any stack: engine version `ENGINE_VERSION`, no
    /// experiments, Molang version of the engine, not a base-game pack.
    ///
    /// The pack is the nil UUID of type `Invalid` with `min_engine_version` and the world's
    /// base-game version both the engine version.
    #[must_use]
    pub fn detached() -> Self {
        let engine = MinEngineVersion::new(
            ENGINE_VERSION.major,
            ENGINE_VERSION.minor,
            ENGINE_VERSION.patch,
        );
        let mut cx = Self::for_pack(
            &LoadOptions::default(),
            Uuid::nil(),
            PackType::Invalid,
            Some(engine),
            false,
        );
        cx.world_base_game_version = BaseGameVersion::new(SemVersion::new(
            ENGINE_VERSION.major,
            ENGINE_VERSION.minor,
            ENGINE_VERSION.patch,
        ));
        cx
    }

    /// The context of one pack of a stack loaded with `options` (built by
    /// `bedrock_addon`'s loaders).
    ///
    /// The Beta APIs and Upcoming Creator Features toggles and their experiments are one
    /// setting each: either one turns on both.
    #[doc(hidden)]
    pub fn for_pack(
        options: &LoadOptions,
        pack_uuid: Uuid,
        pack_type: PackType,
        min_engine_version: Option<MinEngineVersion>,
        is_base_game_pack: bool,
    ) -> Self {
        let mut experiments = options.experiments.clone();
        let beta_apis = options.beta_apis || experiments.contains(Experiment::BetaApis);
        let upcoming_creator_features = options.upcoming_creator_features
            || experiments.contains(Experiment::UpcomingCreatorFeatures);
        if beta_apis {
            experiments.insert(Experiment::BetaApis);
        }
        if upcoming_creator_features {
            experiments.insert(Experiment::UpcomingCreatorFeatures);
        }
        let molang_version = crate::molang::pack_molang_version(min_engine_version.as_ref());
        Self {
            min_engine_version,
            molang_version,
            molang_experiments: experiments.molang_mask(),
            is_base_game_pack,
            is_trusted_pack: false,
            pack_uuid,
            pack_type,
            runs_scripts: false,
            experiments,
            beta_apis,
            upcoming_creator_features,
            engine_version: ENGINE_VERSION,
            world_base_game_version: options.base_game_version.clone(),
            client_render_identifiers: None,
            #[cfg(feature = "molang")]
            molang: options.molang.clone(),
            null_rule: None,
            scalar_rules: crate::json::ScalarRules::PLAIN,
            keeps_unknown: false,
            unknown_severity: crate::diagnostic::Severity::Warning,
            strict_components: false,
            source: DocumentSource::Json,
            max_json_document_bytes: super::MAX_JSON_DOCUMENT_BYTES,
            built_ins: options.built_ins.clone(),
        }
    }

    /// What answers for the game's own content: the host's provider when the stack's
    /// options gave one, else the generated tables
    /// ([`GeneratedBuiltIns`](crate::vanilla::GeneratedBuiltIns)).
    pub fn built_ins(&self) -> &dyn BuiltIns {
        static TABLES: crate::vanilla::GeneratedBuiltIns = crate::vanilla::GeneratedBuiltIns;
        match &self.built_ins {
            Some(host) => host.as_ref(),
            None => &TABLES,
        }
    }

    /// The level sound event `name` names, in the provider's spelling (the tables': lower
    /// case), compared without ASCII case; `None` when it names none.
    pub fn level_sound_event(&self, name: &str) -> Option<crate::id::LevelSoundEventName> {
        self.built_ins()
            .level_sound_event(name)
            .map(crate::id::LevelSoundEventName::parse)
    }

    /// The host's provider, `None` when the host gave none: for the decisions that depend on
    /// whether a host answers at all (the link pass's views).
    pub fn host_built_ins(&self) -> Option<&Arc<dyn BuiltIns>> {
        self.built_ins.as_ref()
    }

    /// Gives the context a host's provider, or takes it away (a detached context has none).
    pub fn set_built_ins(&mut self, built_ins: Option<Arc<dyn BuiltIns>>) {
        self.built_ins = built_ins;
    }
}

/// The render-configuration identifiers the client renderer records for one biome, which
/// client-biome upgrades insert into documents written before those components existed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BiomeRenderIdentifiers {
    /// `minecraft:atmosphere_identifier`.
    pub atmosphere: AtmosphereSettingsId,
    /// `minecraft:color_grading_identifier`.
    pub color_grading: ColorGradingSettingsId,
    /// `minecraft:lighting_identifier`.
    pub lighting: LightingSettingsId,
    /// `minecraft:water_identifier`.
    pub water: WaterSettingsId,
}
