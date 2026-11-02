//! 20 R38 and R39: what a particle effect names. Its material and its texture (R38, the
//! existence half), the events its components fire, the effect and the sound an event starts,
//! and the blocks a particle expires in (R39).

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Phase, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

// The material and atlas files of these worlds are a pack the game ships: the rows
// are tested at their own severity. `tests/link/view.rs` has the other views.
use crate::common::{game_link_with as link_with, hosted, in_file};

const FILE: &str = "particles/a.json";
const BRP: &str = "particle_effect/description/basic_render_parameters";
const COMPONENTS: &str = "particle_effect/components";

/// The two files that give the resource pack the material `mine` with the variant `glow`.
const MATERIALS: [(&str, &str); 2] = [
    (
        "materials/common.json",
        r#"[{"path": "materials/my.material"}]"#,
    ),
    (
        "materials/my.material",
        r#"{"materials": {"version": "1.0.0", "mine": {"variants": [{"glow": {}}]}}}"#,
    ),
];

/// The image at `textures/particle/mine`: a load reads no image, so any bytes do.
const TEXTURE: (&str, &str) = ("textures/particle/mine.png", "no image is read");

/// An effect `demo:a` of `version` drawn with `material` and `texture`, with the JSON
/// `members` (`"components": {..}`, `"events": {..}`) after its description.
fn effect_at(version: &str, material: &str, texture: &str, members: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "particle_effect": {{"description": {{"identifier": "demo:a", "basic_render_parameters": {{"material": "{material}", "texture": "{texture}"}}}}, {members}}}}}"#
    )
}

/// An effect of the newest format drawn with the pack's own material and texture, so that
/// R38 says nothing of it.
fn effect(members: &str) -> String {
    effect_at("1.26.10", "mine", "textures/particle/mine", members)
}

/// The components every effect needs, and `more` beside them.
fn components(more: &str) -> String {
    let separator = if more.is_empty() { "" } else { ", " };
    format!(
        r#""components": {{"minecraft:emitter_lifetime_once": {{}}, "minecraft:emitter_rate_instant": {{"num_particles": 1}}{separator}{more}}}"#
    )
}

/// The resource files of a world whose one effect is `text`, with the pack's material and
/// texture.
fn resources(text: &str) -> Vec<(&'static str, String)> {
    let mut files = vec![(FILE, text.to_owned())];
    files.extend(MATERIALS.map(|(path, text)| (path, text.to_owned())));
    files.push((TEXTURE.0, TEXTURE.1.to_owned()));
    files
}

/// The link findings at the effect's file, in a world with `behavior` files, loaded with
/// `load` and linked with `options`.
fn linked_with(
    text: &str,
    behavior: &[(&'static str, &str)],
    load: LoadOptions,
    options: LinkOptions,
) -> Vec<Diagnostic> {
    let files = resources(text);
    let files: Vec<(&'static str, &str)> = files
        .iter()
        .map(|(path, text)| (*path, text.as_str()))
        .collect();
    let found = link_with(behavior, &files, load, options);
    at_effect(&found)
}

/// [`linked_with`] without a behaviour file, a host or an option.
fn linked(text: &str) -> Vec<Diagnostic> {
    linked_with(text, &[], LoadOptions::default(), LinkOptions::default())
}

/// The findings at the effect's file.
fn at_effect(found: &Diagnostics) -> Vec<Diagnostic> {
    in_file(found, FILE).into_iter().cloned().collect()
}

/// What a finding says of a reference: code, severity, JSON path, kind and name as written.
type Said = (&'static str, Severity, String, &'static str, String);

fn said(finding: &Diagnostic) -> Said {
    let (kind, id) = match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => (*kind, id.clone()),
        other => panic!("no reference: {other:?}"),
    };
    (
        finding.code.name(),
        finding.severity,
        finding.location.json_path.to_string(),
        kind,
        id,
    )
}

fn all_said(found: &[Diagnostic]) -> Vec<Said> {
    found.iter().map(said).collect()
}

/// A finding of the rules here: a warning of the link phase at the effect's file in `rp 2`.
/// The same as an error: the game's link pass logs the reference at the content log's error
/// level.
fn error(code: &'static str, path: &str, kind: &'static str, id: &str) -> Said {
    (code, Severity::Error, path.to_owned(), kind, id.to_owned())
}

/// A finding of ours: the game resolves the name without a line.
fn info(code: &'static str, path: &str, kind: &'static str, id: &str) -> Said {
    (code, Severity::Info, path.to_owned(), kind, id.to_owned())
}

/// Every finding is of the link phase, in the resource pack, at the effect's file.
#[track_caller]
fn placed(found: &[Diagnostic]) {
    for finding in found {
        assert_eq!(finding.phase, Phase::Link, "{finding}");
        let pack = finding
            .location
            .pack
            .as_ref()
            .map(bedrock_addon::pack::PackRef::name);
        assert_eq!(pack, Some("rp 2"), "{finding}");
        let path = finding
            .location
            .path
            .as_ref()
            .map(bedrock_addon::storage::PackPath::as_str);
        assert_eq!(path, Some(FILE), "{finding}");
    }
}

// ---- R38: material -----------------------------------------------------------------------

#[cfg(feature = "material")]
mod material {
    use super::*;

    /// The findings of an effect drawn with `material` (and the pack's texture).
    fn drawn_with(material: &str, load: LoadOptions) -> Vec<Diagnostic> {
        let text = effect_at(
            "1.26.10",
            material,
            "textures/particle/mine",
            &components(""),
        );
        linked_with(&text, &[], load, LinkOptions::default())
    }

    #[test]
    fn r38_a_material_a_pack_defines_is_no_finding() {
        assert_eq!(all_said(&drawn_with("mine", LoadOptions::default())), []);
        // A variant is named `<material>.<variant>`.
        assert_eq!(
            all_said(&drawn_with("mine.glow", LoadOptions::default())),
            []
        );
    }

    #[test]
    fn r38_a_material_nothing_defines_is_an_unresolved_reference_information() {
        let found = drawn_with("no_such_material", LoadOptions::default());
        placed(&found);
        let path = format!("{BRP}/material");
        assert_eq!(
            all_said(&found),
            [info(
                "UnresolvedReference",
                &path,
                "material",
                "no_such_material"
            )]
        );
        // Ours: the finding has no text of vanilla's.
        assert_eq!(found[0].code.to_string(), "unresolved reference");
    }

    #[test]
    fn r38_a_material_name_is_compared_as_written() {
        let path = format!("{BRP}/material");
        for name in ["Mine", "mine.dim", "mine "] {
            assert_eq!(
                all_said(&drawn_with(name, LoadOptions::default())),
                [info("UnresolvedReference", &path, "material", name)],
                "{name}"
            );
        }
    }

    #[test]
    fn r38_a_material_of_the_game_resolves_with_a_host_that_has_it_and_not_without_one() {
        let path = format!("{BRP}/material");
        let missing = [info(
            "UnresolvedReference",
            &path,
            "material",
            "particles_alpha",
        )];
        // Without a host the crate has no list of the game's materials.
        assert_eq!(
            all_said(&drawn_with("particles_alpha", LoadOptions::default())),
            missing
        );
        let host = hosted(&[("material", "particles_alpha")]);
        assert_eq!(all_said(&drawn_with("particles_alpha", host)), []);
        // A host is believed when it says no; the stack's own material still resolves.
        let other = hosted(&[("material", "particles_blend")]);
        assert_eq!(
            all_said(&drawn_with("particles_alpha", other.clone())),
            missing
        );
        assert_eq!(all_said(&drawn_with("mine", other)), []);
    }

    /// a kind is judged only where the stack or a host has a view of it. A stack
    /// whose packs create no material and that has no host cannot tell a material of the
    /// game from a missing one, and says nothing; one material source, or a host, and the
    /// same name is told as the row says.
    #[test]
    fn r38_a_stack_without_a_material_source_and_without_a_host_judges_no_material() {
        let text = effect_at(
            "1.26.10",
            "no_such_material",
            "textures/particle/mine",
            &components(""),
        );
        let path = format!("{BRP}/material");
        let told = [info(
            "UnresolvedReference",
            &path,
            "material",
            "no_such_material",
        )];
        let without = [(FILE, text.as_str()), TEXTURE];
        let linked = |files: &[(&'static str, &str)], load: LoadOptions| {
            at_effect(&link_with(&[], files, load, LinkOptions::default()))
        };
        // No source, no host: nothing is said of the kind.
        assert_eq!(all_said(&linked(&without, LoadOptions::default())), []);
        // One source: the stack has the kind and lacks the name.
        let with = [(FILE, text.as_str()), TEXTURE, MATERIALS[0], MATERIALS[1]];
        assert_eq!(all_said(&linked(&with, LoadOptions::default())), told);
        // A host answers for the kind: its no is believed.
        let host = hosted(&[("material", "particles_alpha")]);
        assert_eq!(all_said(&linked(&without, host)), told);
    }

    #[test]
    fn r38_a_legacy_effect_is_held_to_the_rule_at_the_same_member() {
        let text = effect_at("1.10.0", "nope", "textures/particle/mine", &components(""));
        let path = format!("{BRP}/material");
        assert_eq!(
            all_said(&linked(&text)),
            [info("UnresolvedReference", &path, "material", "nope")]
        );
    }

    /// The presence half is the load's: a legacy effect without a material is told there
    /// (`particle has no material!`), and the link pass adds nothing about the empty name.
    #[test]
    fn r38_an_effect_without_a_material_is_the_loads_finding_alone() {
        let text = format!(
            r#"{{"format_version": "1.10.0", "particle_effect": {{"description": {{"identifier": "demo:a", "basic_render_parameters": {{"texture": "textures/particle/mine"}}}}, {}}}}}"#,
            components("")
        );
        assert_eq!(all_said(&linked(&text)), []);
    }
}

/// Without the `render` feature the stack has no materials and the rule does not exist.
#[cfg(not(feature = "material"))]
#[test]
fn r38_without_the_material_feature_no_material_is_looked_up() {
    let text = effect_at(
        "1.26.10",
        "no_such_material",
        "textures/particle/mine",
        &components(""),
    );
    let found = linked(&text);
    assert!(
        found
            .iter()
            .all(|finding| !finding.location.json_path.to_string().ends_with("material")),
        "{found:?}"
    );
}

// ---- R38: texture ------------------------------------------------------------------------

#[cfg(all(
    feature = "texture_atlas",
    feature = "texture_file",
    feature = "texture_set"
))]
mod texture {
    use super::*;

    /// The texture findings of an effect drawn with `texture`.
    fn drawn_with(texture: &str, load: LoadOptions) -> Vec<Said> {
        let text = effect_at("1.26.10", "mine", texture, &components(""));
        let found = linked_with(&text, &[], load, LinkOptions::default());
        placed(&found);
        let mut said = all_said(&found);
        said.retain(|said| said.0 == "TextureNotFound");
        said
    }

    #[test]
    fn r38_a_texture_a_pack_has_is_no_finding() {
        assert_eq!(
            drawn_with("textures/particle/mine", LoadOptions::default()),
            []
        );
    }

    #[test]
    fn r38_a_texture_no_pack_has_is_a_texture_not_found_error() {
        let path = format!("{BRP}/texture");
        assert_eq!(
            drawn_with("textures/particle/none", LoadOptions::default()),
            [error(
                "TextureNotFound",
                &path,
                "texture",
                "textures/particle/none"
            )]
        );
        // The path is the whole path from the pack's root: a part of it is no texture.
        for texture in ["particle/mine", "textures/particle"] {
            assert_eq!(
                drawn_with(texture, LoadOptions::default()),
                [error("TextureNotFound", &path, "texture", texture)],
                "{texture}"
            );
        }
    }

    /// The frame knows the textures of the stack's packs; a texture of the game's own pack is
    /// missing without a host and found when the host says the game has it (the
    /// game's link pass registers its own files).
    #[test]
    fn r38_a_texture_of_the_game_is_missing_unless_its_pack_is_on_the_stack_or_the_host_has_it() {
        let path = format!("{BRP}/texture");
        let vanilla = "textures/particle/particles";
        let missing = [error("TextureNotFound", &path, "texture", vanilla)];
        assert_eq!(drawn_with(vanilla, LoadOptions::default()), missing);
        assert_eq!(
            drawn_with(
                vanilla,
                hosted(&[("texture", "textures/particle/particles")])
            ),
            []
        );
    }

    /// A particle of a block or of an item is drawn from the atlas, which it names by the
    /// atlas's resource name where another effect has a path.
    #[test]
    fn r38_the_name_of_an_atlas_is_no_texture_path_and_no_finding() {
        for atlas in ["atlas.terrain", "atlas.items"] {
            assert_eq!(drawn_with(atlas, LoadOptions::default()), [], "{atlas}");
        }
        // Exactly the two names the renderers bind.
        let path = format!("{BRP}/texture");
        for other in ["atlas.Terrain", "atlas.blocks", "atlas"] {
            assert_eq!(
                drawn_with(other, LoadOptions::default()),
                [error("TextureNotFound", &path, "texture", other)],
                "{other}"
            );
        }
    }

    #[test]
    fn r38_an_effect_without_a_texture_is_the_loads_finding_alone() {
        let text = format!(
            r#"{{"format_version": "1.10.0", "particle_effect": {{"description": {{"identifier": "demo:a", "basic_render_parameters": {{"material": "mine"}}}}, {}}}}}"#,
            components("")
        );
        assert_eq!(all_said(&linked(&text)), []);
    }
}

/// Without the texture families the stack has no texture list and the rule does not exist.
#[cfg(not(all(
    feature = "texture_atlas",
    feature = "texture_file",
    feature = "texture_set"
)))]
#[test]
fn r38_without_the_texture_families_no_texture_is_looked_up() {
    let text = effect_at("1.26.10", "mine", "textures/particle/none", &components(""));
    let found = linked(&text);
    assert!(
        found
            .iter()
            .all(|finding| finding.code.name() != "TextureNotFound"),
        "{found:?}"
    );
}

// ---- R39: the events a component fires ---------------------------------------------------

/// An effect with `more` components and the events `burst` and `Pop`.
fn firing(more: &str) -> String {
    effect(&format!(
        r#"{}, "events": {{"burst": {{"log": "b"}}, "Pop": {{"log": "p"}}}}"#,
        components(more)
    ))
}

/// R39 is an information: the game fires nothing for an event the effect lacks and says
/// nothing (vanilla's own `obsidian_tear` fires `hit_ground` and defines no event).
fn event(path: &str, name: &str) -> Said {
    (
        "UnresolvedReference",
        Severity::Info,
        path.to_owned(),
        "particle event",
        name.to_owned(),
    )
}

#[test]
fn r39_an_event_of_the_effect_is_no_finding() {
    let text = firing(
        r#""minecraft:emitter_lifetime_events": {"creation_event": "burst", "expiration_event": ["burst", "Pop"], "timeline": {"0.5": "Pop"}, "travel_distance_events": {"2.0": ["burst"]}, "looping_travel_distance_events": [{"distance": 1.0, "effects": "burst"}]}, "minecraft:particle_lifetime_events": {"creation_event": "Pop", "expiration_event": "burst", "timeline": {"1.0": ["burst"]}}, "minecraft:particle_motion_collision": {"collision_radius": 0.1, "events": [{"event": "burst"}, {"event": "Pop", "min_speed": 3.0}]}"#,
    );
    assert_eq!(all_said(&linked(&text)), []);
}

#[test]
fn r39_an_event_name_the_effect_lacks_is_an_unresolved_reference_at_each_member_that_fires_it() {
    let text = firing(
        r#""minecraft:emitter_lifetime_events": {"creation_event": "a", "expiration_event": ["burst", "b"], "timeline": {"0.5": "c", "1.5": ["burst", "d"]}, "travel_distance_events": {"2.0": ["e"]}, "looping_travel_distance_events": [{"distance": 1.0, "effects": "burst"}, {"distance": 2.0, "effects": ["f"]}]}, "minecraft:particle_lifetime_events": {"creation_event": ["g"], "expiration_event": "h", "timeline": {"1.0": "i"}}, "minecraft:particle_motion_collision": {"collision_radius": 0.1, "events": [{"event": "burst"}, {"event": "j"}]}"#,
    );
    let found = linked(&text);
    placed(&found);
    let emitter = format!("{COMPONENTS}/minecraft:emitter_lifetime_events");
    let particle = format!("{COMPONENTS}/minecraft:particle_lifetime_events");
    let collision = format!("{COMPONENTS}/minecraft:particle_motion_collision");
    assert_eq!(
        all_said(&found),
        [
            event(&format!("{emitter}/creation_event"), "a"),
            event(&format!("{emitter}/expiration_event/1"), "b"),
            event(&format!("{emitter}/timeline/0.5"), "c"),
            event(&format!("{emitter}/timeline/1.5/1"), "d"),
            event(&format!("{emitter}/travel_distance_events/2.0/0"), "e"),
            event(
                &format!("{emitter}/looping_travel_distance_events/1/effects/0"),
                "f"
            ),
            event(&format!("{particle}/creation_event/0"), "g"),
            event(&format!("{particle}/expiration_event"), "h"),
            event(&format!("{particle}/timeline/1.0"), "i"),
            event(&format!("{collision}/events/1/event"), "j"),
        ]
    );
}

#[test]
fn r39_an_event_name_is_compared_byte_for_byte() {
    let text = firing(
        r#""minecraft:emitter_lifetime_events": {"creation_event": ["Burst", "pop", "Pop", "burst "]}"#,
    );
    let at = format!("{COMPONENTS}/minecraft:emitter_lifetime_events/creation_event");
    assert_eq!(
        all_said(&linked(&text)),
        [
            event(&format!("{at}/0"), "Burst"),
            event(&format!("{at}/1"), "pop"),
            event(&format!("{at}/3"), "burst "),
        ]
    );
}

#[test]
fn r39_a_single_collision_event_is_told_at_the_object_and_an_effect_without_events_has_none() {
    // No `events` member at all: every name misses.
    let text = effect(&components(
        r#""minecraft:particle_motion_collision": {"collision_radius": 0.1, "events": {"event": "hit"}}"#,
    ));
    let at = format!("{COMPONENTS}/minecraft:particle_motion_collision/events/event");
    assert_eq!(all_said(&linked(&text)), [event(&at, "hit")]);
}

/// The event names of a legacy file are at the same members.
#[test]
fn r39_a_legacy_effect_is_held_to_its_own_events_too() {
    let text = effect_at(
        "1.10.0",
        "mine",
        "textures/particle/mine",
        &format!(
            r#"{}, "events": {{"burst": {{"log": "b"}}}}"#,
            components(
                r#""minecraft:emitter_lifetime_events": {"creation_event": "burst", "expiration_event": "none"}"#
            )
        ),
    );
    let at = format!("{COMPONENTS}/minecraft:emitter_lifetime_events/expiration_event");
    assert_eq!(all_said(&linked(&text)), [event(&at, "none")]);
}

// ---- R39: the effect an event starts -----------------------------------------------------

/// An effect whose events are `events` (the members of the `events` object).
fn with_events(events: &str) -> String {
    effect(&format!(r#"{}, "events": {{{events}}}"#, components("")))
}

fn starts(effect: &str) -> String {
    format!(r#""particle_effect": {{"effect": "{effect}", "type": "emitter"}}"#)
}

#[test]
fn r39_an_effect_of_the_stack_is_no_finding() {
    // The effect itself is one.
    let text = with_events(&format!(r#""again": {{{}}}"#, starts("demo:a")));
    assert_eq!(all_said(&linked(&text)), []);
}

#[test]
fn r39_an_effect_nothing_defines_is_an_unresolved_reference_at_every_node_that_starts_it() {
    let text = with_events(&format!(
        r#""burst": {{{none}, "sequence": [{{{own}}}, {{{b}, "randomize": [{{"weight": 1, {c}}}]}}], "randomize": [{{"weight": 1, {own}}}, {{"weight": 2, {d}}}]}}, "later": {{{e}}}"#,
        none = starts("demo:none"),
        own = starts("demo:a"),
        b = starts("demo:b"),
        c = starts("demo:c"),
        d = starts("demo:d"),
        e = starts("demo:e"),
    ));
    let found = linked(&text);
    placed(&found);
    let at = |node: &str, name: &str| {
        let path = format!("particle_effect/events/{node}/particle_effect/effect");
        info("UnresolvedReference", &path, "particle", name)
    };
    assert_eq!(
        all_said(&found),
        [
            at("burst", "demo:none"),
            at("burst/sequence/1", "demo:b"),
            at("burst/sequence/1/randomize/0", "demo:c"),
            at("burst/randomize/1", "demo:d"),
            at("later", "demo:e"),
        ]
    );
}

#[test]
fn r39_an_effect_identifier_is_compared_as_written_and_a_case_miss_carries_the_hint() {
    use bedrock_addon::diagnostic::ReferenceHint;
    let text = with_events(&format!(
        r#""x": {{{}}}, "y": {{{}}}"#,
        starts("demo:A"),
        starts("a")
    ));
    let found = linked(&text);
    let hints: Vec<(String, Option<ReferenceHint>)> = found
        .iter()
        .map(|finding| match finding.detail.as_deref() {
            Some(DiagnosticDetail::Reference { id, hint, .. }) => (id.clone(), hint.clone()),
            other => panic!("no reference: {other:?}"),
        })
        .collect();
    assert_eq!(
        hints,
        [
            (
                "demo:A".to_owned(),
                Some(ReferenceHint::CaseMismatch("demo:a".to_owned()))
            ),
            // No namespace is put in front of a particle identifier.
            ("a".to_owned(), None),
        ]
    );
}

#[test]
fn r39_an_effect_of_the_game_resolves_with_a_host_that_has_it_and_not_without_one() {
    let text = with_events(&format!(
        r#""x": {{{}}}"#,
        starts("minecraft:basic_smoke_particle")
    ));
    let path = "particle_effect/events/x/particle_effect/effect";
    let missing = [info(
        "UnresolvedReference",
        path,
        "particle",
        "minecraft:basic_smoke_particle",
    )];
    // The crate has no list of the game's particle effects.
    assert_eq!(all_said(&linked(&text)), missing);
    let options = LinkOptions::default();
    let host = hosted(&[("particle", "minecraft:basic_smoke_particle")]);
    assert_eq!(all_said(&linked_with(&text, &[], host, options)), []);
    let other = hosted(&[("particle", "minecraft:basic_flame_particle")]);
    assert_eq!(all_said(&linked_with(&text, &[], other, options)), missing);
}

// ---- R39 behind an element the reader of an effect below 1.20.80 leaves out ----------------

/// An effect below 1.20.80 drawn with the pack's own material and texture.
fn legacy_effect(members: &str) -> String {
    effect_at("1.10.0", "mine", "textures/particle/mine", members)
}

/// The reader of an effect below 1.20.80 leaves out a list element it cannot use (an event
/// name that is no string, an entry that is no object, a looping entry with a negative
/// distance, a collision event without a name, a random option without a weight or with a
/// negative one). A name behind such an element is at the index the file has it at, at
/// every level.
#[test]
fn r39_a_name_behind_an_element_the_older_reader_leaves_out_is_at_the_index_of_the_file() {
    let text = legacy_effect(&format!(
        r#"{}, "events": {{"burst": {{"log": "b", "sequence": [7, {{{b}, "randomize": [{{"weight": -1}}, {{"weight": 1, {c}}}]}}], "randomize": [{{}}, 7, {{"weight": 2, {d}, "sequence": [7, {{{e}}}]}}]}}}}"#,
        components(
            r#""minecraft:emitter_lifetime_events": {"creation_event": [7, "a"], "looping_travel_distance_events": [7, {"distance": -1.0, "effects": "burst"}, {"distance": 2.0, "effects": [7, "f"]}]}, "minecraft:particle_motion_collision": {"collision_radius": 0.1, "events": [7, {"min_speed": 3.0}, {"event": "j"}]}"#
        ),
        b = starts("demo:b"),
        c = starts("demo:c"),
        d = starts("demo:d"),
        e = starts("demo:e"),
    ));
    let found = all_said(&linked(&text));
    let of = |kind: &str| -> Vec<Said> {
        found
            .iter()
            .filter(|said| said.3 == kind)
            .cloned()
            .collect()
    };
    let emitter = format!("{COMPONENTS}/minecraft:emitter_lifetime_events");
    let collision = format!("{COMPONENTS}/minecraft:particle_motion_collision");
    assert_eq!(
        of("particle event"),
        [
            event(&format!("{emitter}/creation_event/1"), "a"),
            event(
                &format!("{emitter}/looping_travel_distance_events/2/effects/1"),
                "f"
            ),
            event(&format!("{collision}/events/2/event"), "j"),
        ],
        "{found:#?}"
    );
    let at = |node: &str, name: &str| {
        let path = format!("particle_effect/events/{node}/particle_effect/effect");
        info("UnresolvedReference", &path, "particle", name)
    };
    assert_eq!(
        of("particle"),
        [
            at("burst/sequence/1", "demo:b"),
            at("burst/sequence/1/randomize/1", "demo:c"),
            at("burst/randomize/2", "demo:d"),
            at("burst/randomize/2/sequence/1", "demo:e"),
        ],
        "{found:#?}"
    );
}

// ---- R39: the sound an event plays -------------------------------------------------------

#[cfg(feature = "sounds_json")]
mod sound {
    use super::*;

    fn plays(name: &str) -> String {
        with_events(&format!(
            r#""x": {{"sequence": [{{"sound_effect": {{"event_name": "{name}"}}}}]}}"#
        ))
    }

    const AT: &str = "particle_effect/events/x/sequence/0/sound_effect/event_name";

    fn unknown(name: &str) -> Said {
        error("UnknownSound", AT, "level sound event", name)
    }

    #[test]
    fn r39_a_level_sound_event_of_the_game_is_no_finding_without_a_host() {
        for name in ["explode", "block.beehive.drip"] {
            assert_eq!(all_said(&linked(&plays(name))), [], "{name}");
        }
    }

    #[test]
    fn r39_a_sound_that_is_no_level_sound_event_is_an_unknown_sound_kept_error() {
        let found = linked(&plays("no_such_sound"));
        placed(&found);
        // The game logs the name at level 3
        //  and the effect keeps the event with
        // `Undefined`: a kept error.
        assert_eq!(all_said(&found), [unknown("no_such_sound")]);
        assert!(found[0].kept);
        assert_eq!(found[0].code.to_string(), "unknown sound event");
    }

    /// Behind a node the reader of an effect below 1.20.80 leaves out.
    #[test]
    fn r39_a_sound_behind_a_node_the_older_reader_leaves_out_is_at_the_index_of_the_file() {
        let text = legacy_effect(&format!(
            r#"{}, "events": {{"x": {{"sequence": [7, {{"sound_effect": {{"event_name": "no_such_sound"}}}}]}}}}"#,
            components("")
        ));
        let at = "particle_effect/events/x/sequence/1/sound_effect/event_name";
        assert_eq!(
            all_said(&linked(&text)),
            [error(
                "UnknownSound",
                at,
                "level sound event",
                "no_such_sound"
            )]
        );
    }

    /// The names of the game's events are lower-case, and no case is folded.
    #[test]
    fn r39_a_sound_name_is_compared_as_written() {
        assert_eq!(all_said(&linked(&plays("Explode"))), [unknown("Explode")]);
        assert_eq!(all_said(&linked(&plays("EXPLODE"))), [unknown("EXPLODE")]);
    }

    #[test]
    fn r39_a_host_answers_for_the_level_sound_events_yes_or_no() {
        let options = LinkOptions::default();
        let host = hosted(&[("level sound event", "demo.boom")]);
        let found = linked_with(&plays("demo.boom"), &[], host.clone(), options);
        assert_eq!(all_said(&found), []);
        // The host is believed when it says no, also for a name the crate knows.
        let found = linked_with(&plays("explode"), &[], host, options);
        assert_eq!(all_said(&found), [unknown("explode")]);
    }
}

/// Without the `sounds_json` feature the rule does not exist.
#[cfg(not(feature = "sounds_json"))]
#[test]
fn r39_without_the_sounds_json_feature_no_sound_is_looked_up() {
    let text = with_events(r#""x": {"sound_effect": {"event_name": "no_such_sound"}}"#);
    assert_eq!(all_said(&linked(&text)), []);
}

// ---- R39: the blocks a particle expires in -----------------------------------------------

#[cfg(feature = "block")]
mod block {
    use super::*;

    const LAMP: &str = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp", "states": {}}, "components": {}}}"#;

    fn expires(inside: &str, outside: &str) -> String {
        effect(&components(&format!(
            r#""minecraft:particle_expire_if_in_blocks": [{inside}], "minecraft:particle_expire_if_not_in_blocks": [{outside}]"#
        )))
    }

    fn block(component: &str, index: usize, name: &str) -> Said {
        let path = format!("{COMPONENTS}/minecraft:{component}/{index}");
        // The game's link pass holds each name.
        error("UnresolvedReference", &path, "block", name)
    }

    fn with_lamp(text: &str, load: LoadOptions) -> Vec<Diagnostic> {
        let behavior = [("blocks/lamp.json", LAMP)];
        linked_with(text, &behavior, load, LinkOptions::default())
    }

    #[test]
    fn r39_a_block_a_pack_defines_or_the_game_has_is_no_finding() {
        let text = expires(
            r#""demo:lamp", "minecraft:water", "stone""#,
            r#""minecraft:air", "demo:lamp""#,
        );
        assert_eq!(all_said(&with_lamp(&text, LoadOptions::default())), []);
    }

    /// Behind a name that is no string, which the reader of an effect below 1.20.80 leaves
    /// out.
    #[test]
    fn r39_a_block_behind_an_element_the_older_reader_leaves_out_is_at_the_index_of_the_file() {
        let text = legacy_effect(&components(
            r#""minecraft:particle_expire_if_in_blocks": [7, "demo:lamp", {}, "demo:no_such_block"], "minecraft:particle_expire_if_not_in_blocks": [7, "demo:other"]"#,
        ));
        assert_eq!(
            all_said(&with_lamp(&text, LoadOptions::default())),
            [
                block("particle_expire_if_in_blocks", 3, "demo:no_such_block"),
                block("particle_expire_if_not_in_blocks", 1, "demo:other"),
            ]
        );
    }

    #[test]
    fn r39_a_block_nothing_defines_is_an_unresolved_reference_in_either_list() {
        let text = expires(
            r#""demo:lamp", "demo:no_such_block""#,
            r#""demo:other", "minecraft:water", "minecraft:no_such_block""#,
        );
        let found = with_lamp(&text, LoadOptions::default());
        placed(&found);
        assert_eq!(
            all_said(&found),
            [
                block("particle_expire_if_in_blocks", 1, "demo:no_such_block"),
                block("particle_expire_if_not_in_blocks", 0, "demo:other"),
                block(
                    "particle_expire_if_not_in_blocks",
                    2,
                    "minecraft:no_such_block"
                ),
            ]
        );
    }

    /// A name without a namespace is a block of `minecraft`, so the pack's `lamp` is not it.
    #[test]
    fn r39_a_bare_block_name_is_one_of_minecraft() {
        let text = expires(r#""water", "lamp""#, "");
        assert_eq!(
            all_said(&with_lamp(&text, LoadOptions::default())),
            [block("particle_expire_if_in_blocks", 1, "lamp")]
        );
    }

    #[test]
    fn r39_a_host_answers_for_the_blocks_of_the_game_yes_or_no() {
        let text = expires(r#""demo:lamp", "minecraft:water", "minecraft:future""#, "");
        let host = hosted(&[("block", "minecraft:future")]);
        assert_eq!(
            all_said(&with_lamp(&text, host)),
            [block("particle_expire_if_in_blocks", 1, "minecraft:water")]
        );
    }
}

/// Without the `block` feature the rule does not exist.
#[cfg(not(feature = "block"))]
#[test]
fn r39_without_the_block_feature_no_block_is_looked_up() {
    let text = effect(&components(
        r#""minecraft:particle_expire_if_in_blocks": ["demo:no_such_block"]"#,
    ));
    assert_eq!(all_said(&linked(&text)), []);
}

// ---- the rows together -------------------------------------------------------------------

/// R38 and the effect half of R39 are informations, so they are lints, left out
/// with the lints as R39's event information is.
#[test]
fn r38_and_r39_are_informations_left_out_without_the_lints() {
    let text = effect_at(
        "1.26.10",
        "mine",
        "textures/particle/mine",
        &format!(
            r#"{}, "events": {{"x": {{{}}}}}"#,
            components(r#""minecraft:particle_lifetime_events": {"creation_event": "none"}"#),
            starts("demo:none")
        ),
    );
    let quiet = LinkOptions {
        lints: false,
        ..Default::default()
    };
    let found = linked_with(&text, &[], LoadOptions::default(), quiet);
    assert_eq!(all_said(&found), []);
    // with the lints both are told, and the R39 event information too
    let found = linked(&text);
    let said = all_said(&found);
    assert!(said.contains(&info(
        "UnresolvedReference",
        "particle_effect/events/x/particle_effect/effect",
        "particle",
        "demo:none"
    )));
    assert!(said.contains(&event(
        &format!("{COMPONENTS}/minecraft:particle_lifetime_events/creation_event"),
        "none"
    )));
}

/// The rules read the effect that is in effect for its identifier, not one a higher pack
/// replaced.
#[test]
fn r39_only_the_effect_in_effect_is_checked() {
    use crate::common::{resource_pack, stacks_of};
    let bad = with_events(&format!(r#""x": {{{}}}"#, starts("demo:none")));
    let good = with_events(&format!(r#""x": {{{}}}"#, starts("demo:a")));
    let mut top: Vec<(&'static str, &str)> = vec![("particles/top.json", good.as_str())];
    top.extend(MATERIALS);
    top.push(TEXTURE);
    let set = stacks_of(
        Vec::new(),
        vec![
            resource_pack(2, &top),
            resource_pack(3, &[("particles/bottom.json", bad.as_str())]),
        ],
        LoadOptions::default(),
    );
    let found = set.link();
    assert!(
        found
            .iter()
            .all(|finding| finding.code.name() != "UnresolvedReference"),
        "{found:?}"
    );
}
