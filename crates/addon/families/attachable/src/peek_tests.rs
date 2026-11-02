//! The peek at a file's root before the chain reads it.

use super::load::root_of;

/// The peek reads what the load reads: of a payload written twice the last stands, whole.
/// An untyped read of the text as written would merge the two objects and name an
/// identifier the loaded document does not have.
#[test]
fn the_peek_of_a_payload_written_twice_reads_the_last_whole() {
    let text = r#"{"format_version": "1.10.0", "minecraft:attachable": {"description": {"identifier": "demo:first", "min_engine_version": "1.8.0"}}, "minecraft:attachable": {"description": {"materials": {}}}}"#;
    let root = root_of(text.as_bytes()).unwrap();
    let description = root
        .as_object()
        .and_then(|root| root.get("minecraft:attachable"))
        .and_then(|payload| payload.as_object())
        .and_then(|payload| payload.get("description"))
        .and_then(|description| description.as_object())
        .unwrap();
    let keys: Vec<&str> = description.iter().map(|(key, _)| key.as_str()).collect();
    assert_eq!(keys, ["materials"]);
}
