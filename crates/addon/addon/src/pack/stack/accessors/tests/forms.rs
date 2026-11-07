//! Every accessor line form over a toy host: one side, both sides, keyed (its keys from another
//! line) and slice, with the `Collection` enum and the `ENTRIES` table the list expands to. A
//! `StackLoader`'s install through an entry gives what the entry's lazy load gives.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, LoadOptions, StackCx, Walk,
    WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackOrigin, PackPath, PackSide};

use bedrock_addon_core::load::Reported;

use crate::pack::stack::ErasedCollection;
use crate::pack::stack::{WalkSource, prepare, walk_items};
use crate::pack::tests::pack_from;
use crate::pack::{Behavior, Pack, PackStack, Resource, Side};

/// What a toy collection holds: each file's path, the side of the stack it was read on and its
/// text, in combine order.
#[derive(Debug, PartialEq)]
struct Files {
    files: Vec<(String, PackSide, String)>,
}

impl Reported for Files {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(std::iter::empty())
    }
}

/// The language codes `texts/languages.json` lists (here: separated by white space).
#[derive(Debug, PartialEq)]
struct Languages(Vec<String>);

impl Reported for Languages {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(std::iter::empty())
    }
}

/// One script bundle of the slice line.
#[derive(Debug, PartialEq)]
struct Bundle {
    path: String,
}

impl Reported for Bundle {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(std::iter::empty())
    }
}

/// The toy families' `load.rs`.
mod toy {
    use super::*;

    const fn walk(directories: &'static [&'static str], accept: fn(&PackPath) -> bool) -> Walk {
        Walk {
            directories,
            depth: ListDepth::Children,
            accept: Accept::Fn(accept),
            order: WalkOrder::BottomUp,
            directory_major: false,
            app_data: &[],
            app_data_place: AppDataPlace::After,
            in_pack: None,
            shadowed: None,
            read_error: None,
        }
    }

    fn any(_: &PackPath) -> bool {
        true
    }

    fn text(file: &WalkFile, _: &LoadContext) -> (String, PackSide, String) {
        (
            file.path.to_string(),
            file.side,
            String::from_utf8_lossy(&file.bytes).into_owned(),
        )
    }

    fn collect(files: Vec<(String, PackSide, String)>, _: &StackCx<'_>) -> Files {
        Files { files }
    }

    /// A one-side line: `plain/` of behaviour packs.
    pub(super) const PLAIN: CollectionLoader<Files, (String, PackSide, String)> =
        CollectionLoader {
            walk: walk(&["plain"], any),
            parse: text,
            combine: collect,
            needs: &[],
            then: None,
        };

    /// A both-sides line: `shared/` of the packs of either stack, each file read on the side
    /// of the stack the walk runs on.
    pub(super) const SHARED: CollectionLoader<Files, (String, PackSide, String)> =
        CollectionLoader {
            walk: walk(&["shared"], any),
            parse: text,
            combine: collect,
            needs: &[],
            then: None,
        };

    fn is_language_list(path: &PackPath) -> bool {
        path.as_str() == "texts/languages.json"
    }

    fn codes(texts: Vec<(String, PackSide, String)>, _: &StackCx<'_>) -> Languages {
        let mut codes: Vec<String> = Vec::new();
        for (_, _, text) in texts {
            for code in text.split_whitespace() {
                if !codes.iter().any(|known| known == code) {
                    codes.push(code.to_owned());
                }
            }
        }
        Languages(codes)
    }

    /// `texts/languages.json` on either side.
    pub(super) const LANGUAGES: CollectionLoader<Languages, (String, PackSide, String)> =
        CollectionLoader {
            walk: walk(&["texts"], is_language_list),
            parse: text,
            combine: codes,
            needs: &[],
            then: None,
        };

    fn is_lang(path: &PackPath) -> bool {
        path.as_str().to_ascii_lowercase().ends_with(".lang")
    }

    /// Every `texts/*.lang` file; the keyed line splits them by language.
    pub(super) const TRANSLATIONS: CollectionLoader<Files, (String, PackSide, String)> =
        CollectionLoader {
            walk: walk(&["texts"], is_lang),
            ..SHARED
        };

    /// The language of `texts/<code>.lang`.
    pub(super) fn language_of(path: &PackPath) -> Option<String> {
        path.as_str()
            .strip_prefix("texts/")?
            .strip_suffix(".lang")
            .map(str::to_owned)
    }

    /// The keys of the keyed line: what the `languages` line lists.
    pub(super) fn listed<S: Side>(host: &FormHost<S>) -> Vec<String> {
        host.languages().0.clone()
    }

    fn is_script(path: &PackPath) -> bool {
        path.as_str().to_ascii_lowercase().ends_with(".js")
    }

    fn bundle(file: &WalkFile, _: &LoadContext) -> Bundle {
        Bundle {
            path: file.path.to_string(),
        }
    }

    fn bundles(parsed: Vec<Bundle>, _: &StackCx<'_>) -> Vec<Bundle> {
        parsed
    }

    /// The slice line: one bundle per script.
    pub(super) const SCRIPTS: CollectionLoader<Vec<Bundle>, Bundle> = CollectionLoader {
        walk: walk(&["scripts"], is_script),
        parse: bundle,
        combine: bundles,
        needs: &[],
        then: None,
    };
}

/// A host with its own caches around a stack, as `PackStack` is its own host.
struct FormHost<S: Side> {
    stack: PackStack<S>,
    caches: FormCaches,
}

impl<S: Side> FormHost<S> {
    fn as_stack(&self) -> &PackStack<S> {
        &self.stack
    }
}

accessors! {
    @list FormCaches for FormHost;
    /// A behaviour-only collection.
    plain: Files = toy::PLAIN, collection = Plain, side = Behavior;
    /// One collection per side.
    shared: Files = toy::SHARED, collection = Shared, side = Both;
    /// The listed languages.
    languages: Languages = toy::LANGUAGES, collection = Languages, side = Both;
    /// The files of one listed language.
    translations[language: String]: Files = toy::TRANSLATIONS, keys = toy::listed,
        key_of = toy::language_of, collection = Translations, side = Both;
    /// The script bundles.
    scripts: [Bundle] = toy::SCRIPTS, collection = Scripts, side = Both;
}

/// A format-2 manifest with UUID `n` and one module of `kind`.
fn manifest(n: u8, kind: &str) -> String {
    format!(
        r#"{{ "format_version": 2,
  "header": {{ "name": "pack {n}", "uuid": "00000000-0000-4000-8000-0000000000{n:02}",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [{{ "type": "{kind}", "uuid": "00000000-0000-4000-9000-0000000000{n:02}", "version": [1, 0, 0] }}] }}"#
    )
}

/// A behaviour pack with a file for every line; `de_DE` has a file but is not listed and
/// `fr_FR` is listed without a file.
fn behavior_pack() -> Pack {
    pack_from(
        &manifest(1, "data"),
        &[
            ("plain/a.json", Some(b"a")),
            ("shared/b.json", Some(b"b")),
            ("texts/languages.json", Some(b"en_US fr_FR")),
            ("texts/en_US.lang", Some(b"bp hello")),
            ("texts/de_DE.lang", Some(b"bp hallo")),
            ("scripts/main.js", Some(b"")),
        ],
        PackOrigin::World,
    )
}

/// A resource pack that lists only `en_US`.
fn resource_pack() -> Pack {
    pack_from(
        &manifest(2, "resources"),
        &[
            ("shared/r.json", Some(b"r")),
            ("texts/languages.json", Some(b"en_US")),
            ("texts/en_US.lang", Some(b"rp hello")),
            ("scripts/client.js", Some(b"")),
        ],
        PackOrigin::World,
    )
}

fn host<S: Side>(pack: Pack) -> FormHost<S> {
    FormHost {
        stack: PackStack::<S>::of([pack], LoadOptions::default()).expect("a pack of the side"),
        caches: FormCaches::default(),
    }
}

fn behavior_host() -> FormHost<Behavior> {
    host(behavior_pack())
}

fn resource_host() -> FormHost<Resource> {
    host(resource_pack())
}

/// The files `files` (path and text), read on `side`.
fn files(side: PackSide, files: &[(&str, &str)]) -> Files {
    Files {
        files: files
            .iter()
            .map(|(path, text)| ((*path).to_owned(), side, (*text).to_owned()))
            .collect(),
    }
}

/// Through the generic impl a `side = Both` line expands to.
fn shared_of<S: Side>(host: &FormHost<S>) -> &Files {
    host.shared()
}

/// The entry of `collection` on `side`.
fn entry(collection: Collection, side: PackSide) -> &'static dyn ErasedCollection {
    ENTRY_COLLECTIONS
        .iter()
        .zip(ENTRIES)
        .find(|(of, entry)| **of == collection && entry.side() == side)
        .map(|(_, entry)| *entry)
        .expect("an entry")
}

#[test]
fn a_one_side_line_reads_its_side() {
    assert_eq!(
        *behavior_host().plain(),
        files(PackSide::Behavior, &[("plain/a.json", "a")])
    );
}

#[test]
fn a_both_sides_line_is_one_generic_accessor_over_each_stack_s_files() {
    let (bp, rp) = (behavior_host(), resource_host());
    assert_eq!(
        *shared_of(&bp),
        files(PackSide::Behavior, &[("shared/b.json", "b")])
    );
    assert_eq!(
        *shared_of(&rp),
        files(PackSide::Resource, &[("shared/r.json", "r")])
    );
    assert!(
        std::ptr::eq(bp.shared(), bp.shared()),
        "the second call returns the cache"
    );
    assert!(std::ptr::eq(rp.shared(), rp.shared()));
}

#[test]
fn a_keyed_line_answers_the_listed_keys_each_loaded_on_first_call() {
    let bp = behavior_host();
    let entry = entry(Collection::Translations, PackSide::Behavior);
    assert_eq!(entry.debug(&bp), None, "nothing loaded yet");
    let english = bp.translations(&"en_US".to_owned()).expect("listed");
    assert_eq!(
        *english,
        files(PackSide::Behavior, &[("texts/en_US.lang", "bp hello")])
    );
    assert!(std::ptr::eq(
        english,
        bp.translations(&"en_US".to_owned()).unwrap()
    ));
    let debug = entry.debug(&bp).expect("one key loaded");
    assert!(
        debug.contains("en_US") && !debug.contains("fr_FR"),
        "only the key asked for is loaded: {debug}"
    );
    assert_eq!(
        bp.translations(&"de_DE".to_owned()),
        None,
        "a file of a language no list names"
    );
    assert_eq!(
        bp.translations(&"fr_FR".to_owned()),
        Some(&files(PackSide::Behavior, &[])),
        "a listed language without files"
    );
    let rp = resource_host();
    assert_eq!(
        rp.translations(&"en_US".to_owned()),
        Some(&files(
            PackSide::Resource,
            &[("texts/en_US.lang", "rp hello")]
        ))
    );
    assert_eq!(rp.translations(&"fr_FR".to_owned()), None);
}

#[test]
fn a_slice_line_returns_its_collection_as_a_slice() {
    let bp = behavior_host();
    let scripts: &[Bundle] = bp.scripts();
    assert_eq!(
        scripts,
        [Bundle {
            path: "scripts/main.js".to_owned()
        }]
    );
    let rp = resource_host();
    assert_eq!(rp.scripts().len(), 1);
    assert!(std::ptr::eq(rp.scripts(), rp.scripts()));
}

#[test]
fn the_collection_enum_and_entries_follow_the_list() {
    assert_eq!(
        Collection::ALL,
        [
            Collection::Plain,
            Collection::Shared,
            Collection::Languages,
            Collection::Translations,
            Collection::Scripts,
        ]
    );
    // A test list's entries name their variant in `ENTRY_COLLECTIONS`; their
    // `ErasedCollection::collection` is `None`, as for every test line (the trait's
    // `Option<load::Collection>` has no toy variants).
    assert!(ENTRIES.iter().all(|entry| entry.collection().is_none()));
    assert_eq!(ENTRIES.len(), ENTRY_COLLECTIONS.len());
    let entries: Vec<(Collection, PackSide)> = ENTRY_COLLECTIONS
        .iter()
        .zip(ENTRIES)
        .map(|(collection, entry)| (*collection, entry.side()))
        .collect();
    let (b, r) = (PackSide::Behavior, PackSide::Resource);
    assert_eq!(
        entries,
        [
            (Collection::Plain, b),
            (Collection::Shared, b),
            (Collection::Shared, r),
            (Collection::Languages, b),
            (Collection::Languages, r),
            (Collection::Translations, b),
            (Collection::Translations, r),
            (Collection::Scripts, b),
            (Collection::Scripts, r),
        ]
    );
}

/// What a `StackLoader` does with an entry over `host`'s stack: read the walk, parse each file
/// with its pack's context, combine, install.
fn install<S: Side>(entry: &dyn ErasedCollection, host: &FormHost<S>) {
    let stack = host.as_stack();
    let walk = entry.walk();
    let mut parsed = Vec::new();
    let mut findings = Diagnostics::default();
    for item in walk_items(&walk, &stack.listed_files(), stack.stack_packs(), None) {
        let WalkSource::Pack(index) = item.source else {
            continue;
        };
        let pack = &stack.stack_packs()[index];
        match prepare(&stack.instances()[index].pack, &item.path) {
            Ok((bytes, _)) => {
                let file = WalkFile {
                    pack: pack.pack.clone(),
                    side: S::SIDE,
                    path: item.path,
                    key: item.key,
                    bytes,
                };
                parsed.push(entry.parse(&file, &pack.context));
            }
            Err(finding) => findings.push(*finding),
        }
    }
    let value = entry.combine(parsed, &StackCx::new(stack.options(), stack.stack_packs()));
    entry.install(host, value, findings);
}

/// Installs every entry of `S`'s side into one host and loads it lazily in another; both must
/// then be loaded and show the same collection.
fn installed_equals_loaded<S: Side>(installed: &FormHost<S>, loaded: &FormHost<S>) {
    for entry in ENTRIES.iter().filter(|entry| entry.side() == S::SIDE) {
        assert!(!entry.is_loaded(installed));
        install(*entry, installed);
        entry.load(loaded);
        assert!(entry.is_loaded(installed));
        assert!(entry.is_loaded(loaded));
        let debug = entry.debug(installed);
        assert!(debug.is_some());
        assert_eq!(debug, entry.debug(loaded));
    }
}

#[test]
fn an_installed_collection_equals_the_one_its_entry_loads() {
    installed_equals_loaded(&behavior_host(), &behavior_host());
    installed_equals_loaded(&resource_host(), &resource_host());
}

#[test]
fn after_a_full_load_a_keyed_line_still_answers_the_listed_keys_only() {
    let bp = behavior_host();
    let entry = entry(Collection::Translations, PackSide::Behavior);
    install(entry, &bp);
    let debug = entry.debug(&bp).expect("loaded");
    assert!(
        debug.contains("de_DE"),
        "a full load parses every key: {debug}"
    );
    assert_eq!(bp.translations(&"de_DE".to_owned()), None);
    assert_eq!(
        bp.translations(&"en_US".to_owned()),
        Some(&files(
            PackSide::Behavior,
            &[("texts/en_US.lang", "bp hello")]
        ))
    );
    assert_eq!(
        bp.translations(&"fr_FR".to_owned()),
        Some(&files(PackSide::Behavior, &[]))
    );
}

#[test]
fn a_keyed_line_reports_what_loading_each_key_found() {
    let pack = pack_from(
        &manifest(3, "data"),
        &[
            ("texts/languages.json", Some(b"en_US")),
            ("texts/en_US.lang", None),
        ],
        PackOrigin::World,
    );
    let bp: FormHost<Behavior> = host(pack);
    assert_eq!(
        bp.translations(&"en_US".to_owned()),
        Some(&files(PackSide::Behavior, &[]))
    );
    let mut out = Vec::new();
    entry(Collection::Translations, PackSide::Behavior).report(&bp, &mut out);
    let codes: Vec<&DiagnosticCode> = out.iter().map(|finding| &finding.code).collect();
    assert_eq!(codes, [&DiagnosticCode::from(&core_codes::FILE_NOT_LOADED)]);
    assert_eq!(
        out[0].location.pack,
        Some(bp.as_stack().stack_packs()[0].pack.clone())
    );
}

#[test]
fn the_hosts_stay_send_and_sync() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<FormHost<Behavior>>();
    send_sync::<FormHost<Resource>>();
}
