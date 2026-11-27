//! What the fog test files share: documents built from members, the pack contexts and the
//! texts both eras close a refused document with.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::families::fog::FogSettings;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;

use super::common::{expect, rp};

pub const FAMILY: &str = "fog";
pub const FAILED: &str = "Failed to validate fog schema";
pub const V1_16_100: FormatVersion = FormatVersion::new(1, 16, 100);
pub const V1_21_0: FormatVersion = FormatVersion::new(1, 21, 0);
pub const V1_21_90: FormatVersion = FormatVersion::new(1, 21, 90);
pub const V1_26_20: FormatVersion = FormatVersion::new(1, 26, 20);

/// The four members a distance entry needs below 1.26.20.
pub const AIR: &str = r##""fog_start": 0.9, "fog_end": 1.0, "fog_color": "#ABD2FF", "render_distance_type": "render""##;

/// Loads `text` as `fogs/x.json`.
pub fn parse(text: &str, cx: &LoadContext) -> Result<Document<FogSettings>, Rejected> {
    FogSettings::parse(&PackPath::from_static("fogs/x.json"), text.as_bytes(), cx)
}

/// A pack that is not the base game's, at `min_engine_version` 1.21.100.
pub fn pack() -> LoadContext {
    rp(1, 21, 100)
}

/// A base-game pack: the only kind that may use the `minecraft` namespace.
pub fn base_game() -> LoadContext {
    let mut cx = rp(1, 21, 100);
    cx.is_base_game_pack = true;
    cx
}

/// A pack loaded by an engine of version `a.b.c`.
pub fn engine(a: u16, b: u16, c: u16) -> LoadContext {
    let mut cx = pack();
    cx.engine_version = FormatVersion::new(a, b, c);
    cx
}

/// A document of `version` named `demo:fog` with `members` beside its description (`""` for
/// none).
pub fn doc(version: &str, members: &str) -> String {
    named(version, "demo:fog", members)
}

/// A document of `version` named `identifier`.
pub fn named(version: &str, identifier: &str, members: &str) -> String {
    let members = if members.is_empty() {
        String::new()
    } else {
        format!(", {members}")
    };
    format!(
        r#"{{"format_version": "{version}", "minecraft:fog_settings": {{"description": {{"identifier": "{identifier}"}}{members}}}}}"#
    )
}

/// `"distance": {"air": {members}}`.
pub fn air(members: &str) -> String {
    format!(r#""distance": {{"air": {{{members}}}}}"#)
}

/// `"volumetric": {"density": {"air": {members}}}`.
pub fn density(members: &str) -> String {
    format!(r#""volumetric": {{"density": {{"air": {{{members}}}}}}}"#)
}

/// Requires the closing line of a refused document.
#[track_caller]
pub fn refused(d: &Diagnostics) {
    expect(d, "fog/failed_to_validate", Severity::Error, FAILED);
}
