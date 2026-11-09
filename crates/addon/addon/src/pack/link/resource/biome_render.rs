//! The fog and the deferred-rendering settings a client biome names, the fogs
//! `biomes_client.json` names, and the fog `biomes_client.json` puts over a client biome's.
//!
//! A client biome's `minecraft:fog_appearance` names a fog definition of `fogs/`, and its
//! five identifier components the lighting, water, atmosphere, colour-grading and cubemap
//! settings of the deferred renderer. The client resolves each when it applies the document;
//! one that names nothing fails the whole document, which still claims its biome
//! ([`Shadowed`]). The merged `biomes_client.json` names fogs too; the client skips one it
//! does not find.

use std::collections::HashSet;

use bedrock_addon_core::document::{ComponentRef, Merged, Origin, RawLayer};

use super::biome::{Claim, Shadowed, claims, is_unknown, path};
use super::biome::{overridden, own_entry};
use crate::diagnostic::{Diagnostic, JsonPath, Severity, sentence};
use crate::families::biomes_client::{LegacyClientBiome, LegacyClientBiomes};
use crate::families::client_biome::component::{ClientBiomeComponent, FogAppearance};
use crate::id::{BiomeId, FogId};
use crate::json::Value;
use crate::pack::link::prelude::*;

/// One component of a client biome that names a definition: its key, its one member, the
/// kind it names and the client's line for a name nothing defines.
#[derive(Debug)]
struct Named {
    component: &'static str,
    member: &'static str,
    kind: &'static str,
    template: &'static str,
}

// The texts are the client's own; the fog text is the fog-appearance component's
// resolve-and-validate line.
const FOG: Named = Named {
    component: "minecraft:fog_appearance",
    member: "fog_identifier",
    kind: kinds::FOG,
    template: "fog_identifier: cannot find fog definition '{}'",
};
const WATER: Named = Named {
    component: "minecraft:water_identifier",
    member: "water_identifier",
    kind: kinds::WATER_SETTINGS,
    template: "water_identifier: cannot find water definition '{}'",
};
const LIGHTING: Named = Named {
    component: "minecraft:lighting_identifier",
    member: "lighting_identifier",
    kind: kinds::LIGHTING_SETTINGS,
    template: "lighting_identifier: cannot find lighting definition '{}'",
};
const ATMOSPHERE: Named = Named {
    component: "minecraft:atmosphere_identifier",
    member: "atmosphere_identifier",
    kind: kinds::ATMOSPHERE_SETTINGS,
    template: "atmosphere_identifier: cannot find atmosphere definition '{}'",
};
const COLOR_GRADING: Named = Named {
    component: "minecraft:color_grading_identifier",
    member: "color_grading_identifier",
    kind: kinds::COLOR_GRADING_SETTINGS,
    template: "color_grading_identifier: cannot find color grading definition '{}'",
};
const CUBEMAP: Named = Named {
    component: "minecraft:cubemap_identifier",
    member: "cubemap_identifier",
    kind: kinds::CUBEMAP_SETTINGS,
    template: "cubemap_identifier: cannot find cubemap definition '{}'",
};

/// The error at the member of `claim` that names `name`, when `target` is missing, and the
/// game's linked-asset line of the same miss, a second finding, also an error.
fn refused(
    claim: &Claim<'_>,
    named: &Named,
    name: &str,
    target: Target<'_>,
) -> Option<[Diagnostic; 2]> {
    let Target::Missing(hint) = target else {
        return None;
    };
    let text = sentence(named.template, &[name.to_owned()]);
    let member = path(&["components", named.component, named.member]);
    // The linked-asset check raises this finding to an error; it is written as the error that
    // comes out.
    let line = At::new(claim.origin, member.clone(), Severity::Error);
    let at = At::new(claim.origin, member, Severity::Error).with_text(text);
    Some([
        unresolved(at, named.kind, name, hint),
        validator_miss(line, Validator::Missing, 1, named.kind, name),
    ])
}

/// What the client says of one component the author wrote: the error of the name it does not
/// find, if it names one.
fn component(
    cx: &LinkCx<'_>,
    claim: &Claim<'_>,
    component: &ClientBiomeComponent,
) -> Option<[Diagnostic; 2]> {
    /// A settings identifier: the empty name is accepted.
    macro_rules! setting {
        ($named:expr, $id:expr, $lookup:ident) => {{
            let id = $id;
            if id.as_str().is_empty() {
                return None;
            }
            refused(claim, &$named, id.as_str(), cx.$lookup(id))
        }};
    }
    match component {
        // Compared as written.
        ClientBiomeComponent::FogAppearance(fog) => refused(
            claim,
            &FOG,
            fog.fog_identifier.as_str(),
            cx.fog(&fog.fog_identifier),
        ),
        ClientBiomeComponent::WaterIdentifier(water) => {
            setting!(WATER, &water.water_identifier, water_settings)
        }
        ClientBiomeComponent::LightingIdentifier(lighting) => {
            setting!(LIGHTING, &lighting.lighting_identifier, lighting_settings)
        }
        ClientBiomeComponent::AtmosphereIdentifier(atmosphere) => setting!(
            ATMOSPHERE,
            &atmosphere.atmosphere_identifier,
            atmosphere_settings
        ),
        ClientBiomeComponent::ColorGradingIdentifier(grading) => setting!(
            COLOR_GRADING,
            &grading.color_grading_identifier,
            color_grading_settings
        ),
        ClientBiomeComponent::CubemapIdentifier(cubemap) => {
            // The game has the default cubemap under its fixed identifier whether or not a
            // pack defines it.
            let defaults = &cx.resource().deferred_settings().defaults;
            if cubemap.cubemap_identifier == defaults.cubemap {
                return None;
            }
            setting!(CUBEMAP, &cubemap.cubemap_identifier, cubemap_settings)
        }
        _ => None,
    }
}

/// On each claiming client biome, the
/// `fog_identifier` of `minecraft:fog_appearance` must be a fog definition of the stack or
/// the game's, compared as written, and the identifier of each of
/// `minecraft:water_identifier`, `lighting_identifier`, `atmosphere_identifier`,
/// `color_grading_identifier` and `cubemap_identifier` a settings definition of its kind;
/// else `UnresolvedReference`, an error with the client's text at the member: the client
/// applies nothing of the document. An empty settings name is accepted, and the default
/// cubemap always exists. A component an upgrade step inserted is not the author's and is
/// not asked: the client fills it from its own registry. A refused claim that shadows lower
/// packs is told once per pack, unless the biome itself is unknown (told with that).
// A biome without the component uses `DeferredSettings::defaults`: nothing to resolve.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let shadowed = Shadowed::default();
    for claim in claims(cx) {
        let mut any = false;
        for entry in claim.components.iter() {
            let ComponentRef::Typed {
                value,
                synthesized: false,
                ..
            } = entry
            else {
                continue;
            };
            if let Some(findings) = component(cx, &claim, value) {
                for finding in findings {
                    out.push(finding);
                }
                any = true;
            }
        }
        if any && !is_unknown(cx, claim.identifier) {
            shadowed.tell(cx, out, &claim);
        }
    }
}

/// The entry `key` of one pack's `biomes_client.json` as the file has it, with the key the
/// file wrote: the name without `minecraft:` first (the game moves such an entry over the
/// namespaced one), then the full name.
fn written_entry<'l>(layer: &'l RawLayer, key: &BiomeId) -> Option<(&'l str, &'l Value)> {
    let biomes = layer.json.as_object()?.get("biomes")?.as_object()?;
    let bare = key.as_str().strip_prefix("minecraft:");
    bare.into_iter()
        .chain([key.as_str()])
        .find_map(|written| biomes.get_key_value(written))
        .map(|(written, entry)| (written.as_str(), entry))
}

/// The style of a biome entry's fog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Style {
    /// The fog of a current entry: `fog_identifier` or an index of `fog_ids_to_merge`.
    Current,
    /// The fog of an old-style entry.
    Old,
}

/// Where the packs' files have the fog `name` of the merged entry `key`: the highest pack
/// whose entry names it, in `fog_identifier` (which the merge turns into the list) or at an
/// index of `fog_ids_to_merge`. `style` is [`Style::Old`] for the fog of an old-style entry, which the game reads
/// from `fog_identifier` alone and gives `minecraft:` when it has no namespace.
fn written<'m>(
    merged: &'m Merged<LegacyClientBiomes, RawLayer>,
    key: &BiomeId,
    name: &str,
    style: Style,
) -> Option<(&'m Origin, JsonPath)> {
    let same = |text: &str| {
        text == name
            || (style == Style::Old
                && !text.contains(':')
                && name.strip_prefix("minecraft:") == Some(text))
    };
    merged.layers().iter().find_map(|layer| {
        let (written_key, entry) = written_entry(layer, key)?;
        let entry = entry.as_object()?;
        let mut at = JsonPath::new();
        at.push("biomes");
        at.push(written_key);
        let identifier = entry.get("fog_identifier").and_then(Value::as_string);
        if identifier.is_some_and(|text| same(text.as_str())) {
            at.push("fog_identifier");
            return Some((&layer.origin, at));
        }
        if style == Style::Old {
            return None;
        }
        let index = entry
            .get("fog_ids_to_merge")
            .and_then(Value::as_array)?
            .iter()
            .position(|element| {
                element
                    .as_string()
                    .is_some_and(|text| text.as_str() == name)
            })?;
        at.push("fog_ids_to_merge");
        at.push_index(index);
        Some((&layer.origin, at))
    })
}

/// The information of one fog name of the merged entry `key` that is no fog definition.
fn legacy_fog(
    merged: &Merged<LegacyClientBiomes, RawLayer>,
    key: &BiomeId,
    name: &FogId,
    style: Style,
    target: Target<'_>,
) -> Option<Diagnostic> {
    let Target::Missing(hint) = target else {
        return None;
    };
    let (origin, at) = written(merged, key, name.as_str(), style).or_else(|| {
        // The merged value alone names it: told at the pack the entry is from.
        let mut at = JsonPath::new();
        at.push("biomes");
        at.push(key.as_str());
        at.push(match style {
            Style::Old => "fog_identifier",
            Style::Current => "fog_ids_to_merge",
        });
        Some((merged.origin(key)?, at))
    })?;
    // The client resolves the fog with no content-log line and the linked-asset check holds no
    // reference of it: the crate reports an information.
    let at = At::new(origin, at, Severity::Info);
    Some(unresolved(at, kinds::FOG, name.as_str(), hint))
}

/// The entries of the merged file, `default` first, each with its key.
fn entries<'l>(
    legacy: &'l LegacyClientBiomes,
    default_key: &'l BiomeId,
) -> impl Iterator<Item = (&'l BiomeId, &'l LegacyClientBiome)> {
    legacy
        .default
        .as_ref()
        .map(|entry| (default_key, entry))
        .into_iter()
        .chain(&legacy.biomes)
}

/// Each fog the merged `biomes_client.json` gives an entry (`fog_ids_to_merge`, which
/// the merge builds from the packs' `fog_identifier`s, and the `fog_identifier` of an
/// old-style fog) must be a fog definition of the stack or the game's, compared as written;
/// else `UnresolvedReference`, an information at the member of the highest pack's file that
/// names it, once per name and entry: the client skips the fog. The text is the crate's own:
/// the client logs nothing for this.
pub(super) fn legacy_fogs(cx: &LinkCx<'_>, out: &mut Findings) {
    let merged = cx.resource().legacy_client_biomes();
    let default_key = BiomeId::parse("default");
    for (key, entry) in entries(merged.value(), &default_key) {
        let mut seen: HashSet<&FogId> = HashSet::new();
        for name in &entry.fog_ids_to_merge {
            if seen.insert(name) {
                out.extend(legacy_fog(merged, key, name, Style::Current, cx.fog(name)));
            }
        }
        let old = entry
            .old_fog
            .as_ref()
            .and_then(|fog| fog.fog_identifier.as_ref());
        if let Some(name) = old
            && seen.insert(name)
        {
            out.extend(legacy_fog(merged, key, name, Style::Old, cx.fog(name)));
        }
    }
}

/// The fog of `biomes_client.json` over a client biome's: a client biome with
/// `minecraft:fog_appearance` whose biome
/// has an entry of its own in the merged `biomes_client.json` that yields fog (an old-style
/// fog, or fogs to merge) is a `ClientBiomeOverriddenByBiomesClient` information at the
/// component, naming the pack of the entry: the client stores the entry's fog over the
/// component's. An entry whose one fog is the component's overrides nothing. The `default`
/// entry is the fog of biomes without one and replaces no biome's.
pub(super) fn fog_overrides(cx: &LinkCx<'_>, out: &mut Findings) {
    let merged = cx.resource().legacy_client_biomes();
    let legacy = merged.value();
    if legacy.biomes.is_empty() {
        return;
    }
    for (identifier, document) in cx.resource().client_biomes() {
        let Some(fog) = document.value().components.get::<FogAppearance>() else {
            continue;
        };
        let Some((key, entry)) = own_entry(legacy, identifier) else {
            continue;
        };
        let yields = entry.old_fog.is_some() || !entry.fog_ids_to_merge.is_empty();
        let same = entry.old_fog.is_none()
            && entry.fog_ids_to_merge.len() == 1
            && entry.fog_ids_to_merge.first() == Some(&fog.fog_identifier);
        if !yields || same {
            continue;
        }
        if let Some(origin) = merged.origin(key) {
            out.push(overridden(
                document.origin(),
                "minecraft:fog_appearance",
                &origin.pack,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::load::LoadOptions;

    use crate::pack::StackSet;
    use crate::pack::link::tests::support::{pack, set_of};
    use crate::pack::link::{LinkOptions, link_with_rules};

    /// `text` for the life of the test process.
    fn leak(text: String) -> &'static str {
        Box::leak(text.into_boxed_str())
    }

    /// A world of two resource packs with `n` client biomes each, for the same `n` biomes no
    /// behaviour pack defines: the top pack's documents name a fog, a lighting definition and
    /// a water surface, the bottom pack defines the fog and the lighting of every second one,
    /// and the top pack's `biomes_client.json` has an entry for each biome. Every collection
    /// the rules read is loaded.
    fn world(n: usize) -> StackSet {
        let mut top: Vec<(&'static str, Option<&str>)> = Vec::new();
        let mut bottom: Vec<(&'static str, Option<&str>)> = Vec::new();
        let mut entries = Vec::new();
        for i in 0..n {
            top.push((
                leak(format!("biomes/b{i}.json")),
                Some(leak(format!(
                    r##"{{"format_version": "1.21.130", "minecraft:client_biome": {{"description": {{"identifier": "demo:b{i}"}},
                        "components": {{"minecraft:fog_appearance": {{"fog_identifier": "demo:f{i}"}},
                        "minecraft:lighting_identifier": {{"lighting_identifier": "demo:l{i}"}},
                        "minecraft:water_appearance": {{"surface_color": "#123456"}}}}}}}}"##
                ))),
            ));
            bottom.push((
                leak(format!("biomes/b{i}.json")),
                Some(leak(format!(
                    r#"{{"format_version": "1.21.130", "minecraft:client_biome": {{"description": {{"identifier": "demo:b{i}"}}, "components": {{}}}}}}"#
                ))),
            ));
            if i % 2 == 0 {
                bottom.push((
                    leak(format!("fogs/f{i}.json")),
                    Some(leak(format!(
                        r#"{{"format_version": "1.26.20", "minecraft:fog_settings": {{"description": {{"identifier": "demo:f{i}"}}}}}}"#
                    ))),
                ));
                bottom.push((
                    leak(format!("lighting/l{i}.json")),
                    Some(leak(format!(
                        r#"{{"format_version": "1.26.0", "minecraft:lighting_settings": {{"description": {{"identifier": "demo:l{i}"}}}}}}"#
                    ))),
                ));
            }
            entries.push(format!(
                r##""demo:b{i}": {{"fog_ids_to_merge": ["demo:f{i}"], "water_surface_color": "#000001"}}"##
            ));
        }
        top.push((
            "biomes_client.json",
            Some(leak(format!(r#"{{"biomes": {{{}}}}}"#, entries.join(", ")))),
        ));
        let set = set_of(
            Vec::new(),
            vec![pack(2, "resources", &top), pack(3, "resources", &bottom)],
            &LoadOptions::default(),
        );
        assert_eq!(set.resource.client_biomes().len(), n);
        assert_eq!(set.resource.fogs().len(), n.div_ceil(2));
        assert_eq!(
            set.resource.deferred_settings().lighting.len(),
            n.div_ceil(2)
        );
        assert_eq!(set.resource.legacy_client_biomes().value().biomes.len(), n);
        assert!(set.behavior.biomes().is_empty());
        set
    }

    /// The cost of the client-biome rules: `n` claiming documents, each refused for its
    /// biome (with the shadowed pack found by reading the lower pack's file again) and half
    /// of them for a fog and a setting too, `n` legacy entries and `n` water overrides,
    /// against four times as many.
    #[test]
    fn the_client_biome_rules_over_n_biomes_cost_n() {
        crate::test_cost::assert_linear_at(
            "link: the client-biome rules over n biomes",
            400,
            world,
            |set: &StackSet| {
                let n = set.resource.client_biomes().len();
                let found = link_with_rules(
                    set,
                    LinkOptions::default(),
                    &[
                        crate::pack::link::resource::biome::run,
                        super::run,
                        super::legacy_fogs,
                        crate::pack::link::resource::biome::water,
                        super::fog_overrides,
                    ],
                );
                // The biome and its shadowed pack for each; a fog, a setting and a legacy fog
                // for every second, the fog and the setting each with the game's linked-asset
                // line; a water override for each; the entry's one fog is the
                // component's, so no fog override.
                assert_eq!(found.len(), 3 * n + 5 * (n / 2));
                found.len()
            },
        );
    }
}
