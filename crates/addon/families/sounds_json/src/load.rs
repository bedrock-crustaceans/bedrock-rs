//! Loading `sounds.json`: the walk, the pre-merge transform of each pack's copy, the JSON merge
//! across the stack (the game's merge strategy) and the typed read of the merged document.

use indexmap::IndexMap;

use super::bindings::{self, SoundBindings, SoundVariants};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{
    Content, Document, Merge, Merged, Origin, Placed, RawLayer, Rejected,
};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_support::sound::fork;
use bedrock_addon_support::sound::merge::{
    SoundCopy, detached_origin, document_text, layers, located, merge_finding, not_json,
    parse_copy, path_member, read_json, readable, unversioned, walk, write_document,
};

/// Where a pack keeps its event bindings.
pub(crate) const BINDINGS_PATH: &str = "sounds.json";

impl bedrock_addon_core::__private::Sealed for SoundBindings {}

impl Merge for SoundBindings {
    /// A top-level section (`block_sounds`, `entity_sounds`).
    type Key = String;
}

/// The findings of one copy of `sounds.json` about the lower copies it clears, judged before the transform inserts its `null`s.
fn cleared_tables(json: &Value, target: &Value, found: &mut Diagnostics) {
    let own = fork::member(fork::member(json, "entity_sounds"), "entities");
    let lower = fork::member(fork::member(target, "entity_sounds"), "entities");
    let mut at = JsonPath::new();
    at.push("entity_sounds");
    at.push("entities");
    // A copy without the section inserts nothing.
    if fork::has_member(json, "entity_sounds") && own.is_null() && !lower.is_null() {
        found.push(merge_finding(
            DiagnosticCode::SoundsJsonTableCleared,
            Severity::Info,
            at,
        ));
        return;
    }
    for name in fork::member_names(own) {
        let entity = fork::member(own, &name);
        let key = fork::member(fork::member(entity, "variants"), "key");
        let lower_key = fork::member(fork::member(fork::member(lower, &name), "variants"), "key");
        if fork::has_member(entity, "variants") && key.is_null() && !lower_key.is_null() {
            let mut at = at.clone();
            at.push(&name);
            at.push("variants");
            at.push("key");
            found.push(merge_finding(
                DiagnosticCode::SoundsJsonTableCleared,
                Severity::Info,
                at,
            ));
        }
    }
}

/// The compile check of the variant keys: every key is compiled at
/// its own version against the client catalogue, for its findings only. Without the check
/// (the `molang` feature, `LoadOptions::molang`) the keys are counted in one `MolangNotChecked`.
/// `cx` is a pack's context, for the load's compile check and the world's experiments.
fn check_variant_keys(value: &SoundBindings, cx: Option<&LoadContext>, found: &mut Diagnostics) {
    fn keyed<'a, T>(
        section: &'static [&'static str],
        name: &'a str,
        variants: Option<&'a SoundVariants<T>>,
    ) -> Option<(JsonPath, &'a bedrock_addon_core::molang::MolangSource)> {
        let key = variants?.key.as_ref()?;
        let mut at = JsonPath::new();
        for part in section {
            at.push(part);
        }
        at.push(name);
        at.push("variants");
        at.push("key");
        Some((at, key))
    }
    let keys: Vec<_> = value
        .entity_sounds
        .entities
        .iter()
        .filter_map(|(name, entry)| {
            keyed(
                &["entity_sounds", "entities"],
                name,
                entry.variants.as_ref(),
            )
        })
        .chain(
            value
                .interactive_sounds
                .entity_sounds
                .entities
                .iter()
                .filter_map(|(name, entry)| {
                    keyed(
                        &["interactive_sounds", "entity_sounds", "entities"],
                        name,
                        entry.variants.as_ref(),
                    )
                }),
        )
        .collect();
    if keys.is_empty() {
        return;
    }
    #[cfg(feature = "molang")]
    if let Some((check, cx)) = cx.and_then(|cx| Some((cx.molang.as_deref()?, cx))) {
        use bedrock_addon_core::molang::{
            EmptyRule, MolangContext, MolangFieldSpec, ReaderKind, Side,
        };
        // The game parses whatever string the key holds, an empty text included.
        const KEY: MolangFieldSpec = MolangFieldSpec {
            reader: ReaderKind::StrictVersioned,
            empty: EmptyRule::Compile,
            // The game parses the `key` without testing the result and registers the entity
            // anyway: the parser's content-log errors are kept errors and the document loads.
            error_severity: Severity::Error,
            error_kept: true,
            ..MolangFieldSpec::DEFAULT
        };
        let context = MolangContext {
            experiments: cx.molang_experiments,
            ..MolangContext::detached()
        };
        for (at, key) in keys {
            let location = Location::new(None, None, at, None);
            check.check_source(key, &KEY, Side::Client, &context, &location, found);
        }
        return;
    }
    let _ = cx;
    let fields = u32::try_from(keys.len()).unwrap_or(u32::MAX);
    found.push(Diagnostic::new(
        DiagnosticCode::MolangNotChecked { fields },
        Severity::Info,
        Phase::Validate,
        Location::default(),
    ));
}

/// Merges the copies of `sounds.json` and reads the result.
fn merge_bindings(copies: Vec<SoundCopy>, cx: &StackCx<'_>) -> Merged<SoundBindings, RawLayer> {
    let mut diagnostics = Diagnostics::default();
    let copies = readable(BINDINGS_PATH, copies, &mut diagnostics);
    let mut origins: IndexMap<String, Origin> = IndexMap::new();
    let mut target: Option<Value> = None;
    for (origin, json, molang_version) in &copies {
        if let Some(target) = &target {
            let mut found = Diagnostics::default();
            cleared_tables(json, target, &mut found);
            located(found, origin, &mut diagnostics);
        }
        let mut copy = json.clone();
        bindings::pre_merge_transform(&mut copy, *molang_version);
        for name in fork::member_names(json) {
            origins.insert(name, origin.clone());
        }
        match &mut target {
            None => target = Some(copy),
            Some(target) => fork::recursive_merge(target, &copy, None),
        }
    }
    let mut found = Diagnostics::default();
    let value = match target {
        Some(merged) => bindings::read_root(&merged, true, &mut found),
        None => SoundBindings::default(),
    };
    check_variant_keys(
        &value,
        cx.packs.first().map(|pack| &pack.context),
        &mut found,
    );
    let top = copies.last().map(|(origin, _, _)| origin);
    for mut finding in found {
        let owner = path_member(&finding.location.json_path, 0)
            .and_then(|section| origins.get(&section))
            .or(top);
        if let Some(origin) = owner {
            finding.location.pack = Some(origin.pack.clone());
            finding.location.path = Some(origin.path.clone());
        }
        diagnostics.push(finding);
    }
    Merged::new(value, origins, layers(&copies), diagnostics)
}

/// The file the walk reads: `sounds.json` in every pack.
fn accepts(path: &PackPath) -> bool {
    path.as_str() == BINDINGS_PATH
}

/// The loader of `rp.sounds_json()`: every resource pack's copy of the file, merged as JSON and
/// read once.
pub const fn load_bindings() -> CollectionLoader<Merged<SoundBindings, RawLayer>, SoundCopy> {
    CollectionLoader {
        walk: walk(accepts),
        parse: parse_copy,
        combine: merge_bindings,
        needs: &[],
        then: None,
    }
}

impl Content for SoundBindings {
    const NAME: &'static str = "sound bindings";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one pack's `sounds.json` on its own, as the only copy of a stack: vanilla's
    /// pre-merge transform with the pack's Molang version, then the typed read and the compile
    /// check of the variant keys. A block sound `base` that names no type of this file is no
    /// finding here (a lower pack may define it); the stack reports it. A file that is no JSON
    /// is rejected.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        // A number no double holds is read as the largest finite one, with a warning.
        bedrock_addon_core::document::FiniteText::load(bytes, |bytes| {
            let origin = detached_origin(path);
            let json = match read_json(bytes) {
                Ok(json) => json,
                Err(message) => return Err(not_json(origin, &message)),
            };
            // The transformed copy is what a merge takes and what the typed members are read from;
            // the document keeps the author's text, which is what it writes.
            let mut transformed = json.clone();
            bindings::pre_merge_transform(&mut transformed, cx.molang_version.0);
            let mut found = Diagnostics::default();
            let mut value = bindings::read_root(&transformed, false, &mut found);
            value.json = json;
            check_variant_keys(&value, Some(cx), &mut found);
            let mut diagnostics = Diagnostics::default();
            located(found, &origin, &mut diagnostics);
            Ok(Document::new(value, origin, unversioned(), diagnostics))
        })
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_document(&self.json, out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        document_text(&self.json)
    }
}

impl Placed for SoundBindings {
    /// `sounds.json`, at the pack root: the one path the client reads.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(BINDINGS_PATH)
    }
}
