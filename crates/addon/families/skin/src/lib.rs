//! Skin packs (`skins.json`): the one file a skin pack is described by. A skin pack is its own
//! pack type and never part of a world's stacks, so there is no stack accessor:
//! `SkinPack::parse` reads the file, `SkinPack::load` reads it as the game loads the pack,
//! against the pack's other files. Unversioned.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod read;
mod types;
mod write;

use std::collections::HashSet;
use std::fmt::Write as _;

use bedrock_addon_core::pack_identity::{PackPath, PackSide};

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, VanillaText,
};
use bedrock_addon_core::document::{
    Content, Document, JsonContent, Origin, Placed, Rejected, unversioned,
};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle, write_json};
use bedrock_addon_core::load::LoadContext;
use read::read_json;

pub use types::{Skin, SkinPack};

/// The path of the skin list, in a skin pack.
pub const SKINS_PATH: &str = "skins.json";
/// The path of a skin pack's custom models.
pub const GEOMETRY_PATH: &str = "geometry.json";
/// The two models the game has for a classic skin; any other `geometry` names a model of the
/// pack's `geometry.json`.
pub const BUILT_IN_GEOMETRIES: [&str; 2] =
    ["geometry.humanoid.custom", "geometry.humanoid.customSlim"];

// The loader's texts.
/// A pack without `skins.json`.
const NOT_FOUND: &str = "Failed to load skinpack, skins.json not found";
/// A `skins.json` that is not JSON.
const NOT_JSON: &str = "Failed to load skinpack, failed to parse manifest";
/// A `skins.json` whose root is no object; the literal `/n` and the two spaces are the
/// game's. The arguments are the pack's name, author and UUID.
const NOT_OBJECT: &str = "Skin manifest failed to load.  Pack Name: {} /nAuthor: {}, /nUUID: {}";
/// The start of the content-log line for a skin whose files are missing.
const INVALID_SKIN: &str = "Trying to add an invalid classic skin.{}";

impl bedrock_addon_core::__private::Sealed for SkinPack {}

fn located(mut diagnostics: Diagnostics, origin: &Origin) -> Diagnostics {
    let found = std::mem::take(&mut diagnostics);
    for mut finding in found {
        finding.location.path = Some(origin.path.clone());
        diagnostics.push(finding);
    }
    diagnostics
}

fn refuse(origin: &Origin, code: DiagnosticCode) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::error(code));
    Rejected::new(origin.clone(), located(diagnostics, origin))
}

impl Content for SkinPack {
    const NAME: &'static str = "skin pack";
    // A skin pack is on neither stack.
    const SIDES: &'static [PackSide] = &[];

    /// Reads one `skins.json` on its own. A file that is not JSON, and one whose root is no
    /// object, is rejected: the game rejects the pack. Every skin entry that is an object is
    /// held, whether or not the pack has its files ([`SkinPack::load`] checks those).
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_placed(path, bytes).map(|(document, _)| document)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&self.to_value(), out, style)
    }
}

impl JsonContent for SkinPack {}

/// `SkinPack::parse`, with the place in the file of each skin held.
fn parse_placed(
    path: &PackPath,
    bytes: &[u8],
) -> Result<(Document<SkinPack>, Vec<JsonPath>), Rejected> {
    // A skin pack is on neither stack; the resource side is the nearer of the two.
    let origin = unversioned::lone(path);
    let mut root = match read_json(bytes) {
        Ok(root) => root,
        Err(error) => {
            let code = DiagnosticCode::MalformedInput {
                message: error.to_string(),
                text: Some(VanillaText::Fixed(NOT_JSON)),
            };
            return Err(refuse(&origin, code));
        }
    };
    if !root.is_object() {
        // The pack's name, author and UUID are the manifest's, which a file read on its own does
        // not have.
        let code = DiagnosticCode::InvalidType {
            text: Some(VanillaText::Formatted {
                template: NOT_OBJECT,
                args: vec![String::new(), String::new(), String::new()],
            }),
        };
        return Err(refuse(&origin, code));
    }
    let mut diagnostics = Diagnostics::default();
    // A number no double holds is read as the largest finite one, with a warning.
    bedrock_addon_core::json::clamp_non_finite_warned(
        &mut root,
        None,
        &JsonPath::new(),
        &mut diagnostics,
    );
    let (pack, places) = SkinPack::from_value_placed(&root, &mut diagnostics);
    let version = unversioned::version();
    let diagnostics = located(diagnostics, &origin);
    Ok((Document::new(pack, origin, version, diagnostics), places))
}

impl Placed for SkinPack {
    /// `skins.json`.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(SKINS_PATH)
    }
}

impl SkinPack {
    /// Loads a skin pack as the game does: `skins_json` is the bytes of the pack's
    /// `skins.json` (`None` when the pack has none), `geometry_json` those of its
    /// `geometry.json`, and `paths` names every file of the pack.
    ///
    /// A skin whose `texture` is no file of the pack, or whose `cape` is given and is none, is
    /// left out, with the game's content-log line as a kept error; a skin whose `geometry` is
    /// neither built-in nor a model of `geometry.json` stays, with an information. A
    /// `geometry.json` that is not JSON is named once, with an information at that file: the
    /// pack loads without custom geometry, and no skin is blamed for a name nobody could look
    /// up.
    ///
    /// # Errors
    ///
    /// The pack is rejected when it has no `skins.json`, when the file is not JSON and when
    /// its root is no object.
    pub fn load<'a>(
        skins_json: Option<&[u8]>,
        geometry_json: Option<&[u8]>,
        paths: impl IntoIterator<Item = &'a str>,
        _cx: &LoadContext,
    ) -> Result<Document<SkinPack>, Rejected> {
        let path = PackPath::from_static(SKINS_PATH);
        let Some(bytes) = skins_json else {
            let code = DiagnosticCode::MissingRequiredField {
                text: Some(VanillaText::Fixed(NOT_FOUND)),
            };
            return Err(refuse(&unversioned::lone(&path), code));
        };
        let (document, places) = parse_placed(&path, bytes)?;
        let files: HashSet<&str> = paths.into_iter().collect();
        // The models of `geometry.json`: the members of its root. `Err` for a file that is
        // not JSON: no rule rejects the pack for it, so it loads without custom geometry.
        let models = geometry_json.map(read_json).transpose();
        let has_model = |name: &str| match &models {
            Ok(models) => models
                .as_ref()
                .and_then(Value::as_object)
                .is_some_and(|models| models.contains_key(name)),
            // Nobody could look the name up: the broken file is what is reported.
            Err(_) => true,
        };
        let mut diagnostics = document.diagnostics().clone();
        let origin = document.origin().clone();
        let version = document.version().clone();
        let mut pack = document.into_value();
        let mut found = Diagnostics::default();
        if let Some(skins) = pack.skins.take() {
            let mut kept = Vec::with_capacity(skins.len());
            for (skin, place) in skins.into_iter().zip(places) {
                // The entry's place in the file.
                let mut at = place;
                let name = skin.name();
                let mut missing = String::new();
                if !files.contains(skin.texture_path().as_ref()) {
                    let _ = write!(missing, " Skin asset is missing from {name}.");
                }
                if skin
                    .cape_path()
                    .is_some_and(|cape| !files.contains(cape.as_ref()))
                {
                    let _ = write!(missing, " Cape is missing from {name}.");
                }
                if !missing.is_empty() {
                    let code = DiagnosticCode::InvalidValue {
                        text: Some(VanillaText::Formatted {
                            template: INVALID_SKIN,
                            args: vec![missing],
                        }),
                    };
                    // The game logs the line at the content log's error level and loads the next
                    // skin: the skin is dropped while the pack is kept, a kept error.
                    found.push(Diagnostic::error(code).at(at).into_kept());
                    continue;
                }
                if !skin.uses_built_in_geometry() && !has_model(&skin.geometry_name()) {
                    // The game falls back without a line, so the crate reports an information.
                    at.push("geometry");
                    let code = DiagnosticCode::InvalidValue { text: None };
                    found.push(Diagnostic::info(code).at(at));
                }
                drop(name);
                kept.push(skin);
            }
            pack.skins = Some(kept);
        }
        diagnostics.extend(located(found, &origin));
        if let Err(message) = models {
            let code = DiagnosticCode::MalformedInput {
                message: message.to_string(),
                text: None,
            };
            // The game reads the geometry file and tests the parse result only for null, with no
            // content-log line and no assert, so the game is silent and the crate reports an
            // information.
            let mut broken = Diagnostic::info(code);
            broken.location.path = Some(PackPath::from_static(GEOMETRY_PATH));
            diagnostics.push(broken);
        }
        Ok(Document::new(pack, origin, version, diagnostics))
    }
}

fn creator_rule(bytes: &[u8], sizes: &[(u32, u32)]) -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    if bedrock_addon_core::shared::png_size(bytes).is_some_and(|size| !sizes.contains(&size)) {
        diagnostics.push(Diagnostic::info(DiagnosticCode::SkinPackCreatorRule));
    }
    diagnostics
}

/// Checks a skin image given its bytes: creator tools want 64 by 64 or 128 by 128 pixels
/// (a `SkinPackCreatorRule` info; the game checks no size). An image that is no PNG is not
/// judged.
#[must_use]
pub fn check_texture(bytes: &[u8]) -> Diagnostics {
    creator_rule(bytes, &[(64, 64), (128, 128)])
}

/// Checks a cape image given its bytes: creator tools want 64 by 32 pixels
/// (a `SkinPackCreatorRule` info).
#[must_use]
pub fn check_cape(bytes: &[u8]) -> Diagnostics {
    creator_rule(bytes, &[(64, 32)])
}
