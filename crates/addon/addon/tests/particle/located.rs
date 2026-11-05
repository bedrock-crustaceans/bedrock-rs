//! Every finding of a loaded or a rejected effect is in the effect's pack and file, and the
//! places the reader records stay for the link pass.

use bedrock_addon::diagnostic::{Diagnostic, JsonPath};
use bedrock_addon::pack::PackRef;

use super::common::{LEGACY, effect, loaded, path, rejected};

/// A legacy effect whose looping entries 0 and 2 are no objects: the reader drops them.
fn two_dropped() -> String {
    effect(
        LEGACY,
        r#""minecraft:emitter_lifetime_events": {"looping_travel_distance_events": [7, {"distance": 1, "effects": "e"}, 7, {"distance": 2, "effects": "e"}]}"#,
    )
}

#[track_caller]
fn assert_in_the_file<'a>(findings: impl IntoIterator<Item = &'a Diagnostic>) {
    let mut count = 0;
    for finding in findings {
        assert_eq!(
            finding.location.pack,
            Some(PackRef::app_data()),
            "{finding:?}"
        );
        assert_eq!(finding.location.path, Some(path()), "{finding:?}");
        count += 1;
    }
    assert!(count > 0, "the input gives findings");
}

#[test]
fn particle_findings_carry_the_file_and_pack() {
    assert_in_the_file(loaded(&two_dropped()).diagnostics().iter());
    assert_in_the_file(rejected("{").iter());
}

#[test]
fn particle_places_stay_after_the_findings_are_located() {
    let document = loaded(&two_dropped());
    let list = JsonPath::new().with(&[
        "particle_effect",
        "components",
        "minecraft:emitter_lifetime_events",
        "looping_travel_distance_events",
    ]);
    // The entry the effect holds second is the file's fourth.
    let place = document.diagnostics().place_of(&list.index(1));
    assert_eq!(place, list.index(3));
}

/// The spans of every finding, loaded or rejected.
fn spans(text: &str) -> Vec<(u32, u32)> {
    let findings = match super::common::load(text, &super::common::context()) {
        Ok(document) => document.diagnostics().iter().cloned().collect::<Vec<_>>(),
        Err(rejected) => rejected.diagnostics.iter().cloned().collect(),
    };
    findings
        .iter()
        .filter_map(|finding| finding.location.span)
        .map(|span| (span.offset, span.len))
        .collect()
}

#[test]
fn a_bom_moves_every_span_by_its_three_bytes() {
    // The BOM is blanked, not cut off: every span is an offset into the file as written.
    // Content after the root value: a finding on that content.
    let text = format!("{} garbage", effect(super::common::CEREAL, ""));
    let plain = spans(&text);
    assert!(!plain.is_empty(), "the input gives a finding with a span");
    let shifted: Vec<(u32, u32)> = plain.iter().map(|&(at, len)| (at + 3, len)).collect();
    assert_eq!(spans(&format!("\u{feff}{text}")), shifted);
}
