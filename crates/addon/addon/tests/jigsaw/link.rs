//! The fixture rows whose expected finding is made by the link pass or at generation. The link rows run through `StackSet::link`; their texts are also pinned on the
//! crate-private checker in `src/families/jigsaw/tests.rs`.
//! The generation texts belong to the consumer that places structures: their rows pin
//! that loading raises none of them.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, WorldPacks};

use super::common::{fixture_packs, load_fixture};

/// The link findings of a fixture's stack.
fn link_fixture(name: &str) -> Diagnostics {
    let packs = fixture_packs(name);
    let entries: Vec<String> = packs
        .iter()
        .map(|pack| {
            format!(
                r#"{{"pack_id": "{}", "version": [1, 0, 0]}}"#,
                pack.id().uuid
            )
        })
        .collect();
    let world = WorldPacks::parse(Some(format!("[{}]", entries.join(",")).as_bytes()), None)
        .expect("a world list");
    let mut repository = PackRepository::new();
    for pack in packs {
        repository.add_pack(pack, PackOrigin::User);
    }
    repository.stacks(&world, LoadOptions::default()).link()
}

/// Whether an error `UnresolvedReference` with exactly `text` was found.
fn has_error(d: &Diagnostics, text: &str) -> bool {
    d.iter().any(|f| {
        f.code.name() == "UnresolvedReference"
            && f.severity == Severity::Error
            && f.code.to_string() == text
    })
}

/// Whether an information `UnresolvedReference` with exactly `text` was found (ours).
fn has(d: &Diagnostics, text: &str) -> bool {
    d.iter().any(|f| {
        f.code.name() == "UnresolvedReference"
            && f.severity == Severity::Info
            && f.code.to_string() == text
    })
}

#[cfg(feature = "block")]
#[test]
fn jig13_unknown_block_is_reported_once_with_vanillas_text() {
    let d = link_fixture("jig13_unknown_block");
    assert!(has_error(&d, "Cannot find block 'test:invalid'"), "{d:?}");
    // One finding of rule 12 per member; beside it the game's validator line, at the
    // block's `name`.
    let texts: Vec<String> = d
        .iter()
        .filter(|f| f.code.name() == "UnresolvedReference")
        .map(|f| f.code.to_string())
        .collect();
    assert_eq!(texts.len(), 2, "{texts:#?}");
    assert!(
        texts
            .iter()
            .any(|t| t.ends_with(":  Missing referenced asset test:invalid")),
        "{texts:#?}"
    );
}

#[test]
fn jig17_set_entry_naming_no_structure_is_reported() {
    let d = link_fixture("jig17_set_entry_unknown_structure");
    // The validator's line, a content-log error.
    assert!(
        has_error(
            &d,
            "-> structures[0] -> structure:  Missing referenced asset demo:missing"
        ),
        "{d:?}"
    );
}

#[cfg(feature = "structure")]
#[test]
fn jig21_pool_element_naming_no_template_is_reported() {
    let d = link_fixture("jig21_generation_texts");
    assert!(
        has(
            &d,
            "[JIGSAW GENERATION FAILURE] Structure name [demo:missing] referenced in template pool [demo:pool] does not exist."
        ),
        "{d:?}"
    );
}

/// Every finding of a fixture's load, at any severity, as `(code, text)`.
fn load_findings(name: &str) -> Vec<(&'static str, String)> {
    let stack = load_fixture(name);
    stack
        .jigsaw()
        .diagnostics()
        .map(|d| (d.code.name(), d.code.to_string()))
        .collect()
}

/// Loading raises no `UnresolvedReference` and none of `texts`, at any severity.
#[track_caller]
fn no_load_finding(name: &str, texts: &[&str]) {
    let found = load_findings(name);
    assert!(
        found
            .iter()
            .all(|(code, text)| *code != "UnresolvedReference" && !texts.contains(&text.as_str())),
        "{name}: {found:?}"
    );
}

#[test]
fn jig21_template_jigsaw_block_final_state_is_not_a_load_finding() {
    // Rules 26 and 30: a template's jigsaw block is read when the structure is placed, by the
    // consumer; the pool that names the template loads without it.
    no_load_finding(
        "jig21_generation_texts",
        &[
            "Failed to map jigsaw structure final block: 'demo:nope'. There is no block registered with that name.",
            "[JIGSAW GENERATION FAILURE] Jigsaw block in structure [demo:missing] has an invalid \"Turns Into\" block: [demo:nope].",
        ],
    );
}

#[test]
fn jig22_unknown_feature_is_reported_with_the_development_text() {
    let d = link_fixture("jig22_feature_pool_element_unknown_feature");
    assert!(
        has(
            &d,
            "Cannot find 'feature' 'demo:missing' for 'feature_pool_element'. Make sure to include the 'namespace_name:' prefix."
        ),
        "{d:?}"
    );
}

#[test]
fn jig23_start_pool_texts_are_not_load_findings() {
    // Rules 31-36: the start pool is drawn when the structure is placed, by the consumer; every document of the row loads without them.
    no_load_finding(
        "jig23_generation_start_pool_texts",
        &[
            "[[Jigsaw Structure]] couldn't be constructed, no pool named `demo:missing`",
            "[[Jigsaw Structure]] couldn't be constructed, pool `demo:empty` has no elements",
            "[[Jigsaw Structure]] Couldn't find anchor named `nope` in element pool 'demo:start'",
            "[[Jigsaw Structure]] Empty or non-existent pool: 'demo:missing'",
        ],
    );
}
