//! The stack collection `languages()`: every pack's `texts/languages.json` and
//! `texts/language_names.json`, merged lowest pack first as vanilla merges them, with our
//! stack-level lints.

use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, ListDepth, ListedFile, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use indexmap::IndexMap;

use super::read::{read_languages, read_names};
use super::{Language, Languages};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{Document, Merged, Origin, Rejected};
use bedrock_addon_support::text::document::{document, origin, reject};
use bedrock_addon_support::text::lang_scan::{
    RESERVED_KEYS, defines_pack_keywords, is_pack_keyword,
};
use bedrock_addon_support::text::language::language_of;

/// The directory the stack loader reads.
const TEXTS: &str = "texts";
/// The directory only vanilla's single-pack append path also tries (L12).
const TEXTS_CAPITAL: &str = "Texts";
const LANGUAGES_JSON: &str = "texts/languages.json";
const LANGUAGE_NAMES_JSON: &str = "texts/language_names.json";

/// What the `languages` line reads of one file below `texts/` or `Texts/`.
#[derive(Clone, Debug)]
pub enum TextsFile {
    /// `texts/languages.json` or `texts/language_names.json`: the pack's layer, or why the
    /// copy does not count.
    Layer(Result<Document<Languages>, Rejected>),
    /// `texts/<code>.lang`: where it is and which pack keywords it defines. Its entries and
    /// findings are the `lang` family's.
    Lang {
        origin: Origin,
        language: Language,
        defines: [bool; 2],
    },
}

/// One `texts/<code>.lang` of a pack, as the keyword rules see it.
pub(super) struct LangFacts {
    /// The file's language.
    pub(super) language: Language,
    /// The file's path in the pack.
    pub(super) path: PackPath,
    /// Whether it defines `pack.name` and `pack.description`, in that order.
    pub(super) defines: [bool; 2],
}

/// What the keyword rules (L5, L6) know of one pack.
pub(super) struct KeywordFacts<'a> {
    /// The pack.
    pub(super) pack: &'a PackRef,
    /// The header `name`.
    pub(super) name: &'a str,
    /// The header `description`; `None` when it is not known.
    pub(super) description: Option<&'a str>,
    /// The codes of the pack's own `texts/languages.json`.
    pub(super) own_codes: &'a [Language],
    /// The pack's `.lang` files.
    pub(super) files: &'a [LangFacts],
}

/// A stack-level finding of ours about `path` of `pack`.
fn finding(
    code: DiagnosticCode,
    severity: Severity,
    pack: &PackRef,
    path: PackPath,
    at: JsonPath,
) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Validate,
        Location::new(Some(pack.clone()), Some(path), at, None),
    )
}

/// L5 and L6 for one pack: a keyword a `.lang` file defines but the pack never reads or its
/// manifest never uses, and a keyword the manifest uses but no file the pack reads defines.
///
/// A pack reads its keywords only from the files of the codes its own `languages.json` lists,
/// so a file of any other code defines them in vain, whatever the manifest says. Whether the
/// manifest uses a keyword needs both header fields; without the description, only "never
/// read" is decided.
// L5 is silent in vanilla; L6 displays the literal keyword.
pub(super) fn keyword_findings(facts: &KeywordFacts<'_>) -> Vec<Diagnostic> {
    let mut found = Vec::new();
    let listed = |file: &LangFacts| facts.own_codes.contains(&file.language);
    // `isPackKeyword` on either header field.
    let uses = |keyword: &str| {
        if facts.name == keyword {
            return Some(true);
        }
        facts.description.map(|description| description == keyword)
    };
    for file in facts.files {
        for (index, keyword) in RESERVED_KEYS.into_iter().enumerate() {
            if !file.defines[index] {
                continue;
            }
            if listed(file) && uses(keyword) != Some(false) {
                continue;
            }
            let mut at = JsonPath::new();
            at.push(keyword);
            found.push(finding(
                DiagnosticCode::IgnoredMember { text: None },
                Severity::Info,
                facts.pack,
                file.path.clone(),
                at,
            ));
        }
    }
    let fields = [
        ("name", Some(facts.name)),
        ("description", facts.description),
    ];
    for (field, text) in fields {
        let Some(keyword) = text.filter(|text| is_pack_keyword(text)) else {
            continue;
        };
        let index = usize::from(keyword != RESERVED_KEYS[0]);
        if facts
            .files
            .iter()
            .any(|file| listed(file) && file.defines[index])
        {
            continue;
        }
        let mut at = JsonPath::new();
        at.push("header");
        at.push(field);
        found.push(finding(
            DiagnosticCode::UnresolvedLangKey,
            Severity::Info,
            facts.pack,
            PackPath::from_static("manifest.json"),
            at,
        ));
    }
    found
}

/// The files below `texts/` whose content `languages()` reads: the two lists and the
/// `.lang` files. Every other file there counts by its path alone (the packs' file lists).
fn is_read(path: &PackPath) -> bool {
    matches!(path.as_str(), LANGUAGES_JSON | LANGUAGE_NAMES_JSON) || language_of(path).is_some()
}

/// A walk of `directories`, lowest pack first, packs only.
// `loadAllVersionsOf` returns every pack's copy, lowest priority first.
const fn walk(
    directories: &'static [&'static str],
    depth: ListDepth,
    accept: fn(&PackPath) -> bool,
) -> Walk {
    Walk {
        directories,
        depth,
        accept,
        order: WalkOrder::BottomUp,
        directory_major: false,
        app_data: &[],
        app_data_place: AppDataPlace::After,
        in_pack: None,
        shadowed: None,
        read_error: None,
    }
}

/// The `Parse` job of `languages()`: the two JSON files are read; of a `.lang` file only
/// whether it defines the pack keywords; of any other file only its place.
fn parse_texts(file: &WalkFile, _: &LoadContext) -> TextsFile {
    let origin = origin(file.pack.clone(), file.side, &file.path);
    match file.key.as_str() {
        LANGUAGES_JSON => TextsFile::Layer(match read_languages(&file.bytes) {
            Ok((languages, diagnostics)) => Ok(document(languages, origin, diagnostics)),
            Err(finding) => Err(reject(origin, finding)),
        }),
        LANGUAGE_NAMES_JSON => TextsFile::Layer(match read_names(&file.bytes) {
            Ok((names, diagnostics)) => {
                let languages = Languages {
                    codes: Vec::new(),
                    names,
                };
                Ok(document(languages, origin, diagnostics))
            }
            Err(finding) => Err(reject(origin, finding)),
        }),
        // Only the keyword lines are read: the file's entries and findings are the
        // `translations` line's, and `languages()` must not cost a parse of every language.
        // The walk hands over the two lists and the `.lang` files only.
        _ => TextsFile::Lang {
            language: language_of(&file.key).unwrap_or_else(|| Language::new("")),
            origin,
            defines: defines_pack_keywords(&file.bytes),
        },
    }
}

/// The layers top first; within one pack the later copy first. The walks are lowest pack
/// first, so that is the walk order reversed.
fn top_first<T>(layers: &mut [Document<T>]) {
    layers.reverse();
}

/// The `Combine` job of `languages()`: the union of the codes in first-seen order and the
/// names by vanilla's rule, both lowest pack first; then our findings about the files below
/// `texts/` and the pack keywords.
// `findAvailableLanguages` and `findAvailableLanguageNames`
fn combine_languages(parsed: Vec<TextsFile>, cx: &StackCx<'_>) -> Merged<Languages> {
    let mut value = Languages::default();
    let mut origins: IndexMap<Language, Origin> = IndexMap::new();
    let mut layers: Vec<Document<Languages>> = Vec::new();
    let mut diagnostics = Diagnostics::default();
    let mut lang_files: Vec<(Origin, Language, [bool; 2])> = Vec::new();
    for file in parsed {
        match file {
            TextsFile::Layer(Ok(layer)) => {
                diagnostics.extend(layer.diagnostics().iter().cloned());
                for code in &layer.codes {
                    if !value.codes.contains(code) {
                        value.codes.push(code.clone());
                        origins.insert(code.clone(), layer.origin().clone());
                    }
                }
                value.names.append(&layer.names);
                layers.push(layer);
            }
            TextsFile::Layer(Err(rejected)) => diagnostics.extend(rejected.diagnostics),
            TextsFile::Lang {
                origin,
                language,
                defines,
            } => lang_files.push((origin, language, defines)),
        }
    }
    // L7: only listed codes are iterated, so a `.lang` file of any other code is never
    // loaded. `vanilla = "silent"`.
    for (origin, language, _) in &lang_files {
        if !value.contains(language) {
            diagnostics.push(finding(
                DiagnosticCode::IgnoredFile { text: None },
                Severity::Info,
                &origin.pack,
                origin.path.clone(),
                JsonPath::new(),
            ));
        }
    }
    // The files that are neither a list nor a language file, by their paths alone: lowest
    // pack first, `Texts/` before `texts/` as the bytes sort.
    let other_files: Vec<(&PackRef, &ListedFile)> = cx
        .packs
        .iter()
        .rev()
        .flat_map(|pack| {
            let capital = pack.files_below(TEXTS_CAPITAL).iter();
            let texts = pack.files_below(TEXTS).iter();
            capital
                .chain(texts.filter(|file| !is_read(&file.key)))
                .map(move |file| (&pack.pack, file))
        })
        .collect();
    other_file_findings(&other_files, &value, &mut diagnostics);
    // L5 and L6, pack by pack, lowest first.
    for pack in cx.packs.iter().rev() {
        let own_codes: Vec<Language> = layers
            .iter()
            .filter(|layer| layer.origin().pack == pack.pack)
            .flat_map(|layer| layer.codes.iter().cloned())
            .collect();
        let files: Vec<LangFacts> = lang_files
            .iter()
            .filter(|(origin, ..)| origin.pack == pack.pack)
            .map(|(origin, language, defines)| LangFacts {
                language: language.clone(),
                path: origin.path.clone(),
                defines: *defines,
            })
            .collect();
        diagnostics.extend(keyword_findings(&KeywordFacts {
            pack: &pack.pack,
            name: pack.pack.name(),
            // A header without a description uses no keyword there.
            description: Some(pack.description.as_deref().unwrap_or_default()),
            own_codes: &own_codes,
            files: &files,
        }));
    }
    top_first(&mut layers);
    Merged::new(value, origins, layers, diagnostics)
}

/// The findings about the files that are neither a language file nor one of the two lists.
fn other_file_findings(
    files: &[(&PackRef, &ListedFile)],
    languages: &Languages,
    diagnostics: &mut Diagnostics,
) {
    let mut capital: Vec<&PackRef> = Vec::new();
    let mut dead_overlays: Vec<(&PackRef, &str)> = Vec::new();
    for &(pack, listed) in files {
        let key = &listed.key;
        // A file in a folder of `texts/` is a file of the locale overlay `texts/<code>/`.
        let overlay = key
            .as_str()
            .strip_prefix(TEXTS)
            .and_then(|rest| rest.strip_prefix('/'))
            .and_then(|rest| rest.split_once('/'));
        if let Some((code, file)) = overlay {
            if !languages.contains(&Language::new(code)) {
                // L16: the overlay of a language no `languages.json` lists is never
                // installed; one finding per pack and folder. Silent in vanilla.
                if !dead_overlays.contains(&(pack, code)) {
                    dead_overlays.push((pack, code));
                    let folder = PackPath::new(&format!("{TEXTS}/{code}"))
                        .unwrap_or_else(|_| listed.path.clone());
                    diagnostics.push(finding(
                        DiagnosticCode::IgnoredFile { text: None },
                        Severity::Info,
                        pack,
                        folder,
                        JsonPath::new(),
                    ));
                }
            } else if !file.starts_with("font/") {
                // L15: for clients of that language the file shadows the pack's root copy
                // of the same path; glyph sheets are what the overlay is for.
                // `vanilla = "silent per-file overlay
                // (ResourcePack::getResource)"`.
                diagnostics.push(finding(
                    DiagnosticCode::ShadowedDefinition { text: None },
                    Severity::Info,
                    pack,
                    listed.path.clone(),
                    JsonPath::new(),
                ));
            }
        }
        let in_capital = key
            .as_str()
            .strip_prefix(TEXTS_CAPITAL)
            .is_some_and(|rest| rest.starts_with('/'));
        // L12: the stack loader builds `texts/<code>.lang` and reads nothing else; one finding
        // per pack. `vanilla = "only the single-pack
        // appendLanguageStrings path falls back to Texts/"`.
        if in_capital && !capital.contains(&pack) {
            capital.push(pack);
            diagnostics.push(finding(
                DiagnosticCode::IgnoredFile { text: None },
                Severity::Info,
                pack,
                PackPath::from_static(TEXTS_CAPITAL),
                JsonPath::new(),
            ));
        }
    }
}

/// The loader of `languages()` on the stack of `side`: every file below `texts/` and
/// `Texts/`. The two lists are read; the other files are looked at for what they are, which
/// the lints need.
pub fn load_languages(side: PackSide) -> CollectionLoader<Merged<Languages>, TextsFile> {
    let _ = side;
    CollectionLoader {
        walk: walk(&[TEXTS], ListDepth::Recursive, is_read),
        parse: parse_texts,
        combine: combine_languages,
        needs: &[],
        then: None,
    }
}
