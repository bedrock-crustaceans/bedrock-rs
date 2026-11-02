//! Helpers of the item tests.

#![allow(dead_code)]

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_item::Item;
use uuid::Uuid;

/// One of the ten vanilla pack UUIDs that satisfy `LegacyCreatorFeatures`.
pub const ALLOWLISTED: Uuid = Uuid::from_u128(0xda09_e399_6f5e_4287_a013_4cf1_3de3_2166);
/// A third-party pack UUID.
pub const THIRD_PARTY: Uuid = Uuid::from_u128(0x1234_5678_9abc_4def_8123_4567_89ab_cdef);

/// A third-party behaviour pack's context.
pub fn third_party() -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.pack_uuid = THIRD_PARTY;
    cx
}

/// An allowlisted (legacy creator features) pack's context.
pub fn allowlisted() -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.pack_uuid = ALLOWLISTED;
    cx
}

/// Loads `json` as an item file.
pub fn load(json: &str, cx: &LoadContext) -> Result<Document<Item>, Rejected> {
    Item::parse(
        &PackPath::from_static("items/test.json"),
        json.as_bytes(),
        cx,
    )
}

/// Loads `json`, which must load.
#[track_caller]
pub fn loads(json: &str, cx: &LoadContext) -> Document<Item> {
    match load(json, cx) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// Loads `json`, which must be rejected; its findings.
#[track_caller]
pub fn rejects(json: &str, cx: &LoadContext) -> Diagnostics {
    match load(json, cx) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The findings of a load, loaded or not.
pub fn findings(json: &str, cx: &LoadContext) -> Diagnostics {
    match load(json, cx) {
        Ok(document) => document.diagnostics().clone(),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The findings whose code is `name`.
pub fn with_code<'a>(d: &'a Diagnostics, name: &str) -> Vec<&'a Diagnostic> {
    d.iter().filter(|f| f.code.name() == name).collect()
}

/// Whether a finding's message contains `text`.
pub fn has_text(d: &Diagnostics, text: &str) -> bool {
    d.iter().any(|f| f.code.to_string().contains(text))
}

/// The findings above info.
pub fn above_info(d: &Diagnostics) -> Vec<&Diagnostic> {
    d.iter().filter(|f| f.severity > Severity::Info).collect()
}

/// The `UnrecognizedField` findings of `d`, at any severity.
pub fn unknown_members(d: &Diagnostics) -> Vec<&Diagnostic> {
    d.iter()
        .filter(|f| f.code.name() == "UnrecognizedField")
        .collect()
}

/// A minimal document at `version` with `components` (a JSON object's members) and
/// `identifier`.
pub fn item(version: &str, identifier: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "{identifier}"}}, "components": {{{components}}}}}}}"#
    )
}

/// Whether `d` holds a finding whose code matches.
pub fn has(d: &Diagnostics, pred: impl Fn(&DiagnosticCode) -> bool) -> bool {
    d.iter().any(|f| pred(&f.code))
}

/// Whether the newest writer refused the item at `path` (`minecraft:item/…`): a member the
/// strict 1.26.50 format does not bind.
#[track_caller]
pub fn refused_at(
    written: &Result<String, bedrock_addon_core::json::EncodeError>,
    path: &str,
) -> bool {
    matches!(
        written,
        Err(bedrock_addon_core::json::EncodeError::NotRepresentable { at, .. }) if at.to_string() == path
    )
}

/// The text the as-written writer gives of `json` (the struct its version selects, in its own
/// format): where an older document keeps what the newest format refuses.
#[track_caller]
pub fn written_as_read(json: &str, cx: &LoadContext) -> String {
    let document = Item::parse_as_written(
        &PackPath::from_static("items/test.json"),
        json.as_bytes(),
        cx,
    )
    .unwrap_or_else(|rejected| panic!("rejected: {:#?}", rejected.diagnostics));
    document.value().to_json().expect("written as read")
}

/// The payload the newest struct holds of `item` (`VersionStruct::write`), as JSON text: what
/// the newest writer writes, without its refusal of the members the strict 1.26.50 format does
/// not bind; for the tests of what a step carries.
pub fn newest_payload(item: &Item) -> String {
    let payload = bedrock_addon_core::versioning::VersionStruct::write(item).expect("the payload");
    let mut out = Vec::new();
    bedrock_addon_core::json::write_json(
        &payload,
        &mut out,
        bedrock_addon_core::json::WriteStyle::default(),
    )
    .expect("the payload's text");
    String::from_utf8(out).expect("UTF-8")
}

/// The newest writer's text when it wrote the item as a 1.26.0 legacy component document: the item holds a member the strict 1.26.50 format refuses, and that
/// text reads back as the same item.
#[track_caller]
pub fn written_at_1_26_0(
    written: &Result<String, bedrock_addon_core::json::EncodeError>,
) -> String {
    match written {
        Ok(text) if text.contains("\"format_version\": \"1.26.0\"") => text.clone(),
        other => panic!("not written at 1.26.0: {other:?}"),
    }
}
