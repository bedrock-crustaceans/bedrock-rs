//! The client-biome family's vanilla content-log texts (`VanillaText::Fixed`, and the
//! templates of `VanillaText::Formatted`, whose `{}` stand for vanilla's `%s` / `{}`). Every
//! text here is vanilla's, quoted as the game prints it; findings of the crate's own carry no
//! text.

use bedrock_addon_core::diagnostic::VanillaText;

/// Any failure of a version struct's strict load.
pub(crate) const INVALID_FILE: &str = "Invalid Client Biome file";
/// The content log after it; `{}` is the file.
pub(crate) const FAILED_TO_LOAD: &str = "Failed to load client biome document: {}";
/// A version from 1.0.0 up to the floor; `{}` are the file and the version.
pub(crate) const CANNOT_LOAD: &str = "{}: Cannot load client_biome with format_version {}";
/// A document below 1.21.130 whose root has no `format_version` (the version was found
/// elsewhere in the text); `{}` is the file.
pub(crate) const NO_VERSION_STRING: &str = "{}: No format_version string found";
/// A syntax error in a document below 1.21.130; `{}` is the parser's message.
pub(crate) const LEGACY_SYNTAX: &str = "failed to parse file into a json document: {}";
/// A root that is no object in a document below 1.21.130; `{}` is the file.
pub(crate) const LEGACY_ROOT: &str = "failed to parse client biome file {} into a json document, root value needs to be a json object";
/// The identifier's regex below 1.21.90.
pub(crate) const IDENTIFIER_UPPER: &str = "identifier has an upper case letter";
/// The identifier's regex from 1.21.90.
pub(crate) const IDENTIFIER_NAMESPACE: &str =
    "identifier has an upper case letter or does not specify any namespace";
/// `minecraft:water_appearance` without a member.
pub(crate) const WATER_EMPTY: &str =
    "No surface color or opacity specified. Specify one or remove the component.";
/// A colour that is neither `"#RRGGBB"` nor three integers.
pub(crate) const COLOR: &str = "Expected [r, g, b], with values in the 0-255 range, or a hex value with 6 characters (e.g. RGB as #FFFFFF).";
/// `minecraft:biome_music` check.
pub(crate) const VOLUME: &str = "Volume multiplier must be between 0 and 1, inclusive.";
/// `minecraft:water_appearance` check.
pub(crate) const OPACITY: &str = "surface_opacity field must be between 0 and 1, inclusive.";
/// `minecraft:ambient_sounds` check.
pub(crate) const NO_SOUND: &str = "No addition, loop, mood, or underwater variant sound specified. Specify at least one or remove the component.";
/// `minecraft:precipitation` check.
pub(crate) const PRECIPITATION: &str = "precipitation: multiple precipitation types set for biome";

/// A fixed text.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every caller fills an `Option<VanillaText>` field"
)]
pub(crate) fn fixed(text: &'static str) -> Option<VanillaText> {
    Some(VanillaText::Fixed(text))
}

/// A template with its arguments.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every caller fills an `Option<VanillaText>` field"
)]
pub(crate) fn formatted(template: &'static str, args: &[&str]) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.iter().map(|a| (*a).to_owned()).collect(),
    })
}
