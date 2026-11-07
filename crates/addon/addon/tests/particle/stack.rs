//! The registry of `rp.particles()`: the walk of `particles/`, the effect
//! loaded last per identifier, a failed file, and the references the link pass will check.

use bedrock_addon::diagnostic::{Code, Diagnostic, Diagnostics, Severity};
use bedrock_addon::families::particle::component::EmitterRateInstant;
use bedrock_addon::id::ParticleId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{
    Pack, PackOrigin, PackRepository, PackSelection, PackStack, Resource, WorldPacks,
};

use super::common::{BASE, CEREAL, DESCRIPTION, LEGACY, document, number, options, pack};

/// An effect named `identifier` that emits `count` particles: what tells the packs apart.
fn spark(version: &str, identifier: &str, count: u32) -> String {
    document(
        version,
        &format!(
            r#""components": {{"minecraft:emitter_lifetime_once": {{}}, "minecraft:emitter_rate_instant": {{"num_particles": {count}}}}}"#
        ),
    )
    .replace("demo:spark", identifier)
}

/// An effect named `identifier` that the 1.20.80 reader refuses.
fn broken(identifier: &str) -> String {
    spark(CEREAL, identifier, 0).replace(r#"{"num_particles": 0}"#, "{}")
}

/// The resource stack of `packs`, top first (named `top`, `middle`, `bottom`, `fourth` in
/// that order, however many there are); each pack is its `(path, text)` files.
fn stack(packs: Vec<Vec<(&str, String)>>) -> PackStack<Resource> {
    let names = ["top", "middle", "bottom", "fourth"];
    let packs: Vec<Pack> = packs
        .into_iter()
        .enumerate()
        .map(|(i, files)| {
            let files = files
                .into_iter()
                .map(|(path, text)| (path.to_owned(), text))
                .collect();
            pack(10 + i as u128, names[i], files)
        })
        .collect();
    PackStack::<Resource>::of(packs, options()).expect("a stack")
}

/// How many particles the effective `identifier` emits.
fn count_of(stack: &PackStack<Resource>, identifier: &str) -> Option<f32> {
    let particles = stack.particles();
    let effect = particles.get(&ParticleId::parse(identifier))?;
    let rate = effect.components.get::<EmitterRateInstant>()?;
    rate.num_particles.number()
}

/// The pack and file of a finding.
fn place(diagnostic: &Diagnostic) -> (String, String) {
    (
        diagnostic.location.pack.as_ref().unwrap().name().to_owned(),
        diagnostic
            .location
            .path
            .as_ref()
            .unwrap()
            .as_str()
            .to_owned(),
    )
}

/// The findings with a code `code` accepts, as `(pack, file)`.
fn found(stack: &PackStack<Resource>, code: impl Fn(&Code) -> bool) -> Vec<(String, String)> {
    let particles = stack.particles();
    particles
        .diagnostics()
        .filter(|d| code(&d.code))
        .map(place)
        .collect()
}

fn shadowed(code: &Code) -> bool {
    // By legacy name: the variant and the definitions that were it.
    matches!(code.id(), "core/shadowed_definition")
}

#[test]
fn the_higher_pack_s_effect_is_the_effective_one() {
    let stack = stack(vec![
        vec![("particles/a.json", spark(CEREAL, "demo:a", 1))],
        vec![
            ("particles/other_name.json", spark(LEGACY, "demo:a", 2)),
            ("particles/b.particle.json", spark(LEGACY, "demo:b", 3)),
        ],
    ]);
    let particles = stack.particles();
    assert_eq!(particles.len(), 2);
    assert_eq!(count_of(&stack, "demo:a"), Some(1.0));
    assert_eq!(count_of(&stack, "demo:b"), Some(3.0));
    // The replaced effect is noted at info, where it is.
    let effective = particles.get(&ParticleId::parse("demo:a")).unwrap();
    assert_eq!(effective.origin().pack.name(), "top");
    assert_eq!(
        found(&stack, shadowed),
        [("middle".to_owned(), "particles/other_name.json".to_owned())]
    );
    assert!(
        particles
            .diagnostics()
            .all(|d| d.severity <= Severity::Info)
    );
}

#[test]
fn three_packs_define_one_effect_and_the_highest_wins() {
    let stack = stack(vec![
        vec![("particles/a.json", spark(CEREAL, "demo:a", 1))],
        vec![("particles/a.json", spark(CEREAL, "demo:a", 2))],
        vec![("particles/a.json", spark(LEGACY, "demo:a", 3))],
    ]);
    assert_eq!(count_of(&stack, "demo:a"), Some(1.0));
    assert_eq!(stack.particles().len(), 1);
    assert_eq!(
        found(&stack, shadowed),
        [
            ("bottom".to_owned(), "particles/a.json".to_owned()),
            ("middle".to_owned(), "particles/a.json".to_owned()),
        ]
    );
}

/// A file that fails to load is skipped ("Failed to load particle effect"): it binds
/// nothing, so the effect of a lower pack stays the effective one; in two packs and in
/// three, whichever of them fails.
#[test]
fn a_file_that_fails_leaves_the_effect_below_it_in_place() {
    let two = stack(vec![
        vec![("particles/a.json", broken("demo:a"))],
        vec![("particles/a.json", spark(CEREAL, "demo:a", 2))],
    ]);
    assert_eq!(count_of(&two, "demo:a"), Some(2.0));
    let rejected: Vec<_> = two
        .particles()
        .rejected()
        .iter()
        .map(|rejected| rejected.origin.pack.name().to_owned())
        .collect();
    assert_eq!(rejected, ["top"]);
    assert_eq!(found(&two, shadowed), []);

    let top_fails = stack(vec![
        vec![("particles/a.json", broken("demo:a"))],
        vec![("particles/a.json", spark(CEREAL, "demo:a", 2))],
        vec![("particles/a.json", spark(CEREAL, "demo:a", 3))],
    ]);
    assert_eq!(count_of(&top_fails, "demo:a"), Some(2.0));
    assert_eq!(
        found(&top_fails, shadowed),
        [("bottom".to_owned(), "particles/a.json".to_owned())]
    );

    let middle_fails = stack(vec![
        vec![("particles/a.json", spark(CEREAL, "demo:a", 1))],
        vec![("particles/a.json", broken("demo:a"))],
        vec![("particles/a.json", spark(CEREAL, "demo:a", 3))],
    ]);
    assert_eq!(count_of(&middle_fails, "demo:a"), Some(1.0));
    assert_eq!(middle_fails.particles().rejected().len(), 1);

    let all_fail = stack(vec![
        vec![("particles/a.json", broken("demo:a"))],
        vec![("particles/a.json", broken("demo:a"))],
    ]);
    assert_eq!(count_of(&all_fail, "demo:a"), None);
    assert_eq!(all_fail.particles().len(), 0);
    assert_eq!(all_fail.particles().rejected().len(), 2);
    // The rejection is at the file, with its reason.
    let errors: Vec<_> = two
        .particles()
        .diagnostics()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| (place(d), d.location.json_path.to_string(), d.code.id()))
        .collect();
    assert_eq!(
        errors,
        [(
            ("top".to_owned(), "particles/a.json".to_owned()),
            "particle_effect/components/minecraft:emitter_rate_instant/num_particles".to_owned(),
            "core/missing_required_field"
        )]
    );
}

/// The effect of `identifier`: the pack it is from and how many components it has.
fn effective(stack: &PackStack<Resource>, identifier: &str) -> Option<(String, usize)> {
    let particles = stack.particles();
    let effect = particles.get(&ParticleId::parse(identifier))?;
    Some((
        effect.origin().pack.name().to_owned(),
        effect.components.len(),
    ))
}

/// The packs whose file was rejected, in walk order (bottom first).
fn rejected_in(stack: &PackStack<Resource>) -> Vec<String> {
    let particles = stack.particles();
    let rejected = particles.rejected().iter();
    rejected
        .map(|rejected| rejected.origin.pack.name().to_owned())
        .collect()
}

/// `particles/a.json` of the pack `name`.
fn a_of(name: &str) -> (String, String) {
    (name.to_owned(), "particles/a.json".to_owned())
}

/// A 1.10.0 document naming `demo:a` whose payload is `description` and `rest`.
fn legacy_a(description: &str, rest: &str) -> String {
    format!(
        r#"{{"format_version": "1.10.0", "particle_effect": {{"description": {description}{rest}}}}}"#
    )
}

/// 1.10.0 files of `demo:a` the legacy parser stops in after it has read the identifier:
/// the game ignores the parser's result, upgrades what was parsed
/// and registers it.
fn stopped_legacy_files() -> Vec<(&'static str, String)> {
    let components = format!(r#", "components": {{{BASE}}}"#);
    let with = |parameters: &str| {
        legacy_a(
            &format!(r#"{{"identifier": "demo:a", "basic_render_parameters": {parameters}}}"#),
            &components,
        )
    };
    let described =
        r#"{"identifier": "demo:a", "basic_render_parameters": {"material": "m", "texture": "t"}}"#;
    vec![
        (
            "no basic_render_parameters",
            legacy_a(r#"{"identifier": "demo:a"}"#, &components),
        ),
        ("basic_render_parameters that is no object", with("1")),
        ("no material", with(r#"{"texture": "t"}"#)),
        (
            "a material that is no string",
            with(r#"{"material": 1, "texture": "t"}"#),
        ),
        ("no texture", with(r#"{"material": "m"}"#)),
        ("no components", legacy_a(described, "")),
        (
            "components that is no object",
            legacy_a(described, r#", "components": []"#),
        ),
    ]
}

/// A 1.10.0 file the legacy parser stops in claims its identifier: what was parsed is
/// registered, without a component, and the effect of every lower pack is replaced, in two
/// packs and in three, wherever the file is.
#[test]
fn a_legacy_file_the_parser_stops_in_replaces_the_effect_below_it() {
    for (what, text) in stopped_legacy_files() {
        let two = stack(vec![
            vec![("particles/a.json", text.clone())],
            vec![("particles/a.json", spark(CEREAL, "demo:a", 2))],
        ]);
        assert_eq!(
            effective(&two, "demo:a"),
            Some(("top".to_owned(), 0)),
            "{what}"
        );
        assert_eq!(found(&two, shadowed), [a_of("middle")], "{what}");
        assert_eq!(rejected_in(&two), [] as [&str; 0], "{what}");

        let on_top = stack(vec![
            vec![("particles/a.json", text.clone())],
            vec![("particles/a.json", spark(CEREAL, "demo:a", 2))],
            vec![("particles/a.json", spark(LEGACY, "demo:a", 3))],
        ]);
        assert_eq!(
            effective(&on_top, "demo:a"),
            Some(("top".to_owned(), 0)),
            "{what}"
        );
        assert_eq!(
            found(&on_top, shadowed),
            [a_of("bottom"), a_of("middle")],
            "{what}"
        );
        assert_eq!(rejected_in(&on_top), [] as [&str; 0], "{what}");

        // In the middle it replaces the bottom pack's and is replaced by the top pack's.
        let between = stack(vec![
            vec![("particles/a.json", spark(CEREAL, "demo:a", 1))],
            vec![("particles/a.json", text.clone())],
            vec![("particles/a.json", spark(CEREAL, "demo:a", 3))],
        ]);
        assert_eq!(count_of(&between, "demo:a"), Some(1.0), "{what}");
        assert_eq!(
            found(&between, shadowed),
            [a_of("bottom"), a_of("middle")],
            "{what}"
        );

        // Alone it is the effect.
        let alone = stack(vec![vec![("particles/a.json", text)]]);
        assert_eq!(
            effective(&alone, "demo:a"),
            Some(("top".to_owned(), 0)),
            "{what}"
        );
    }
}

/// Files the loader skips ("Failed to load particle effect"), whatever they name: a Puv
/// failure of the legacy slice, a 1.10.0 file whose identifier cannot be read (ours; the
/// game registers an effect named `""`), and every failure from 1.20.80.
fn skipped_files() -> Vec<(&'static str, String)> {
    let components = format!(r#", "components": {{{BASE}}}"#);
    let cereal = |members: &str| document(CEREAL, members).replace("demo:spark", "demo:a");
    vec![
        // Below 1.20.80.
        ("1.10.0: no JSON", r#"{"format_version": "1.10.0", "particle_effect": {"#.to_owned()),
        ("1.10.0: a root that is no object", "[]".to_owned()),
        (
            "a format_version that is no version",
            spark(LEGACY, "demo:a", 0).replace("1.10.0", "one"),
        ),
        ("1.10.0: no particle_effect", r#"{"format_version": "1.10.0"}"#.to_owned()),
        ("below the floor", spark(LEGACY, "demo:a", 0).replace("1.10.0", "1.9.0")),
        (
            "1.10.0: no description",
            format!(r#"{{"format_version": "1.10.0", "particle_effect": {{"components": {{{BASE}}}}}}}"#),
        ),
        (
            "1.10.0: an identifier that is no string",
            legacy_a(
                r#"{"identifier": 1, "basic_render_parameters": {"material": "m", "texture": "t"}}"#,
                &components,
            ),
        ),
        // From 1.20.80.
        ("1.26.10: no JSON", r#"{"format_version": "1.26.10", "particle_effect": {"#.to_owned()),
        (
            "1.26.10: a third root member",
            spark(CEREAL, "demo:a", 0).replacen('{', r#"{"more": 1, "#, 1),
        ),
        ("1.26.10: no payload", r#"{"format_version": "1.26.10"}"#.to_owned()),
        (
            "1.26.10: a payload under another key",
            spark(CEREAL, "demo:a", 0).replace("particle_effect", "particle"),
        ),
        ("1.26.10: a required member missing", broken("demo:a")),
        (
            "1.26.10: no basic_render_parameters",
            spark(CEREAL, "demo:a", 0).replace(
                r#", "basic_render_parameters": {"material": "particles_alpha", "texture": "textures/particle/particles"}"#,
                "",
            ),
        ),
        ("1.26.10: no components", cereal(r#""curves": {}"#)),
        (
            "1.26.10: a value of the wrong type",
            spark(CEREAL, "demo:a", 0).replace(r#"{"num_particles": 0}"#, r#"{"num_particles": []}"#),
        ),
        (
            "1.26.10: a value outside its set",
            cereal(&format!(
                r#""components": {{{BASE}}}, "events": {{"e": {{"particle_effect": {{"effect": "demo:b", "type": "none"}}}}}}"#
            )),
        ),
        ("above the engine", spark(CEREAL, "demo:a", 0).replace("1.26.10", "99.0.0")),
    ]
}

/// A skipped file binds nothing: the effect of the pack below it stays the effective one, in
/// two packs and in three, whichever of them is skipped.
#[test]
fn a_skipped_file_leaves_the_effect_below_it_in_place() {
    for (what, text) in skipped_files() {
        let two = stack(vec![
            vec![("particles/a.json", text.clone())],
            vec![("particles/a.json", spark(CEREAL, "demo:a", 2))],
        ]);
        assert_eq!(count_of(&two, "demo:a"), Some(2.0), "{what}");
        assert_eq!(
            effective(&two, "demo:a"),
            Some(("middle".to_owned(), 2)),
            "{what}"
        );
        assert_eq!(found(&two, shadowed), [], "{what}");
        assert_eq!(rejected_in(&two), ["top"], "{what}");
        assert_eq!(two.particles().len(), 1, "{what}");

        let on_top = stack(vec![
            vec![("particles/a.json", text.clone())],
            vec![("particles/a.json", spark(LEGACY, "demo:a", 2))],
            vec![("particles/a.json", spark(CEREAL, "demo:a", 3))],
        ]);
        assert_eq!(count_of(&on_top, "demo:a"), Some(2.0), "{what}");
        assert_eq!(found(&on_top, shadowed), [a_of("bottom")], "{what}");
        assert_eq!(rejected_in(&on_top), ["top"], "{what}");
        assert_eq!(on_top.particles().len(), 1, "{what}");

        let between = stack(vec![
            vec![("particles/a.json", spark(CEREAL, "demo:a", 1))],
            vec![("particles/a.json", text.clone())],
            vec![("particles/a.json", spark(CEREAL, "demo:a", 3))],
        ]);
        assert_eq!(count_of(&between, "demo:a"), Some(1.0), "{what}");
        assert_eq!(found(&between, shadowed), [a_of("bottom")], "{what}");
        assert_eq!(rejected_in(&between), ["middle"], "{what}");

        let alone = stack(vec![vec![("particles/a.json", text)]]);
        assert_eq!(alone.particles().len(), 0, "{what}");
        assert_eq!(rejected_in(&alone), ["top"], "{what}");
    }
}

/// Two files of one pack that name one effect: the one loaded last is bound (files are
/// walked in path order), the other is noted.
#[test]
fn two_files_of_one_pack_naming_one_effect() {
    let stack = stack(vec![vec![
        ("particles/a.json", spark(CEREAL, "demo:a", 1)),
        ("particles/b.json", spark(CEREAL, "demo:a", 2)),
        ("particles/c.json", spark(CEREAL, "demo:c", 3)),
    ]]);
    assert_eq!(stack.particles().len(), 2);
    assert_eq!(count_of(&stack, "demo:a"), Some(2.0));
    assert_eq!(
        found(&stack, shadowed),
        [("top".to_owned(), "particles/a.json".to_owned())]
    );
}

/// The walk: `.json` files below `particles/`, sub-folders included; the extension is
/// compared as written, and no other directory is read.
#[test]
fn the_walk_reads_json_files_below_particles() {
    let stack = stack(vec![vec![
        ("particles/a.json", spark(CEREAL, "demo:a", 1)),
        (
            "particles/deep/er/b.particle.json",
            spark(CEREAL, "demo:b", 2),
        ),
        ("particles/c.JSON", spark(CEREAL, "demo:c", 3)),
        ("particles/d.txt", spark(CEREAL, "demo:d", 4)),
        ("particle/e.json", spark(CEREAL, "demo:e", 5)),
        ("textures/particles/f.json", spark(CEREAL, "demo:f", 6)),
    ]]);
    let particles = stack.particles();
    let mut names: Vec<_> = particles
        .iter()
        .map(|(id, _)| id.as_str().to_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["demo:a", "demo:b"]);
    assert_eq!(particles.diagnostics().count(), 0);
}

/// An effect's identifier is what the file says, whatever the file is called, and is kept as
/// written (a particle identifier has no namespace rule).
#[test]
fn an_effect_is_registered_under_the_identifier_it_writes() {
    let stack = stack(vec![vec![
        ("particles/x.json", spark(CEREAL, "No Namespace", 1)),
        ("particles/y.json", spark(CEREAL, "", 2)),
    ]]);
    assert_eq!(count_of(&stack, "No Namespace"), Some(1.0));
    assert_eq!(count_of(&stack, ""), Some(2.0));
    assert_eq!(count_of(&stack, "no namespace"), None);
}

/// A document whose payload is written twice is read as its last copy, also for the
/// identifier it is registered under.
#[test]
fn a_doubled_payload_is_registered_under_its_last_copy() {
    let text = format!(
        r#"{{"format_version": "1.26.10", "particle_effect": {{"description": {{"identifier": "demo:first", "basic_render_parameters": {{"material": "m", "texture": "t"}}}}, "components": {{}}}}, "particle_effect": {{{DESCRIPTION}, "components": {{"minecraft:emitter_rate_instant": {{"num_particles": 9}}}}}}}}"#
    );
    let stack = stack(vec![vec![("particles/a.json", text)]]);
    assert_eq!(count_of(&stack, "demo:spark"), Some(9.0));
    assert_eq!(count_of(&stack, "demo:first"), None);
    assert_eq!(number(9.0).number(), Some(9.0));
}

// ---- the link pass ----------------------------------------------------------

/// The link findings of one resource pack with `files`.
fn link(files: Vec<(&str, String)>) -> Diagnostics {
    let files = files
        .into_iter()
        .map(|(path, text)| (path.to_owned(), text))
        .collect();
    let pack = pack(20, "linked", files);
    let world = WorldPacks {
        behavior: Vec::new(),
        resource: vec![PackSelection {
            pack_id: pack.id().uuid,
            version: pack.id().version.clone(),
            subpack: None,
        }],
    };
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package); // the pack with the sources is one the game ships
    repository.stacks(&world, LoadOptions::default()).link()
}

/// The `(path, code)` of the link findings about `particles/` files.
fn unresolved(d: &Diagnostics) -> Vec<(String, &'static str)> {
    d.iter()
        .filter(|f| {
            let path = f.location.path.as_ref();
            path.is_some_and(|path| path.as_str().starts_with("particles/"))
        })
        .map(|f| (f.location.json_path.to_string(), f.code.id()))
        .collect()
}

/// `material` names a material of the stack, `texture` a texture file. The stack
/// has a material source, so the material is judged (without one, and without a
/// host, nothing is said of any material).
#[test]
fn r38_a_material_or_texture_no_pack_has_is_unresolved() {
    let found = link(vec![
        (
            "particles/a.json",
            spark(CEREAL, "demo:a", 1)
                .replace("particles_alpha", "no_such_material")
                .replace(
                    "textures/particle/particles",
                    "textures/particle/no_such_texture",
                ),
        ),
        // (K127)
        (
            "materials/common.json",
            r#"[{"path": "materials/particles.material"}]"#.to_owned(),
        ),
        (
            "materials/particles.material",
            r#"{"materials": {"version": "1.0.0", "particles_alpha": {}}}"#.to_owned(),
        ),
    ]);
    let brp = "particle_effect/description/basic_render_parameters";
    assert_eq!(
        unresolved(&found),
        [
            (format!("{brp}/material"), "core/unresolved_reference"),
            (format!("{brp}/texture"), "core/texture_not_found_worded"),
        ]
    );
    // The texture is the validator's file reference, a content-log error;
    // the material is looked up at draw time without a line: ours, an information.
    assert!(found.iter().all(|f| f.severity
        == if f.code.id() == "core/texture_not_found_worded" {
            Severity::Error
        } else {
            Severity::Info
        }));
}

/// An event's `particle_effect.effect` names a particle of the stack, the event
/// names of a component name events of the effect, a block list names blocks.
#[test]
fn r39_a_reference_to_a_particle_event_sound_or_block_that_does_not_exist_is_unresolved() {
    let text = document(
        CEREAL,
        r#""components": {"minecraft:emitter_rate_instant": {"num_particles": 1}, "minecraft:emitter_lifetime_events": {"creation_event": "missing_event", "expiration_event": ["burst"]}, "minecraft:particle_expire_if_in_blocks": ["demo:no_such_block"]}, "events": {"burst": {"particle_effect": {"effect": "demo:no_such_particle", "type": "emitter"}, "sound_effect": {"event_name": "no_such_sound"}}, "again": {"particle_effect": {"effect": "demo:spark", "type": "emitter"}}}"#,
    );
    // The effect's own material and texture are the pack's, so that the material and texture check has nothing to say
    // of this effect.
    let found = link(vec![
        ("particles/a.json", text),
        (
            "materials/common.json",
            r#"[{"path": "materials/particles.material"}]"#.to_owned(),
        ),
        (
            "materials/particles.material",
            r#"{"materials": {"version": "1.0.0", "particles_alpha": {}}}"#.to_owned(),
        ),
        (
            "textures/particle/particles.png",
            "no image is read".to_owned(),
        ),
    ]);
    let components = "particle_effect/components";
    assert_eq!(
        unresolved(&found),
        [
            (
                format!("{components}/minecraft:emitter_lifetime_events/creation_event"),
                "core/unresolved_reference"
            ),
            (
                format!("{components}/minecraft:particle_expire_if_in_blocks/0"),
                "core/unresolved_reference_worded"
            ),
            (
                "particle_effect/events/burst/particle_effect/effect".to_owned(),
                "core/unresolved_reference"
            ),
            (
                "particle_effect/events/burst/sound_effect/event_name".to_owned(),
                "core/unknown_sound"
            ),
        ]
    );
    // The block list is held by the game's validator: a content-log error;
    // an event the effect lacks is an information; a sound that is no
    // level sound event is the game's level-3 line, the effect kept: a kept
    // error; an effect nothing defines is resolved in silence: an information.
    assert!(found.iter().all(|f| {
        let at = f.location.json_path.to_string();
        f.severity
            == if at.contains("particle_expire_if_in_blocks") {
                Severity::Error
            } else if at.ends_with("creation_event") || at.ends_with("particle_effect/effect") {
                Severity::Info
            } else {
                assert!(f.kept, "{f:?}");
                Severity::Error
            }
    }));
}
