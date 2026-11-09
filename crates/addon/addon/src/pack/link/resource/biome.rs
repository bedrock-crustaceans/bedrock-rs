//! The biome a client biome is for, what the claim of a document the link pass refuses costs
//! the packs below it, and the water surface `biomes_client.json` puts over a client biome's.
//!
//! The client applies a client-biome document only to a biome the server names. The stack is
//! walked top first and the first document of an identifier that loads claims it, before
//! anything else is asked of it: a document the client then refuses (no such biome, a fog or
//! render setting nothing defines) applies nothing, and the documents of the packs below it
//! for the same biome are not used either. The pass changes no registry: it tells the
//! refusal as an error and the claim as one information for each pack it shadows.
//!
//! `biomes_client.json` is applied after the client biomes and writes the same water
//! surface: where it replaces a value a client biome's `minecraft:water_appearance` set, the
//! pass says so at the component.
// The registry is as the load left it.

use std::cell::OnceCell;
use std::collections::HashMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::{ComponentMap, Origin, VersionInfo};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

use crate::diagnostic::{Code, Diagnostic, JsonPath, ReferenceHint, Severity, sentence};
#[cfg(feature = "biomes_client")]
use crate::families::biomes_client::{LegacyClientBiome, LegacyClientBiomes};
use crate::families::client_biome::component::ClientBiomeComponent;
#[cfg(feature = "biomes_client")]
use crate::families::client_biome::component::WaterAppearance;
use crate::id::BiomeId;
use crate::pack::link::prelude::*;

/// The client's line for a client biome of a biome the server does not name. The client logs
/// it at the error level (content log, `BiomeRegistry`).
const NO_BIOME: &str = "Loaded client biome but no biome with that name exists: {}";

/// The key of the entry of `biomes_client.json` every biome takes its water from.
#[cfg(feature = "biomes_client")]
const DEFAULT: &str = "default";

/// A client-biome document that claims its identifier: the effective document of the
/// registry, stable or beta.
#[derive(Debug)]
pub(super) struct Claim<'a> {
    /// The biome the document is for (`description.identifier`, with `minecraft:` when an
    /// older document wrote it bare).
    pub(super) identifier: &'a BiomeId,
    /// The document's pack and file.
    pub(super) origin: &'a Origin,
    /// The components, upgraded to the newest format.
    #[cfg_attr(
        not(any(
            all(
                feature = "biomes_client",
                feature = "fog",
                feature = "deferred_settings"
            ),
            feature = "sounds_json"
        )),
        allow(
            dead_code,
            reason = "read by the rules of the `fog`, `deferred_settings` and `sounds_json` features"
        )
    )]
    pub(super) components: &'a ComponentMap<ClientBiomeComponent>,
    /// The version the file was written at.
    #[cfg_attr(
        not(feature = "sounds_json"),
        allow(dead_code, reason = "read by the rule of the `sounds_json` feature")
    )]
    pub(super) version: &'a VersionInfo,
}

/// Every claiming document the registry holds, the stable ones then the beta ones, each in
/// load order. A document a higher pack shadows is not one of them: the client never looks
/// at it.
pub(super) fn claims<'a>(cx: &LinkCx<'a>) -> impl Iterator<Item = Claim<'a>> {
    let registry = cx.resource().client_biomes();
    let stable = registry.iter().map(|(identifier, document)| Claim {
        identifier,
        origin: document.origin(),
        components: &document.value().components,
        version: document.version(),
    });
    let beta = registry.beta().into_iter().flat_map(|beta| {
        beta.iter().map(|(identifier, document)| Claim {
            identifier,
            origin: document.origin(),
            components: &document.value().components,
            version: document.version(),
        })
    });
    stable.chain(beta)
}

/// `minecraft:client_biome/<keys…>`: a place in a client-biome file.
pub(super) fn path(keys: &[&str]) -> JsonPath {
    let mut path = JsonPath::new();
    path.push("minecraft:client_biome");
    for key in keys {
        path.push(key);
    }
    path
}

/// Whether the server names no biome `identifier`: no behaviour pack of the stack defines it
/// and it is none of the game's.
#[cfg(all(
    feature = "biomes_client",
    feature = "fog",
    feature = "deferred_settings"
))]
pub(super) fn is_unknown(cx: &LinkCx<'_>, identifier: &BiomeId) -> bool {
    cx.biome(identifier).is_missing()
}

/// The error at the identifier of a document whose biome the server does not name.
fn unknown(cx: &LinkCx<'_>, identifier: &BiomeId, origin: &Origin) -> Option<Diagnostic> {
    let Target::Missing(hint) = cx.biome(identifier) else {
        return None;
    };
    let text = sentence(NO_BIOME, &[identifier.as_str().to_owned()]);
    let at = At::new(
        origin,
        path(&["description", "identifier"]),
        Severity::Error,
    )
    .with_text(text);
    Some(unresolved(at, kinds::BIOME, identifier.as_str(), hint))
}

/// The identifier the file `path` of `pack` names when it loads as a client biome: read
/// again from the file's bytes, since the registry keeps neither a document the client's
/// component checks refused nor one a higher pack shadows. `None` for a file that does not
/// load or whose bytes the pack does not hold.
fn identifier_of(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    pack: &PackRef,
    path: &PackPath,
) -> Option<BiomeId> {
    let entry = cx.stack_pack(PackSide::Resource, pack)?;
    let bytes = cx.bytes(PackSide::Resource, pack, path, out)?;
    let origin = Origin::file(pack.clone(), PackSide::Resource, path.clone());
    crate::families::client_biome::identifier_in(&origin, &bytes, &entry.context)
}

/// The packs whose client-biome documents a higher pack's claim shadows, by biome: found on
/// the first claim the pass refuses, from the load's own notes of the shadowed files.
#[derive(Debug, Default)]
pub(super) struct Shadowed<'a> {
    by_identifier: OnceCell<HashMap<BiomeId, Vec<&'a PackRef>>>,
}

impl<'a> Shadowed<'a> {
    /// Tells what the refused `claim` costs: one `ClientBiomeShadowsLowerPacks` information at
    /// the claiming document's identifier for each pack below it that has a document for the
    /// same biome, which the client does not use either. The pack is named in the detail. The
    /// client says nothing of it, so the crate reports an information. The code carries no
    /// payload, so the shadowed pack is the finding's `Reference` detail, by its name.
    pub(super) fn tell(&self, cx: &LinkCx<'a>, out: &mut Findings, claim: &Claim<'_>) {
        let by_identifier = self.by_identifier.get_or_init(|| shadowed(cx, out));
        let Some(packs) = by_identifier.get(claim.identifier) else {
            return;
        };
        for pack in packs {
            let at = At::new(
                claim.origin,
                path(&["description", "identifier"]),
                Severity::Info,
            );
            out.push(at.reference(
                Code::from(&core_codes::CLIENT_BIOME_SHADOWS_LOWER_PACKS),
                kinds::PACK,
                pack.name(),
                Some(ReferenceHint::Shadowed),
            ));
        }
    }
}

/// Every pack with a client-biome document a higher pack's shadows, by the document's biome,
/// top first.
fn shadowed<'a>(cx: &LinkCx<'a>, out: &mut Findings) -> HashMap<BiomeId, Vec<&'a PackRef>> {
    let mut by_identifier: HashMap<BiomeId, Vec<&'a PackRef>> = HashMap::new();
    // The load tells each loaded document a higher pack shadows as a `ShadowedDefinition`
    // at its file.
    for finding in cx.resource().client_biomes().diagnostics() {
        if !matches!(
            finding.code.id(),
            "core/shadowed_definition" | "core/shadowed_definition_worded"
        ) {
            continue;
        }
        let (Some(pack), Some(file)) = (&finding.location.pack, &finding.location.path) else {
            continue;
        };
        let Some(identifier) = identifier_of(cx, out, pack, file) else {
            continue;
        };
        let packs = by_identifier.entry(identifier).or_default();
        if !packs.contains(&pack) {
            packs.push(pack);
        }
    }
    by_identifier
}

/// A client biome whose identifier names no biome of the behaviour stack and none of the
/// game's is an `UnresolvedReference` error with the client's text, at
/// `description.identifier`: the client applies nothing of the document.
/// A document the component checks of the load refused is asked the same (it still claimed
/// its identifier). A refused claim that shadows lower packs is told once per pack.
// The pass suppresses nothing: the load's component error stands beside this one.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let registry = cx.resource().client_biomes();
    let shadowed = Shadowed::default();
    for claim in claims(cx) {
        if let Some(finding) = unknown(cx, claim.identifier, claim.origin) {
            out.push(finding);
            shadowed.tell(cx, out, &claim);
        }
    }
    // A claim the load's component checks refused is no document of the registry; the load
    // told what it shadows.
    for rejected in registry.rejected() {
        let origin = &rejected.origin;
        let Some(identifier) = identifier_of(cx, out, &origin.pack, &origin.path) else {
            continue;
        };
        out.extend(unknown(cx, &identifier, origin));
    }
}

/// The entry of `legacy` for `biome` with its key: under the full name, else, for a
/// `minecraft` biome, under the name without the namespace
/// ([`LegacyClientBiomes::get`]).
#[cfg(feature = "biomes_client")]
pub(super) fn own_entry<'l>(
    legacy: &'l LegacyClientBiomes,
    biome: &BiomeId,
) -> Option<(&'l BiomeId, &'l LegacyClientBiome)> {
    legacy.biomes.get_key_value(biome).or_else(|| {
        let (namespace, name) = biome.as_str().split_once(':')?;
        (namespace == "minecraft")
            .then(|| legacy.biomes.get_key_value(&BiomeId::parse(name)))
            .flatten()
    })
}

/// The `ClientBiomeOverriddenByBiomesClient` information at the component `component` of
/// the document at `origin`, naming the pack whose `biomes_client.json` overrides it.
// The game overwrites the component's values in silence, so the crate reports an
// information.
#[cfg(feature = "biomes_client")]
pub(super) fn overridden(origin: &Origin, component: &str, by: &PackRef) -> Diagnostic {
    let at = At::new(origin, path(&["components", component]), Severity::Info);
    at.reference(
        Code::from(&core_codes::CLIENT_BIOME_OVERRIDDEN_BY_BIOMES_CLIENT),
        kinds::PACK,
        by.name(),
        None,
    )
}

/// The water surface of `biomes_client.json` over a client biome's: a client biome whose
/// `minecraft:water_appearance` sets a value that the merged `biomes_client.json` replaces
/// by another (its `default` entry, for every biome, or the biome's own entry) is a
/// `ClientBiomeOverriddenByBiomesClient` information at the component, once for each pack whose
/// entry writes a member the component set; the pack is named in the detail. Entries that
/// write the values the component has override nothing.
#[cfg(feature = "biomes_client")]
pub(super) fn water(cx: &LinkCx<'_>, out: &mut Findings) {
    let merged = cx.resource().legacy_client_biomes();
    let legacy = merged.value();
    if legacy.default.is_none() && legacy.biomes.is_empty() {
        return;
    }
    let default_key = BiomeId::parse(DEFAULT);
    for (identifier, document) in cx.resource().client_biomes() {
        let client = document.value();
        let Some(water) = client.components.get::<WaterAppearance>() else {
            continue;
        };
        let (color, opacity) = client.water_surface();
        if !legacy
            .effective_water_surface(color, opacity, identifier)
            .overridden
        {
            continue;
        }
        let writes = |entry: &LegacyClientBiome| {
            (entry.water_surface_color.is_some() && water.surface_color.is_some())
                || (entry.water_surface_transparency.is_some() && water.surface_opacity.is_some())
        };
        let default = legacy.default.as_ref().map(|entry| (&default_key, entry));
        let mut packs: Vec<&PackRef> = Vec::new();
        for (key, entry) in default.into_iter().chain(own_entry(legacy, identifier)) {
            let Some(origin) = merged.origin(key) else {
                continue;
            };
            if writes(entry) && !packs.contains(&&origin.pack) {
                packs.push(&origin.pack);
            }
        }
        for pack in packs {
            out.push(overridden(
                document.origin(),
                "minecraft:water_appearance",
                pack,
            ));
        }
    }
}
