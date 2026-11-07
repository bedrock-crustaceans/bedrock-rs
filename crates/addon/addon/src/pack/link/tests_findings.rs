//! The constructors of the findings: the code, the severity, the phase, the place and the
//! detail of each.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, JsonPath, Phase, ReferenceHint, Severity, sentence,
};

use super::super::{
    At, cycle, invalid, kinds, texture_not_found, unknown_sound, unresolved, unresolved_lang_key,
};
use bedrock_addon_core::pack_identity::PackPath;

use super::support::origin;

fn path(keys: &[&str]) -> JsonPath {
    let mut path = JsonPath::new();
    for key in keys {
        path.push(key);
    }
    path
}

/// Everything of a finding but its code: the severity, the place and the detail.
fn check(
    finding: &Diagnostic,
    severity: Severity,
    kind: &'static str,
    id: &str,
    hint: Option<ReferenceHint>,
) {
    assert_eq!(finding.severity, severity);
    assert_eq!(finding.phase, Phase::Link);
    let location = &finding.location;
    assert_eq!(
        location
            .pack
            .as_ref()
            .map(bedrock_addon_core::pack_identity::PackRef::name),
        Some("pack 7")
    );
    assert_eq!(
        location.path.as_ref().map(PackPath::as_str),
        Some("entities/pet.json")
    );
    // The path is the one the rule gave, a key with `:` and `/` in it kept whole.
    assert_eq!(
        location.json_path,
        path(&["minecraft:entity", "components", "minecraft:loot", "a/b"])
    );
    assert_eq!(location.span, None);
    assert_eq!(
        finding.detail.as_deref(),
        Some(&DiagnosticDetail::Reference {
            kind,
            id: id.to_owned(),
            hint,
        })
    );
}

fn at(origin: &bedrock_addon_core::document::Origin, severity: Severity) -> At<'_> {
    At::new(
        origin,
        path(&["minecraft:entity", "components", "minecraft:loot", "a/b"]),
        severity,
    )
}

#[test]
fn an_unresolved_reference_names_the_kind_and_the_name_as_written_with_its_hint() {
    let origin = origin("entities/pet.json");
    let hint = Some(ReferenceHint::CaseMismatch("demo:lamp".to_owned()));
    let finding = unresolved(
        at(&origin, Severity::Warning),
        kinds::BLOCK,
        "Demo:Lamp",
        hint.clone(),
    );
    assert_eq!(finding.code, Code::from(&core_codes::UNRESOLVED_REFERENCE));
    check(&finding, Severity::Warning, "block", "Demo:Lamp", hint);
    assert_eq!(
        finding.to_string(),
        "[warning] pack 7 | entities/pet.json | minecraft:entity/components/minecraft:loot/a~1b \
         | unresolved reference: block 'Demo:Lamp' (did you mean 'demo:lamp'?)"
    );
    // The severity is the row's: the same constructor gives an information or an error.
    let info = unresolved(at(&origin, Severity::Info), kinds::LOOT_TABLE, "x", None);
    check(&info, Severity::Info, "loot table", "x", None);
    let error = unresolved(at(&origin, Severity::Error), kinds::BIOME, "y", None);
    check(&error, Severity::Error, "biome", "y", None);
}

#[test]
fn a_finding_that_vanilla_words_carries_vanillas_text_and_ours_none() {
    let origin = origin("entities/pet.json");
    let text = sentence("{} is not a valid item name.", &["demo:none".to_owned()]);
    let worded = unresolved(
        at(&origin, Severity::Warning).with_text(text.clone()),
        kinds::ITEM,
        "demo:none",
        None,
    );
    assert_eq!(
        worded.code,
        core_codes::UNRESOLVED_REFERENCE_WORDED.with([text.clone()])
    );
    check(&worded, Severity::Warning, "item", "demo:none", None);
    for (finding, own) in [
        (
            invalid(
                at(&origin, Severity::Error).with_text(text.clone()),
                kinds::BLOCK,
                "b",
            ),
            Code::from(&core_codes::INVALID_REFERENCE),
        ),
        (
            cycle(
                at(&origin, Severity::Warning).with_text(text.clone()),
                kinds::BLOCK,
                "b",
            ),
            Code::from(&core_codes::REFERENCE_CYCLE),
        ),
        (
            texture_not_found(at(&origin, Severity::Warning).with_text(text.clone()), "b"),
            Code::from(&core_codes::TEXTURE_NOT_FOUND),
        ),
        (
            unknown_sound(
                at(&origin, Severity::Warning).with_text(text.clone()),
                kinds::BLOCK,
                "b",
            ),
            Code::from(&core_codes::UNKNOWN_SOUND),
        ),
    ] {
        assert_ne!(finding.code, own, "the text is carried");
        assert_eq!(finding.code.name(), own.name());
    }
}

#[test]
fn an_invalid_reference_and_a_cycle_have_their_own_codes() {
    let origin = origin("entities/pet.json");
    let finding = invalid(at(&origin, Severity::Error), kinds::BLOCK, "demo:lamp");
    assert_eq!(finding.code, Code::from(&core_codes::INVALID_REFERENCE));
    check(&finding, Severity::Error, "block", "demo:lamp", None);
    let finding = cycle(
        at(&origin, Severity::Warning),
        kinds::BEHAVIOR_TREE,
        "demo:tree",
    );
    assert_eq!(finding.code, Code::from(&core_codes::REFERENCE_CYCLE));
    check(
        &finding,
        Severity::Warning,
        "behavior tree",
        "demo:tree",
        None,
    );
}

#[test]
fn a_missing_texture_a_sound_and_a_lang_key_have_their_own_codes() {
    let origin = origin("entities/pet.json");
    let finding = texture_not_found(at(&origin, Severity::Warning), "textures/Entity/Pet");
    assert_eq!(finding.code, Code::from(&core_codes::TEXTURE_NOT_FOUND));
    check(
        &finding,
        Severity::Warning,
        "texture",
        "textures/Entity/Pet",
        None,
    );
    let finding = unknown_sound(
        at(&origin, Severity::Info),
        kinds::SOUND_EVENT,
        "mob.pet.say",
    );
    assert_eq!(finding.code, Code::from(&core_codes::UNKNOWN_SOUND));
    check(&finding, Severity::Info, "sound event", "mob.pet.say", None);
    let finding = unknown_sound(
        at(&origin, Severity::Warning),
        kinds::LEVEL_SOUND_EVENT,
        "Boom",
    );
    check(
        &finding,
        Severity::Warning,
        "level sound event",
        "Boom",
        None,
    );
    let finding = unresolved_lang_key(at(&origin, Severity::Info), "Item.Pet.Name");
    assert_eq!(finding.code, Code::from(&core_codes::UNRESOLVED_LANG_KEY));
    check(&finding, Severity::Info, "lang key", "Item.Pet.Name", None);
}

#[test]
fn the_names_of_the_kinds_are_the_identifier_kinds_names() {
    assert_eq!(kinds::BLOCK, "block");
    assert_eq!(kinds::ATLAS_KEY, "atlas key");
    assert_eq!(kinds::LEVEL_SOUND_EVENT, "level sound event");
    assert_eq!(kinds::ITEM_TAG, "item tag");
    assert_eq!(kinds::BLOCK_STATE, "block state");
}

#[test]
fn a_finding_of_any_code_is_built_with_the_frames_place_severity_and_phase() {
    let origin = origin("entities/pet.json");
    let hint = Some(ReferenceHint::MissingNamespace);
    let code = Code::from(&core_codes::INVALID_VALUE);
    let with =
        at(&origin, Severity::Error).reference(code.clone(), kinds::BLOCK, "lamp", hint.clone());
    assert_eq!(with.code, code);
    check(&with, Severity::Error, "block", "lamp", hint);
    // Without a detail: the same place, severity and phase, and nothing named.
    let plain = at(&origin, Severity::Info).diagnostic(Code::from(&core_codes::FILE_NOT_LOADED));
    assert_eq!(plain.code, Code::from(&core_codes::FILE_NOT_LOADED));
    assert_eq!(plain.severity, Severity::Info);
    assert_eq!(plain.phase, Phase::Link);
    assert_eq!(plain.location, with.location);
    assert_eq!(plain.detail, None);
}

#[test]
fn link_types_implement_debug() {
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<super::super::Findings>();
    assert_debug::<At<'static>>();
    assert_debug::<super::super::LinkCx<'static>>();
}
