//! The documents and contexts the rows share.

#![allow(
    dead_code,
    reason = "two test binaries include this file and each uses part of it"
)]

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::{LoadContext, LoadOptions};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::version::MinEngineVersion;
use bedrock_addon_family_server_sound_definitions::SoundMetadataFile;
use uuid::Uuid;

pub const PATH: &str = "sounds/sound_definitions.json";

/// A 1.26.60 document around `definitions` (the array's elements).
pub fn file(definitions: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.60", "minecraft:server_sound_definitions": [{definitions}]}}"#
    )
}

/// A context with the Beta APIs toggle as given.
pub fn context(beta_apis: bool, base_game: bool) -> LoadContext {
    let options = LoadOptions {
        beta_apis,
        ..LoadOptions::default()
    };
    LoadContext::for_pack(
        &options,
        Uuid::from_u128(7),
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 40)),
        base_game,
    )
}

pub fn parse_with(text: &str, cx: &LoadContext) -> Result<Document<SoundMetadataFile>, Rejected> {
    SoundMetadataFile::parse(&PackPath::new(PATH).unwrap(), text.as_bytes(), cx)
}

/// Loads `text` with Beta APIs on (which the family does not ask).
pub fn parse(text: &str) -> Result<Document<SoundMetadataFile>, Rejected> {
    parse_with(text, &context(true, false))
}

#[track_caller]
pub fn loads(text: &str) -> Document<SoundMetadataFile> {
    parse(text).unwrap_or_else(|rejected| panic!("{:#?}", lines(&rejected.diagnostics)))
}

#[track_caller]
pub fn rejected(text: &str) -> Diagnostics {
    match parse(text) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

pub fn lines(diagnostics: &Diagnostics) -> Vec<String> {
    diagnostics
        .iter()
        .map(|d| format!("{} {:?} | {d}", d.code.name(), d.severity))
        .collect()
}

/// Requires `diagnostics` to hold exactly one error, named `code`, whose line contains `text`.
#[track_caller]
pub fn one_error(diagnostics: &Diagnostics, code: &str, text: &str) {
    let errors: Vec<&Diagnostic> = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 1, "{:#?}", lines(diagnostics));
    assert_eq!(errors[0].code.name(), code, "{:#?}", lines(diagnostics));
    assert!(
        errors[0].to_string().contains(text),
        "{:#?}",
        lines(diagnostics)
    );
}

pub const CLICK: &str = r#"{ "sound_event_name": "demo:click" }"#;
