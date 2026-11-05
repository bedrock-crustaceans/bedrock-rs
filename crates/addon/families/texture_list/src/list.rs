//! Texture lists: `textures/textures_list.json`, the textures a pack asks the client to
//! preload. Vanilla validates nothing here; the client keeps whatever its JSON library made of
//! the file, and falls back to scanning `textures/` when that is empty.

use std::collections::{BTreeMap, HashSet};

use indexmap::{IndexMap, IndexSet};

use super::texts;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{Content, Document, JsonContent, Placed, Rejected};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle, strip_bom, write_json};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_support::texture::document::{detached, unversioned};
use bedrock_addon_support::texture::files::{PackFiles, TEXTURE_SET, VERBATIM_OR_IMAGE};
use bedrock_addon_support::texture::image::ImageFormat;
use bedrock_addon_support::texture::jsoncpp::{root_end, skip_trivia, string_end};

/// The file's path in a resource pack.
pub(crate) const PATH: &str = "textures/textures_list.json";
/// The name several shipped packs use, which no client reads.
pub(super) const MISNAMED: &str = "textures/texture_list.json";
/// The directory a pack's textures live in.
const TEXTURES: &str = "textures/";

/// What the root of a texture list file is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextureListRoot {
    /// The pack has no list.
    #[default]
    Missing,
    /// The file is empty, or its root is unreadable.
    Invalid,
    /// `null`.
    Null,
    /// An array: its string elements are the entries.
    Array,
    /// An object: the string values of its members are the entries, in byte order of the keys.
    Object,
    /// A string, a number or a bool: no entry, and no scan either.
    Scalar,
}

/// The `textures/textures_list.json` of one pack or subpack.
#[doc(alias = "textures_list.json")]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextureList {
    /// What the root is.
    pub root: TextureListRoot,
    /// The string entries, verbatim, in document order (for an object root, the member values
    /// in byte order of their keys, the last of a repeated key).
    pub entries: Vec<String>,
    /// The number of elements that are no strings; the client skips them.
    pub skipped_non_strings: usize,
    /// Whether the text broke off inside the root: the client then holds one more, empty,
    /// element, so even a list without an entry is no empty list.
    pub partial: bool,
}

/// What the client does with a pack's list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextureListMode {
    /// The list is empty: the client scans the pack's `textures/` directory.
    Scan,
    /// The list is not empty: the client preloads these paths and scans nothing. The entries
    /// may be none (a scalar root).
    List {
        /// The paths, verbatim.
        entries: Vec<String>,
    },
}

/// One top-level element of a list, as the scan of the text finds it.
enum Element {
    Text(String),
    Other,
}

/// What the scan of a list's text finds.
struct Scanned {
    root: TextureListRoot,
    elements: Vec<Element>,
    /// The text is no strict JSON.
    malformed: bool,
    /// The text broke off inside the root.
    partial: bool,
}

/// The value that starts at `at`: its end, and its text when it is a string.
fn value_at(text: &str, at: usize) -> Option<(usize, Element)> {
    let rest = text.get(at..)?;
    let end = at + root_end(rest)?;
    let slice = text.get(at..end)?;
    if slice.starts_with('"') {
        let decoded = bedrock_addon_core::json::from_str::<String>(slice).ok()?;
        return Some((end, Element::Text(decoded)));
    }
    bedrock_addon_core::json::from_str_jsonc::<Value>(slice).ok()?;
    Some((end, Element::Other))
}

/// Reads `text` the way the client's JSON library leaves it: the top-level elements before
/// the first error stay.
///
/// The reader keeps what it read, and the failed element is appended as `null`.
fn scan(text: &str) -> Scanned {
    let bytes = text.as_bytes();
    let start = skip_trivia(bytes, 0);
    let mut scanned = Scanned {
        root: TextureListRoot::Invalid,
        elements: Vec::new(),
        malformed: true,
        partial: false,
    };
    let (close, object) = match bytes.get(start) {
        Some(b'[') => (b']', false),
        Some(b'{') => (b'}', true),
        _ => {
            // A scalar root, `null`, or nothing readable.
            if let Some((end, element)) = value_at(text, start) {
                let literal = text.get(start..end).unwrap_or_default();
                scanned.root = match element {
                    Element::Other if literal == "null" => TextureListRoot::Null,
                    _ => TextureListRoot::Scalar,
                };
                scanned.malformed = skip_trivia(bytes, end) < bytes.len();
            }
            return scanned;
        }
    };
    scanned.root = if object {
        TextureListRoot::Object
    } else {
        TextureListRoot::Array
    };
    let mut members: BTreeMap<Vec<u8>, Element> = BTreeMap::new();
    let mut at = skip_trivia(bytes, start + 1);
    let mut first = true;
    let closed = loop {
        if bytes.get(at) == Some(&close) && first {
            break Some(at + 1);
        }
        let mut key = None;
        if object {
            // `"key" :`
            let Some(end) = (bytes.get(at) == Some(&b'"'))
                .then(|| string_end(bytes, at))
                .flatten()
            else {
                break None;
            };
            let name = text.get(at..end).unwrap_or_default();
            let Ok(name) = bedrock_addon_core::json::from_str::<String>(name) else {
                break None;
            };
            at = skip_trivia(bytes, end);
            if bytes.get(at) != Some(&b':') {
                // The member is there, with a `null` value.
                members.insert(name.into_bytes(), Element::Other);
                break None;
            }
            at = skip_trivia(bytes, at + 1);
            key = Some(name.into_bytes());
        }
        let Some((end, element)) = value_at(text, at) else {
            if let Some(key) = key {
                members.insert(key, Element::Other);
            }
            break None;
        };
        match key {
            // Of a repeated key the last value stands.
            Some(key) => {
                members.insert(key, element);
            }
            None => scanned.elements.push(element),
        }
        first = false;
        at = skip_trivia(bytes, end);
        match bytes.get(at) {
            Some(b',') => at = skip_trivia(bytes, at + 1),
            Some(byte) if *byte == close => break Some(at + 1),
            _ => break None,
        }
    };
    if object {
        // `std::map` order: bytewise by key.
        scanned.elements.extend(members.into_values());
    }
    match closed {
        Some(end) => scanned.malformed = skip_trivia(bytes, end) < bytes.len(),
        None => scanned.partial = true,
    }
    scanned
}

/// The path of element `index`.
fn at_index(index: usize) -> JsonPath {
    let mut at = JsonPath::new();
    at.push_index(index);
    at
}

/// A finding of the crate's own about a value: vanilla reads it silently.
fn invalid() -> DiagnosticCode {
    DiagnosticCode::InvalidValue { text: None }
}

/// Whether `entry` ends in an image or texture-set extension, in any case.
fn has_extension(entry: &str) -> bool {
    let lower = entry.to_ascii_lowercase();
    lower.ends_with(TEXTURE_SET) || ImageFormat::from_path(&lower).is_some()
}

/// Whether `entry` is no normalised pack-relative path.
fn unnormalised(entry: &str) -> bool {
    entry.contains('\\')
        || entry.starts_with('/')
        || entry.starts_with("./")
        || entry.contains("//")
        || entry.trim() != entry
}

impl TextureList {
    /// Reads a list from the bytes of its file (`None` when the pack has none), as the client
    /// does: a BOM is skipped, comments are allowed, any root is accepted, and of a text that
    /// breaks off the elements read so far stay. Never fails; `out` gets the lint findings.
    ///
    /// Every finding is the crate's own and an information: vanilla logs nothing.
    #[must_use]
    pub fn read(bytes: Option<&[u8]>, out: &mut Diagnostics) -> TextureList {
        let Some(bytes) = bytes else {
            return TextureList::default();
        };
        let text = String::from_utf8_lossy(strip_bom(bytes));
        let scanned = scan(&text);
        let mut list = TextureList {
            root: scanned.root,
            partial: scanned.partial,
            ..TextureList::default()
        };
        if scanned.malformed || scanned.partial {
            // Clients keep the entries read before the error.
            let code = DiagnosticCode::MalformedInput {
                message: "not valid JSON; clients keep only the entries read before the error"
                    .to_owned(),
                text: None,
            };
            out.push(Diagnostic::info(code).phase(Phase::Validate));
        }
        match list.root {
            // No preload and no scan.
            TextureListRoot::Scalar => {
                out.push(Diagnostic::info(invalid()).phase(Phase::Validate));
            }
            // The values are the list.
            TextureListRoot::Object if !scanned.elements.is_empty() => {
                out.push(Diagnostic::info(invalid()).phase(Phase::Validate));
            }
            _ => {}
        }
        for (index, element) in scanned.elements.into_iter().enumerate() {
            match element {
                Element::Text(entry) => list.entries.push(entry),
                Element::Other => {
                    list.skipped_non_strings += 1;
                    // An element that is no string, for an array's own elements.
                    if list.root == TextureListRoot::Array {
                        out.push(
                            Diagnostic::info(invalid())
                                .at(at_index(index))
                                .phase(Phase::Validate),
                        );
                    }
                }
            }
        }
        // An object's member that broke off is no element of the document.
        if list.root == TextureListRoot::Object && list.partial {
            list.skipped_non_strings = 0;
        }
        list.lint_entries(out);
        list
    }

    /// The findings about the entries themselves.
    fn lint_entries(&self, out: &mut Diagnostics) {
        let mut counts: IndexMap<&str, usize> = IndexMap::new();
        for (index, entry) in self.entries.iter().enumerate() {
            *counts.entry(entry).or_default() += 1;
            // Once per distinct entry.
            if counts.get(entry.as_str()) != Some(&1) {
                continue;
            }
            if has_extension(entry) {
                out.push(
                    Diagnostic::info(invalid())
                        .at(at_index(index))
                        .phase(Phase::Validate),
                );
            }
            // An entry that is not normalised, or one outside `textures/`: one finding for both,
            // the crate's own.
            if unnormalised(entry) || !entry.starts_with(TEXTURES) {
                out.push(
                    Diagnostic::info(invalid())
                        .at(at_index(index))
                        .phase(Phase::Validate),
                );
            }
        }
        let mut seen = HashSet::new();
        for (index, entry) in self.entries.iter().enumerate() {
            // At the first of the equal entries.
            if seen.insert(entry.as_str()) && counts.get(entry.as_str()).is_some_and(|n| *n > 1) {
                let code = DiagnosticCode::DuplicateIdentifier { text: None };
                out.push(
                    Diagnostic::info(code)
                        .at(at_index(index))
                        .phase(Phase::Validate),
                );
            }
        }
    }

    /// Whether the client sees an empty list, and so scans `textures/`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self.root {
            TextureListRoot::Missing | TextureListRoot::Invalid | TextureListRoot::Null => true,
            TextureListRoot::Scalar => false,
            TextureListRoot::Array | TextureListRoot::Object => {
                self.entries.is_empty() && self.skipped_non_strings == 0 && !self.partial
            }
        }
    }

    /// What the client does with a pack whose own list is this one and whose active subpack's
    /// list is `subpack`.
    ///
    /// The subpack's elements are appended to the base's. When the subpack contributes any, a
    /// scalar base is replaced by them, and the entries of an object base are dropped.
    #[must_use]
    pub fn mode(&self, subpack: Option<&TextureList>) -> TextureListMode {
        let appended = subpack.filter(|subpack| !subpack.is_empty());
        let Some(subpack) = appended else {
            return if self.is_empty() {
                TextureListMode::Scan
            } else {
                TextureListMode::List {
                    entries: self.entries.clone(),
                }
            };
        };
        let mut entries = match self.root {
            TextureListRoot::Array => self.entries.clone(),
            _ => Vec::new(),
        };
        entries.extend(subpack.entries.iter().cloned());
        TextureListMode::List { entries }
    }

    /// The list a pack's own images make, as vanilla's developer tool writes it, but sorted
    /// and without repeats: of `paths`, the paths of the pack's files, every image below
    /// `textures/` without its extension. `None` when the pack has no image (vanilla then
    /// writes no file).
    #[must_use]
    pub fn generate<'a>(paths: impl IntoIterator<Item = &'a str>) -> Option<String> {
        let mut entries: Vec<&str> = paths
            .into_iter()
            .filter(|path| path.starts_with(TEXTURES))
            .filter_map(image_stem)
            .collect();
        entries.sort_unstable();
        entries.dedup();
        if entries.is_empty() {
            return None;
        }
        let list = TextureList {
            root: TextureListRoot::Array,
            entries: entries.into_iter().map(str::to_owned).collect(),
            ..TextureList::default()
        };
        list.to_json().ok()
    }
}

/// The claim of an image file: its path without the last extension, when that is an image
/// extension in lower case.
fn image_stem(path: &str) -> Option<&str> {
    let format = ImageFormat::from_path(path)?;
    path.strip_suffix(format.extension())
}

/// The claim of a texture set file: a path that ends in `texture_set.json` (no dot is asked
/// for before it), without `.json` and then without `.texture_set`.
fn set_stem(path: &str) -> Option<&str> {
    if !path.ends_with("texture_set.json") {
        return None;
    }
    let stem = path.strip_suffix(".json")?;
    Some(stem.strip_suffix(".texture_set").unwrap_or(stem))
}

/// What the client's scan of a `textures/` directory claims: the texture sets first, then the
/// images, each in the order given, without the images that are a layer of one of the pack's
/// texture sets (`layers`: the layer paths, folder included).
#[must_use]
pub fn scan_claims<'a>(
    files: impl IntoIterator<Item = &'a str> + Clone,
    layers: &[String],
) -> Vec<String> {
    let below = |path: &&str| path.starts_with(TEXTURES);
    let layers: HashSet<&str> = layers.iter().map(String::as_str).collect();
    let sets = files.clone().into_iter().filter(below).filter_map(set_stem);
    let images = files
        .into_iter()
        .filter(below)
        .filter_map(image_stem)
        .filter(|stem| !layers.contains(stem));
    // Each claim once, at its first place.
    let claims: IndexSet<&str> = sets.chain(images).collect();
    claims.into_iter().map(str::to_owned).collect()
}

/// A texture path a pack of the stack claims for preloading.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TextureClaim {
    /// The path, as listed or as scanned.
    pub location: String,
    /// The highest pack that claims it.
    pub pack: PackRef,
    /// That pack's place in the stack; 0 is the top.
    pub stack_index: usize,
}

/// The findings of one pack's list against the pack's own files, and of the files themselves.
pub(crate) struct PackLint<'a> {
    /// The pack's files.
    pub(super) files: &'a PackFiles,
    /// The layer paths of the pack's texture sets.
    pub(super) layers: &'a [String],
    /// Whether the pack is a base-game pack.
    pub(super) base_game: bool,
}

impl PackLint<'_> {
    fn keys(&self) -> impl Iterator<Item = &str> + Clone + '_ {
        self.files.files.keys().map(String::as_str)
    }

    /// What a scan of the pack claims.
    pub(super) fn scan(&self) -> Vec<String> {
        scan_claims(self.keys(), self.layers)
    }

    /// Whether the pack has a file for `entry`: the path itself, an image or a texture set.
    fn has(&self, entry: &str) -> bool {
        VERBATIM_OR_IMAGE
            .iter()
            .chain([&TEXTURE_SET])
            .any(|extension| {
                self.files
                    .files
                    .contains_key(&format!("{entry}{extension}"))
            })
    }

    /// Reports the entries of `list` the pack has no file for.
    pub(super) fn stale(&self, list: &TextureList, out: &mut Diagnostics) {
        if list.is_empty() {
            return;
        }
        let mut listed = HashSet::new();
        for (index, entry) in list.entries.iter().enumerate() {
            // Once per distinct entry, at its first place.
            if listed.insert(entry.as_str()) && !self.has(entry) {
                // An information: vanilla validates nothing, and another pack may have it.
                let code = DiagnosticCode::TextureNotFound { text: None };
                out.push(
                    Diagnostic::info(code)
                        .at(at_index(index))
                        .phase(Phase::Validate),
                );
            }
        }
    }

    /// Reports whether `entries`, the list the client uses for the pack (its own list with the
    /// active subpack's appended), omits images of the pack.
    pub(super) fn omitted(&self, entries: &[String], out: &mut Diagnostics) {
        let listed: HashSet<&str> = entries.iter().map(String::as_str).collect();
        let omitted = self
            .scan()
            .into_iter()
            .any(|claim| !listed.contains(claim.as_str()));
        if omitted {
            out.push(Diagnostic::info(invalid()).phase(Phase::Validate));
        }
    }

    /// The findings about the pack's files, whatever its list. `empty` says
    /// whether the pack's list is empty.
    pub(super) fn pack(&self, empty: bool, out: &mut Diagnostics) {
        let at = |path: &str| {
            let path = PackPath::new(path).ok();
            Location::new(Some(self.files.pack.clone()), path, JsonPath::new(), None)
        };
        let has_textures = self.keys().any(|key| key.starts_with(TEXTURES));
        if self.base_game && empty && has_textures {
            // The game's own line, which goes to its developer log, not the content log: an
            // information with the game's text.
            let args = [PATH.to_owned(), self.files.pack.name().to_owned()];
            let code = DiagnosticCode::MissingRequiredField {
                text: bedrock_addon_support::texture::texts::formatted(
                    texts::NO_LIST_IN_BASE_PACK,
                    args,
                ),
            };
            let finding = Diagnostic::new(code, Severity::Info, Phase::Validate, at(PATH));
            out.push(finding);
        }
        let has = |path: &str| self.files.files.contains_key(path);
        if has(MISNAMED) && !has(PATH) {
            let code = DiagnosticCode::IgnoredFile { text: None };
            out.push(Diagnostic::new(
                code,
                Severity::Info,
                Phase::Validate,
                at(MISNAMED),
            ));
        }
        for key in self.keys().filter(|key| key.starts_with(TEXTURES)) {
            let upper = ImageFormat::from_path(key).is_none()
                && ImageFormat::from_path_ignoring_case(key).is_some();
            if upper {
                let code = DiagnosticCode::IgnoredFile { text: None };
                out.push(Diagnostic::new(
                    code,
                    Severity::Info,
                    Phase::Validate,
                    at(key),
                ));
            }
        }
    }
}

impl bedrock_addon_core::__private::Sealed for TextureList {}

impl Content for TextureList {
    const NAME: &'static str = "texture list";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `textures_list.json` ([`TextureList::read`]). Never rejects: the client
    /// takes whatever the file holds.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        let mut diagnostics = Diagnostics::default();
        let list = TextureList::read(Some(bytes), &mut diagnostics);
        Ok(unversioned(list, detached(path), diagnostics))
    }

    /// Writes the entries as a JSON array of strings.
    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&self.entries, out, style)
    }
}

impl JsonContent for TextureList {}

impl Placed for TextureList {
    /// `textures/textures_list.json`.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(PATH)
    }
}
