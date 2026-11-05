//! Loading the materials of a resource stack: one walk over the list files and every
//! `.material` file, one parse job per file, and one combine that processes the lists as the
//! client does, merges each named file over the packs and creates the materials.

use std::collections::{HashMap, HashSet};

use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, LoadContext, StackCx, Walk, WalkFile,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bytes::Bytes;
use indexmap::{IndexMap, IndexSet};

use super::Materials;
use super::file::{KeyRules, MaterialFile, MaterialFileForm, base_name};
use super::group::{FileSource, MaterialGroup, MaterialGroupKind};
use super::list::{MaterialList, MaterialListKind};
use super::patch::{self, Trace, patch_traced};
use super::read::{
    Unwrapped, check_duplicates, path_in_file, read_copy, read_file, read_list_placed,
    report_in_file, unwrap_copy,
};
use super::span;
use super::write::{parse_error, rejected, stem_of, unversioned, wrong_root};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity,
};
use bedrock_addon_core::document::{Document, Extra, Merged, Origin, RawLayer, Rejected};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::load::Reported;

/// The extension of a material file.
const EXTENSION: &str = ".material";

/// The one path the client's stack upgrades: a request for it falls back, in each pack that
/// lacks it, to that pack's [`SPRITE`] file. Never the reverse.
const BARRIER: &str = "materials/barrier.material";
/// What a pack without the [`BARRIER`] file contributes to a request for it.
const SPRITE: &str = "materials/cameraFacingSprite.material";

/// One pack's copy of a material file, as its parse job leaves it.
#[derive(Debug)]
struct FileCopy {
    /// Where the copy is.
    origin: Origin,
    /// The copy as read and past the wrapper gate, unless it is no JSON object.
    read: Option<(RawLayer, Unwrapped)>,
    /// Where the copy has a number past the range of a double, by its path in the file.
    clamped: IndexSet<JsonPath>,
    /// What reading the copy found.
    findings: Diagnostics,
    /// The copy's text, for the spans of the findings the merge places in it.
    bytes: Bytes,
}

/// What one walked file is.
#[derive(Debug)]
enum Parsed {
    /// A list file, and its text for the findings the groups place in it.
    List(Box<Result<Document<MaterialList>, Rejected>>, ListText),
    /// One pack's copy of a material file.
    File(Box<FileCopy>),
}

/// One walked file: where it is, and what its parse job made of it.
#[derive(Debug)]
pub struct MaterialParsed {
    /// The pack.
    pack: PackRef,
    /// The key the stack reads the file at.
    key: PackPath,
    /// The file's content.
    parsed: Parsed,
}

/// The four list files, and every `.material` file: a list may name one anywhere in the pack.
fn is_read(path: &PackPath) -> bool {
    let path = path.as_str();
    MaterialListKind::from_path(path).is_some() || path.ends_with(EXTENSION)
}

/// The material files anywhere in the pack, lowest pack first.
const WALK: Walk = Walk::json_tree_in(&[""])
    .accept(Accept::Fn(is_read))
    .app_data(&[], AppDataPlace::Before);

/// The origin of a walked file.
fn origin_of(file: &WalkFile) -> Origin {
    Origin::file(file.pack.clone(), PackSide::Resource, file.path.clone())
}

/// One list file of the walk, and the index each of its entries has in the file.
fn parse_list(
    kind: MaterialListKind,
    file: &WalkFile,
) -> (Result<Document<MaterialList>, Rejected>, Vec<usize>) {
    let origin = origin_of(file);
    let mut findings = Diagnostics::default();
    let copy = match read_copy(&file.bytes) {
        Ok(copy) => copy,
        Err(message) => {
            findings.push(parse_error(&file.path, message));
            return (Err(rejected(origin, findings)), Vec::new());
        }
    };
    copy.report_clamped(&mut findings);
    let list = read_list_placed(kind, &copy.root, &mut findings);
    span::fill_all(&file.bytes, &mut findings);
    if let Some((list, places)) = list {
        return (Ok(unversioned(list, origin, findings)), places);
    }
    findings.push(wrong_root());
    (Err(rejected(origin, findings)), Vec::new())
}

/// The text of a list file and the index in it of each entry its list holds: where the
/// groups place what they find about an entry.
#[derive(Debug)]
struct ListText {
    bytes: Bytes,
    places: Vec<usize>,
}

/// One pack's copy of a material file: read, passed through the wrapper gate and checked for
/// two keys of one name. A copy that is no JSON object is left out of the merge.
fn parse_copy(file: &WalkFile) -> FileCopy {
    let origin = origin_of(file);
    let mut findings = Diagnostics::default();
    let mut clamped = IndexSet::new();
    let read = match read_copy(&file.bytes) {
        Ok(mut copy) => {
            if let Some(root) = copy.root.as_object() {
                let unwrapped = unwrap_copy(root.clone(), &mut findings);
                copy.report_clamped(&mut findings);
                let mut found = Diagnostics::default();
                check_duplicates(&unwrapped.map, file.path.as_str(), &mut found);
                report_in_file(found, unwrapped.wrapped.is_some(), &mut findings);
                clamped = std::mem::take(&mut copy.clamped);
                Some((RawLayer::new(origin.clone(), copy.root), unwrapped))
            } else {
                findings.push(wrong_root());
                None
            }
        }
        Err(message) => {
            findings.push(parse_error(&file.path, message));
            None
        }
    };
    findings.locate(&origin);
    span::fill_all(&file.bytes, &mut findings);
    FileCopy {
        origin,
        read,
        clamped,
        findings,
        bytes: file.bytes.clone(),
    }
}

/// One file of the walk.
fn parse_walk_file(file: &WalkFile, _: &LoadContext) -> MaterialParsed {
    let parsed = match MaterialListKind::from_path(file.key.as_str()) {
        Some(kind) => {
            let (list, places) = parse_list(kind, file);
            let text = ListText {
                bytes: file.bytes.clone(),
                places,
            };
            Parsed::List(Box::new(list), text)
        }
        None => Parsed::File(Box::new(parse_copy(file))),
    };
    MaterialParsed {
        pack: file.pack.clone(),
        key: file.key.clone(),
        parsed,
    }
}

/// One merged file with what the groups need of it.
struct MergedFile {
    /// The merge.
    merged: Merged<MaterialFile, RawLayer>,
    /// Per material: whether it is refused.
    refused: Vec<bool>,
    /// Who wrote what of the merged map.
    writers: Writers,
}

/// Who wrote each member of a merged material map: the copies it was merged from, and the
/// trace of the merge.
struct Writers {
    /// Per copy, lowest pack first: where it is, and whether its map is inside a wrapper.
    copies: Vec<(Origin, bool)>,
    /// Which writer wrote what, and under which keys.
    trace: Trace,
    /// The copy each writer of the trace stands for.
    stands_for: patch::Writers,
    /// The text of each of `copies`.
    texts: span::Texts,
}

impl Writers {
    /// Where the member at `at` of the merged map was written: which of `copies`, and the
    /// path the member has in that copy's file.
    fn written(&self, at: &JsonPath) -> Option<(usize, JsonPath)> {
        let (writer, path) = self.trace.written(at);
        let copy = usize::try_from(self.stands_for.copy_of(writer)?).ok()?;
        let (_, wrapped) = self.copies.get(copy)?;
        Some((copy, path_in_file(&path, *wrapped)))
    }

    /// Places a finding at a path of the merged map where its member was written: the pack
    /// and the file of the copy that wrote it, and the path the member has in that file.
    /// The copy it was placed in, if any.
    fn place(&self, finding: &mut Diagnostic) -> Option<usize> {
        let (copy, path) = self.written(&finding.location.json_path)?;
        let (origin, _) = self.copies.get(copy)?;
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        finding.location.json_path = path;
        Some(copy)
    }

    /// Places every finding of `found` ([`place`](Self::place)) and gives each the span its
    /// member has in the text of the copy it was placed in; each copy's text is gone through
    /// once for all of its findings.
    fn place_all(&self, found: impl IntoIterator<Item = Diagnostic>) -> Vec<Diagnostic> {
        let mut placed: Vec<(Option<usize>, Diagnostic)> = found
            .into_iter()
            .map(|mut finding| (self.place(&mut finding), finding))
            .collect();
        self.texts.fill(&mut placed);
        placed.into_iter().map(|(_, finding)| finding).collect()
    }
}

/// Reports the keys `rekeyed` of the copy at `origin`, each of which takes over a lower key
/// under another spelling. The finding is about the copy's key: one for it, however many
/// lower keys of its name it takes over.
fn report_rekeyed(rekeyed: &[String], wrapped: bool, origin: &Origin, out: &mut Diagnostics) {
    let mut reported: HashSet<&str> = HashSet::new();
    for key in rekeyed {
        if !reported.insert(key.as_str()) {
            continue;
        }
        let at = JsonPath::new().child(key);
        let at = path_in_file(&at, wrapped);
        let code = DiagnosticCode::ShadowedDefinition { text: None };
        let finding = Diagnostic::info(code)
            .at(at)
            .phase(Phase::Validate)
            .in_file(origin);
        out.push(finding);
    }
}

/// Merges the copies `sources` of the file at `path`, lowest pack first, and reads the result.
/// `before` are findings about the merge as a whole, reported first.
///
/// Each copy past the gate is laid over what the lower copies gave; a key that
/// takes over a lower key under another spelling drops or changes its parent (reported
/// once per key of the copy, also where it takes over two lower keys of its name). The key
/// rules of a material are those of the highest copy that names it: the client stamps each
/// copy's materials with the copy's version before it merges. A finding of the merged map is
/// placed in the copy that wrote the member it is about.
fn merge_file(path: &PackPath, sources: &[&FileCopy], before: Diagnostics) -> MergedFile {
    let mut diagnostics = before;
    let mut merged = Value::NULL;
    let mut trace = Trace::Whole(0);
    let mut trace_writers = patch::Writers::default();
    // Per copy that was merged: where it is, and whether it is wrapped.
    let mut copies = Vec::new();
    // Per copy that was merged: where it has a number past the doubles.
    let mut clamped_in = Vec::new();
    let mut layers = Vec::new();
    // Per copy that was merged: its text.
    let mut texts = span::Texts::default();
    // Per name before the first colon: the highest copy that names it.
    let mut named: HashMap<&str, usize> = HashMap::new();
    let mut top = None;
    for (index, source) in sources.iter().enumerate() {
        diagnostics.extend(source.findings.iter().cloned());
        let Some((layer, unwrapped)) = &source.read else {
            continue;
        };
        layers.push(layer.clone());
        for key in unwrapped.map.keys() {
            named.insert(base_name(key.as_str()), index);
        }
        let mut rekeyed = Vec::new();
        let copy = unwrapped.map.clone().into_value();
        let by = trace_writers.add(u32::try_from(copies.len()).unwrap_or(u32::MAX));
        let wrapped = unwrapped.wrapped.is_some();
        copies.push((source.origin.clone(), wrapped));
        clamped_in.push(&source.clamped);
        let told = &mut |_: &str, upper: &str| rekeyed.push(upper.to_owned());
        let by = Trace::Whole(by);
        (merged, trace) = patch_traced(merged, trace, copy, by, told, &mut trace_writers);
        let mut shadowing = Diagnostics::default();
        report_rekeyed(&rekeyed, wrapped, &source.origin, &mut shadowing);
        span::fill_all(&source.bytes, &mut shadowing);
        diagnostics.extend(shadowing);
        texts.push(source.bytes.clone());
        top = Some(index);
    }
    // Top first, as `Merged::layers` promises.
    layers.reverse();
    let empty = facet_value::VObject::new();
    let map = merged.as_object().unwrap_or(&empty);
    let wrapped_by = |index: usize| {
        let read = sources.get(index).and_then(|source| source.read.as_ref());
        read.and_then(|(_, unwrapped)| unwrapped.wrapped.clone())
    };
    let rules = |key: &str| {
        let wrapped = named.get(base_name(key)).copied().and_then(wrapped_by);
        Some(if wrapped.is_some() {
            KeyRules::Versioned
        } else {
            KeyRules::Legacy
        })
    };
    // A finding of the merged document belongs to the copy that wrote the member it is
    // about.
    let writers = Writers {
        copies,
        trace,
        stands_for: trace_writers,
        texts,
    };
    // A number of the merged map was clamped when the copy that wrote it has it past the
    // doubles.
    let clamped = |at: &JsonPath| {
        writers.written(at).is_some_and(|(copy, path)| {
            clamped_in
                .get(copy)
                .is_some_and(|clamped| clamped.contains(&path))
        })
    };
    let mut found = Diagnostics::default();
    let mut read = read_file(
        map,
        top.and_then(wrapped_by),
        stem_of(path.as_str()),
        &rules,
        &clamped,
        &mut found,
    );
    let origin_of = |index: usize| sources.get(index).map(|source| &source.origin);
    let single = matches!(read.file.form, MaterialFileForm::Single { .. });
    let mut origins = IndexMap::new();
    for key in read.file.materials.keys() {
        let from = if single {
            top
        } else {
            named.get(key.base_name().as_str()).copied()
        };
        if let Some(origin) = from.and_then(origin_of) {
            origins.insert(key.clone(), origin.clone());
        }
    }
    diagnostics.extend(writers.place_all(found));
    if top.is_none() {
        read.file.extra = Extra::default();
    }
    MergedFile {
        merged: Merged::new(read.file, origins, layers, diagnostics),
        refused: read.refused,
        writers,
    }
}

/// Everything the combine works on.
struct Loading<'a> {
    /// The stack.
    stack: &'a StackCx<'a>,
    /// Each walked material file's copies by path, lowest pack first.
    copies: IndexMap<String, Vec<FileCopy>>,
    /// The merged files in the order they were first asked for.
    files: IndexMap<PackPath, MergedFile>,
    /// The paths a list asked for.
    requested: HashSet<String>,
    /// The findings of the load itself.
    diagnostics: Diagnostics,
    /// The group findings already reported: a file two groups load reports each once.
    reported: HashSet<(usize, JsonPath, &'static str)>,
}

impl Loading<'_> {
    /// The copies a request for `path` merges, lowest pack first: each pack's own copy, and
    /// for the barrier file the sprite file of each pack that has no barrier file.
    fn sources(&self, path: &str) -> Vec<&FileCopy> {
        let own = self.copies.get(path).map_or(&[][..], Vec::as_slice);
        if path != BARRIER {
            return own.iter().collect();
        }
        let fallback = self.copies.get(SPRITE).map_or(&[][..], Vec::as_slice);
        let has_own: HashSet<Option<usize>> =
            own.iter().map(|copy| copy.origin.pack.index()).collect();
        let mut sources: Vec<&FileCopy> = own.iter().collect();
        sources.extend(
            fallback
                .iter()
                .filter(|copy| !has_own.contains(&copy.origin.pack.index())),
        );
        // Lowest pack first: the highest stack index.
        sources.sort_by_key(|copy| std::cmp::Reverse(copy.origin.pack.index()));
        sources
    }

    /// The index in `files` of the merge of the file at `path`, merging it on first call;
    /// `None` when no pack has a copy.
    fn merged(&mut self, path: &str) -> Option<usize> {
        let key = PackPath::new(path).ok()?;
        if let Some(index) = self.files.get_index_of(&key) {
            return Some(index);
        }
        let sources = self.sources(path);
        if sources.is_empty() {
            return None;
        }
        // The client reads the sprite file under the barrier file's name in silence.
        let mut fallbacks = Diagnostics::default();
        for source in &sources {
            if source.origin.path.as_str() != path {
                let code = DiagnosticCode::ShadowedDefinition { text: None };
                fallbacks.push(
                    Diagnostic::info(code)
                        .phase(Phase::Validate)
                        .in_file(&source.origin),
                );
            }
        }
        let file = merge_file(&key, &sources, fallbacks);
        Some(self.files.insert_full(key, file).0)
    }

    /// Whether a pack of the stack has a file at `path`, by the stack's listing.
    fn listed(&self, path: &str) -> bool {
        self.stack.packs.iter().any(|pack| {
            pack.files()
                .binary_search_by(|file| file.key.as_str().cmp(path))
                .is_ok()
        })
    }

    /// Processes one group's list copies, highest pack first, as the client's loader does: a
    /// path the group already loaded is skipped, so the first entry for a path, with its
    /// defines, is the one that counts. `places` gives, per list, the index each of its
    /// entries has in the file: a finding about an entry is placed there.
    fn group(
        &mut self,
        kind: MaterialGroupKind,
        lists: &[Document<MaterialList>],
        places: &[Vec<usize>],
    ) -> MaterialGroup {
        let mut group = MaterialGroup::new(kind);
        let mut loaded: HashSet<&str> = HashSet::new();
        let of_kind = lists.iter().zip(places);
        for (list, places) in of_kind.filter(|(list, _)| list.kind == kind.list()) {
            let mut here: HashSet<&str> = HashSet::new();
            for (index, entry) in list.entries.iter().enumerate() {
                let path = entry.path.as_str();
                let at_entry = |member: Option<&str>, code, severity| {
                    let mut at = JsonPath::new();
                    // The reader drops entries, so the place in the list is not the file's.
                    at.push_index(places.get(index).copied().unwrap_or(index));
                    if let Some(member) = member {
                        at.push(member);
                    }
                    Diagnostic::error(code)
                        .severity(severity)
                        .at(at)
                        .phase(Phase::Validate)
                        .in_file(list.origin())
                };
                let repeated = !here.insert(path);
                if !loaded.insert(path) {
                    // The game skips the later listing, and its defines, in silence.
                    if repeated {
                        let code = DiagnosticCode::NoEffect { text: None };
                        self.diagnostics.push(at_entry(None, code, Severity::Info));
                    }
                    continue;
                }
                self.requested.insert(path.to_owned());
                let Some(file) = self.merged(path) else {
                    // The crate reads only `.material` files; the game reads any path.
                    if !path.ends_with(EXTENSION) && self.listed(path) {
                        let code = DiagnosticCode::IgnoredFile { text: None };
                        let finding = at_entry(Some("path"), code, Severity::Info);
                        self.diagnostics.push(finding);
                    }
                    continue;
                };
                self.create(&mut group, file, entry.base_defines());
            }
        }
        group
    }

    /// Creates the materials of the merged file `file` in `group`.
    fn create(&mut self, group: &mut MaterialGroup, file: usize, base_defines: Vec<String>) {
        let Some((path, merged)) = self.files.get_index(file) else {
            return;
        };
        let source = FileSource {
            index: u32::try_from(file).unwrap_or(u32::MAX),
            path,
            file: merged.merged.value(),
            refused: &merged.refused,
        };
        let mut found = Vec::new();
        group.load_file(&source, base_defines, &mut found);
        let mut fresh = Vec::new();
        for (_, finding) in found {
            let at = finding.location.json_path.clone();
            if self.reported.insert((file, at, finding.code.name())) {
                fresh.push(finding);
            }
        }
        self.diagnostics.extend(merged.writers.place_all(fresh));
    }
}

/// The list files of a walk apart: those that loaded, each with its text, and those that did
/// not, both top pack first and within a pack in the walk's order, by path.
struct Lists {
    loaded: Vec<(Document<MaterialList>, ListText)>,
    rejected: Vec<Rejected>,
}

/// Sorts the parsed files of a walk: the copies of material files go to `loading` by path,
/// the lists are returned. A list that does not load stays a rejected file of the group,
/// whole; its findings are the group's as well, in the walk's order.
fn sort_parsed(parsed: Vec<MaterialParsed>, loading: &mut Loading<'_>) -> Lists {
    let mut loaded = Vec::new();
    let mut rejected = Vec::new();
    for file in parsed {
        match file.parsed {
            Parsed::List(list, text) => match *list {
                Ok(list) => loaded.push((file.pack.index(), (list, text))),
                Err(file_rejected) => {
                    loading
                        .diagnostics
                        .extend(file_rejected.diagnostics.iter().cloned());
                    rejected.push((file.pack.index(), file_rejected));
                }
            },
            Parsed::File(copy) => loading
                .copies
                .entry(file.key.as_str().to_owned())
                .or_default()
                .push(*copy),
        }
    }
    loaded.sort_by_key(|(index, _)| *index);
    rejected.sort_by_key(|(index, _)| *index);
    Lists {
        loaded: loaded.into_iter().map(|(_, list)| list).collect(),
        rejected: rejected.into_iter().map(|(_, file)| file).collect(),
    }
}

/// The group, from every parsed file in walk order.
fn combine(parsed: Vec<MaterialParsed>, stack: &StackCx<'_>) -> Materials {
    let mut loading = Loading {
        stack,
        copies: IndexMap::new(),
        files: IndexMap::new(),
        requested: HashSet::new(),
        diagnostics: Diagnostics::default(),
        reported: HashSet::new(),
    };
    let Lists {
        loaded: lists,
        rejected,
    } = sort_parsed(parsed, &mut loading);
    let mut list_texts = span::Texts::default();
    // Per list: the index in the file of each entry the list holds.
    let mut places = Vec::with_capacity(lists.len());
    let lists: Vec<Document<MaterialList>> = lists
        .into_iter()
        .map(|(list, text)| {
            list_texts.push(text.bytes);
            places.push(text.places);
            list
        })
        .collect();
    let common = loading.group(MaterialGroupKind::Common, &lists, &places);
    let fancy = loading.group(MaterialGroupKind::Fancy, &lists, &places);
    let sad = loading.group(MaterialGroupKind::Sad, &lists, &places);
    // What the groups found at a list's entries gets its span from that list's text.
    let list_at: HashMap<(Option<usize>, &str), usize> = lists
        .iter()
        .enumerate()
        .map(|(place, list)| {
            let origin = list.origin();
            ((origin.pack.index(), origin.path.as_str()), place)
        })
        .collect();
    let mut of_lists: Vec<(Option<usize>, Diagnostic)> = std::mem::take(&mut loading.diagnostics)
        .into_iter()
        .map(|finding| {
            let location = &finding.location;
            let list = location.pack.as_ref().zip(location.path.as_ref());
            let place = list.and_then(|(pack, path)| list_at.get(&(pack.index(), path.as_str())));
            (place.copied(), finding)
        })
        .collect();
    list_texts.fill(&mut of_lists);
    loading
        .diagnostics
        .extend(of_lists.into_iter().map(|(_, finding)| finding));
    // The files no list asked for are merged too, by path, and create nothing.
    let mut rest: Vec<String> = loading
        .copies
        .keys()
        .filter(|path| !loading.requested.contains(*path))
        .cloned()
        .collect();
    rest.sort_unstable();
    for path in rest {
        if path != BARRIER {
            loading.merged(&path);
            continue;
        }
        // The game never reads the file unless a list asks for it. It is merged from its
        // own copies only: nothing asked for the fallback.
        let own: Vec<&FileCopy> = loading.copies.get(BARRIER).into_iter().flatten().collect();
        let mut unread = Diagnostics::default();
        for copy in &own {
            let code = DiagnosticCode::IgnoredFile { text: None };
            unread.push(
                Diagnostic::info(code)
                    .phase(Phase::Validate)
                    .in_file(&copy.origin),
            );
        }
        if let Ok(key) = PackPath::new(&path) {
            let file = merge_file(&key, &own, unread);
            loading.files.insert(key, file);
        }
    }
    let files = loading
        .files
        .into_iter()
        .map(|(path, file)| (path, file.merged))
        .collect();
    Materials {
        lists,
        files,
        common,
        fancy,
        sad,
        rejected,
        diagnostics: loading.diagnostics,
    }
}

/// The loader of `rp.materials()`: the list files and every material file of every resource
/// pack, combined into the group.
pub const fn load_group() -> CollectionLoader<Materials, MaterialParsed> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}

impl Reported for Materials {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

/// One file of a test's walk, parsed: the file at `path` of the pack at stack index `pack`.
#[cfg(test)]
pub(super) fn parse_bytes(pack: usize, path: &str, bytes: &[u8]) -> MaterialParsed {
    use bedrock_addon_core::pack_identity::{PackIdVersion, PackType};
    let id = PackIdVersion::new(
        uuid::Uuid::from_u128(pack as u128 + 1),
        bedrock_addon_core::version::SemVersion::new(1, 0, 0),
        PackType::Invalid,
    );
    let path = PackPath::new(path).expect("a pack path");
    let file = WalkFile {
        pack: PackRef::new(pack, id, format!("pack {pack}")),
        side: PackSide::Resource,
        path: path.clone(),
        key: path,
        bytes: bytes::Bytes::copy_from_slice(bytes),
    };
    parse_walk_file(&file, &LoadContext::detached())
}

/// The group of a test's parsed files, in walk order (lowest pack first), over a stack that
/// lists no file.
#[cfg(test)]
pub(super) fn combine_parsed(parsed: Vec<MaterialParsed>) -> Materials {
    let options = bedrock_addon_core::load::LoadOptions::default();
    combine(parsed, &StackCx::new(&options, &[]))
}
