//! The checks over every loaded document, run once when the group combines: which type a block
//! belongs to and the tag hierarchy.

use indexmap::{IndexMap, IndexSet};

use super::validate::part_of_a_cycle;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Phase,
};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::id::{BlockId, Kind, PoiTagId, PoiTypeId, kind};

/// What the pass needs of one effective type document.
pub(crate) struct TypeRecord {
    /// The type.
    pub(crate) id: PoiTypeId,
    /// Its file.
    pub(crate) origin: Origin,
    /// `blocks`, as loaded.
    pub(crate) blocks: Vec<BlockId>,
    /// `tags`, as loaded.
    pub(crate) tags: Vec<PoiTagId>,
}

/// What the pass needs of one loaded tag document (every one: the documents of an identifier
/// are unioned).
pub(crate) struct TagRecord {
    /// The tag.
    pub(crate) id: PoiTagId,
    /// Its file.
    pub(crate) origin: Origin,
    /// `children`, as loaded.
    pub(crate) children: Vec<PoiTagId>,
    /// `parents`, as loaded.
    pub(crate) parents: Vec<PoiTagId>,
}

/// What the documents resolve to together.
#[derive(Debug, Default)]
pub(crate) struct Resolved {
    /// The type of each block, in the order the blocks were first claimed.
    pub(crate) blocks: IndexMap<BlockId, PoiTypeId>,
    /// Every tag, in the order they came to exist.
    pub(crate) tags: IndexMap<PoiTagId, TagNode>,
}

/// One tag of the hierarchy.
#[derive(Debug, Default)]
pub(crate) struct TagNode {
    /// The types that name the tag, in load order, each once.
    pub(crate) direct: IndexSet<PoiTypeId>,
    /// The types the tag holds: its own, then its narrower tags'.
    pub(crate) members: Vec<PoiTypeId>,
    /// The broader tags, in the order their edges were read.
    pub(crate) parents: Vec<PoiTagId>,
    /// The narrower tags, in the order their edges were read.
    pub(crate) children: Vec<PoiTagId>,
}

/// An edge of the hierarchy, by the tags' positions: `(narrower, broader)`.
type Edge = (usize, usize);

/// A finding at `path` of the document at `origin`, naming `id` of kind `K`.
fn finding<K: Kind>(
    code: DiagnosticCode,
    origin: &Origin,
    path: JsonPath,
    id: &bedrock_addon_core::id::Id<K>,
) -> Diagnostic {
    // The crate's own notes are informations: the game's release build ignores a tag that
    // names no tag in silence, and its registry overwrites a block's type without a word.
    Diagnostic::info(code)
        .at(path)
        .phase(Phase::Validate)
        .in_file(origin)
        .detail(DiagnosticDetail::Reference {
            kind: K::NAME,
            id: id.as_str().to_owned(),
            hint: None,
        })
}

/// The path of element `index` of the member `key` of the payload `payload`.
fn entry(payload: &str, key: &str, index: usize) -> JsonPath {
    let mut path = JsonPath::new();
    path.push(payload);
    path.push(key);
    path.push_index(index);
    path
}

/// Resolves the effective types, given in the order they were loaded, and every loaded tag
/// document, in walk order.
pub(crate) fn resolve(
    types: &[TypeRecord],
    tags: &[TagRecord],
    findings: &mut Diagnostics,
) -> Resolved {
    let mut resolved = Resolved::default();
    claim_blocks(types, &mut resolved, findings);
    create_tags(types, tags, &mut resolved);
    let edges = read_edges(tags, &resolved, findings);
    let edges = without_cycles(&edges, tags, &resolved, findings);
    let mut narrower = vec![Vec::new(); resolved.tags.len()];
    for &(child, parent) in &edges {
        narrower[parent].push(child);
        let (child_id, parent_id) = match (
            resolved.tags.get_index(child),
            resolved.tags.get_index(parent),
        ) {
            (Some((child_id, _)), Some((parent_id, _))) => (child_id.clone(), parent_id.clone()),
            _ => continue,
        };
        resolved.tags[child].parents.push(parent_id);
        resolved.tags[parent].children.push(child_id);
    }
    collect_members(&mut resolved.tags, &narrower);
    resolved
}

/// The edges the tag documents state, each once: `A.parents ∋ B` and `B.children ∋ A` are the
/// same edge. An entry naming a tag that does not exist is dropped with a finding and the tag
/// is not created (vanilla ignores it silently in a release build; the game's own unit test
/// of the case expects that the missing tag does not come to exist).
fn read_edges(
    tags: &[TagRecord],
    resolved: &Resolved,
    findings: &mut Diagnostics,
) -> IndexSet<Edge> {
    let mut edges = IndexSet::new();
    for record in tags {
        let Some(own) = resolved.tags.get_index_of(&record.id) else {
            continue;
        };
        for (key, list, own_is_narrower) in [
            ("children", &record.children, false),
            ("parents", &record.parents, true),
        ] {
            for (i, other) in list.iter().enumerate() {
                match resolved.tags.get_index_of(other) {
                    None => findings.push(finding::<kind::PoiTag>(
                        DiagnosticCode::UnresolvedReference { text: None },
                        &record.origin,
                        entry("minecraft:poi_tag", key, i),
                        other,
                    )),
                    // Never the tag itself: it lost that entry when its document loaded.
                    Some(other) if own_is_narrower => {
                        edges.insert((own, other));
                    }
                    Some(other) => {
                        edges.insert((other, own));
                    }
                }
            }
        }
    }
    edges
}

/// The edges that are part of no cycle. The edges between the tags of a cycle (a set of tags
/// that reach one another) are ignored, so those tags keep the types that name them and
/// inherit nothing through the cycle, as the game's own unit test of a dependency cycle
/// expects. An edge between such a tag and a tag outside the cycle stays; that case is the
/// crate's own reading. Each tag of a cycle is told so, at its first document when it has one;
/// vanilla's release build is silent about it.
fn without_cycles(
    edges: &IndexSet<Edge>,
    tags: &[TagRecord],
    resolved: &Resolved,
    findings: &mut Diagnostics,
) -> Vec<Edge> {
    let count = resolved.tags.len();
    let mut broader = vec![Vec::new(); count];
    for &(child, parent) in edges {
        broader[child].push(parent);
    }
    let component = components(&broader);
    let mut in_cycle = vec![false; count];
    let mut kept = Vec::with_capacity(edges.len());
    for &(child, parent) in edges {
        if component[child] == component[parent] {
            // Every tag of a cycle is the narrower end of one of its edges.
            in_cycle[child] = true;
        } else {
            kept.push((child, parent));
        }
    }
    for (tag, _) in resolved
        .tags
        .keys()
        .zip(in_cycle)
        .filter(|(_, in_cycle)| *in_cycle)
    {
        // The text is the game's unit test's; vanilla itself has only a development assert,
        // so the crate's own note is an information.
        let mut finding = Diagnostic::info(part_of_a_cycle(tag)).phase(Phase::Validate);
        if let Some(record) = tags.iter().find(|record| record.id == *tag) {
            let mut path = JsonPath::new();
            path.push("minecraft:poi_tag");
            path.push("description");
            path.push("identifier");
            finding = finding.at(path).in_file(&record.origin);
        }
        findings.push(finding);
    }
    kept
}

/// The strongly connected component of each node of the graph `successors` (Tarjan's
/// algorithm, without recursion): two nodes have the same number exactly when each reaches
/// the other.
pub(crate) fn components(successors: &[Vec<usize>]) -> Vec<usize> {
    const UNSEEN: usize = usize::MAX;
    let count = successors.len();
    let mut index = vec![UNSEEN; count];
    let mut low = vec![0; count];
    let mut component = vec![UNSEEN; count];
    let mut waiting = Vec::new();
    let mut on_waiting = vec![false; count];
    let mut next_index = 0;
    let mut next_component = 0;
    for root in 0..count {
        if index[root] != UNSEEN {
            continue;
        }
        let mut path = vec![(root, 0)];
        while let Some((node, next)) = path.last_mut() {
            let node = *node;
            if *next == 0 && index[node] == UNSEEN {
                index[node] = next_index;
                low[node] = next_index;
                next_index += 1;
                waiting.push(node);
                on_waiting[node] = true;
            }
            if let Some(&successor) = successors[node].get(*next) {
                *next += 1;
                if index[successor] == UNSEEN {
                    path.push((successor, 0));
                } else if on_waiting[successor] {
                    low[node] = low[node].min(index[successor]);
                }
                continue;
            }
            path.pop();
            if let Some(&(parent, _)) = path.last() {
                low[parent] = low[parent].min(low[node]);
            }
            if low[node] == index[node] {
                while let Some(member) = waiting.pop() {
                    on_waiting[member] = false;
                    component[member] = next_component;
                    if member == node {
                        break;
                    }
                }
                next_component += 1;
            }
        }
    }
    component
}

/// Fills each tag's members: the types that name it, then those of its narrower tags
/// (`narrower[i]` are the positions of tag `i`'s), each type once. Depth-first without
/// recursion, a tag after its narrower ones.
///
/// It ends on any graph: a tag is entered once (`open` while it is on the path, `done`
/// after), so the path is never longer than the tags and every edge is looked at once. The
/// caller removes the edges of cycles first; were one left, the edge back to a tag still on
/// the path is not followed and adds nothing.
pub(crate) fn collect_members(tags: &mut IndexMap<PoiTagId, TagNode>, narrower: &[Vec<usize>]) {
    let mut done = vec![false; tags.len()];
    let mut open = vec![false; tags.len()];
    for root in 0..tags.len() {
        if done[root] {
            continue;
        }
        open[root] = true;
        let mut path = vec![(root, 0)];
        while let Some((node, next)) = path.last_mut() {
            let node = *node;
            if let Some(&child) = narrower[node].get(*next) {
                *next += 1;
                if !done[child] && !open[child] {
                    open[child] = true;
                    path.push((child, 0));
                }
                continue;
            }
            open[node] = false;
            let mut members = tags[node].direct.clone();
            for &child in &narrower[node] {
                members.extend(tags[child].members.iter().cloned());
            }
            tags[node].members = members.into_iter().collect();
            done[node] = true;
            path.pop();
        }
    }
}

/// The tags that exist: the ones the types name, created on first use as vanilla's tag
/// registrar creates a builder (a tag only types name is a leaf tag, with no finding), then the
/// ones with a document. Each holds the types that name it.
fn create_tags(types: &[TypeRecord], tags: &[TagRecord], resolved: &mut Resolved) {
    for record in types {
        for tag in &record.tags {
            let node = resolved.tags.entry(tag.clone()).or_default();
            node.direct.insert(record.id.clone());
        }
    }
    for record in tags {
        resolved.tags.entry(record.id.clone()).or_default();
    }
}

/// One block belongs to one type, the one loaded last among those listing it; the type that
/// takes a block from another is told so (the crate's own note: vanilla overwrites silently).
fn claim_blocks(types: &[TypeRecord], resolved: &mut Resolved, findings: &mut Diagnostics) {
    for record in types {
        for (i, block) in record.blocks.iter().enumerate() {
            // Another type's: each effective type is here once, and its list holds a block
            // once.
            let earlier = resolved.blocks.insert(block.clone(), record.id.clone());
            if let Some(earlier) = earlier {
                findings.push(finding::<kind::PoiType>(
                    DiagnosticCode::PoiBlockClaimedTwice,
                    &record.origin,
                    entry("minecraft:poi_block", "blocks", i),
                    &earlier,
                ));
            }
        }
    }
}
