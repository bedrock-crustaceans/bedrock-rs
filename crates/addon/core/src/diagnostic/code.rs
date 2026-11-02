//! The one `DiagnosticCode` enum and its message table: every code, its typed payload and
//! the text `Display` renders. The vanilla content-log templates live in [`VanillaText`]; a code
//! vanilla reports with several texts, or with a family's own text, carries one in its `text`
//! field.

use std::fmt::{self, Display as _};

use crate::pack_identity::PackIdVersion;
use crate::version::FormatVersion;
use crate::visual::ShadowReason;

/// Declares [`DiagnosticCode`] and its name table from one variant list, so the names cannot
/// drift from the variants.
macro_rules! diagnostic_codes {
    // For one bound field: return it when it is the `text` field. The name is matched
    // as a literal and the binding passed again so that it keeps the caller's hygiene.
    (@one text $binding:ident) => { return $binding.as_ref(); };
    (@one $other:ident $binding:ident) => {};
    (
        $(#[$enum_meta:meta])*
        pub enum DiagnosticCode {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $({
                    $(
                        $(#[$field_meta:meta])*
                        $field:ident : $field_ty:ty
                    ),* $(,)?
                })?
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        pub enum DiagnosticCode {
            $(
                $(#[$variant_meta])*
                $variant $({
                    $(
                        $(#[$field_meta])*
                        $field: $field_ty
                    ),*
                })?
            ),*
        }

        impl DiagnosticCode {
            /// Every code's name, in declaration order: the variant names, which are stable.
            pub const NAMES: &'static [&'static str] = &[$(stringify!($variant)),*];

            /// The code's name: its variant name (`"UnrecognizedField"`), for filtering and
            /// reports that must not depend on the message text.
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant { .. } => stringify!($variant),)*
                }
            }

            /// The game's text the code carries in its `text` field, if it has the field and a
            /// text in it: what tells a finding with the game's words from one with only its
            /// kind's.
            #[doc(hidden)]
            #[must_use]
            #[allow(
                unused_variables,
                unreachable_code,
                clippy::needless_return,
                reason = "every field is bound; only `text` is read, returned from its arm"
            )]
            pub fn game_text(&self) -> Option<&VanillaText> {
                match self {
                    $(Self::$variant $({ $($field),* })? => {
                        $($(diagnostic_codes!(@one $field $field);)*)?
                        None
                    })*
                }
            }
        }
    };
}

diagnostic_codes! {
    /// What happened: one severity-neutral code per kind of finding, with its typed payload.
    ///
    /// The emitting rule picks the [`Severity`](super::Severity): the same code is an error in one
    /// family and a warning in another. `Display` renders vanilla's content-log text where
    /// vanilla has one (a [`VanillaText`] in the `text` field, or the code's own fixed text),
    /// otherwise a one-line message of the crate's own. Tests match on variants, never on the text.
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum DiagnosticCode {
        // ---- document level ----
        /// A `format_version` no struct of the family covers: below the floor, above the
        /// ceiling, or between two exact versions.
        UnsupportedVersion {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A beta document (`"format_version": "beta"`) without the Beta APIs toggle.
        BetaFeaturesNotEnabled {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A prerelease-tagged `format_version` (`1.21.60-beta.3`) that loaded; it selects the
        /// struct below its release, as vanilla's semantic-version order does.
        PrereleaseFormatVersion {
            /// The version as written.
            version: String,
        },
        /// Content that needs a toggle or a requirement the load does not satisfy.
        MissingToggles {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A deprecated member: moved to a side-car below its deprecation version, dropped at
        /// or above it.
        DeprecatedMember {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A member the schema does not declare.
        UnrecognizedField {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A component name the family's table does not know.
        UnrecognizedComponent {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A value outside what the member accepts (range, length, emptiness, UUID form).
        InvalidValue {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A required member is absent.
        MissingRequiredField {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A value clamped into its range.
        ClampedValue {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A component dropped by a rule; the rest of the document loads.
        DroppedComponent {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A member dropped by a rule; the rest of the document loads.
        DroppedField {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A member the load read and does not keep: nothing reads it, the loaded document
        /// does not hold it and a written document does not have it. Unlike
        /// [`UnrecognizedField`](Self::UnrecognizedField), whose member is kept and written
        /// back, and unlike [`DroppedField`](Self::DroppedField), no rule removes it: it is a
        /// member no schema declares, at a place that keeps no unknown members.
        MemberNotKept {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A value the load holds in another form than the file has it in: the member stays,
        /// and a written document has the value as it was read, not as it was written (a
        /// name in another case, a boolean for a number). Also a value the written format
        /// has no spelling for, written as the nearest one it has. Nothing is dropped.
        ValueRewritten {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// Two versions of one component in one section (a component and the one it
        /// supersedes); the older one is dropped.
        DuplicateComponentVersion {
            /// The superseded component (`minecraft:foil`).
            old: String,
            /// The component that supersedes it (`minecraft:glint`).
            new: String,
        },
        /// A sound event name that no sound definition provides, or a level sound event name
        /// the game does not know.
        UnknownSound {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A generic AI goal whose name lacks the required suffix.
        GenericGoalSuffixMissing,
        /// A JSON syntax error, or a root or envelope that is not the expected document shape.
        MalformedInput {
            /// The reader's message (facet-json's text), shown when `text` is `None`.
            message: String,
            /// Vanilla's text, when the family or the envelope has one.
            text: Option<VanillaText>,
        },

        // ---- resource-pack side and shared ----
        /// A `format_version` above the engine version, parsed as the newest struct.
        FormatVersionAboveEngine {
            /// The version as written.
            version: String,
        },
        /// A client entity document that defines nothing: superseded by a higher-priority
        /// pack, or ignored by the shared identifier map for the `reason` given.
        ClientEntityShadowed {
            /// Why the shared identifier map ignored the document, when it did.
            reason: Option<ShadowReason>,
        },
        /// An attachable document that defines nothing: superseded by a higher-priority pack,
        /// or ignored by the shared identifier map for the `reason` given.
        AttachableShadowed {
            /// Why the shared identifier map ignored the document, when it did.
            reason: Option<ShadowReason>,
        },
        /// A lower-priority item binding was superseded.
        ItemBindingShadowed,
        /// A lower-priority render controller was superseded.
        RenderControllerShadowed,
        /// Two distinct definitions land on one key (a client entity and an attachable sharing
        /// an identifier; one item placed twice in the catalog).
        IdentifierCollision {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A name that must resolve (in another family, the same family or a registry)
        /// resolves to nothing. The name is in the diagnostic's
        /// [`DiagnosticDetail::Reference`](super::DiagnosticDetail::Reference).
        UnresolvedReference {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A reference resolves, but to something the use cannot accept.
        InvalidReference {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A referenced texture file is in no pack of the stack.
        TextureNotFound {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// The pack's splashes replace the lower packs' splashes.
        SplashesReplaced,
        /// The stack has no splash texts.
        NoSplashes,
        /// A localisation key that no `.lang` file defines.
        UnresolvedLangKey,
        /// A file the loader does not read (wrong extension, wrong place).
        IgnoredFile {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// The pack has no `pack_icon.png`.
        PackIconMissing,
        /// `pack_icon.png` is not square or not a power of two.
        PackIconShape,
        /// A glyph sheet whose size is not a 16 × 16 grid.
        GlyphSheetShape,
        /// A skin pack's name does not match its localisation.
        SkinPackNameMismatch,
        /// A skin pack breaks a creator rule.
        SkinPackCreatorRule,
        /// A member that is read but dropped, or that has no effect.
        IgnoredMember {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A UI `where` / `target` condition that matched no element.
        UnmatchedCondition,
        /// An array-valued UI `where`, which vanilla never matches.
        WhereArrayNeverMatches,
        /// A UI overlay `name@base` key shadows a base short name.
        RebasedOverride,
        /// A definition from a lower-priority source was replaced, or a higher one ignored.
        ShadowedDefinition {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A file could not be read from its pack.
        ReadError {
            /// What the host reported.
            kind: std::io::ErrorKind,
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// An envelope without the family's payload key.
        MissingPayload {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A namespace rule failed, or an identifier that must be namespaced is not.
        NamespaceRule {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A member of the wrong JSON type or shape, including a failed typed deserialisation.
        InvalidType {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A binary cache document recognised by its version string; its body was not parsed.
        SkippedCacheDocument,

        // ---- family codes: version and envelope ----
        /// `format_version` is absent (or, where the family says so, unreadable).
        MissingFormatVersion {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// `format_version` is present but neither a semantic-version string nor three integers,
        /// or of a shape the family's envelope rejects. Without `text`: vanilla's
        /// "Malformed format version".
        MalformedFormatVersion {
            /// The family's own vanilla text, when it is not the generic one.
            text: Option<VanillaText>,
        },
        /// The file is empty, or parses to nothing / `null`.
        EmptyDocument {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A version between two known slices, parsed by the lower one.
        UnusualFormatVersion,
        /// A version the crate accepts that vanilla 1.26.20 has no parser for.
        VanillaRejectsVersion,
        /// `format_version` given as an array of three integers (accepted).
        FormatVersionArray,
        /// The payload member is not the family's payload key; loaded anyway.
        PayloadKeyMismatch,
        /// The payload key names no registered feature type.
        UnknownPayloadKey {
            /// The payload key.
            name: String,
        },
        /// A second payload in one file is ignored.
        IgnoredPayload,
        /// The closing error of a document that failed schema validation.
        DocumentRejected {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A document without a usable identifier.
        MissingIdentifier {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A `{ "sound_definitions": … }` root without a version of at least 1.14.0, which
        /// vanilla wraps again.
        SoundDefinitionsWrapperWithoutVersion,
        /// The pack has no `min_engine_version`, so the command version is unset.
        MissingMinEngineVersion,
        /// A field newer than the pack's `min_engine_version` (or the configured target
        /// engine, for server configuration).
        FieldSinceEngine {
            /// The engine version that introduced the field.
            since: FormatVersion,
        },
        /// A field the configured target engine no longer has, although an earlier engine
        /// had it.
        FieldUntilEngine {
            /// The last engine version known to have the field.
            last: FormatVersion,
        },
        /// A value form the configured engine does not support yet.
        UnsupportedValueForm,

        // ---- family codes: values, types and expressions ----
        /// A string or discriminator outside its enum.
        InvalidEnumValue {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// One parser message of a Molang source, or a rejecting lint. The compiler's message
        /// is the diagnostic's [`DiagnosticDetail::Molang`](super::DiagnosticDetail) (feature
        /// `molang`).
        InvalidMolang {
            /// The expression as written.
            expression: String,
        },
        /// A non-rejecting Molang lint, or a behaviour-pack expression the Molang crate cannot
        /// evaluate.
        MolangLint {
            /// The expression as written.
            expression: String,
        },
        /// Molang stored unchecked, one per document: the `molang` feature is off, or it is on
        /// and no Molang check was given (`LoadOptions::molang` is `None`, the default). The
        /// text names the one that holds.
        MolangNotChecked {
            /// How many Molang fields the document holds.
            fields: u32,
        },
        /// A keyframe key outside `[0, 1]`; the key is dropped.
        KeyframeOutOfDomain,
        /// A value written back out loses precision.
        LossyConversion,
        /// A literal ignored because a key form takes precedence, or because a second literal
        /// is parsed into the same member.
        LiteralIgnored,
        /// A member name matched case-insensitively.
        NonCanonicalMemberCase,
        /// A transition expression lower-cased below 1.17.30.
        CaseFoldedTransition,
        /// `remap_curve` keys whose string order differs from their numeric order.
        UnorderedRemapCurve,
        /// No `initial_state` and no `default` state; the first state is used.
        ImplicitInitialState,
        /// An animation event string that is neither a command, an entity event nor Molang.
        UnrecognizedEventString,
        /// A behavior-tree child that is not an object with exactly one member.
        InvalidNode,
        /// `minecraft:root` is null, not an object, or not exactly one member.
        InvalidRoot,
        /// An unknown behavior-tree node type.
        UnknownNode {
            /// The node type.
            name: String,
        },
        /// An item parsed with the legacy (non-component) schema.
        LegacyItemFormat {
            /// Why the legacy schema was chosen.
            reason: &'static str,
        },
        /// A persona-piece pack placed in the resource-pack directory.
        PersonaPiecePack,
        /// A `.lang` key ends, or its value starts, with a space.
        LangKeyTrailingSpace,
        /// A scatter parameter vanilla logs and loads anyway; vanilla's replacement applies.
        ScatterParamValue {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// `<=`, `>=` or `%` used as an operator in a UI binding expression.
        UnsupportedOperator,

        // ---- family codes: assets ----
        /// An image that is not decodable or has the wrong proportions.
        InvalidTexture {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A texture-set layer image cannot be decoded.
        LayerImageUndecodable,
        /// A texture-set layer image has an unsupported channel count.
        LayerImageFormat,
        /// Content that only takes effect with a manifest capability the pack does not declare.
        CapabilityHint {
            /// The family's sentence, when it has one.
            text: Option<VanillaText>,
        },
        /// A pack in the stack disables Vibrant Visuals.
        VibrantVisualsDisabledByPack,
        /// An audio file in a format the client does not play.
        UnsupportedAudioFormat,
        /// A font's data file is found under none of the tried paths.
        FontFileMissing {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A later `fonts[]` entry replaces an earlier one of the same name.
        FontNameOverridden,
        /// A font page's files come from different packs.
        MixedFontPage,
        /// `filter_lighting: true` on an atomic client entity's controller is ignored.
        AtomicFilterLightingIgnored,
        /// A render controller uses features atomic entities do not support; skipped for that
        /// entity.
        AtomicUnsupportedRenderController,

        // ---- family codes: identity, merge and linking ----
        /// Two definitions of one identifier where one is dropped by a rule other than pack
        /// priority.
        DuplicateIdentifier {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// Two bones that collide after lower-casing.
        DuplicateBone,
        /// A lower-priority atomic client entity was superseded.
        AtomicClientEntityShadowed,
        /// A lower-priority animation controller was superseded.
        AnimationControllerShadowed,
        /// A pack tree replaced by a built-in tree of the same name.
        ShadowedByBuiltin,
        /// References that form a cycle.
        ReferenceCycle {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// A client biome that fails validation still claims its identifier from lower packs.
        ClientBiomeShadowsLowerPacks,
        /// `biomes_client.json` overrides a client biome's appearance.
        ClientBiomeOverriddenByBiomesClient,
        /// The base-game `biomes_client.json` is not loaded for this world version.
        VanillaBiomesClientSkipped,
        /// A legacy fog entry for a biome no earlier file has is not used.
        LegacyFogIgnored,
        /// A biome document in a format vanilla no longer loads (below 1.21.110), with how to
        /// move it to a supported one.
        BiomeFormatHint {
            /// The document's format version; `None` for the 1.12 shape, whose only root
            /// member is named after the biome.
            version: Option<FormatVersion>,
            /// The document uses the 1.20.60-era surface components
            /// (`minecraft:surface_parameters`, `minecraft:*_surface`).
            surface_components: bool,
        },
        /// A member that has no effect where it is written.
        NoEffect {
            /// Vanilla's text, when the family has one.
            text: Option<VanillaText>,
        },
        /// An upgrade step inserted a component.
        SynthesizedComponent,
        /// An upgrade step prefixed `minecraft:` to an identifier.
        IdentifierNamespaced,
        /// An upgrade table missed an identifier because it was not namespaced.
        UpgradeTableMissedBareIdentifier,
        /// Two `Always` item bindings target one item.
        ItemBindingOverlap,
        /// A failed higher-priority render controller falls back to the lower definition.
        RenderControllerFallback,
        /// A render controller file's case folding did not reach some text, which kept its
        /// capitals: raised beside the error that text caused, and at a controller key or an
        /// array name that kept capitals, which the game registers under a name its
        /// lower-cased references never reach.
        CaseFoldingDisabled {
            /// The 1-based line of the apostrophe that switched the folding off; `None` when
            /// the capital was written as an escape, which the folding skips.
            line: Option<u32>,
            /// The text is a name others refer to (a controller key, an array name).
            unresolvable: bool,
        },
        /// One block listed by two POI types; the last load wins.
        PoiBlockClaimedTwice,
        /// A POI type with an empty `blocks` list.
        PoiNoBlocks,
        /// A pack's `sound_definitions.json` clears the lower packs' definitions.
        SoundDefinitionsCleared,
        /// A `sounds.json` table absent from a pack clears the lower packs' entries.
        SoundsJsonTableCleared,
        /// A legacy overlay clears a definition's sounds.
        SoundsClearedByLegacyOverlay,
        /// A legacy overlay resets a lower pack's `max_distance`.
        DistanceResetByLegacyOverlay,
        /// A UI property the control's type never reads.
        UnusedProperty,
        /// The combined `ui_defs` list is empty.
        NoUiDefinitions,

        // ---- family codes: pack settings ----
        /// A saved or admin-set value, or a setting's `type`, that does not match the
        /// setting's kind.
        SettingTypeMismatch,
        /// A dropdown or multiselect value that names no option.
        SettingUnknownOption,
        /// A saved value for a setting that is not defined.
        SettingUnknownName,
        /// A saved slider value clamped to `[min, max]`.
        SettingClamped,
        /// A legacy manifest setting element other than a label is dropped.
        LegacySettingIgnored,
        /// A client settings-change packet ignored on a dedicated server.
        PackSettingChangeIgnored,

        // ---- family codes: loading ----
        /// A listed entry name that is lossy on the host or breaks the path rules; skipped.
        PathRejected {
            /// The entry name as listed (lossily decoded).
            entry: String,
        },
        /// An accessor needs a file that is indexed but not resident.
        FileNotLoaded,
        /// A job panicked; what it was for is rejected, absent or installed empty.
        InternalError,
        /// A file whose format the loader cannot read: an encrypted file without its content
        /// key, a binary (`\x7FMCB`) document (vanilla `unsupported_file_format`).
        UnsupportedFileFormat,
        /// A path an archive holds more than once; the last entry is kept (`duplicate_entry`).
        DuplicateEntry,

        // ---- pack stack composition ----
        /// A pack dependency that is in no registered pack; the dependent pack is hidden
        /// (vanilla `missing_dependency`).
        MissingDependency {
            /// The dependent pack.
            pack: PackIdVersion,
            /// The dependency, as the manifest names it.
            dependency: PackIdVersion,
        },
        /// A resource pack a behaviour pack depends on was added to the resource stack after the
        /// listed ones (`dependency_auto_added`).
        DependencyAutoAdded {
            /// The behaviour pack.
            pack: PackIdVersion,
            /// The resource pack added.
            dependency: PackIdVersion,
        },
        /// A pack a world list or the base-game layer names is not registered; the entry is
        /// skipped.
        PackNotFound {
            /// The pack named.
            pack: PackIdVersion,
        },

        // ---- family codes: scripting ----
        /// A second or later script module in one pack is skipped.
        MultipleScriptModules {
            /// `Plugin [{}]`: `<header name> - <module version>`.
            plugin: String,
        },
        /// A native-module dependency the pack may not use.
        ScriptModulePermission {
            /// Vanilla's text, when the rule has one.
            text: Option<VanillaText>,
        },
        /// No version in the engine's module catalog satisfies a requested version.
        ScriptModuleVersionUnavailable {
            /// The pack header's name (stage-3 lines print the name only).
            plugin: String,
            /// The requesting module, `<name> - <version>`.
            module: String,
            /// The requested version.
            requested: String,
            /// The native module requested.
            dependency: String,
            /// The catalog's version with the request's pre-release tag, when it has one:
            /// `(tag, version)`.
            current: Option<(String, String)>,
            /// The versions the catalog lists.
            available: Vec<String>,
        },
        /// The requests for one module are satisfiable separately but not by one version.
        ScriptModuleVersionConflict {
            /// The pack header's name.
            plugin: String,
            /// The native module.
            module: String,
            /// Each request: `(version, requested by)`.
            requests: Vec<(String, String)>,
        },
        /// A request served by a higher version because of another request.
        ScriptModulePromoted {
            /// The pack header's name.
            plugin: String,
            /// The native module.
            module: String,
            /// The requested version.
            from: String,
            /// The version served.
            to: String,
            /// Who requested the higher version.
            requested_by: String,
        },
        /// An `alpha` selector or a `-alpha` version.
        ScriptAlphaDependency,
        /// `entry` lacks the `scripts/` prefix but the main file is found.
        ScriptEntryOutsideRoot,
        /// Invalid UTF-8, import attributes, or a lexer or parser error in a script file.
        ScriptSyntaxError {
            /// The engine's text, where it is known: invalid UTF-8 and import attributes. A
            /// general syntax error is worded by the engine's `QuickJS`, which nothing here
            /// reproduces.
            text: Option<VanillaText>,
        },
        /// An import specifier that normalises to no file of the bundle.
        ScriptImportNotFound {
            /// The normalised import name.
            specifier: String,
            /// Why it likely failed, when a near miss exists; data only, `Display` prints
            /// vanilla's text.
            hint: Option<NotFoundHint>,
        },
        /// A specifier whose leading `./` / `../` run reaches its end: vanilla's normaliser
        /// terminates the server.
        ScriptImportCrashesEngine,
        /// An `@…` import that is not a module known to the context.
        ScriptModuleUnrecognized {
            /// The module name.
            module: String,
        },
        /// An import of a restricted native module.
        ScriptModuleRestricted {
            /// The module name.
            module: String,
            /// The importing file.
            referrer: String,
        },
        /// `import()` with a non-literal argument.
        ScriptImportNotChecked,
        /// A `.js` file outside `scripts/`, or `scripts/` in a pack without a script module.
        ScriptFileNotLoadable,
        /// `eval(` / `Function(` tokens in a pack without the `script_eval` capability.
        ScriptEvalWithoutCapability,
        /// An `import.meta.<member>` read; `import.meta` has no members in vanilla.
        ScriptImportMetaEmpty,
        /// Import scanning disabled; one per bundle.
        ScriptsNotChecked,
        /// Script files the entry reaches that the JavaScript parser was kept from, because
        /// they could nest more deeply than it has stack for; one per bundle.
        ScriptSyntaxNotChecked {
            /// How many files.
            files: u32,
        },
        /// A pack's own `config/<uuid>/` folder hides `config/default/` and lacks a file that
        /// `default` has.
        PackConfigShadowsDefault,
        /// A folder under `config/` that matches no script-module UUID of the stack.
        UnmatchedConfigFolder {
            /// The folder name.
            folder: String,
            /// The script-module UUID it looks like, in canonical form, when it looks like one.
            expected: Option<uuid::Uuid>,
        },
        /// The `@minecraft/server-net` entry of `module_permissions` failed to load.
        NetConfigRejected {
            /// The module whose configuration failed.
            module: String,
            /// The loader's error list, as vanilla prints it.
            errors: String,
        },

        // ---- manifest (the game's pack-parse error kinds) ----
        /// A manifest property with an invalid value.
        InvalidProperty {
            /// Vanilla's text, where it is known.
            text: Option<VanillaText>,
        },
        /// A manifest version property that is malformed.
        MalformedPropertyVersion {
            /// Vanilla's text, where it is known.
            text: Option<VanillaText>,
        },
        /// A manifest property of the wrong type.
        WrongTypeProperty {
            /// Vanilla's text, where it is known.
            text: Option<VanillaText>,
        },
        /// A required manifest property is absent.
        MissingProperty {
            /// Vanilla's text, where it is known.
            text: Option<VanillaText>,
        },
        /// A manifest property that is empty; the game reports a different kind once the pack is
        /// upgraded.
        EmptyProperty,
        /// A manifest UUID that is malformed or nil (a different kind once the pack is upgraded);
        /// vanilla replaces it by a hash of the text.
        MalformedPropertyUuid,
        /// A module UUID equal to the header UUID, or two modules with one UUID (a different
        /// kind once the pack is upgraded).
        DuplicateUuid,
        /// A deprecated (format 0) manifest without modules.
        MissingModules,
        /// Modules that declare different pack types (a different kind once the pack is
        /// upgraded).
        MultipleModules,
        /// A manifest capability the host's registry does not grant.
        InvalidCapability {
            /// The capability as written.
            name: String,
        },
        /// A manifest property read but ignored for this pack type or format.
        IgnoredProperty,
        /// A manifest version above the highest accepted one.
        VersionTooHigh {
            /// The version as read.
            version: String,
            /// The highest version accepted.
            limit: String,
        },
        /// A manifest version below the lowest accepted one.
        VersionTooLow {
            /// The version as read.
            version: String,
            /// The lowest version accepted.
            limit: String,
        },
        /// A format-0/1 manifest's `min_engine_version` of 1.13.0 or above, capped to 1.12.0.
        MinEngineVersionCapFormatVersion1,
    }
}

/// Why an import specifier likely resolved to no file: the near miss a lint found.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NotFoundHint {
    /// The specifier leaves the pack's `scripts/` root.
    EscapesRoot,
    /// `.` or `..` segments in the middle of the path.
    MidPathDots,
    /// A file matches when case is ignored.
    CaseMismatch,
    /// A directory import (`./dir` for `./dir/index.js`).
    DirectoryIndex,
    /// The file exists with `.js` appended.
    AppendedJs,
    /// A JSON file imported as a module.
    ImportedAsJson,
    /// A bare package name (`lodash`), which no bundle resolves.
    BarePackage,
    /// The specifier repeats the `scripts/` prefix.
    ScriptsPrefix,
}

impl fmt::Display for NotFoundHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EscapesRoot => "the path leaves the scripts root",
            Self::MidPathDots => "the path has `.` or `..` in the middle",
            Self::CaseMismatch => "a file matches when case is ignored",
            Self::DirectoryIndex => "directory imports are not resolved",
            Self::AppendedJs => "the file exists with `.js` appended",
            Self::ImportedAsJson => "a JSON file cannot be imported",
            Self::BarePackage => "bare package names are not resolved",
            Self::ScriptsPrefix => "the path repeats the `scripts/` prefix",
        })
    }
}

/// A vanilla content-log text with its arguments: the crate's message table. `Display` renders
/// the text byte for byte as the game prints it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VanillaText {
    /// `Unknown/unsupported version value specified in "format_version" field: <version>`
    UnknownFormatVersion {
        /// The version as written.
        version: String,
    },
    /// `unsupported version: no parser available for version <version>` (the versioned
    /// loader's text).
    NoParserForVersion {
        /// The version as written.
        version: String,
    },
    /// `no format_version field was given` (legacy parsers).
    NoFormatVersionField,
    /// `warning parsing : the string format_version field was not given;` (the game's older
    /// schema-tree path).
    FormatVersionNotGiven,
    /// `no valid version passed by the caller or found in the input` (the versioned loaders
    /// that do not check the version themselves).
    NoValidVersion,
    /// `A Minecraft Document must have exactly two members: "format_version" and a payload member`
    DocumentMemberCount,
    /// `Failed to parse the Minecraft Document JSON: <message>`
    DocumentJson {
        /// The reader's message.
        message: String,
    },
    /// `To use <what> '<name>', use json format version <version> or higher`
    UseFormatVersion {
        /// What is gated: `component`, `entity`, `item`, ….
        what: &'static str,
        /// Its name.
        name: String,
        /// The version it needs.
        version: FormatVersion,
    },
    /// `To use <what> '<name>', turn on the '<toggle>' toggle in settings`
    TurnOnToggle {
        /// What is gated.
        what: &'static str,
        /// Its name.
        name: String,
        /// The toggle's name as the settings screen shows it.
        toggle: &'static str,
    },
    /// `To use <what> '<name>' in json formats earlier than <version>, turn on the '<toggle>'
    /// toggle in settings`
    TurnOnToggleBefore {
        /// What is gated.
        what: &'static str,
        /// Its name.
        name: String,
        /// The version from which no toggle is needed.
        version: FormatVersion,
        /// The toggle's name as the settings screen shows it.
        toggle: &'static str,
    },
    /// `Turn on the Beta APIs toggle in settings to allow beta documents.`: the JSON item
    /// path's gate on a beta `format_version`.
    BetaDocuments,
    /// `Using a beta format_version is only allowed for base packs or when the BetaApis toggle
    /// is enabled.` (the check of the versioned cereal loaders)
    BetaFormatVersion,
    /// `Minecraft Document JSON exceeds the maximum size of <max> bytes`
    DocumentTooLarge {
        /// The limit in bytes.
        max: usize,
    },
    /// `JSON document is approaching the maximum allowed size: <loaded> bytes loaded, <max>
    /// bytes maximum. Consider reducing or splitting the document where possible, or raise
    /// this issue if further increases in size are expected.`
    DocumentNearlyTooLarge {
        /// The document's length in bytes.
        loaded: usize,
        /// The limit in bytes.
        max: usize,
    },
    /// `<member> has been deprecated and is not available in json format <version>`
    Deprecated {
        /// The deprecated member.
        member: String,
        /// The document's version.
        version: FormatVersion,
    },
    /// `Error: actor resource file has format version: <version> exceeds game version:
    /// <engine>. Skipping.`
    ActorAboveEngine {
        /// The version as written.
        version: String,
        /// The engine version.
        engine: FormatVersion,
    },
    /// A fixed vanilla text a static metadata table holds (a component's
    /// `deprecated_message`).
    Fixed(&'static str),
    /// A vanilla text with arguments: `template` with each `{}` replaced by the next of `args`
    /// (vanilla's `%s` / `%f` / `{}` format strings of the block loader).
    Formatted {
        /// The text, with `{}` where each argument goes.
        template: &'static str,
        /// The arguments, in order.
        args: Vec<String>,
    },
    /// `Plugin [<plugin>] - module uuid [<uuid>] with version [<version>] already exists,
    /// skipping.`
    PluginModuleExists {
        /// `<header name> - <module version>`.
        plugin: String,
        /// The script module's UUID.
        uuid: uuid::Uuid,
        /// The script module's version.
        version: String,
    },
    /// `Plugin [<plugin>] - using unsupported runtime [<runtime>].`
    PluginUnsupportedRuntime {
        /// `<header name> - <module version>`.
        plugin: String,
        /// The runtime as vanilla prints it (sanitised: empty for an unknown language).
        runtime: String,
    },
    /// `Plugin [<plugin>] - entry [<entry>] not supported for runtime [<runtime>].`
    PluginEntryNotSupported {
        /// `<header name> - <module version>`.
        plugin: String,
        /// The `entry` as written.
        entry: String,
        /// The runtime (`JavaScript`).
        runtime: String,
    },
    /// `Provided '<path>' element has an invalid value in pack manifest.` (the legacy manifest
    /// reader's `InvalidProperty`; the format-3 reader prints the same with its own path).
    ManifestInvalidValue {
        /// The element as vanilla prints it: `/modules/0/type` (legacy), `modules/type`
        /// (format 3).
        path: String,
    },
    /// `Provided '<path>' element is not SemVer (semver.org) compliant in pack manifest.`
    ManifestNotSemVer {
        /// The element as vanilla prints it: `/dependencies/0/version`.
        path: String,
    },
    /// `Required '<path>' element is the wrong type in pack manifest.`
    ManifestWrongType {
        /// The element as vanilla prints it: `/dependencies/0/version`.
        path: String,
    },
    /// `dependency <module> invalid version: <version>` (the format-3 manifest validator).
    DependencyInvalidVersion {
        /// The native module's name.
        module: String,
        /// The version as vanilla prints it: `0.0.0` for a string it could not read.
        version: String,
    },
    /// `SyntaxError: invalid UTF-8 sequence    at <file>:<line>` (the engine's `QuickJS`).
    /// The engine reports `at main.js:1`; a file in a sub-folder and a later line follow the
    /// same form.
    ScriptInvalidUtf8 {
        /// The script file, by its path below `scripts/`.
        file: String,
        /// The line, from 1.
        line: u32,
    },
    /// `SyntaxError: expecting ';'    at <file>:<line>` (the engine's `QuickJS`, which has no
    /// import attributes).
    ScriptExpectingSemicolon {
        /// The script file, by its path below `scripts/`.
        file: String,
        /// The line, from 1.
        line: u32,
    },
    /// `Plugin [<plugin>] - does not contain main file [<main>].`
    PluginMissingMainFile {
        /// `<header name> - <module version>`.
        plugin: String,
        /// The main file, relative to the scripts root.
        main: String,
    },
    /// `Plugin [<plugin>] - requesting dependency on beta APIs [<dependency>], but the Beta
    /// APIs experiment is not enabled.`
    PluginBetaApis {
        /// `<header name> - <module version>`.
        plugin: String,
        /// The dependency as vanilla prints it.
        dependency: String,
    },
    /// `Plugin [<plugin>] - requesting dependency on [<module>], but the Beta APIs experiment
    /// is not enabled.`
    PluginBetaApisModule {
        /// `<header name> - <module version>`.
        plugin: String,
        /// The module name.
        module: String,
    },
    /// `Plugin [<plugin>] - requesting dependency on internal APIs [<dependency>], but does
    /// not have permission to access them.`
    PluginInternalApis {
        /// `<header name> - <module version>`.
        plugin: String,
        /// The dependency as vanilla prints it.
        dependency: String,
    },
    /// `Plugin [<plugin>] - requesting dependency on module [<module>] but it is not
    /// configured to use it.`
    PluginModuleNotConfigured {
        /// `<header name> - <module version>`.
        plugin: String,
        /// The module name.
        module: String,
    },
    /// `Plugin [<plugin>] - skipping dependency on [<module>] because this is not an editor
    /// world.`
    PluginNotEditorWorld {
        /// `<header name> - <module version>`.
        plugin: String,
        /// The module name.
        module: String,
    },
    /// `Plugin [<plugin>] dependency by UUID is deprecated and only supports version 0.1.0, use
    /// module_name instead.`
    PluginDependencyByUuid {
        /// `<header name> - <module version>`.
        plugin: String,
    },
    /// `Plugin [<plugin>] - module [<module>] depends on unknown module [<dependency>].`
    PluginUnknownModule {
        /// The pack header's name.
        plugin: String,
        /// The requesting module.
        module: String,
        /// The unknown module name.
        dependency: String,
    },
    /// `Failed to open file '<file>'` (server script configuration).
    ConfigOpenFailed {
        /// The file as vanilla prints it.
        file: String,
    },
    /// `Failed to parse JSON in file '<file>'` (server script configuration).
    ConfigParseFailed {
        /// The file as vanilla prints it.
        file: String,
    },
    /// `Failed to validate fog schema`
    FogSchema,
    /// `Atomic Entity - light_color_multiplier expressions are not supported.`
    AtomicLightColorExpression,
    /// `Filter test not found -> <name>`
    FilterTestNotFound {
        /// The test name as written.
        name: String,
    },
    /// `ItemDescriptor requires a string, {"item": string}, or {"tags": string} or
    /// {"item_tag": string}. ItemDescriptor cannot parse <value>` and a newline.
    ItemDescriptorShape {
        /// The rejected JSON, as compact text.
        value: String,
    },
    /// `BlockDescriptor requires a string, block object, or any_tag object. Cannot parse
    /// <value>`
    BlockDescriptorShape {
        /// The rejected JSON, as compact text.
        value: String,
    },
    /// `Only strings are valid for name. The invalid name value: <value>` and a newline.
    BlockDescriptorName {
        /// The rejected `name`, as compact JSON text.
        value: String,
    },
    /// `Invalid state type during block parsing: block name <block>, state name <state>,
    /// state type <state_type>. State type can only be bool, int, or string.`
    BlockDescriptorStateType {
        /// The descriptor's block name.
        block: String,
        /// The state's name.
        state: String,
        /// jsoncpp's value type number of the rejected value.
        state_type: i32,
    },
    /// `Cannot have both a tag expression and block states in a block descriptor for block:
    /// <block>`
    BlockDescriptorTagsAndStates {
        /// The descriptor's block name (empty without one).
        block: String,
    },
    /// `Cannot have both a block name and tag expression in a block descriptor for block:
    /// <block>`
    BlockDescriptorNameAndTags {
        /// The descriptor's block name.
        block: String,
    },
    /// `duplicate pack setting name found: <name>`
    DuplicatePackSettingName {
        /// The setting's name.
        name: String,
    },
    /// `slider <name> max must be greater than min`
    SliderMaxNotAboveMin {
        /// The setting's name.
        name: String,
    },
    /// `slider <name> step <step> must be <= to max - min`
    SliderStepTooLarge {
        /// The setting's name.
        name: String,
        /// The step, as `{}` prints an `f32`.
        step: String,
    },
    /// `slider <name> default <default> is out of range`
    SliderDefaultOutOfRange {
        /// The setting's name.
        name: String,
        /// The default, as `{}` prints an `f32`.
        default: String,
    },
    /// `dropdown <name> default value <default> is not in options`
    DropdownDefaultNotInOptions {
        /// The setting's name.
        name: String,
        /// The default as written.
        default: String,
    },
    /// `multiselect <name> default value <default> is not in options` (the crate's own text:
    /// the multiselect form of the dropdown text)
    MultiselectDefaultNotInOptions {
        /// The setting's name.
        name: String,
        /// The default as written.
        default: String,
    },
    /// `Error: animation script object has <size> items - size must be 1.  Skipping.`
    AnimationScriptObjectSize {
        /// The object's member count.
        size: usize,
    },
    /// `Error loading 'entity_version' as it contains multiple upgrade entries for version
    /// <version>.`
    EntityVersionDuplicate {
        /// The repeated version.
        version: u32,
    },
    /// `Error loading '<identifier>' as it contains a non-contiguous version between <from> and
    /// <to>.`
    EntityVersionGap {
        /// The entity's identifier.
        identifier: String,
        /// The version before the gap.
        from: u32,
        /// The version after the gap.
        to: u32,
    },
    /// `upgrade handler '<name>' is not defined in 'upgrades'` (the crate's own text:
    /// the game's failed lookup has no text of its own)
    UpgradeHandlerUndefined {
        /// The handler's name.
        name: String,
    },
    /// `` `<member>` was removed by the <version> upgrade`` (the crate's own text:
    /// the game's upgrade drops it silently)
    RemovedByUpgrade {
        /// The removed component or member.
        member: String,
        /// The version of the step that removed it.
        version: FormatVersion,
    },
    /// `component group '<group>' is null and was removed` (the crate's own text)
    NullGroupRemoved {
        /// The group's name.
        group: String,
    },
    /// `member "<member>" is deprecated` (cereal's text for a deprecated member name).
    MemberDeprecated {
        /// The member's JSON name.
        member: String,
    },
    /// `'<value>' is not a valid '<member>' value.`
    NotAValidValue {
        /// The value as written.
        value: String,
        /// The member's JSON name.
        member: &'static str,
    },
    /// `Block <block> on "eat_and_replace_block_pairs" on "minecraft:behavior.eat_block" is not a
    /// valid block.`
    EatBlockInvalidBlock {
        /// The block name as written.
        block: String,
    },
    /// `Item '<identifier>' with format_version 1.16.100 or greater cannot be loaded from
    /// resource packs.`
    ItemInResourcePack {
        /// The item.
        identifier: String,
    },
    /// `Resource pack has item definitions not found in the behavior pack. Item
    /// '<identifier>' is being ignored.`
    ItemNotInBehaviorPack {
        /// The item.
        identifier: String,
    },
    /// `Item '<identifier>' (format_version: <version>) cannot be registered because no
    /// 1.16.100+ Vanilla format version exists!`
    ItemVanillaIsLegacy {
        /// The item.
        identifier: String,
        /// The document's version as written.
        version: String,
    },
    /// `Item '<identifier>' (format_version: <version>) cannot be registered because the
    /// Vanilla item only exists as a 1.16.100+ format version!`
    ItemVanillaIsComponentBased {
        /// The item.
        identifier: String,
        /// The document's version as written.
        version: String,
    },
    /// `Non-Vanilla item '<identifier>' should not be using the "minecraft" namespace!`
    ItemNonVanillaNamespace {
        /// The item.
        identifier: String,
    },
    /// `Items with the '<component>' component and 'slot' set to '<slot>' will default to a
    /// max stack size of 1. Use the 'minecraft:max_stack_size' component to override.`
    ItemWearableStackSize {
        /// The component (`minecraft:wearable`).
        component: &'static str,
        /// The slot as written.
        slot: &'static str,
    },
    /// `"minecraft:kinetic_weapon"'s "<condition>" cannot have a "max_duration" of zero. To
    /// disable the effect, leave the "<condition>" field undefined.`
    ItemKineticZeroDuration {
        /// The condition member (`damage_conditions`).
        condition: &'static str,
    },
    /// `Within "minecraft:kinetic_weapon"'s "<condition>", "min_speed" and
    /// "min_relative_speed" are mutually exclusive. Specify only one.`
    ItemKineticSpeedExclusive {
        /// The condition member (`damage_conditions`).
        condition: &'static str,
    },
    /// `` `minecraft:enchantable`: `<slot>` is an invalid enchant slot, `none` has been used
    /// instead ``
    ItemEnchantSlot {
        /// The slot as written.
        slot: String,
    },
    /// `unknown saturation_modifier type '<name>' in food component, defaulted to 0.6` (the
    /// value used is always `normal`'s 0.6, printed with `{:g}`).
    ItemSaturationModifier {
        /// The name as written.
        name: String,
    },
    /// `minecraft:creative_category has been deprecated and is not available in json format
    /// <version> and above. Use description->menu_category->group instead.`
    ItemCreativeCategoryDeprecated {
        /// The document's version.
        version: FormatVersion,
    },
    /// `The "<component>" component requires "minecraft:use_modifiers > use_duration" to be set
    /// with a non-zero value.`
    ItemUseDurationRequired {
        /// The component (`minecraft:food`).
        component: &'static str,
    },
    /// `The "<component>" component requires a "minecraft:durability" component with positive
    /// "max_durability"`
    ItemDurabilityRequired {
        /// The component (`minecraft:damage_absorption`).
        component: &'static str,
    },
    /// `Item component '<component>' is only available to legacy vanilla packs` (the crate's
    /// own text: the game has no text for a legacy-creator-features component without a release
    /// version).
    ItemLegacyOnly {
        /// The component.
        component: String,
    },
    /// `Item[<item>] Failed to load <what> with name[<name>] from Json. bad name?`
    ItemLegacyEffectName {
        /// The item.
        item: String,
        /// `effect` or `remove_effect`.
        what: &'static str,
        /// The name as written.
        name: String,
    },
}

/// `major.minor.patch`, as vanilla prints a semantic version without a tag.
struct Version(FormatVersion);

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FormatVersion {
            major,
            minor,
            patch,
        } = self.0;
        write!(f, "{major}.{minor}.{patch}")
    }
}

impl fmt::Display for VanillaText {
    #[allow(clippy::too_many_lines)] // one arm per vanilla text: the message table
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFormatVersion { version } => write!(
                f,
                "Unknown/unsupported version value specified in \"format_version\" field: {version}"
            ),
            Self::NoParserForVersion { version } => {
                write!(f, "unsupported version: no parser available for version {version}")
            }
            Self::NoFormatVersionField => f.write_str("no format_version field was given"),
            Self::FormatVersionNotGiven => {
                f.write_str("warning parsing : the string format_version field was not given;")
            }
            Self::NoValidVersion => {
                f.write_str("no valid version passed by the caller or found in the input")
            }
            Self::DocumentMemberCount => f.write_str(
                "A Minecraft Document must have exactly two members: \"format_version\" and a payload member",
            ),
            Self::DocumentJson { message } => {
                write!(f, "Failed to parse the Minecraft Document JSON: {message}")
            }
            Self::UseFormatVersion {
                what,
                name,
                version,
            } => write!(
                f,
                "To use {what} '{name}', use json format version {} or higher",
                Version(*version)
            ),
            Self::TurnOnToggle { what, name, toggle } => write!(
                f,
                "To use {what} '{name}', turn on the '{toggle}' toggle in settings"
            ),
            Self::TurnOnToggleBefore {
                what,
                name,
                version,
                toggle,
            } => write!(
                f,
                "To use {what} '{name}' in json formats earlier than {}, turn on the '{toggle}' toggle in settings",
                Version(*version)
            ),
            Self::BetaDocuments => {
                f.write_str("Turn on the Beta APIs toggle in settings to allow beta documents.")
            }
            Self::BetaFormatVersion => f.write_str(
                "Using a beta format_version is only allowed for base packs or when the BetaApis toggle is enabled.",
            ),
            Self::DocumentTooLarge { max } => {
                write!(f, "Minecraft Document JSON exceeds the maximum size of {max} bytes")
            }
            Self::DocumentNearlyTooLarge { loaded, max } => write!(
                f,
                "JSON document is approaching the maximum allowed size: {loaded} bytes loaded, {max} bytes maximum. Consider reducing or splitting the document where possible, or raise this issue if further increases in size are expected."
            ),
            Self::Deprecated { member, version } => write!(
                f,
                "{member} has been deprecated and is not available in json format {}",
                Version(*version)
            ),
            Self::ActorAboveEngine { version, engine } => write!(
                f,
                "Error: actor resource file has format version: {version} exceeds game version: {}. Skipping.",
                Version(*engine)
            ),
            Self::Fixed(text) => f.write_str(text),
            Self::Formatted { template, args } => {
                let mut args = args.iter();
                let mut parts = template.split("{}");
                if let Some(first) = parts.next() {
                    f.write_str(first)?;
                }
                for part in parts {
                    f.write_str(args.next().map_or("", String::as_str))?;
                    f.write_str(part)?;
                }
                Ok(())
            }
            Self::PluginModuleExists {
                plugin,
                uuid,
                version,
            } => write!(
                f,
                "Plugin [{plugin}] - module uuid [{uuid}] with version [{version}] already exists, skipping."
            ),
            Self::PluginUnsupportedRuntime { plugin, runtime } => {
                write!(f, "Plugin [{plugin}] - using unsupported runtime [{runtime}].")
            }
            Self::PluginEntryNotSupported {
                plugin,
                entry,
                runtime,
            } => write!(
                f,
                "Plugin [{plugin}] - entry [{entry}] not supported for runtime [{runtime}]."
            ),
            Self::ManifestInvalidValue { path } => write!(
                f,
                "Provided '{path}' element has an invalid value in pack manifest."
            ),
            Self::ManifestNotSemVer { path } => write!(
                f,
                "Provided '{path}' element is not SemVer (semver.org) compliant in pack manifest."
            ),
            Self::ManifestWrongType { path } => write!(
                f,
                "Required '{path}' element is the wrong type in pack manifest."
            ),
            Self::DependencyInvalidVersion { module, version } => {
                write!(f, "dependency {module} invalid version: {version}")
            }
            Self::ScriptInvalidUtf8 { file, line } => {
                write!(f, "SyntaxError: invalid UTF-8 sequence    at {file}:{line}")
            }
            Self::ScriptExpectingSemicolon { file, line } => {
                write!(f, "SyntaxError: expecting ';'    at {file}:{line}")
            }
            Self::PluginMissingMainFile { plugin, main } => {
                write!(f, "Plugin [{plugin}] - does not contain main file [{main}].")
            }
            Self::PluginBetaApis { plugin, dependency } => write!(
                f,
                "Plugin [{plugin}] - requesting dependency on beta APIs [{dependency}], but the Beta APIs experiment is not enabled."
            ),
            Self::PluginBetaApisModule { plugin, module } => write!(
                f,
                "Plugin [{plugin}] - requesting dependency on [{module}], but the Beta APIs experiment is not enabled."
            ),
            Self::PluginInternalApis { plugin, dependency } => write!(
                f,
                "Plugin [{plugin}] - requesting dependency on internal APIs [{dependency}], but does not have permission to access them."
            ),
            Self::PluginModuleNotConfigured { plugin, module } => write!(
                f,
                "Plugin [{plugin}] - requesting dependency on module [{module}] but it is not configured to use it."
            ),
            Self::PluginNotEditorWorld { plugin, module } => write!(
                f,
                "Plugin [{plugin}] - skipping dependency on [{module}] because this is not an editor world."
            ),
            Self::PluginDependencyByUuid { plugin } => write!(
                f,
                "Plugin [{plugin}] dependency by UUID is deprecated and only supports version 0.1.0, use module_name instead."
            ),
            Self::PluginUnknownModule {
                plugin,
                module,
                dependency,
            } => write!(
                f,
                "Plugin [{plugin}] - module [{module}] depends on unknown module [{dependency}]."
            ),
            Self::ConfigOpenFailed { file } => write!(f, "Failed to open file '{file}'"),
            Self::ConfigParseFailed { file } => write!(f, "Failed to parse JSON in file '{file}'"),
            Self::FogSchema => f.write_str("Failed to validate fog schema"),
            Self::AtomicLightColorExpression => f.write_str(
                "Atomic Entity - light_color_multiplier expressions are not supported.",
            ),
            Self::FilterTestNotFound { name } => write!(f, "Filter test not found -> {name}"),
            Self::ItemDescriptorShape { value } => writeln!(
                f,
                "ItemDescriptor requires a string, {{\"item\": string}}, or {{\"tags\": string}} or {{\"item_tag\": string}}. ItemDescriptor cannot parse {value}"
            ),
            Self::BlockDescriptorShape { value } => write!(
                f,
                "BlockDescriptor requires a string, block object, or any_tag object. Cannot parse {value}"
            ),
            Self::BlockDescriptorName { value } => writeln!(
                f,
                "Only strings are valid for name. The invalid name value: {value}"
            ),
            Self::BlockDescriptorStateType {
                block,
                state,
                state_type,
            } => write!(
                f,
                "Invalid state type during block parsing: block name {block}, state name {state}, state type {state_type}. State type can only be bool, int, or string."
            ),
            Self::BlockDescriptorTagsAndStates { block } => write!(
                f,
                "Cannot have both a tag expression and block states in a block descriptor for block: {block}"
            ),
            Self::BlockDescriptorNameAndTags { block } => write!(
                f,
                "Cannot have both a block name and tag expression in a block descriptor for block: {block}"
            ),
            Self::DuplicatePackSettingName { name } => {
                write!(f, "duplicate pack setting name found: {name}")
            }
            Self::SliderMaxNotAboveMin { name } => {
                write!(f, "slider {name} max must be greater than min")
            }
            Self::SliderStepTooLarge { name, step } => {
                write!(f, "slider {name} step {step} must be <= to max - min")
            }
            Self::SliderDefaultOutOfRange { name, default } => {
                write!(f, "slider {name} default {default} is out of range")
            }
            Self::DropdownDefaultNotInOptions { name, default } => {
                write!(f, "dropdown {name} default value {default} is not in options")
            }
            Self::MultiselectDefaultNotInOptions { name, default } => {
                write!(f, "multiselect {name} default value {default} is not in options")
            }
            Self::AnimationScriptObjectSize { size } => write!(
                f,
                "Error: animation script object has {size} items - size must be 1.  Skipping."
            ),
            Self::EntityVersionDuplicate { version } => write!(
                f,
                "Error loading 'entity_version' as it contains multiple upgrade entries for version {version}."
            ),
            Self::EntityVersionGap {
                identifier,
                from,
                to,
            } => write!(
                f,
                "Error loading '{identifier}' as it contains a non-contiguous version between {from} and {to}."
            ),
            Self::UpgradeHandlerUndefined { name } => {
                write!(f, "upgrade handler '{name}' is not defined in 'upgrades'")
            }
            Self::NullGroupRemoved { group } => {
                write!(f, "component group '{group}' is null and was removed")
            }
            Self::RemovedByUpgrade { member, version } => write!(
                f,
                "`{member}` was removed by the {} upgrade",
                Version(*version)
            ),
            Self::MemberDeprecated { member } => write!(f, "member \"{member}\" is deprecated"),
            Self::NotAValidValue { value, member } => {
                write!(f, "'{value}' is not a valid '{member}' value.")
            }
            Self::EatBlockInvalidBlock { block } => write!(
                f,
                "Block {block} on \"eat_and_replace_block_pairs\" on \"minecraft:behavior.eat_block\" is not a valid block."
            ),
            Self::ItemInResourcePack { identifier } => write!(
                f,
                "Item '{identifier}' with format_version 1.16.100 or greater cannot be loaded from resource packs."
            ),
            Self::ItemNotInBehaviorPack { identifier } => write!(
                f,
                "Resource pack has item definitions not found in the behavior pack. Item '{identifier}' is being ignored."
            ),
            Self::ItemVanillaIsLegacy {
                identifier,
                version,
            } => write!(
                f,
                "Item '{identifier}' (format_version: {version}) cannot be registered because no 1.16.100+ Vanilla format version exists!"
            ),
            Self::ItemVanillaIsComponentBased {
                identifier,
                version,
            } => write!(
                f,
                "Item '{identifier}' (format_version: {version}) cannot be registered because the Vanilla item only exists as a 1.16.100+ format version!"
            ),
            Self::ItemNonVanillaNamespace { identifier } => write!(
                f,
                "Non-Vanilla item '{identifier}' should not be using the \"minecraft\" namespace!"
            ),
            Self::ItemWearableStackSize { component, slot } => write!(
                f,
                "Items with the '{component}' component and 'slot' set to '{slot}' will default to a max stack size of 1. Use the 'minecraft:max_stack_size' component to override."
            ),
            Self::ItemKineticZeroDuration { condition } => write!(
                f,
                "\"minecraft:kinetic_weapon\"'s \"{condition}\" cannot have a \"max_duration\" of zero. To disable the effect, leave the \"{condition}\" field undefined."
            ),
            Self::ItemKineticSpeedExclusive { condition } => write!(
                f,
                "Within \"minecraft:kinetic_weapon\"'s \"{condition}\", \"min_speed\" and \"min_relative_speed\" are mutually exclusive. Specify only one."
            ),
            Self::ItemEnchantSlot { slot } => write!(
                f,
                "`minecraft:enchantable`: `{slot}` is an invalid enchant slot, `none` has been used instead"
            ),
            Self::ItemSaturationModifier { name } => write!(
                f,
                "unknown saturation_modifier type '{name}' in food component, defaulted to 0.6"
            ),
            Self::ItemCreativeCategoryDeprecated { version } => write!(
                f,
                "minecraft:creative_category has been deprecated and is not available in json format {} and above. Use description->menu_category->group instead.",
                Version(*version)
            ),
            Self::ItemUseDurationRequired { component } => write!(
                f,
                "The \"{component}\" component requires \"minecraft:use_modifiers > use_duration\" to be set with a non-zero value."
            ),
            Self::ItemDurabilityRequired { component } => write!(
                f,
                "The \"{component}\" component requires a \"minecraft:durability\" component with positive \"max_durability\""
            ),
            Self::ItemLegacyOnly { component } => write!(
                f,
                "Item component '{component}' is only available to legacy vanilla packs"
            ),
            Self::ItemLegacyEffectName { item, what, name } => write!(
                f,
                "Item[{item}] Failed to load {what} with name[{name}] from Json. bad name?"
            ),
        }
    }
}

/// Writes `text` when present, else the crate's own one-line `fallback`.
fn text_or(f: &mut fmt::Formatter<'_>, text: Option<&VanillaText>, fallback: &str) -> fmt::Result {
    match text {
        Some(text) => text.fmt(f),
        None => f.write_str(fallback),
    }
}

/// The sentence of a document of `what` (a client entity, an attachable) that defines
/// nothing: why the shared identifier map ignored it. Both families always give the
/// reason; without one the sentence says only that another definition of the identifier
/// stands, since these maps go by `min_engine_version` and not by pack priority alone.
fn shadowed(f: &mut fmt::Formatter<'_>, what: &str, reason: Option<ShadowReason>) -> fmt::Result {
    match reason {
        Some(ShadowReason::OlderMinEngineVersion) => write!(
            f,
            "{what} ignored: its min_engine_version is below the loaded definition's"
        ),
        Some(ShadowReason::MinEngineVersionAboveStack) => {
            write!(
                f,
                "{what} ignored: its min_engine_version is above the stack's"
            )
        }
        // No reason, or one a newer `visual` adds.
        _ => write!(
            f,
            "{what} superseded by another definition of its identifier"
        ),
    }
}

impl fmt::Display for DiagnosticCode {
    #[allow(clippy::too_many_lines)] // one arm per code: the message table
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion { text } => {
                text_or(f, text.as_ref(), "unsupported format_version")
            }
            Self::BetaFeaturesNotEnabled { text } => text_or(
                f,
                text.as_ref(),
                "beta document without the Beta APIs toggle",
            ),
            Self::PrereleaseFormatVersion { version } => write!(
                f,
                "format_version {version} has a pre-release tag and selects the schema below its release"
            ),
            Self::MissingToggles { text } => {
                text_or(f, text.as_ref(), "needs a toggle that is not enabled")
            }
            Self::DeprecatedMember { text } => text_or(f, text.as_ref(), "deprecated member"),
            Self::UnrecognizedField { text } => text_or(f, text.as_ref(), "unrecognized field"),
            Self::UnrecognizedComponent { text } => {
                text_or(f, text.as_ref(), "unrecognized component")
            }
            Self::InvalidValue { text } => text_or(f, text.as_ref(), "invalid value"),
            Self::MissingRequiredField { text } => {
                text_or(f, text.as_ref(), super::cereal::MISSING_REQUIRED_FIELD_TEXT)
            }
            Self::ClampedValue { text } => {
                text_or(f, text.as_ref(), "value clamped into its range")
            }
            Self::DroppedComponent { text } => text_or(f, text.as_ref(), "component dropped"),
            Self::DroppedField { text } => text_or(f, text.as_ref(), "field dropped"),
            Self::MemberNotKept { text } => text_or(f, text.as_ref(), "member read and not kept"),
            Self::ValueRewritten { text } => {
                text_or(f, text.as_ref(), "value held in another form than it is written in")
            }
            Self::DuplicateComponentVersion { old, new } => write!(
                f,
                "Two versions of the same component found: {old} and {new}. Please remove {new}, or upgrade your format version and remove {old}."
            ),
            Self::UnknownSound { text } => text_or(f, text.as_ref(), "unknown sound event"),
            Self::GenericGoalSuffixMissing => {
                f.write_str("generic goal name lacks its required suffix")
            }
            Self::MalformedInput { message, text } => match text {
                Some(text) => text.fmt(f),
                None if message.is_empty() => f.write_str("malformed input"),
                None => f.write_str(message),
            },
            Self::FormatVersionAboveEngine { version } => write!(
                f,
                "format_version {version} is above the engine version {}; parsed as the newest schema",
                Version(crate::version::ENGINE_VERSION)
            ),
            Self::ClientEntityShadowed { reason } => shadowed(f, "client entity", *reason),
            Self::AttachableShadowed { reason } => shadowed(f, "attachable", *reason),
            Self::ItemBindingShadowed => {
                f.write_str("item binding superseded by a higher-priority pack")
            }
            Self::RenderControllerShadowed => {
                f.write_str("render controller superseded by a higher-priority pack")
            }
            Self::IdentifierCollision { text } => {
                text_or(f, text.as_ref(), "two definitions share one identifier")
            }
            Self::UnresolvedReference { text } => text_or(f, text.as_ref(), "unresolved reference"),
            Self::InvalidReference { text } => text_or(
                f,
                text.as_ref(),
                "reference resolves to something this use cannot accept",
            ),
            Self::TextureNotFound { text } => text_or(f, text.as_ref(), "texture not found"),
            Self::SplashesReplaced => f.write_str("splashes replace the lower packs' splashes"),
            Self::NoSplashes => f.write_str("no splash texts"),
            Self::UnresolvedLangKey => f.write_str("localisation key not defined"),
            Self::IgnoredFile { text } => text_or(f, text.as_ref(), "file ignored"),
            Self::PackIconMissing => f.write_str("pack_icon.png missing"),
            Self::PackIconShape => f.write_str("pack_icon.png is not a square image"),
            Self::GlyphSheetShape => f.write_str("glyph sheet is not a 16 x 16 grid"),
            Self::SkinPackNameMismatch => {
                f.write_str("skin pack name does not match its localisation")
            }
            Self::SkinPackCreatorRule => f.write_str("skin pack breaks a creator rule"),
            Self::IgnoredMember { text } => text_or(f, text.as_ref(), "member ignored"),
            Self::UnmatchedCondition => f.write_str("condition matched no element"),
            Self::WhereArrayNeverMatches => f.write_str("an array-valued `where` never matches"),
            Self::RebasedOverride => f.write_str("overlay `name@base` key shadows a base name"),
            Self::ShadowedDefinition { text } => text_or(
                f,
                text.as_ref(),
                "definition shadowed by a higher-priority source",
            ),
            Self::ReadError { kind, text } => match text {
                Some(text) => text.fmt(f),
                None => write!(f, "cannot read the file ({kind})"),
            },
            Self::MissingPayload { text } => {
                text_or(f, text.as_ref(), "the document has no payload member")
            }
            Self::NamespaceRule { text } => text_or(f, text.as_ref(), "namespace rule failed"),
            Self::InvalidType { text } => text_or(f, text.as_ref(), "wrong JSON type"),
            Self::SkippedCacheDocument => f.write_str("cache document not parsed"),
            Self::MissingFormatVersion { text } => text_or(f, text.as_ref(), "no format_version"),
            Self::MalformedFormatVersion { text } => {
                text_or(f, text.as_ref(), "Malformed format version")
            }
            Self::EmptyDocument { text } => text_or(f, text.as_ref(), "empty document"),
            Self::UnusualFormatVersion => {
                f.write_str("format_version between two known versions; parsed by the lower one")
            }
            Self::VanillaRejectsVersion => {
                f.write_str("vanilla has no parser for this format_version")
            }
            Self::FormatVersionArray => f.write_str("format_version given as an array"),
            Self::PayloadKeyMismatch => {
                f.write_str("payload member is not the family's payload key")
            }
            Self::UnknownPayloadKey { name } => write!(
                f,
                "Could not find registered feature type that matches name '{name}'"
            ),
            Self::IgnoredPayload => f.write_str("second payload ignored"),
            Self::DocumentRejected { text } => {
                text_or(f, text.as_ref(), "document failed schema validation")
            }
            Self::MissingIdentifier { text } => text_or(f, text.as_ref(), "no usable identifier"),
            Self::SoundDefinitionsWrapperWithoutVersion => f.write_str(
                "`sound_definitions` wrapper without a format_version of at least 1.14.0",
            ),
            Self::MissingMinEngineVersion => f.write_str("the pack has no min_engine_version"),
            Self::FieldSinceEngine { since } => {
                write!(f, "field needs engine version {} or newer", Version(*since))
            }
            Self::FieldUntilEngine { last } => write!(
                f,
                "field is gone from this engine; engine version {} was the last known to have it",
                Version(*last)
            ),
            Self::UnsupportedValueForm => {
                f.write_str("value form not supported by the configured engine")
            }
            Self::InvalidEnumValue { text } => text_or(f, text.as_ref(), "invalid enum value"),
            Self::InvalidMolang { expression } | Self::MolangLint { expression } => {
                f.write_str(expression)
            }
            // With the feature the finding is raised only where no check was given.
            Self::MolangNotChecked { fields } => write!(
                f,
                "{fields} Molang field(s) not checked: {}",
                if cfg!(feature = "molang") {
                    "no Molang check was given (`LoadOptions::molang` is `None`)"
                } else {
                    "the `molang` feature is off"
                }
            ),
            Self::KeyframeOutOfDomain => f.write_str("keyframe key outside [0, 1]; dropped"),
            Self::LossyConversion => f.write_str("value loses precision when written"),
            Self::LiteralIgnored => f.write_str("literal ignored"),
            Self::NonCanonicalMemberCase => f.write_str("member name matched ignoring case"),
            Self::CaseFoldedTransition => f.write_str("transition expression lower-cased"),
            Self::UnorderedRemapCurve => f.write_str("remap_curve keys out of numeric order"),
            Self::ImplicitInitialState => {
                f.write_str("no initial_state and no default state; the first state is used")
            }
            Self::UnrecognizedEventString => {
                f.write_str("event is neither a command, an entity event nor Molang")
            }
            Self::InvalidNode => f.write_str("node is not an object with exactly one member"),
            Self::InvalidRoot => {
                f.write_str("minecraft:root is not an object with exactly one member")
            }
            Self::UnknownNode { name } => write!(f, "Unknown behavior {name}"),
            Self::LegacyItemFormat { reason } => {
                write!(f, "item parsed with the legacy schema: {reason}")
            }
            Self::PersonaPiecePack => {
                f.write_str("persona-piece pack in the resource-pack directory")
            }
            Self::LangKeyTrailingSpace => f.write_str("key ends, or value starts, with a space"),
            Self::ScatterParamValue { text } => {
                text_or(f, text.as_ref(), "invalid scatter parameter")
            }
            Self::UnsupportedOperator => f.write_str("`<=`, `>=` and `%` are not operators"),
            Self::InvalidTexture { text } => text_or(
                f,
                text.as_ref(),
                "image not decodable or of the wrong proportions",
            ),
            Self::LayerImageUndecodable => f.write_str("layer image cannot be decoded"),
            Self::LayerImageFormat => f.write_str("layer image has an unsupported channel count"),
            Self::CapabilityHint { text } => text_or(
                f,
                text.as_ref(),
                "takes effect only with a capability the pack does not declare",
            ),
            Self::VibrantVisualsDisabledByPack => {
                f.write_str("a pack in the stack disables Vibrant Visuals")
            }
            Self::UnsupportedAudioFormat => f.write_str("audio format not supported"),
            Self::FontFileMissing { text } => text_or(f, text.as_ref(), "font file not found"),
            Self::FontNameOverridden => f.write_str("font replaced by a later entry"),
            Self::MixedFontPage => f.write_str("font page files come from different packs"),
            // The game's own texts: neither has an argument.
            Self::AtomicFilterLightingIgnored => {
                f.write_str("Atomic Entity - filter_lighting is not supported.")
            }
            Self::AtomicUnsupportedRenderController => f.write_str(
                "Skipping an Atomic Entity - Unsupported render controller features (conditional enable, dynamic resources, multiple materials, or part visibility).",
            ),
            Self::DuplicateIdentifier { text } => text_or(f, text.as_ref(), "duplicate identifier"),
            Self::DuplicateBone => f.write_str("two bones collide after lower-casing"),
            Self::AtomicClientEntityShadowed => {
                f.write_str("atomic client entity superseded by a higher-priority pack")
            }
            Self::AnimationControllerShadowed => {
                f.write_str("animation controller superseded by a higher-priority pack")
            }
            Self::ShadowedByBuiltin => f.write_str("replaced by a built-in definition"),
            Self::ReferenceCycle { text } => {
                text_or(f, text.as_ref(), "references form a cycle")
            }
            Self::ClientBiomeShadowsLowerPacks => {
                f.write_str("invalid client biome still shadows lower packs")
            }
            Self::ClientBiomeOverriddenByBiomesClient => {
                f.write_str("biomes_client.json overrides this client biome")
            }
            Self::VanillaBiomesClientSkipped => {
                f.write_str("base-game biomes_client.json not loaded for this world version")
            }
            Self::LegacyFogIgnored => f.write_str("legacy fog entry not used"),
            Self::BiomeFormatHint {
                version: None,
                ..
            } => f.write_str(
                "biome files in the 1.12 format are no longer supported; use the minecraft:biome document format with format_version 1.21.110 or higher",
            ),
            Self::BiomeFormatHint {
                version: Some(version),
                surface_components,
            } => {
                write!(
                    f,
                    "biome format {version} was experimental and is not loaded; set format_version to 1.21.110 (move climate ash/white_ash/red_spores/blue_spores to the client biome's minecraft:precipitation"
                )?;
                if *surface_components {
                    f.write_str(
                        " and replace minecraft:surface_parameters / minecraft:*_surface with minecraft:surface_builder",
                    )?;
                }
                f.write_str(")")
            }
            Self::NoEffect { text } => text_or(f, text.as_ref(), "member has no effect"),
            Self::SynthesizedComponent => f.write_str("component inserted by an upgrade step"),
            Self::IdentifierNamespaced => {
                f.write_str("identifier prefixed with minecraft: by an upgrade step")
            }
            Self::UpgradeTableMissedBareIdentifier => {
                f.write_str("upgrade table missed an identifier without namespace")
            }
            Self::ItemBindingOverlap => f.write_str("two Always item bindings target one item"),
            Self::RenderControllerFallback => {
                f.write_str("render controller falls back to the lower-priority definition")
            }
            Self::CaseFoldingDisabled { line, unresolvable } => {
                match line {
                    Some(line) => write!(f, "an apostrophe at line {line} disabled case folding")?,
                    None => f.write_str("a capital letter the game's case folding does not reach")?,
                }
                if *unresolvable {
                    f.write_str("; the game resolves no reference to this name")?;
                }
                Ok(())
            }
            Self::PoiBlockClaimedTwice => f.write_str("block listed by two POI types"),
            Self::PoiNoBlocks => f.write_str("POI type with no blocks"),
            Self::SoundDefinitionsCleared => {
                f.write_str("sound_definitions.json clears the lower packs' definitions")
            }
            Self::SoundsJsonTableCleared => {
                f.write_str("absent sounds.json table clears the lower packs' entries")
            }
            Self::SoundsClearedByLegacyOverlay => {
                f.write_str("legacy overlay clears the definition's sounds")
            }
            Self::DistanceResetByLegacyOverlay => f.write_str("legacy overlay resets max_distance"),
            Self::UnusedProperty => f.write_str("property not read by this control type"),
            Self::NoUiDefinitions => f.write_str("no UI definition files"),
            Self::SettingTypeMismatch => f.write_str("value does not match the setting's type"),
            Self::SettingUnknownOption => f.write_str("value names no option of the setting"),
            Self::SettingUnknownName => f.write_str("no setting of this name"),
            Self::SettingClamped => f.write_str("slider value clamped to its range"),
            Self::LegacySettingIgnored => f.write_str("legacy setting element dropped"),
            Self::PackSettingChangeIgnored => {
                f.write_str("settings change ignored on a dedicated server")
            }
            Self::PathRejected { entry } => write!(f, "entry '{entry}' skipped: path rejected"),
            Self::FileNotLoaded => f.write_str("file indexed but not loaded"),
            Self::InternalError => f.write_str("internal error: a loader job panicked"),
            Self::UnsupportedFileFormat => f.write_str("unsupported file format"),
            Self::DuplicateEntry => f.write_str("duplicate archive entry; the last one is kept"),
            Self::MissingDependency { pack, dependency } => write!(
                f,
                "pack {pack} depends on {dependency}, which is not available; the pack is hidden"
            ),
            Self::DependencyAutoAdded { pack, dependency } => write!(
                f,
                "resource pack {dependency} added to the resource stack as a dependency of {pack}"
            ),
            Self::PackNotFound { pack } => write!(f, "pack {pack} not found"),
            Self::MultipleScriptModules { plugin } => write!(
                f,
                "Plugin [{plugin}] - skipped, only one script module allowed per pack."
            ),
            Self::ScriptModulePermission { text } => {
                text_or(f, text.as_ref(), "native module not permitted")
            }
            Self::ScriptModuleVersionUnavailable {
                plugin,
                module,
                requested,
                dependency,
                current,
                available,
            } => {
                write!(
                    f,
                    "Plugin [{plugin}] - module [{module}] requested invalid version [{requested}] of [{dependency}]."
                )?;
                if let Some((tag, version)) = current {
                    write!(f, "\nCurrent [{tag}] version is [{version}].")?;
                }
                f.write_str("\nAvailable versions:")?;
                for version in available {
                    write!(f, "\n{version}")?;
                }
                Ok(())
            }
            Self::ScriptModuleVersionConflict {
                plugin,
                module,
                requests,
            } => {
                write!(
                    f,
                    "Plugin [{plugin}] - version conflict for module [{module}]."
                )?;
                // One line per request, a comma between lines and none after the last.
                for (i, (version, by)) in requests.iter().enumerate() {
                    let separator = if i == 0 { "\n" } else { ",\n" };
                    write!(f, "{separator} [{version}] requested by [{by}]")?;
                }
                Ok(())
            }
            Self::ScriptModulePromoted {
                plugin,
                module,
                from,
                to,
                requested_by,
            } => write!(
                f,
                "Plugin [{plugin}] - promoted [{module}] from [{from}] to [{to}] requested by [{requested_by}]."
            ),
            Self::ScriptAlphaDependency => f.write_str("alpha script API dependency"),
            Self::ScriptEntryOutsideRoot => f.write_str("entry lacks the scripts/ prefix"),
            Self::ScriptSyntaxError { text } => {
                text_or(f, text.as_ref(), "syntax error in a script file")
            }
            // The hint is data for the caller; the text is vanilla's alone.
            Self::ScriptImportNotFound { specifier, hint: _ } => {
                write!(f, "ReferenceError: Import [{specifier}] not found.")
            }
            Self::ScriptImportCrashesEngine => {
                f.write_str("this import specifier terminates the server")
            }
            Self::ScriptModuleUnrecognized { module } => write!(
                f,
                "Module [{module}] is unrecognized. If this is a Native module, did you remember to add it to your manifest.json?"
            ),
            Self::ScriptModuleRestricted { module, referrer } => write!(
                f,
                "Import failed, can not load restricted module '{module}' from '{referrer}'."
            ),
            Self::ScriptImportNotChecked => f.write_str("dynamic import not checked"),
            Self::ScriptFileNotLoadable => f.write_str("script file can never be loaded"),
            Self::ScriptEvalWithoutCapability => {
                f.write_str("eval without the script_eval capability")
            }
            Self::ScriptImportMetaEmpty => f.write_str("import.meta has no members"),
            Self::ScriptsNotChecked => f.write_str("script imports not checked"),
            Self::ScriptSyntaxNotChecked { files } => write!(
                f,
                "syntax of {files} script files not checked: nested too deeply for the parser"
            ),
            Self::PackConfigShadowsDefault => {
                f.write_str("pack config folder hides config/default")
            }
            Self::UnmatchedConfigFolder { folder, expected } => match expected {
                Some(uuid) => write!(
                    f,
                    "config folder '{folder}' is never used; the script module's folder is '{uuid}'"
                ),
                None => write!(
                    f,
                    "config folder '{folder}' matches no script module of the stack"
                ),
            },
            Self::NetConfigRejected { module, errors } => write!(
                f,
                "Failed to load config for {module} due to error(s): {errors}"
            ),
            Self::InvalidProperty { text } => {
                text_or(f, text.as_ref(), "invalid manifest property")
            }
            Self::MalformedPropertyVersion { text } => {
                text_or(f, text.as_ref(), "malformed manifest version")
            }
            Self::WrongTypeProperty { text } => {
                text_or(f, text.as_ref(), "manifest property of the wrong type")
            }
            Self::MissingProperty { text } => {
                text_or(f, text.as_ref(), "missing manifest property")
            }
            Self::EmptyProperty => f.write_str("empty manifest property"),
            Self::MalformedPropertyUuid => f.write_str("malformed manifest UUID"),
            Self::DuplicateUuid => f.write_str("duplicate UUID in the manifest"),
            Self::MissingModules => f.write_str("the manifest has no modules"),
            Self::MultipleModules => {
                f.write_str("the manifest's modules declare different pack types")
            }
            Self::InvalidCapability { name } => write!(
                f,
                "The pack manifest contains an invalid capability: {name}"
            ),
            Self::IgnoredProperty => f.write_str("manifest property ignored"),
            Self::VersionTooHigh { version, limit } => {
                write!(f, "version {version} is above {limit}")
            }
            Self::VersionTooLow { version, limit } => {
                write!(f, "version {version} is below {limit}")
            }
            Self::MinEngineVersionCapFormatVersion1 => {
                f.write_str("min_engine_version of a format 1 manifest is capped to 1.12.0")
            }
        }
    }
}

#[cfg(test)]
mod game_text_tests {
    use super::{DiagnosticCode, VanillaText};

    /// A code with a game text in its `text` field gives it; one with `text: None`, or a
    /// variant without the field, gives none.
    #[test]
    fn game_text_is_the_text_field_when_it_holds_one() {
        let with = DiagnosticCode::UnrecognizedField {
            text: Some(VanillaText::Fixed("child 'zz' not valid here.")),
        };
        assert_eq!(
            with.game_text(),
            Some(&VanillaText::Fixed("child 'zz' not valid here."))
        );
        assert_eq!(
            DiagnosticCode::UnrecognizedField { text: None }.game_text(),
            None
        );
        assert_eq!(DiagnosticCode::GenericGoalSuffixMissing.game_text(), None);
    }
}
