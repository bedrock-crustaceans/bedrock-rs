//! Loading `.lang` files: `Content` for one file, and the merge of one language along the
//! stack, with the translations accessor's loader.

use std::collections::btree_map::Entry;

use bedrock_addon_core::load::{Accept, CollectionLoader, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use indexmap::IndexMap;

use super::{LangFile, Language, Translations};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{
    Content, Document, Merge, Merged, Origin, Placed, Rejected, unversioned,
};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_support::text::language::language_of;

/// Both stacks: a behaviour pack carries `texts/` for its manifest name.
const SIDES: &[PackSide] = &[PackSide::Behavior, PackSide::Resource];

/// The directory the stack loader reads.
const TEXTS: &str = "texts";

impl bedrock_addon_core::__private::Sealed for LangFile {}
impl bedrock_addon_core::__private::Sealed for Translations {}

/// Keyed by the stored key: `Merged::origin` names the pack whose value won.
impl Merge for Translations {
    type Key = String;
}

impl Content for LangFile {
    const NAME: &'static str = "lang file";
    const SIDES: &'static [PackSide] = SIDES;

    /// Reads one `.lang` file; its language is the file name of `path` without `.lang`
    /// (`texts/en_US.lang`). Never rejects: vanilla drops bad lines one at a time.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = unversioned::lone(path);
        let language = language_of_file(path);
        let (file, diagnostics) = LangFile::from_bytes(language, bytes);
        Ok(unversioned::document(file, origin, diagnostics))
    }

    /// Writes the `.lang` text ([`LangFile::write_text`]); the style is JSON's and unused.
    fn write_json(&self, out: &mut Vec<u8>, _: WriteStyle) -> Result<(), EncodeError> {
        self.write_text(out)
    }

    /// The `.lang` text ([`LangFile::to_text`]).
    fn to_json(&self) -> Result<String, EncodeError> {
        self.to_text()
    }
}

impl Placed for LangFile {
    /// `texts/<code>.lang`.
    fn default_path(&self) -> PackPath {
        PackPath::new(&format!("texts/{}.lang", self.language))
            .unwrap_or_else(|_| PackPath::from_static("texts/_.lang"))
    }
}

/// The language a single `.lang` file is read as: the code of `texts/<code>.lang`; for a file
/// anywhere else, its name without the extension.
fn language_of_file(path: &PackPath) -> Language {
    language_of(path).unwrap_or_else(|| {
        let name = path.segments().last().unwrap_or_default();
        Language::new(name.strip_suffix(".lang").unwrap_or(name))
    })
}

/// What the `translations` line reads of one `texts/<code>.lang`: the file's table for the
/// language, and the keywords its pack reads of it.
#[derive(Debug)]
pub struct LangLayer {
    document: Document<Translations>,
    keywords: Translations,
}

/// The `Parse` job of `translations(&language)`: the file's table as the stack loader reads
/// it, and the keywords its pack reads of it.
fn parse_lang(file: &WalkFile, _: &LoadContext) -> LangLayer {
    let origin = unversioned::origin_of(file);
    let language = language_of(&file.key).unwrap_or_else(|| Language::new(""));
    let (lang, diagnostics) = LangFile::from_bytes(language, &file.bytes);
    LangLayer {
        keywords: lang.pack_keywords(file.pack.id()),
        document: unversioned::document(lang.translations(), origin, diagnostics),
    }
}

/// The `Combine` job of `translations(&language)`: each copy is appended on the lower ones,
/// so the highest pack's value wins per key; the pack keywords keep their first definition.
// `loadFromResourcePackManager`: `appendTranslations` once per copy, lowest priority first,
// each assigning. Pack-keyword locales merge with `addKeys`
// (`try_emplace` only).
fn combine_translations(parsed: Vec<LangLayer>, _: &StackCx<'_>) -> Merged<Translations> {
    let mut value = Translations::new();
    let mut origins: IndexMap<String, Origin> = IndexMap::new();
    let mut layers: Vec<Document<Translations>> = Vec::new();
    let mut diagnostics = Diagnostics::default();
    for LangLayer { document, keywords } in parsed {
        diagnostics.extend(document.diagnostics().iter().cloned());
        value.may_contain_identifier |=
            document.may_contain_identifier || keywords.may_contain_identifier;
        for (key, text) in &document.strings {
            value.strings.insert(key.clone(), text.clone());
            origins.insert(key.clone(), document.origin().clone());
        }
        for (key, text) in keywords.strings {
            if let Entry::Vacant(slot) = value.strings.entry(key) {
                origins.insert(slot.key().clone(), document.origin().clone());
                slot.insert(text);
            }
        }
        layers.push(document);
    }
    // The layers top first; the walks are lowest pack first, so that is the walk order
    // reversed.
    layers.reverse();
    Merged::new(value, origins, layers, diagnostics)
}

/// The loader of `translations(&language)`, on either stack: `texts/*.lang`, which the
/// accessor splits by language ([`language_of`]).
pub const LOADER: CollectionLoader<Merged<Translations>, LangLayer> = CollectionLoader {
    // A `.lang` file, by the extension vanilla builds the path with (`getLangFilePath`
    // appends `.lang` as written).
    walk: Walk::json_children_in(&[TEXTS]).accept(Accept::Ext("lang")),
    parse: parse_lang,
    combine: combine_translations,
    needs: &[],
    then: None,
};
