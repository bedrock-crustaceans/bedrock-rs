//! Every scalar member of a definition, read from every JSON type: each line pins what the
//! load says (code, severity, path, span) and, when the file loads, the definition written
//! back. The members were strict wrappers; this table is what they read, and the oracle of
//! the change that makes them plain.

use super::common::{file, parse};

/// The values each member is read from.
const VALUES: &[&str] = &[
    "\"demo:x\"",
    "\"5\"",
    "5",
    "0.5",
    "-1",
    "true",
    "null",
    "1e39",
    "18446744073709551616",
    "[]",
    "{}",
];

/// The definitions the table reads, `{}` standing for the value.
const PLACES: &[&str] = &[
    r#"{"sound_event_name": {}}"#,
    r#"{"sound_event_name": "demo:x", "broadcast_range": {}}"#,
    r#"{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": {}}}"#,
    r#"{"sound_event_name": "demo:x", "music_info": {"artist": {}}}"#,
    r#"{"sound_event_name": "demo:x", "music_info": {"genres": [{}]}}"#,
];

/// `text` on one line: each line trimmed, the lines joined.
fn compact(text: &str) -> String {
    text.lines().map(str::trim).collect()
}

/// One line of the table.
fn line(place: &str, value: &str) -> String {
    let definition = place.replace("{}", value);
    let text = file(&definition);
    let (loaded, diagnostics) = match parse(&text) {
        Ok(document) => (
            format!(
                "loaded {}",
                compact(&document.to_json().unwrap_or_default())
            ),
            document.diagnostics().clone(),
        ),
        Err(rejected) => ("rejected".to_owned(), rejected.diagnostics),
    };
    let findings: Vec<String> = diagnostics
        .iter()
        .map(|d| {
            let span = d
                .location
                .span
                .map_or("-".to_owned(), |s| format!("{}+{}", s.offset, s.len));
            format!(
                "{} {:?} {} {span}",
                d.code.name(),
                d.severity,
                d.location.json_path
            )
        })
        .collect();
    format!("{definition} => {loaded} [{}]", findings.join("; "))
}

#[test]
fn every_scalar_member_reads_as_pinned() {
    let mut table = String::new();
    for place in PLACES {
        for value in VALUES {
            table.push_str(&line(place, value));
            table.push('\n');
        }
    }
    assert!(
        table == EXPECTED,
        "--- actual ---
{table}--- end ---"
    );
}

/// The table as the strict wrappers read it.
const EXPECTED: &str = r#"{"sound_event_name": "demo:x"} => loaded {"format_version": "1.26.60","minecraft:server_sound_definitions": [{"sound_event_name": "demo:x"}]} []
{"sound_event_name": "5"} => rejected [NamespaceRule Error minecraft:server_sound_definitions/0/sound_event_name -]
{"sound_event_name": 5} => rejected [InvalidType Error minecraft:server_sound_definitions/0/sound_event_name 70+18]
{"sound_event_name": 0.5} => rejected [InvalidType Error minecraft:server_sound_definitions/0/sound_event_name 70+18]
{"sound_event_name": -1} => rejected [InvalidType Error minecraft:server_sound_definitions/0/sound_event_name 70+18]
{"sound_event_name": true} => rejected [InvalidType Error minecraft:server_sound_definitions/0/sound_event_name 70+18]
{"sound_event_name": null} => rejected [InvalidType Error minecraft:server_sound_definitions/0/sound_event_name 70+18]
{"sound_event_name": 1e39} => rejected [InvalidType Error minecraft:server_sound_definitions/0/sound_event_name 70+18]
{"sound_event_name": 18446744073709551616} => rejected [InvalidType Error minecraft:server_sound_definitions/0/sound_event_name 70+18]
{"sound_event_name": []} => rejected [InvalidType Error minecraft:server_sound_definitions/0/sound_event_name/0 91+1]
{"sound_event_name": {}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/sound_event_name 91+1]
{"sound_event_name": "demo:x", "broadcast_range": "demo:x"} => rejected [InvalidType Error minecraft:server_sound_definitions/0/broadcast_range 100+17]
{"sound_event_name": "demo:x", "broadcast_range": "5"} => rejected [InvalidType Error minecraft:server_sound_definitions/0/broadcast_range 100+17]
{"sound_event_name": "demo:x", "broadcast_range": 5} => loaded {"format_version": "1.26.60","minecraft:server_sound_definitions": [{"sound_event_name": "demo:x","broadcast_range": 5.0}]} []
{"sound_event_name": "demo:x", "broadcast_range": 0.5} => loaded {"format_version": "1.26.60","minecraft:server_sound_definitions": [{"sound_event_name": "demo:x","broadcast_range": 0.5}]} []
{"sound_event_name": "demo:x", "broadcast_range": -1} => loaded {"format_version": "1.26.60","minecraft:server_sound_definitions": [{"sound_event_name": "demo:x","broadcast_range": -1.0}]} [InvalidValue Info minecraft:server_sound_definitions/0/broadcast_range -]
{"sound_event_name": "demo:x", "broadcast_range": true} => rejected [InvalidType Error minecraft:server_sound_definitions/0/broadcast_range 100+17]
{"sound_event_name": "demo:x", "broadcast_range": null} => rejected [InvalidType Error minecraft:server_sound_definitions/0/broadcast_range 119+4]
{"sound_event_name": "demo:x", "broadcast_range": 1e39} => rejected [InvalidType Error minecraft:server_sound_definitions/0/broadcast_range 100+17]
{"sound_event_name": "demo:x", "broadcast_range": 18446744073709551616} => loaded {"format_version": "1.26.60","minecraft:server_sound_definitions": [{"sound_event_name": "demo:x","broadcast_range": 1.8446744e19}]} []
{"sound_event_name": "demo:x", "broadcast_range": []} => rejected [InvalidType Error minecraft:server_sound_definitions/0/broadcast_range/0 120+1]
{"sound_event_name": "demo:x", "broadcast_range": {}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/broadcast_range 120+1]
{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": "demo:x"}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/duration_info/seconds 139+9]
{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": "5"}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/duration_info/seconds 139+9]
{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": 5}} => loaded {"format_version": "1.26.60","minecraft:server_sound_definitions": [{"sound_event_name": "demo:x","duration_info": {"mode": "game_time","seconds": 5.0}}]} []
{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": 0.5}} => loaded {"format_version": "1.26.60","minecraft:server_sound_definitions": [{"sound_event_name": "demo:x","duration_info": {"mode": "game_time","seconds": 0.5}}]} []
{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": -1}} => loaded {"format_version": "1.26.60","minecraft:server_sound_definitions": [{"sound_event_name": "demo:x","duration_info": {"mode": "game_time","seconds": -1.0}}]} [InvalidValue Info minecraft:server_sound_definitions/0/duration_info/seconds -]
{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": true}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/duration_info/seconds 139+9]
{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": null}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/duration_info/seconds 139+9]
{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": 1e39}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/duration_info/seconds 139+9]
{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": 18446744073709551616}} => loaded {"format_version": "1.26.60","minecraft:server_sound_definitions": [{"sound_event_name": "demo:x","duration_info": {"mode": "game_time","seconds": 1.8446744e19}}]} []
{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": []}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/duration_info/seconds/0 151+1]
{"sound_event_name": "demo:x", "duration_info": {"mode": "game_time", "seconds": {}}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/duration_info/seconds 151+1]
{"sound_event_name": "demo:x", "music_info": {"artist": "demo:x"}} => loaded {"format_version": "1.26.60","minecraft:server_sound_definitions": [{"sound_event_name": "demo:x","music_info": {"artist": "demo:x"}}]} []
{"sound_event_name": "demo:x", "music_info": {"artist": "5"}} => loaded {"format_version": "1.26.60","minecraft:server_sound_definitions": [{"sound_event_name": "demo:x","music_info": {"artist": "5"}}]} []
{"sound_event_name": "demo:x", "music_info": {"artist": 5}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/artist 115+8]
{"sound_event_name": "demo:x", "music_info": {"artist": 0.5}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/artist 115+8]
{"sound_event_name": "demo:x", "music_info": {"artist": -1}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/artist 115+8]
{"sound_event_name": "demo:x", "music_info": {"artist": true}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/artist 115+8]
{"sound_event_name": "demo:x", "music_info": {"artist": null}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/artist 125+4]
{"sound_event_name": "demo:x", "music_info": {"artist": 1e39}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/artist 115+8]
{"sound_event_name": "demo:x", "music_info": {"artist": 18446744073709551616}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/artist 115+8]
{"sound_event_name": "demo:x", "music_info": {"artist": []}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/artist/0 126+1]
{"sound_event_name": "demo:x", "music_info": {"artist": {}}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/artist 126+1]
{"sound_event_name": "demo:x", "music_info": {"genres": ["demo:x"]}} => loaded {"format_version": "1.26.60","minecraft:server_sound_definitions": [{"sound_event_name": "demo:x","music_info": {"genres": ["demo:x"]}}]} []
{"sound_event_name": "demo:x", "music_info": {"genres": ["5"]}} => loaded {"format_version": "1.26.60","minecraft:server_sound_definitions": [{"sound_event_name": "demo:x","music_info": {"genres": ["5"]}}]} []
{"sound_event_name": "demo:x", "music_info": {"genres": [5]}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/genres 125+1]
{"sound_event_name": "demo:x", "music_info": {"genres": [0.5]}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/genres 125+1]
{"sound_event_name": "demo:x", "music_info": {"genres": [-1]}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/genres 125+1]
{"sound_event_name": "demo:x", "music_info": {"genres": [true]}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/genres 125+1]
{"sound_event_name": "demo:x", "music_info": {"genres": [null]}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/genres 125+1]
{"sound_event_name": "demo:x", "music_info": {"genres": [1e39]}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/genres 125+1]
{"sound_event_name": "demo:x", "music_info": {"genres": [18446744073709551616]}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/genres 125+1]
{"sound_event_name": "demo:x", "music_info": {"genres": [[]]}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/genres/0/0 127+1]
{"sound_event_name": "demo:x", "music_info": {"genres": [{}]}} => rejected [InvalidType Error minecraft:server_sound_definitions/0/music_info/genres/0 127+1]
"#;
