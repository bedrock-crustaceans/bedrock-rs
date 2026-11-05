//! Loading fonts: `Content` of the two JSON files, the walk of `font/`, and the group the
//! stack makes of them.

use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use indexmap::IndexMap;

use super::stack::Index;
use super::{
    EMOTICONS_PATH, EmoticonList, FONT_METADATA_PATH, FontFile, FontMetadata, Fonts, emoticons,
    metadata,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Severity, VanillaText,
};
use bedrock_addon_core::document::{Content, Document, Merge, Merged, Origin, Placed, Rejected};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::Reported;
use bedrock_addon_support::fixed_path::{
    Authored, FileCopy, document, finding, json_string, json_text, lone, malformed, origin, reject,
};
use bedrock_addon_support::jsoncpp::read;

impl bedrock_addon_core::__private::Sealed for FontMetadata {}
impl bedrock_addon_core::__private::Sealed for EmoticonList {}

/// Keyed by a font's or an alias's name: `Merged::origin` names the pack whose file is read.
impl Merge for FontMetadata {
    type Key = String;
}

/// Keyed by an emoticon's name.
impl Merge for EmoticonList {
    type Key = String;
}

/// Reads one `font/font_metadata.json`.
fn read_metadata(origin: Origin, bytes: &[u8]) -> FileCopy<FontMetadata> {
    let mut root = match read(bytes) {
        Ok(root) => root,
        Err(message) => {
            // The game keeps its default fonts and writes its line to the developer log, not the
            // content log: an information with its text, the file still refused.
            let code = malformed(message.to_string(), Some(metadata::NOT_JSON));
            return Err(reject(
                origin,
                finding(code, Severity::Info, JsonPath::new()),
            ));
        }
    };
    if !FontMetadata::version_is_supported(&root) {
        // The file is ignored; the game's line goes to the developer log, so this is an
        // information, and the file is still refused.
        let code = DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::Fixed(metadata::VERSION)),
        };
        let mut at = JsonPath::new();
        at.push("version");
        return Err(reject(origin, finding(code, Severity::Info, at)));
    }
    let mut diagnostics = Diagnostics::default();
    // A number no double holds is read as the largest finite one, with a warning.
    let mut beyond = Diagnostics::default();
    bedrock_addon_core::json::clamp_non_finite_warned(
        &mut root,
        None,
        &JsonPath::new(),
        &mut beyond,
    );
    let metadata = FontMetadata::from_value(&root, &mut diagnostics);
    // A scale is held at the width of a float and says so itself at the number.
    for finding in beyond {
        let at = &finding.location.json_path;
        let is_scale = matches!(
            at.segments().last(),
            Some(JsonPathSegment::Key(
                "font_scale" | "font_scale_factor_override"
            ))
        );
        let said = is_scale
            && diagnostics.iter().any(|own| {
                matches!(own.code, DiagnosticCode::ClampedValue { .. })
                    && own.location.json_path == *at
            });
        if !said {
            diagnostics.push(finding);
        }
    }
    Ok(document(metadata, origin, diagnostics))
}

/// Reads one `font/emoticons.json`.
fn read_emoticons(origin: Origin, bytes: &[u8]) -> FileCopy<EmoticonList> {
    let mut root = match read(bytes) {
        Ok(root) => root,
        Err(message) => {
            // No emoticons; the game's "Malformed JSON" is a developer assert: an information
            // with its text, the file still refused.
            let code = malformed(message.to_string(), Some(emoticons::MALFORMED));
            return Err(reject(
                origin,
                finding(code, Severity::Info, JsonPath::new()),
            ));
        }
    };
    let mut diagnostics = Diagnostics::default();
    // A number no double holds is read as the largest finite one, with a warning.
    bedrock_addon_core::json::clamp_non_finite_warned(
        &mut root,
        None,
        &JsonPath::new(),
        &mut diagnostics,
    );
    let list = EmoticonList::from_value(&root, &mut diagnostics);
    Ok(document(list, origin, diagnostics))
}

impl Content for FontMetadata {
    const NAME: &'static str = "font metadata";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `font/font_metadata.json`. A file that is not JSON, and one whose `version`
    /// is not the integer 1 (`1.0` and `"1"` are not), is rejected: the game ignores it.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        read_metadata(lone(path), bytes)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        json_text(&self.to_value(), out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        json_string(&self.to_value())
    }
}

impl Placed for FontMetadata {
    /// `font/font_metadata.json`.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(FONT_METADATA_PATH)
    }
}

impl Content for EmoticonList {
    const NAME: &'static str = "emoticon list";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `font/emoticons.json`. Only a file that is not JSON is rejected, with an
    /// error; a root that is no array is a list without emoticons, with an information.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        read_emoticons(lone(path), bytes)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        json_text(&self.to_value(), out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        json_string(&self.to_value())
    }
}

impl Placed for EmoticonList {
    /// `font/emoticons.json`.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(EMOTICONS_PATH)
    }
}

/// One of the two JSON files of `font/`, read.
#[derive(Debug)]
pub enum FontCopy {
    /// `font/font_metadata.json`.
    Metadata(FileCopy<FontMetadata>),
    /// `font/emoticons.json`.
    Emoticons(FileCopy<EmoticonList>),
}

fn is_font_json(path: &PackPath) -> bool {
    matches!(path.as_str(), FONT_METADATA_PATH | EMOTICONS_PATH)
}

fn parse_font(file: &WalkFile, _: &LoadContext) -> FontCopy {
    let origin = origin(file.pack.clone(), file.side, &file.path);
    if file.key.as_str() == FONT_METADATA_PATH {
        FontCopy::Metadata(read_metadata(origin, &file.bytes))
    } else {
        FontCopy::Emoticons(read_emoticons(origin, &file.bytes))
    }
}

/// The highest copy of a file read whole: its value, the origin of each of its `names`, every
/// copy that reads as a layer, and the highest copy's findings alone. A highest copy that is
/// refused leaves the default value: the game does not turn to a lower pack's file.
fn highest<T: Merge<Key = String> + Clone + Default>(
    copies: Vec<FileCopy<T>>,
    names: impl Fn(&T) -> Vec<String>,
) -> Merged<T> {
    let mut layers = Vec::new();
    let mut diagnostics = Diagnostics::default();
    let mut origins = IndexMap::new();
    let mut value = T::default();
    let count = copies.len();
    for (place, copy) in copies.into_iter().enumerate() {
        let top = place + 1 == count;
        match copy {
            Ok(layer) => {
                if top {
                    diagnostics.extend(layer.diagnostics().iter().cloned());
                    value = layer.value().clone();
                    for name in names(&value) {
                        origins.insert(name, layer.origin().clone());
                    }
                }
                layers.push(layer);
            }
            Err(rejected) if top => diagnostics.extend(rejected.diagnostics),
            Err(_) => {}
        }
    }
    layers.reverse();
    Merged::new(value, origins, layers, diagnostics)
}

/// The `Combine` job of `rp.fonts()`.
fn combine(parsed: Vec<FontCopy>, cx: &StackCx<'_>) -> Fonts {
    let mut metadata_copies = Vec::new();
    let mut emoticon_copies = Vec::new();
    // Where the highest font list lies, read or refused.
    let mut metadata_file = None;
    for copy in parsed {
        match copy {
            FontCopy::Metadata(copy) => {
                let origin = match &copy {
                    Ok(layer) => layer.origin(),
                    Err(rejected) => &rejected.origin,
                };
                metadata_file = Some(FontFile::new(&origin.pack, &origin.path));
                metadata_copies.push(copy);
            }
            FontCopy::Emoticons(copy) => emoticon_copies.push(copy),
        }
    }
    let metadata = highest(metadata_copies, |metadata: &FontMetadata| {
        let fonts = metadata.fonts.iter().flatten().map(|f| f.font_name.clone());
        let aliases = metadata.font_aliases.iter().flatten();
        fonts.chain(aliases.map(|a| a.alias.clone())).collect()
    });
    let emoticons = highest(emoticon_copies, |list: &EmoticonList| {
        list.emoticons
            .iter()
            .map(|emoticon| emoticon.name.clone())
            .collect()
    });
    let index = Index::build(cx.packs, metadata.value(), metadata_file.as_ref());
    Fonts {
        metadata,
        emoticons,
        files: index.files,
        pages: index.pages,
        diagnostics: index.diagnostics,
    }
}

impl Reported for Fonts {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

/// The loader of `rp.fonts()`: `font/font_metadata.json` and `font/emoticons.json` of every
/// resource pack. The other font files are known from each pack's listing; none is read.
pub const fn load_fonts() -> CollectionLoader<Fonts, FontCopy> {
    CollectionLoader {
        walk: Walk {
            directories: &["font"],
            depth: ListDepth::Children,
            accept: Accept::Fn(is_font_json),
            order: WalkOrder::BottomUp,
            directory_major: false,
            app_data: &[],
            app_data_place: AppDataPlace::After,
            in_pack: None,
            shadowed: None,
            read_error: None,
        },
        parse: parse_font,
        combine,
        needs: &[],
        then: None,
    }
}
