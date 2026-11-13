//! The registries a client builds from the decoded payload of a
//! `TrimDataPacket`, the lists a server sends, and vanilla's one content-log text (V3).

use bedrock_addon_core::diagnostic::{Phase, Severity};
use bedrock_addon_family_item::codes as item_codes;
use bedrock_addon_family_item::trim::{FormatCode, TrimData};

use super::common::{
    item, material_entry, material_rows, pattern_entry, pattern_rows, payload, vanilla_table,
};

#[test]
fn pk01_a_decoded_payload_builds_the_vanilla_registries() {
    let (patterns, materials) = payload();
    assert_eq!(patterns.len(), 18, "the payload's patterns");
    assert_eq!(materials.len(), 11, "the payload's materials");

    let data = TrimData::from_packet(&patterns, &materials);

    // The server's table is the engine's: equal whatever order the patterns came in.
    assert_eq!(data, *TrimData::vanilla());
    assert!(data.diagnostics().is_empty());
    // The materials are a list, kept in the order sent, which is the build order.
    assert_eq!(material_rows(&data), vanilla_table().materials);
    // Every pattern is there, found both ways.
    let mut expected = vanilla_table().patterns;
    expected.sort_by(|a, b| a.item_name.cmp(&b.item_name));
    assert_eq!(pattern_rows(&data), expected);
    for row in &expected {
        assert_eq!(
            data.pattern(&row.pattern_id).unwrap().item_name.as_str(),
            row.item_name
        );
        assert_eq!(
            data.pattern_for_item(&item(&row.item_name))
                .unwrap()
                .pattern_id
                .as_str(),
            row.pattern_id
        );
    }
}

/// The payload's patterns come as the server's hash map gives them: neither in build order nor
/// sorted, so the order `from_packet` gives them is its own.
#[test]
fn pk01_the_patterns_are_sorted_by_template_item_whatever_the_wire_order() {
    let (patterns, materials) = payload();
    let wire: Vec<&str> = patterns
        .iter()
        .map(|entry| entry.template_item.as_str())
        .collect();
    let build: Vec<String> = vanilla_table()
        .patterns
        .into_iter()
        .map(|row| row.item_name)
        .collect();
    let mut sorted = wire.clone();
    sorted.sort_unstable();
    assert_ne!(wire, build, "the payload is not in build order");
    assert_ne!(wire, sorted, "the payload is not sorted");

    let data = TrimData::from_packet(&patterns, &materials);
    let items: Vec<&str> = data
        .patterns()
        .map(|pattern| pattern.item_name.as_str())
        .collect();
    assert_eq!(items, sorted);

    // The same registries from the reversed list.
    let reversed: Vec<_> = patterns.iter().rev().cloned().collect();
    let again = TrimData::from_packet(&reversed, &materials);
    assert_eq!(again, data);
    assert_eq!(pattern_rows(&again), pattern_rows(&data));
}

#[test]
fn pk02_the_vanilla_table_round_trips_through_the_packet_lists() {
    let vanilla = TrimData::vanilla();
    let (patterns, materials) = vanilla.to_packet();
    assert_eq!(patterns.len(), 18);
    assert_eq!(materials.len(), 11);

    let decoded = TrimData::from_packet(&patterns, &materials);
    assert_eq!(decoded, *vanilla);
    assert!(decoded.diagnostics().is_empty());
    assert_eq!(material_rows(&decoded), material_rows(vanilla));
}

/// The lists are the payload's fields: template item and pattern; material, colour and item.
#[test]
fn pk02_the_lists_carry_every_field_of_the_table() {
    let (patterns, materials) = TrimData::vanilla().to_packet();
    let expected = vanilla_table();
    let sent: Vec<(&str, &str)> = patterns
        .iter()
        .map(|entry| (entry.pattern.as_str(), entry.template_item.as_str()))
        .collect();
    let rows: Vec<(&str, &str)> = expected
        .patterns
        .iter()
        .map(|row| (row.pattern_id.as_str(), row.item_name.as_str()))
        .collect();
    assert_eq!(sent, rows);
    let sent: Vec<(&str, &str, &str)> = materials
        .iter()
        .map(|entry| (entry.id.as_str(), entry.color.as_str(), entry.item.as_str()))
        .collect();
    let rows: Vec<(&str, &str, &str)> = expected
        .materials
        .iter()
        .map(|row| {
            (
                row.material_id.as_str(),
                row.color.as_str(),
                row.item_name.as_str(),
            )
        })
        .collect();
    assert_eq!(sent, rows);
}

/// Real data both ways: the payload's own lists come back from the registries they built.
#[test]
fn pk02_a_decoded_payload_round_trips() {
    let (patterns, materials) = payload();
    let (mut sent_patterns, sent_materials) =
        TrimData::from_packet(&patterns, &materials).to_packet();
    assert_eq!(sent_materials, materials);
    let mut patterns = patterns;
    patterns.sort_by(|a, b| a.template_item.as_str().cmp(b.template_item.as_str()));
    sent_patterns.sort_by(|a, b| a.template_item.as_str().cmp(b.template_item.as_str()));
    assert_eq!(sent_patterns, patterns);
}

#[test]
fn empty_lists_build_empty_registries() {
    let data = TrimData::from_packet(&[], &[]);
    assert_eq!(data, TrimData::default());
    assert_eq!(data.patterns().len(), 0);
    assert!(data.materials().is_empty());
    assert_eq!(data.to_packet(), (Vec::new(), Vec::new()));
    assert_ne!(data, *TrimData::vanilla());
}

/// The client assigns the received vector: order and repeats are kept, and
/// a lookup returns the first match, by item name or by material id.
#[test]
fn the_materials_are_taken_as_sent_and_a_lookup_finds_the_first() {
    let materials = [
        material_entry("quartz", "§h", "minecraft:quartz"),
        material_entry("quartz", "§x", "demo:gem"),
        material_entry("ruby", "red", "minecraft:quartz"),
    ];
    let data = TrimData::from_packet(&[], &materials);

    assert_eq!(data.materials().len(), 3);
    assert_eq!(data.to_packet().1, materials.to_vec());
    let by_id = data.material("quartz").unwrap();
    assert_eq!(by_id.item_name.as_str(), "minecraft:quartz");
    assert_eq!(by_id.color.as_str(), "§h");
    let by_item = data.material_for_item(&item("minecraft:quartz")).unwrap();
    assert_eq!(by_item.material_id.as_str(), "quartz");
    assert_eq!(
        data.material_for_item(&item("demo:gem"))
            .unwrap()
            .color
            .as_str(),
        "§x"
    );
    let ruby = data.material("ruby").unwrap();
    assert_eq!(ruby.color.as_str(), "red");
    assert_eq!(ruby.color.letter(), None);
    assert!(data.diagnostics().is_empty());
}

/// V3: the client logs `Overwriting pattern template. Duplicate entries with
/// template item: "%s"` at error level (level 3) and overwrites
/// the entry. Nothing is refused, so ours is a kept error carrying that text.
#[test]
fn v3_a_template_item_sent_twice_is_overwritten_with_a_kept_error_in_vanillas_words() {
    let patterns = [
        pattern_entry("demo:a_template", "one"),
        pattern_entry("demo:b_template", "two"),
        pattern_entry("demo:a_template", "three"),
    ];
    let data = TrimData::from_packet(&patterns, &[]);

    assert_eq!(data.patterns().len(), 2);
    let a = data.pattern_for_item(&item("demo:a_template")).unwrap();
    assert_eq!(a.pattern_id.as_str(), "three", "the later entry wins");
    assert_eq!(
        data.pattern("three").unwrap().item_name.as_str(),
        "demo:a_template"
    );
    assert_eq!(
        data.pattern("two").unwrap().item_name.as_str(),
        "demo:b_template"
    );

    let findings: Vec<_> = data.diagnostics().iter().collect();
    assert_eq!(findings.len(), 1, "{findings:?}");
    let finding = findings[0];
    assert!(finding.code.is(&item_codes::DUPLICATE_TEMPLATE));
    assert_eq!((finding.severity, finding.kept), (Severity::Error, true));
    assert_eq!(finding.phase, Phase::Parse);
    assert_eq!(finding.location.json_path.to_string(), "patterns/2");
    assert_eq!(
        finding.code.to_string(),
        "Overwriting pattern template. Duplicate entries with template item: \"demo:a_template\""
    );
}

/// The client writes its two maps independently and never clears them, so the identifier of
/// an overwritten entry still names its template item (the pattern-to-template map keeps it).
#[test]
fn v3_the_overwritten_pattern_is_still_found_by_its_id() {
    let patterns = [
        pattern_entry("demo:a_template", "one"),
        pattern_entry("demo:a_template", "three"),
    ];
    let data = TrimData::from_packet(&patterns, &[]);
    let stale = data.pattern("one").expect("vanilla's reverse map keeps it");
    assert_eq!(stale.pattern_id.as_str(), "one");
    assert_eq!(stale.item_name.as_str(), "demo:a_template");
    assert_eq!(data.patterns().len(), 1);
    // Such registries are not the ones a clean list builds.
    assert_ne!(
        data,
        TrimData::from_packet(&[pattern_entry("demo:a_template", "three")], &[])
    );
}

/// A server walks its template-item map (`getAllEntries`): one pattern per template item,
/// whatever the other map still holds.
#[test]
fn v3_the_lists_sent_have_one_pattern_per_template_item() {
    let data = TrimData::from_packet(
        &[
            pattern_entry("demo:a_template", "one"),
            pattern_entry("demo:a_template", "three"),
        ],
        &[],
    );
    assert_eq!(
        data.to_packet().0,
        vec![pattern_entry("demo:a_template", "three")]
    );
}

/// Only the template item is checked for a duplicate: two items may send the same pattern,
/// and the identifier then names the item written last.
#[test]
fn v3_one_pattern_for_two_template_items_is_no_duplicate() {
    let patterns = [
        pattern_entry("demo:b_template", "shared"),
        pattern_entry("demo:a_template", "shared"),
    ];
    let data = TrimData::from_packet(&patterns, &[]);
    assert!(data.diagnostics().is_empty());
    assert_eq!(data.patterns().len(), 2);
    assert_eq!(
        data.pattern("shared").unwrap().item_name.as_str(),
        "demo:a_template",
        "the entry written last, not the first by template item"
    );
    for template in ["demo:a_template", "demo:b_template"] {
        assert_eq!(
            data.pattern_for_item(&item(template))
                .unwrap()
                .pattern_id
                .as_str(),
            "shared"
        );
    }
}

/// A repeated material is no finding: the list is assigned as it is.
#[test]
fn v3_a_material_sent_twice_is_no_finding() {
    let materials = [
        material_entry("quartz", "§h", "minecraft:quartz"),
        material_entry("quartz", "§h", "minecraft:quartz"),
    ];
    let data = TrimData::from_packet(&[], &materials);
    assert!(data.diagnostics().is_empty());
    assert_eq!(data.materials().len(), 2);
}

/// The colour is "a two-character `§` formatting code".
#[test]
fn a_format_code_is_the_section_sign_and_one_character() {
    assert_eq!(FormatCode::new("§h").letter(), Some('h'));
    assert_eq!(FormatCode::new("§h").as_str(), "§h");
    for other in ["", "§", "h", "xh", "§hh", "h§", "red", "§§h"] {
        assert_eq!(FormatCode::new(other).letter(), None, "{other:?}");
        assert_eq!(FormatCode::new(other).as_str(), other);
    }
    let letters: String = TrimData::vanilla()
        .materials()
        .iter()
        .map(|material| material.color.letter().expect("a two-character code"))
        .collect();
    assert_eq!(letters, "hijmnpqstuv");
}

/// Two values are equal when both pattern maps and the material lists are; what building
/// them found is not compared, and neither is the order of the patterns.
#[test]
fn equality_is_the_two_maps_and_the_material_list() {
    let quartz = material_entry("quartz", "§h", "minecraft:quartz");
    let iron = material_entry("iron", "§i", "minecraft:iron_ingot");
    let one = pattern_entry("demo:a_template", "one");
    let two = pattern_entry("demo:b_template", "two");

    // The findings are not compared.
    let twice = TrimData::from_packet(&[one.clone(), one.clone()], &[]);
    let single = TrimData::from_packet(std::slice::from_ref(&one), &[]);
    assert_eq!(twice.diagnostics().len(), 1);
    assert!(single.diagnostics().is_empty());
    assert_eq!(twice, single);

    // The patterns are compared as maps, the materials as a list.
    let base = TrimData::from_packet(&[one.clone(), two.clone()], &[quartz.clone(), iron.clone()]);
    assert_eq!(
        base,
        TrimData::from_packet(&[two.clone(), one.clone()], &[quartz.clone(), iron.clone()])
    );
    assert_ne!(
        base,
        TrimData::from_packet(&[one.clone(), two.clone()], &[iron.clone(), quartz.clone()]),
        "the material order"
    );
    assert_ne!(
        base,
        TrimData::from_packet(&[one.clone(), two.clone()], std::slice::from_ref(&quartz)),
        "a material less"
    );
    assert_ne!(
        base,
        TrimData::from_packet(
            &[one.clone(), two.clone()],
            &[
                quartz.clone(),
                material_entry("iron", "§x", "minecraft:iron_ingot")
            ]
        ),
        "a material's colour"
    );
    assert_ne!(
        base,
        TrimData::from_packet(std::slice::from_ref(&one), &[quartz.clone(), iron.clone()]),
        "a pattern less"
    );
    assert_ne!(
        base,
        TrimData::from_packet(
            &[one.clone(), pattern_entry("demo:b_template", "other")],
            &[quartz.clone(), iron.clone()]
        ),
        "a template item's pattern"
    );
    assert_ne!(
        base,
        TrimData::from_packet(
            &[one, pattern_entry("demo:c_template", "two")],
            &[quartz, iron]
        ),
        "a pattern's template item"
    );
}

/// The item map is compared too: the same two entries in the other order leave the same
/// identifier map and another pattern for the item.
#[test]
fn equality_compares_the_item_map_when_the_identifier_map_agrees() {
    let one = pattern_entry("demo:a_template", "one");
    let two = pattern_entry("demo:a_template", "two");
    let ends_with_two = TrimData::from_packet(&[one.clone(), two.clone()], &[]);
    let ends_with_one = TrimData::from_packet(&[two, one], &[]);
    for data in [&ends_with_two, &ends_with_one] {
        assert!(data.pattern("one").is_some());
        assert!(data.pattern("two").is_some());
    }
    assert_ne!(ends_with_two, ends_with_one);
}

/// The sort key is the template item, not the pattern: here the two orders differ.
#[test]
fn the_patterns_are_sorted_by_the_item_not_by_the_pattern() {
    let data = TrimData::from_packet(
        &[
            pattern_entry("demo:b_template", "alpha"),
            pattern_entry("demo:a_template", "zeta"),
            pattern_entry("demo:c_template", "beta"),
        ],
        &[],
    );
    let order: Vec<&str> = data
        .patterns()
        .map(|pattern| pattern.pattern_id.as_str())
        .collect();
    assert_eq!(order, ["zeta", "alpha", "beta"]);
}
