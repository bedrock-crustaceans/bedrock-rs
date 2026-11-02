//! One loaded piece: its metadata, the files beside it and what the game makes of them.

use bedrock_addon_core::pack_identity::{PackPath, PackType};
use uuid::Uuid;

use super::{PieceMetadata, PieceStoreInfo, PieceType, text};
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::Document;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{MolangVersion, RawVersion};
use bedrock_addon_core::version::MinEngineVersion;

/// The Molang version every expression of an emote animation is compiled at, whatever the
/// piece pack's manifest says: the game loads emotes with a fixed minimum engine version of
/// 1.14.0, which is before Molang was versioned.
pub const EMOTE_MOLANG_VERSION: MolangVersion = MolangVersion::V0;

/// One persona piece: a directory of a persona pack, read as the client reads it.
#[doc(alias = "persona_piece")]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct PersonaPiece {
    /// The piece's directory, from the pack root: the key the client groups the files by.
    pub directory: PackPath,
    /// The piece's identity: its `piece_id` in the client's built-in packs, the pack's header
    /// UUID in every other pack.
    pub id: Uuid,
    /// The `<name>.meta.json`, with the findings about that file alone.
    pub meta: Document<PieceMetadata>,
    /// The `<name>.store_info.json`, when the piece has one that is a JSON object.
    pub store_info: Option<Document<PieceStoreInfo>>,
    /// The `<name>.geometry.json`, when the piece has one that is JSON.
    pub geometry: Option<PieceGeometry>,
    /// The entries of `animation_sources` the game keeps: those whose file is a JSON object
    /// holding the named animation.
    pub animations: Vec<PieceAnimation>,
    /// An emote's animation file as the game loads it: the file of the first kept animation.
    pub emote: Option<EmoteAnimation>,
    /// The piece's own `texts/` folder, when it holds an `en_US.lang` (the client then gives
    /// the piece its own translations).
    pub texts: Option<PackPath>,
    /// `persona.offer.title` of the piece's own `texts/en_US.lang`: the name the client shows
    /// instead of [`name_key`](Self::name_key)'s translation.
    pub offer_title: Option<String>,
    /// The findings that need more than one file: the identity, references to textures, models
    /// and animations, the name.
    pub(super) findings: Diagnostics,
}

/// A piece's `<name>.geometry.json`: models keyed by name, held as JSON.
///
/// The game applies no schema when it loads the piece. When a character is put together it
/// takes the root member a geometry source names and hands it, with the file's
/// `format_version`, to its geometry loader: [`model_file`](Self::model_file) builds that text.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct PieceGeometry {
    /// The file, from the pack root.
    pub path: PackPath,
    /// The file's root, with every bone parent `hip` changed to `waist` as the game changes it.
    pub json: Value,
}

impl PieceGeometry {
    /// The model named `name`: that root member of the file.
    #[must_use]
    pub fn model(&self, name: &str) -> Option<&Value> {
        self.json
            .as_object()?
            .get(name)
            .filter(|model| !model.is_null())
    }

    /// The file's `format_version` as the game reads it: `"1.0"` when the member is absent or
    /// not a string.
    #[must_use]
    pub fn format_version(&self) -> &str {
        self.json
            .as_object()
            .and_then(|root| root.get("format_version"))
            .and_then(Value::as_string)
            .map_or("1.0", |version| version.as_str())
    }

    /// The names of the file's models: its root members but `format_version`.
    pub fn models(&self) -> impl Iterator<Item = &str> + '_ {
        self.json
            .as_object()
            .into_iter()
            .flat_map(|root| root.keys())
            .map(facet_value::VString::as_str)
            .filter(|key| *key != "format_version")
    }

    /// The model `name` as a geometry file of its own (`format_version` and the one model), for
    /// the geometry family's reader; `None` when the file has no such model.
    #[must_use]
    pub fn model_file(&self, name: &str) -> Option<Value> {
        let model = self.model(name)?;
        let mut file = facet_value::VObject::new();
        file.insert("format_version", self.format_version());
        file.insert(name, model.clone());
        Some(file.into())
    }
}

/// One animation of a piece that the game keeps.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PieceAnimation {
    /// The animation's key in its file (`animation_sources[].name`).
    pub name: String,
    /// The name the game stores it by: `<name>.<piece id>`.
    pub stored_name: String,
    /// The animation file, from the pack root.
    pub file: PackPath,
}

/// An emote's animation file as the game's animation loader sees it.
///
/// It is a client animation file; read it with the animation family's reader in a context
/// whose Molang version is [`EMOTE_MOLANG_VERSION`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct EmoteAnimation {
    /// The file, from the pack root.
    pub path: PackPath,
    /// The file's root after the game's two changes: every key of `animations` has
    /// `.<piece id>` appended, and a bone named `hip` is named `waist`.
    pub json: Value,
}

impl EmoteAnimation {
    /// The context the game loads an emote's animation in, whatever `cx` (the piece pack's)
    /// says: a client animation, with a minimum engine version of 1.14.0 and with it Molang
    /// version 0.
    #[must_use]
    pub fn context(&self, cx: &LoadContext) -> LoadContext {
        let mut cx = cx.clone();
        // A client animation, whatever the type of the pack the piece is in.
        cx.pack_type = PackType::Resource;
        cx.min_engine_version = Some(MinEngineVersion::new(1, 14, 0));
        cx.molang_version = RawVersion::from(EMOTE_MOLANG_VERSION);
        cx
    }
}

impl PersonaPiece {
    /// The piece's type as the game takes it.
    #[must_use]
    pub fn piece_type(&self) -> PieceType {
        self.meta.value().piece_type()
    }

    /// The translation key of the piece's name: built from the store-info's
    /// `piece_store_name` when the piece has a store-info object (from the empty text when that
    /// object lacks the member, as the game does), else from `piece_name`.
    #[must_use]
    pub fn name_key(&self) -> String {
        let name = match &self.store_info {
            Some(store) => store.value().piece_store_name.as_deref(),
            None => self.meta.value().piece_name.as_deref(),
        };
        text::name_key(name.unwrap_or_default())
    }

    /// The image the Character Creator lists the piece by, from the pack root: the empty text
    /// when a store-info object lacks `piece_thumbnail`, `None` without a store-info object.
    #[must_use]
    pub fn thumbnail(&self) -> Option<&str> {
        let store = self.store_info.as_ref()?;
        Some(store.value().piece_thumbnail.as_deref().unwrap_or_default())
    }

    /// The findings that need more than one of the piece's files: its identity, the textures,
    /// models and animations it refers to, its name.
    #[must_use]
    pub fn findings(&self) -> &Diagnostics {
        &self.findings
    }

    /// Every finding about the piece: the metadata's, the store-info's, then
    /// [`findings`](Self::findings).
    pub fn report(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.meta
            .diagnostics()
            .iter()
            .chain(
                self.store_info
                    .iter()
                    .flat_map(|store| store.diagnostics().iter()),
            )
            .chain(self.findings.iter())
    }
}
