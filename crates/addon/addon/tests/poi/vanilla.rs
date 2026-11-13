//! The POI documents vanilla 1.26.60 ships: the 20 MCB files of `experimental_poi`
//! (`poi/blocks/*.json`, `poi/tags/village.json`, header 1.26.40), decoded with the field order
//! of the schema (every member is written, defaults included), and the four
//! hand-written documents (`fixtures/poi/engine/vanilla-*.json`).
//! The MCB originals load through the storage's POI body decoder to the same documents
//! as the decoded twins.

use std::path::PathBuf;

use bedrock_addon::document::Content;
use bedrock_addon::families::poi::{PoiTag, PoiType};
use bedrock_addon::id::{BlockId, PoiTagId, PoiTypeId};
use bedrock_addon::pack::PackOrigin;

use super::common::{describe, fixture, loads};
use super::stacks::{fixture_packs_from, pack_from, stack};

/// The decoded set, as a pack of the game.
const SET: &str = "poi42_experimental_poi";

/// A file below `tests/fixtures/poi/`.
fn file(rest: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/poi")
        .join(rest);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Loads, writes, loads again: the value is the same and the second write is the first.
#[track_caller]
fn round_trips<T: Content + PartialEq + std::fmt::Debug>(text: &str) -> String {
    let first = loads::<T>(text);
    assert!(
        first.diagnostics().is_empty(),
        "{}",
        describe(first.diagnostics().iter())
    );
    let written = first.to_json().unwrap();
    let again = loads::<T>(&written);
    assert_eq!(again.value(), first.value());
    assert_eq!(
        again.to_json().unwrap(),
        written,
        "the second write differs"
    );
    written
}

#[test]
fn the_experimental_poi_set_loads_from_a_game_pack_without_a_finding() {
    let stack = stack(fixture_packs_from(SET, PackOrigin::Package));
    let poi = stack.poi();
    assert_eq!(
        (poi.types.len(), poi.tags.len(), poi.rejected().count()),
        (19, 1, 0)
    );
    assert_eq!(
        poi.diagnostics().count(),
        0,
        "{}",
        describe(poi.diagnostics())
    );
    let village = PoiTagId::from_static("minecraft:village");
    // `village` holds `home` and `meeting` itself and the job sites through its child.
    for member in ["minecraft:home", "minecraft:meeting", "minecraft:cleric"] {
        assert!(poi.has_tag(&PoiTypeId::parse(member), &village), "{member}");
    }
    assert!(poi.contains_tag(&PoiTagId::from_static("minecraft:acquirable_job_site")));
    assert_eq!(
        poi.type_of_block(&BlockId::from_static(
            "minecraft:waxed_oxidized_lightning_rod"
        )),
        Some(&PoiTypeId::from_static("minecraft:lightning_rod"))
    );
}

#[test]
fn every_document_of_the_set_round_trips_at_the_value_level() {
    let dir = super::common::fixture_dir(SET).join("poi");
    let mut count = 0;
    for (kind, is_tag) in [("blocks", false), ("tags", true)] {
        let mut names: Vec<_> = std::fs::read_dir(dir.join(kind))
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        for name in names {
            let text = fixture(&format!("{SET}/poi/{kind}/{name}"));
            if is_tag {
                round_trips::<PoiTag>(&text);
            } else {
                round_trips::<PoiType>(&text);
            }
            count += 1;
        }
    }
    assert_eq!(count, 20);
}

#[test]
fn the_hand_written_fixtures_are_the_documents_of_the_set() {
    for (written, decoded) in [
        ("vanilla-cleric", "blocks/cleric"),
        ("vanilla-home", "blocks/home"),
        ("vanilla-lightning_rod", "blocks/lightning_rod"),
    ] {
        let written = String::from_utf8(file(&format!("engine/{written}.json"))).unwrap();
        let ours = fixture(&format!("{SET}/poi/{decoded}.json"));
        assert_eq!(
            loads::<PoiType>(&written).value(),
            loads::<PoiType>(&ours).value()
        );
        round_trips::<PoiType>(&written);
    }
    let written = String::from_utf8(file("engine/vanilla-tag-village.json")).unwrap();
    let ours = fixture(&format!("{SET}/poi/tags/village.json"));
    assert_eq!(
        loads::<PoiTag>(&written).value(),
        loads::<PoiTag>(&ours).value()
    );
    let cleric = loads::<PoiType>(&String::from_utf8(file("engine/vanilla-cleric.json")).unwrap());
    assert_eq!((cleric.tickets(), cleric.usable_distance()), (1, 1));
}

/// The MCB originals (header 1.26.40), the files the game ships: every one of the 20 loads on
/// a base-game stack without a finding, to the same documents as the decoded twins.
#[test]
fn the_mcb_originals_load_on_a_game_stack_without_a_finding() {
    let mut files = Vec::new();
    for kind in ["blocks", "tags"] {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/poi/mcb/poi")
            .join(kind);
        let mut names: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        for name in names {
            files.push((
                format!("poi/{kind}/{name}"),
                file(&format!("mcb/poi/{kind}/{name}")),
            ));
        }
    }
    assert_eq!(files.len(), 20);
    let header = file("mcb/poi/blocks/cleric.json");
    assert_eq!(&header[..12], b"\x7fMCB\x01\x00\x1a\x00\x28\x00\x00\x00");
    let baked = stack(vec![pack_from(10, PackOrigin::Package, files)]);
    let poi = baked.poi();
    assert_eq!(
        (poi.types.len(), poi.tags.len(), poi.rejected().count()),
        (19, 1, 0)
    );
    assert_eq!(
        poi.diagnostics().count(),
        0,
        "{}",
        describe(poi.diagnostics())
    );
    assert_eq!(baked.report().iter().count(), 0);
    let decoded = stack(fixture_packs_from(SET, PackOrigin::Package));
    for (id, document) in &poi.types {
        assert_eq!(
            Some(document.value()),
            decoded
                .poi()
                .types
                .get(id)
                .map(bedrock_addon::document::Document::value),
            "{id:?}"
        );
    }
    let village = PoiTagId::from_static("minecraft:village");
    assert_eq!(
        poi.tags
            .get(&village)
            .map(bedrock_addon::document::Document::value),
        decoded
            .poi()
            .tags
            .get(&village)
            .map(bedrock_addon::document::Document::value)
    );
}
