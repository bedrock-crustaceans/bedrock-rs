//! Loading a persona pack: which files make a piece, and each piece as the client builds it.

use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};

use bedrock_addon_core::pack_identity::{PackIdVersion, PackPath, PackRef, PackSide};
use bytes::Bytes;
use facet_value::VObject;

use super::piece::{EmoteAnimation, PersonaPiece, PieceAnimation, PieceGeometry};
use super::read::{read_meta, read_root, read_store_info, written_in};
use super::text::{self, unresolved, vanilla};
use super::{PackKind, PersonaPieces, PieceMetadata, PieceStoreInfo, PieceType};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Severity,
};
use bedrock_addon_core::document::{Document, Origin, Rejected, unversioned};
use bedrock_addon_core::json::Value;

/// The files of one pack, as [`PersonaPiece::load_all`] asks for them.
///
/// The module cannot name the crate's `Pack`; a pack is handed over through this trait
/// (`id` is `Pack::id`, `paths` is `Pack::paths`, `read` is `Pack::read`).
pub trait PieceSource {
    /// The pack's identity: its header UUID and version and, from its manifest's modules, its
    /// type.
    fn id(&self) -> &PackIdVersion;
    /// The pack's name, for findings.
    #[allow(
        clippy::unnecessary_literal_bound,
        reason = "an implementor returns its own name"
    )]
    fn name(&self) -> &str {
        ""
    }
    /// Every file of the pack, from the pack root, in any order.
    fn paths(&self) -> impl Iterator<Item = &PackPath> + '_;
    /// The bytes of one file. Only the JSON and `.lang` files the load parses are asked for:
    /// images are known by their paths.
    ///
    /// # Errors
    ///
    /// The kind of failure, when the file cannot be read.
    fn read(&self, path: &PackPath) -> Result<Bytes, std::io::ErrorKind>;
}

/// The model pack at the root of a persona pack, by its exact name.
const SHARED_GEOMETRY: &str = "geometry.json";

/// `findings` as the diagnostics of the file at `origin`.
fn located(findings: Vec<Diagnostic>, origin: &Origin) -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    for mut found in findings {
        if found.location.path.is_none() {
            found.location.path = Some(origin.path.clone());
        }
        if !origin.pack.is_app_data() {
            found.location.pack = Some(origin.pack.clone());
        }
        diagnostics.push(found);
    }
    diagnostics
}

/// Reads one file of the family on its own.
pub(super) fn parse_file<T>(
    origin: Origin,
    bytes: &[u8],
    refusal: fn(&PackPath, String) -> Diagnostic,
    read: fn(&Value, &mut Vec<Diagnostic>) -> T,
) -> Result<Document<T>, Rejected> {
    let mut findings = Vec::new();
    let root = match read_root(bytes, &mut findings) {
        Ok(root) if root.is_object() => root,
        Ok(_) => {
            findings.push(refusal(
                &origin.path,
                "the root is not an object".to_owned(),
            ));
            return Err(Rejected::new(origin.clone(), located(findings, &origin)));
        }
        Err(message) => {
            findings.push(refusal(&origin.path, message.to_string()));
            return Err(Rejected::new(origin.clone(), located(findings, &origin)));
        }
    };
    let value = read(&root, &mut findings);
    if findings
        .iter()
        .any(|found| found.severity == Severity::Error)
    {
        // The 1.26.40+ reader refused the file; the client logs the reader's findings and
        // the file's own line, and skips the piece.
        findings.push(refusal(
            &origin.path,
            "the 1.26.40 reader refused the file".to_owned(),
        ));
        return Err(Rejected::new(origin.clone(), located(findings, &origin)));
    }
    let diagnostics = located(findings, &origin);
    Ok(Document::new(
        value,
        origin,
        unversioned::version(),
        diagnostics,
    ))
}

/// A meta file that is not JSON, or whose root is no object.
pub(super) fn meta_not_parsed(path: &PackPath, message: String) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::MalformedInput {
        message,
        text: vanilla(text::META_NOT_PARSED, [path.as_str()]),
    })
}

/// A store-info file that is not JSON, or whose root is no object. The game ignores the
/// file in silence.
pub(super) fn store_info_not_parsed(_: &PackPath, message: String) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::MalformedInput {
        message,
        text: None,
    })
}

impl PersonaPiece {
    /// Loads the pieces of one pack as the client does.
    ///
    /// What happens depends on the pack ([`PackKind`]): the client's built-in packs hold many
    /// pieces, each identified by its `piece_id`; a `persona_piece` pack holds one, identified
    /// by the pack's UUID; in any other pack the piece files are dead content, reported and
    /// not read. A piece the client would skip is in [`PersonaPieces::rejected`]; everything
    /// the client skips in silence is a finding.
    ///
    /// The piece names are looked up in the pack's own `texts/*.lang` only: see
    /// [`load_all_with_texts`](Self::load_all_with_texts).
    #[must_use]
    pub fn load_all<S: PieceSource>(pack: &S) -> PersonaPieces {
        Self::load_all_with_texts(pack, &[])
    }

    /// [`load_all`](Self::load_all), with the bytes of further `.lang` files the client has
    /// when it shows the pieces (the vanilla resource pack's `texts/en_US.lang`): a piece name
    /// one of them translates is not reported as untranslated.
    #[must_use]
    pub fn load_all_with_texts<S: PieceSource>(pack: &S, texts: &[&[u8]]) -> PersonaPieces {
        let mut paths: Vec<&PackPath> = pack.paths().collect();
        // The client's walk has no stated order: by name, so that a load is repeatable.
        paths.sort_unstable();
        let index = index(&paths);
        let kind = PackKind::of(pack.id());
        let mut loaded = PersonaPieces {
            kind,
            pieces: Vec::new(),
            rejected: Vec::new(),
            shared_geometry: None,
            diagnostics: Diagnostics::default(),
        };
        let pack_ref = PackRef::new(0, pack.id().clone(), pack.name());
        // The findings about the pack and its folders; each names the pack.
        let mut of_pack: Vec<Diagnostic> = Vec::new();
        let named = |findings: Vec<Diagnostic>| {
            let mut diagnostics = Diagnostics::default();
            for mut found in findings {
                found.location.pack = Some(pack_ref.clone());
                diagnostics.push(found);
            }
            diagnostics
        };
        if kind == PackKind::Ordinary {
            // Only the built-in pack is searched for pieces among resource packs.
            for path in &index.piece_files {
                if !path.as_str().starts_with("models/") {
                    of_pack.push(at_file(
                        DiagnosticCode::IgnoredFile { text: None },
                        Severity::Info,
                        path,
                    ));
                }
            }
            loaded.diagnostics = named(of_pack);
            return loaded;
        }
        if index.files.contains(SHARED_GEOMETRY) {
            loaded.shared_geometry = Some(PackPath::from_static(SHARED_GEOMETRY));
        }
        let load = Load {
            pack,
            kind,
            pack_ref: pack_ref.clone(),
            index: &index,
            supplied: texts
                .iter()
                .flat_map(|bytes| text::lang_entries(bytes).into_keys())
                .collect(),
            names: OnceCell::new(),
        };
        // Count the folders that made a piece, and remember the second of them. A folder whose
        // meta file is refused made none and shares no identity.
        let mut made = 0usize;
        let mut second = None;
        for (directory, folder) in &index.folders {
            let Some(meta) = folder.metas.last() else {
                // The client makes no piece of a folder without a meta file, in silence: an
                // information of the crate's own.
                of_pack.push(at_file(
                    DiagnosticCode::MissingRequiredField { text: None },
                    Severity::Info,
                    &directory_path(directory),
                ));
                continue;
            };
            if folder.metas.len() > 1 {
                // A later meta file replaces the earlier in the folder's slot, in silence: an
                // information of the crate's own.
                of_pack.push(at_file(
                    DiagnosticCode::ShadowedDefinition { text: None },
                    Severity::Info,
                    &directory_path(directory),
                ));
            }
            match load.piece(directory, folder, meta) {
                Ok(piece) => {
                    made += 1;
                    if made == 2 {
                        second = Some(*directory);
                    }
                    loaded.pieces.push(piece);
                }
                Err(rejected) => loaded.rejected.push(rejected),
            }
        }
        if kind == PackKind::PersonaPiece
            && let Some(second) = second
        {
            // Every piece of such a pack gets the pack's UUID and the client keeps one. One
            // finding, at the first folder that is not the kept one. The game says nothing: an
            // information of the crate's own.
            of_pack.push(at_file(
                DiagnosticCode::IdentifierCollision { text: None },
                Severity::Info,
                &directory_path(second),
            ));
        }
        shared_identities_and_unused_texts(kind, &loaded.pieces, &index, &mut of_pack);
        loaded.diagnostics = named(of_pack);
        loaded
    }
}

/// The built-in pieces that share an id, and the `texts/` folders the client
/// does not use.
fn shared_identities_and_unused_texts(
    kind: PackKind,
    pieces: &[PersonaPiece],
    index: &Index<'_>,
    of_pack: &mut Vec<Diagnostic>,
) {
    if kind == PackKind::BuiltIn {
        // Of two pieces with one id the client keeps the first, in silence: an information of
        // the crate's own.
        let mut seen = HashSet::new();
        for piece in pieces {
            if !seen.insert(piece.id) {
                let mut found = at_file(
                    DiagnosticCode::IdentifierCollision { text: None },
                    Severity::Info,
                    &piece.meta.origin().path,
                );
                found.location.json_path.push("piece_id");
                of_pack.push(found);
            }
        }
    }
    // The client gives a piece its own texts only when the folder has `en_US.lang`.
    for (directory, folder) in &index.folders {
        let unused = !folder.metas.is_empty()
            && folder.texts.is_none()
            && index.with_texts.contains(directory);
        if unused {
            of_pack.push(at_file(
                DiagnosticCode::IgnoredFile { text: None },
                Severity::Info,
                &directory_path(&join(directory, "texts")),
            ));
        }
    }
}

/// A finding about the file (or folder) `path`.
fn at_file(code: DiagnosticCode, severity: Severity, path: &PackPath) -> Diagnostic {
    let mut found = Diagnostic::error(code).severity(severity);
    found.location.path = Some(path.clone());
    found
}

/// A folder's name as a pack path.
fn directory_path(directory: &str) -> PackPath {
    if directory.is_empty() {
        PackPath::ROOT
    } else {
        PackPath::new(directory).unwrap_or(PackPath::ROOT)
    }
}

/// The files of one folder that make a piece: the slots of the client's walk.
#[derive(Debug, Default)]
pub(super) struct Folder<'a> {
    /// Every `*.meta.json`, by name; the client reads the last.
    pub(super) metas: Vec<&'a PackPath>,
    /// The `*.geometry.json` (the last by name).
    pub(super) geometry: Option<&'a PackPath>,
    /// The `*.store_info.json` (the last by name).
    pub(super) store_info: Option<&'a PackPath>,
    /// `texts/en_US.lang` below the folder.
    pub(super) texts: Option<&'a PackPath>,
}

/// A pack's files, grouped as the client's walk groups them.
#[derive(Debug, Default)]
pub(super) struct Index<'a> {
    /// The folders with at least one slot filled, by name.
    pub(super) folders: std::collections::BTreeMap<&'a str, Folder<'a>>,
    /// Every file of the pack.
    pub(super) files: HashSet<&'a str>,
    /// Every meta, store-info and geometry file, in path order.
    pub(super) piece_files: Vec<&'a PackPath>,
    /// The folders that have a `texts/` folder with some `.lang` file.
    pub(super) with_texts: HashSet<&'a str>,
    /// The pack's own `texts/*.lang`.
    pub(super) names: Vec<&'a PackPath>,
    /// The pack's own `texts/en_US.lang`: the texts of a piece whose folder is the pack root.
    pub(super) root_texts: Option<&'a PackPath>,
}

/// The folder of a file and its name.
fn split(path: &str) -> (&str, &str) {
    path.rsplit_once('/').unwrap_or(("", path))
}

/// Groups `paths` (sorted) by the folder each file belongs to, by suffix alone and
/// case-sensitively, as the client does.
#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "the client compares the suffixes byte for byte"
)]
pub(super) fn index<'a>(paths: &[&'a PackPath]) -> Index<'a> {
    let mut index = Index::default();
    for path in paths {
        let text = path.as_str();
        index.files.insert(text);
        let (directory, name) = split(text);
        if name.ends_with(".meta.json") {
            index.folders.entry(directory).or_default().metas.push(path);
            index.piece_files.push(path);
        } else if name.ends_with(".geometry.json") {
            index.folders.entry(directory).or_default().geometry = Some(path);
            index.piece_files.push(path);
        } else if name.ends_with(".store_info.json") {
            index.folders.entry(directory).or_default().store_info = Some(path);
            index.piece_files.push(path);
        } else if name.ends_with(".lang") {
            let (parent, folder) = split(directory);
            if folder != "texts" {
                continue;
            }
            if parent.is_empty() {
                // The pack's own names: they make no folder a piece's, and are the
                // own texts of a piece at the pack root.
                index.names.push(path);
                if name == "en_US.lang" {
                    index.root_texts = Some(path);
                }
                continue;
            }
            index.with_texts.insert(parent);
            if name == "en_US.lang" {
                index.folders.entry(parent).or_default().texts = Some(path);
            }
        }
    }
    index
}

/// One pack being loaded.
struct Load<'a, S> {
    pack: &'a S,
    kind: PackKind,
    pack_ref: PackRef,
    index: &'a Index<'a>,
    /// The keys of the `.lang` files the caller supplied.
    supplied: HashSet<String>,
    /// The keys of the pack's own `texts/*.lang`, once a name was looked up.
    names: OnceCell<HashSet<String>>,
}

impl<S: PieceSource> Load<'_, S> {
    /// The piece of one folder, or the meta file the client skips.
    fn piece(
        &self,
        directory: &str,
        folder: &Folder<'_>,
        meta_path: &PackPath,
    ) -> Result<PersonaPiece, Rejected> {
        let origin = unversioned::origin(self.pack_ref.clone(), PackSide::Resource, meta_path);
        let bytes = match self.pack.read(meta_path) {
            Ok(bytes) => bytes,
            Err(kind) => {
                // A meta file that cannot be read: the client skips the piece.
                let found = Diagnostic::error(DiagnosticCode::ReadError {
                    kind,
                    text: vanilla(text::META_NOT_FOUND, [meta_path.as_str()]),
                });
                return Err(Rejected::new(origin.clone(), located(vec![found], &origin)));
            }
        };
        let meta = parse_file(origin.clone(), &bytes, meta_not_parsed, read_meta)?;
        let mut findings = Vec::new();
        let id = if self.kind == PackKind::BuiltIn {
            let Some(id) = meta.value().piece_uuid() else {
                // The client drops a built-in piece without an id of its own.
                let mut diagnostics = meta.diagnostics().clone();
                let found = Diagnostic::error(DiagnosticCode::InvalidValue { text: None })
                    .at(member("piece_id"));
                diagnostics.extend(located(vec![found], &origin));
                return Err(Rejected::new(origin, diagnostics));
            };
            id
        } else {
            // The pack's UUID identifies the piece. `piece_id` is required from 1.26.40, so
            // every loaded meta file has one and its being ignored is no finding.
            self.pack.id().uuid
        };
        let value = meta.value();
        missing_images(&self.index.files, directory, value, &mut findings);
        let geometry = self.geometry(directory, folder, value, &mut findings);
        let mut open = |path: &PackPath| {
            let bytes = self.pack.read(path)?;
            Ok(animation_file(&bytes))
        };
        let (animations, emote) = kept_animations(
            &self.index.files,
            directory,
            value,
            &id.to_string(),
            &mut findings,
            &mut open,
        );
        let store_info = self.store_info(folder, &mut findings);
        let own = if directory.is_empty() {
            // the slot's key is the grandparent of `texts/en_US.lang`.
            self.index.root_texts
        } else {
            folder.texts
        };
        let (texts, offer_title) = self.texts(own);
        let mut piece = PersonaPiece {
            directory: directory_path(directory),
            id,
            meta,
            store_info,
            geometry,
            animations,
            emote,
            texts,
            offer_title,
            findings: Diagnostics::default(),
        };
        // The name the client shows needs a translation, or the piece's own offer title.
        let key = piece.name_key();
        if piece.offer_title.is_none() && !self.is_named(&key) {
            findings.push(Diagnostic::info(DiagnosticCode::UnresolvedLangKey).detail(
                DiagnosticDetail::Reference {
                    kind: "translation",
                    id: key,
                    hint: None,
                },
            ));
        }
        piece.findings = located(findings, &origin);
        Ok(piece)
    }

    /// The piece's geometry file and the models its sources name.
    fn geometry(
        &self,
        directory: &str,
        folder: &Folder<'_>,
        meta: &PieceMetadata,
        findings: &mut Vec<Diagnostic>,
    ) -> Option<PieceGeometry> {
        // `Err`: the file is there and nobody can look a name up in it.
        let geometry = match folder.geometry {
            None => Ok(None),
            Some(path) => match self.pack.read(path) {
                Err(kind) => {
                    // The game logs the line at the content log's error level and leaves the
                    // geometry empty; the piece is kept: a kept error.
                    findings.push(
                        at_file(
                            DiagnosticCode::ReadError {
                                kind,
                                text: vanilla(text::GEOMETRY_NOT_FOUND, [path.as_str()]),
                            },
                            Severity::Error,
                            path,
                        )
                        .into_kept(),
                    );
                    Err(())
                }
                Ok(bytes) => {
                    let mut of_file = Vec::new();
                    let read = read_root(&bytes, &mut of_file);
                    findings.extend(of_file.into_iter().map(|mut found| {
                        found.location.path = Some(path.clone());
                        found
                    }));
                    match read {
                        Ok(mut json) => {
                            retarget_hip_parents(&mut json);
                            Ok(Some(PieceGeometry {
                                path: path.clone(),
                                json,
                            }))
                        }
                        Err(message) => {
                            // The game logs it at the content log's error level and leaves the
                            // geometry empty; the piece is kept: a kept error.
                            findings.push(
                                at_file(
                                    DiagnosticCode::MalformedInput {
                                        message: message.to_string(),
                                        text: vanilla(text::GEOMETRY_NOT_PARSED, [path.as_str()]),
                                    },
                                    Severity::Error,
                                    path,
                                )
                                .into_kept(),
                            );
                            Err(())
                        }
                    }
                }
            },
        };
        let Ok(geometry) = geometry else {
            return None;
        };
        let on = folder.geometry.map_or(directory, PackPath::as_str);
        missing_models(geometry.as_ref(), on, meta, findings);
        geometry
    }

    /// The piece's store-info, when it is a JSON object, and its thumbnail.
    fn store_info(
        &self,
        folder: &Folder<'_>,
        findings: &mut Vec<Diagnostic>,
    ) -> Option<Document<PieceStoreInfo>> {
        let path = folder.store_info?;
        let bytes = match self.pack.read(path) {
            Ok(bytes) => bytes,
            Err(kind) => {
                // The game ignores a store-info it cannot read, in silence, so an
                // information.
                findings.push(at_file(
                    DiagnosticCode::ReadError { kind, text: None },
                    Severity::Info,
                    path,
                ));
                return None;
            }
        };
        let origin = unversioned::origin(self.pack_ref.clone(), PackSide::Resource, path);
        match parse_file(origin, &bytes, store_info_not_parsed, read_store_info) {
            Ok(store) => {
                // The thumbnail is named from the pack root, with or without its extension.
                if let Some(thumbnail) = &store.value().piece_thumbnail
                    && !has_image(&self.index.files, thumbnail)
                {
                    // The game logs nothing about a missing thumbnail: an information of the
                    // crate's own.
                    let mut found =
                        unresolved("texture", thumbnail, None, member("piece_thumbnail"));
                    found.severity = Severity::Info;
                    found.location.path = Some(path.clone());
                    findings.push(found);
                }
                Some(store)
            }
            Err(rejected) => {
                // The file is refused and the piece loads without it. The client ignores it
                // in silence, so each finding is an information.
                for found in rejected.diagnostics {
                    let mut found = found;
                    found.severity = found.severity.min(Severity::Info);
                    findings.push(found);
                }
                None
            }
        }
    }

    /// The piece's own `texts/` folder and the offer title of its `en_US.lang`.
    fn texts(&self, own: Option<&PackPath>) -> (Option<PackPath>, Option<String>) {
        let Some(path) = own else {
            return (None, None);
        };
        let (texts, _) = split(path.as_str());
        let title = self
            .pack
            .read(path)
            .ok()
            .and_then(|bytes| text::lang_entries(&bytes).remove("persona.offer.title"));
        (Some(directory_path(texts)), title)
    }

    /// Whether some `.lang` file of the pack, or one the caller supplied, has `key`. The
    /// pack's files are read the first time a name is looked up.
    fn is_named(&self, key: &str) -> bool {
        if self.supplied.contains(key) {
            return true;
        }
        self.names
            .get_or_init(|| {
                let mut names = HashSet::new();
                for path in &self.index.names {
                    if let Ok(bytes) = self.pack.read(path) {
                        names.extend(text::lang_entries(&bytes).into_keys());
                    }
                }
                names
            })
            .contains(key)
    }
}

/// Whether `files` has a file `name`, as written or with one of the image loader's
/// extensions.
fn has_image(files: &HashSet<&str>, name: &str) -> bool {
    const EXTENSIONS: [&str; 5] = ["", ".tga", ".png", ".jpg", ".jpeg"];
    let mut path = String::with_capacity(name.len() + 5);
    EXTENSIONS.iter().any(|extension| {
        path.clear();
        path.push_str(name);
        path.push_str(extension);
        files.contains(path.as_str())
    })
}

/// The place of the member `key` of entry `index` of the list `list`.
fn entry(list: &str, index: usize, key: &str) -> JsonPath {
    member(list).index(index).child(key)
}

/// Reports every image the piece in `directory` names that is not one of `files`, in its
/// folder. The game has no content-log line for a missing one (its texture builder only
/// asserts "Source pack '%s' is missing Persona texture '%s'." in developer builds), so
/// each miss is an information of the crate's own.
pub(super) fn missing_images(
    files: &HashSet<&str>,
    directory: &str,
    meta: &PieceMetadata,
    findings: &mut Vec<Diagnostic>,
) {
    let mut check = |name: Option<&String>, at: JsonPath| {
        let Some(name) = name else {
            return;
        };
        if !has_image(files, &join(directory, name)) {
            let mut found = unresolved("texture", name, None, at);
            found.severity = Severity::Info;
            findings.push(found);
        }
    };
    check(meta.clothing_map.as_ref(), member("clothing_map"));
    check(meta.face_clothing_map.as_ref(), member("face_clothing_map"));
    for (index, source) in meta.texture_sources.iter().flatten().enumerate() {
        // The game reads nothing of an entry without a texture.
        if source.texture.as_deref().is_none_or(str::is_empty) {
            continue;
        }
        check(
            source.texture.as_ref(),
            entry("texture_sources", index, "texture"),
        );
        check(
            source.tint_map.as_ref(),
            entry("texture_sources", index, "tint_map"),
        );
    }
    for (index, source) in meta.geometry_sources.iter().flatten().enumerate() {
        if source.texture.as_deref().is_none_or(str::is_empty) {
            continue;
        }
        check(
            source.texture.as_ref(),
            entry("geometry_sources", index, "texture"),
        );
        check(
            source.tint_map.as_ref(),
            entry("geometry_sources", index, "tint_map"),
        );
    }
}

/// The client looks each source's `geometry` up among the root members of the piece's
/// geometry file; `on` is what the game's text says the name was looked up on.
pub(super) fn missing_models(
    geometry: Option<&PieceGeometry>,
    on: &str,
    meta: &PieceMetadata,
    findings: &mut Vec<Diagnostic>,
) {
    for (index, source) in meta.geometry_sources.iter().flatten().enumerate() {
        if source.geometry.is_none() && written_in(&source.extra, "geometry") {
            // A name of another JSON type has its own finding.
            continue;
        }
        let name = source.geometry.as_deref().unwrap_or_default();
        if geometry.is_none_or(|geometry| geometry.model(name).is_none()) {
            // The game logs the line at the content log's error level and goes on; the piece
            // is kept: a kept error.
            findings.push(
                unresolved(
                    "geometry",
                    name,
                    vanilla(text::GEOMETRY_NAME_NOT_FOUND, [name, on]),
                    entry("geometry_sources", index, "geometry"),
                )
                .into_kept(),
            );
        }
    }
}

/// What opening an animation file gives: its root when it is a JSON object with an
/// `animations` member, `None` when it is not, or the kind of failure when it cannot be read.
pub(super) type Opened = Result<Option<Value>, std::io::ErrorKind>;

/// The animations of `animation_sources` whose file holds them, and an emote's file.
/// `open` reads one animation file; it is called once per file, however many entries name
/// it.
///
/// Only an emote's first entry is told, as the game checks it: an emote with no entry has
/// an invalid animation definition; a file that is not there has its own line; a file
/// that does not hold the animation, whatever the reason (not JSON, no `animations`, the name
/// not a key), has the line after the animation load. Each is a
/// kept error: the piece stays. The other entries, and every entry of a piece of
/// another type, are not checked by the game and are not told.
pub(super) fn kept_animations(
    files: &HashSet<&str>,
    directory: &str,
    meta: &PieceMetadata,
    id: &str,
    findings: &mut Vec<Diagnostic>,
    open: &mut dyn FnMut(&PackPath) -> Opened,
) -> (Vec<PieceAnimation>, Option<EmoteAnimation>) {
    let emote_piece = meta.piece_type() == PieceType::Emote;
    let mut opened: HashMap<String, Option<Value>> = HashMap::new();
    let mut kept = Vec::new();
    let mut first_kept = false;
    for (index, source) in meta.animation_sources.iter().flatten().enumerate() {
        let told = emote_piece && index == 0;
        let name = source.name.as_deref().unwrap_or_default();
        let joined = join(
            directory,
            source.animation_file.as_deref().unwrap_or_default(),
        );
        let path = PackPath::new(&joined)
            .ok()
            .filter(|_| files.contains(joined.as_str()));
        let mut root = None;
        let mut missing = None;
        if let Some(path) = &path {
            if !opened.contains_key(&joined) {
                let read = open(path).unwrap_or_else(|kind| {
                    missing = Some(kind);
                    None
                });
                opened.insert(joined.clone(), read);
            }
            root = opened.get(&joined).and_then(Option::as_ref);
        } else {
            missing = Some(std::io::ErrorKind::NotFound);
        }
        let named = root
            .and_then(|root| root.as_object())
            .and_then(|root| root.get("animations"))
            .and_then(Value::as_object)
            .is_some_and(|animations| animations.contains_key(name));
        match (path, named) {
            (Some(path), true) => {
                first_kept |= index == 0;
                kept.push(PieceAnimation {
                    name: name.to_owned(),
                    stored_name: format!("{name}.{id}"),
                    file: path,
                });
            }
            _ if !told => {}
            _ if missing.is_some() => {
                // The game's error-level line for a missing animation file; the piece is kept.
                findings.push(
                    Diagnostic::error(DiagnosticCode::ReadError {
                        kind: missing.unwrap_or(std::io::ErrorKind::NotFound),
                        text: vanilla(text::ANIMATION_FILE_MISSING, [id, &joined]),
                    })
                    .at(entry("animation_sources", index, "animationFile"))
                    .into_kept(),
                );
            }
            _ => {
                // The game's error-level line for an animation the file lacks; the piece is kept.
                findings.push(
                    unresolved(
                        "animation",
                        name,
                        vanilla(text::ANIMATION_NOT_IN_FILE, [name, id, &joined]),
                        entry("animation_sources", index, "name"),
                    )
                    .into_kept(),
                );
            }
        }
    }
    if !emote_piece {
        return (kept, None);
    }
    // The game builds the emote from the first entry alone.
    let emote = kept.first().filter(|_| first_kept).and_then(|first| {
        let root = opened.get(first.file.as_str())?.as_ref()?;
        let mut lost = Vec::new();
        let json = as_emote(root, id, &mut lost);
        findings.extend(lost.into_iter().map(|at| {
            // The game's rename has no text for a bone it replaces, so the crate reports an
            // information.
            let mut found = Diagnostic::info(DiagnosticCode::DuplicateBone).at(at);
            found.location.path = Some(first.file.clone());
            found
        }));
        Some(EmoteAnimation {
            path: first.file.clone(),
            json,
        })
    });
    if meta.animation_sources.iter().flatten().next().is_none() {
        // The game logs the invalid-definition line at the content log's error level and
        // stops reading the emote; the piece is kept: a kept error.
        findings.push(
            Diagnostic::error(DiagnosticCode::MissingRequiredField {
                text: vanilla(text::EMOTE_INVALID, [id]),
            })
            .at(member("animation_sources"))
            .into_kept(),
        );
    }
    (kept, emote)
}

/// One animation file as the game can use it: a JSON object with an `animations` member,
/// else `None`. The game says nothing of the file itself; an emote's first entry is told
/// about it by `kept_animations`.
fn animation_file(bytes: &[u8]) -> Option<Value> {
    let root = read_root(bytes, &mut Vec::new()).ok()?;
    let has_animations = root
        .as_object()
        .and_then(|root| root.get("animations"))
        .is_some_and(|animations| !animations.is_null());
    (root.is_object() && has_animations).then_some(root)
}

/// `name` in the folder `directory`, joined as the game joins them.
fn join(directory: &str, name: &str) -> String {
    if directory.is_empty() {
        name.to_owned()
    } else {
        format!("{directory}/{name}")
    }
}

/// Changes every bone parent `hip` of the models in `geometry` to `waist`.
pub(super) fn retarget_hip_parents(geometry: &mut Value) {
    let Some(models) = geometry.as_object_mut() else {
        return;
    };
    for model in models.values_mut() {
        let bones = model
            .as_object_mut()
            .and_then(|model| model.get_mut("bones"))
            .and_then(Value::as_array_mut);
        for bone in bones
            .into_iter()
            .flat_map(facet_value::VArray::as_mut_slice)
        {
            let parent = bone.as_object_mut().and_then(|bone| bone.get_mut("parent"));
            if let Some(parent) = parent
                && parent
                    .as_string()
                    .is_some_and(|name| name.as_str() == "hip")
            {
                *parent = Value::from("waist");
            }
        }
    }
}

/// An animation file as the game loads it for the emote `id`: every key of `animations` has
/// `.<id>` appended and a bone named `hip` is named `waist`. `lost` takes the place of every
/// bone the rename replaces: of a `hip` and a `waist` in one animation the later stands.
pub(super) fn as_emote(root: &Value, id: &str, lost: &mut Vec<JsonPath>) -> Value {
    let Some(members) = root.as_object() else {
        return root.clone();
    };
    let mut file = VObject::with_capacity(members.len());
    for (key, value) in members {
        let animations = value.as_object().filter(|_| key.as_str() == "animations");
        let Some(animations) = animations else {
            file.insert(key.as_str(), value.clone());
            continue;
        };
        let mut renamed = VObject::with_capacity(animations.len());
        for (name, animation) in animations {
            renamed.insert(
                format!("{name}.{id}", name = name.as_str()),
                waist(animation, name.as_str(), lost),
            );
        }
        file.insert("animations", renamed);
    }
    file.into()
}

/// The animation `name` with its bone `hip` named `waist`.
fn waist(animation: &Value, name: &str, lost: &mut Vec<JsonPath>) -> Value {
    let Some(members) = animation.as_object() else {
        return animation.clone();
    };
    let mut renamed = VObject::with_capacity(members.len());
    for (key, value) in members {
        let bones = value.as_object().filter(|_| key.as_str() == "bones");
        let Some(bones) = bones else {
            renamed.insert(key.as_str(), value.clone());
            continue;
        };
        let mut named = VObject::with_capacity(bones.len());
        // The name the bone `waist` so far was written under.
        let mut written = None;
        for (bone, channels) in bones {
            let as_written = bone.as_str();
            let bone = if as_written == "hip" {
                "waist"
            } else {
                as_written
            };
            if bone == "waist"
                && let Some(earlier) = written.replace(as_written)
            {
                let at = member("animations")
                    .child(name)
                    .child("bones")
                    .child(earlier);
                lost.push(at);
            }
            named.insert(bone, channels.clone());
        }
        renamed.insert("bones", named);
    }
    renamed.into()
}

/// The path of a root member.
fn member(key: &str) -> JsonPath {
    JsonPath::new().child(key)
}
