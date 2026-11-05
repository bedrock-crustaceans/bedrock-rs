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
