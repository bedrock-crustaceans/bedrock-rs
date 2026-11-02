//! One material group (the common, the fancy or the sad list's): the materials its files
//! create, parent first, and what each inherits from.

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use bedrock_addon_core::pack_identity::PackPath;
use indexmap::IndexMap;

use super::file::{KeyRules, MaterialDef, MaterialFile, MaterialFileForm, MaterialKey};
use super::list::MaterialListKind;
use super::set::NameSet;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, JsonPath, Location, Phase, Severity, VanillaText,
};

/// Which group of the client a [`MaterialGroup`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MaterialGroupKind {
    /// The group of `materials/common.json`, loaded in every graphics mode.
    Common,
    /// The switchable group as `materials/fancy.json` fills it (Fancy graphics).
    Fancy,
    /// The switchable group as `materials/sad.json` fills it (every other graphics mode).
    Sad,
}

impl MaterialGroupKind {
    /// The three groups.
    pub const ALL: [Self; 3] = [Self::Common, Self::Fancy, Self::Sad];

    /// The list file that fills the group.
    #[must_use]
    pub fn list(self) -> MaterialListKind {
        match self {
            Self::Common => MaterialListKind::Common,
            Self::Fancy => MaterialListKind::Fancy,
            Self::Sad => MaterialListKind::Sad,
        }
    }
}

/// One material a group created.
///
/// A link of a parent chain: it names its parent and holds nothing of it. What it inherits is
/// worked out by [`Materials::resolve`](super::Materials::resolve).
///
/// Two materials are equal when they are the same link: the same name, parent name, file, key
/// rules and place in the creation order, and the same variants of their own (the names the
/// material's `variants` or `+variants` put in, and whether `variants` dropped the inherited
/// ones). The parent is compared by its name and never followed, so what the two inherit is
/// not compared: comparing a link costs the link, whatever the length of its chain. Two equal
/// links can therefore have different variants, where the materials above them differ or
/// where a parent got a variant before the one was created from it and after the other was.
/// Equal [`MaterialGroup`]s leave no such room: a group also compares what every step of its
/// creation did to the variants, so the materials of two equal groups have the same variants.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Material {
    /// The material's name.
    pub name: String,
    /// The material it was created from, as its key names it; `None` for a material that
    /// starts from its list entry's defines.
    pub parent: Option<String>,
    /// The file that defines it; after a `+name` key of a later file extended it, that file.
    pub file: PackPath,
    /// The key rules its members are read with.
    pub key_rules: KeyRules,
    /// The variants it has, its own and the inherited ones.
    pub(super) variants: NameSet,
    /// Its place in the group's creation order.
    pub(super) node: u32,
    /// What the material did to the variants it inherits.
    pub(super) own: OwnVariants,
}

/// What `variants` and `+variants` did to a variant set: of a material, the part of its set
/// that is the link's own, over every definition that made it; of one step of a group's
/// creation, what that step's definition did to the set it started from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct OwnVariants {
    /// Whether `variants` emptied the inherited set first.
    replaced: bool,
    /// The names put in that were not there, in the order they were written.
    added: Vec<String>,
}

impl OwnVariants {
    /// What was done once `step` is done after it.
    fn then(&mut self, step: &Self) {
        if step.replaced {
            self.clone_from(step);
        } else {
            self.added.extend(step.added.iter().cloned());
        }
    }
}

/// The link, not what it inherits: see the type.
impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
            && self.key_rules == other.key_rules
            && self.name == other.name
            && self.parent == other.parent
            && self.file == other.file
            && self.own == other.own
    }
}

impl Material {
    /// Whether the material has the variant `variant`, so that `<name>.<variant>` is a name of
    /// the group. A child has its parent's variants unless its own `variants` replaces them.
    #[must_use]
    pub fn has_variant(&self, variant: &str) -> bool {
        self.variants.contains(variant)
    }

    /// The material's variants, bytewise.
    pub fn variants(&self) -> impl Iterator<Item = &str> + '_ {
        self.variants.iter()
    }

    /// How many variants the material has.
    #[must_use]
    pub fn variant_count(&self) -> usize {
        self.variants.len()
    }
}

/// What a name of the group stands for.
#[derive(Debug, PartialEq)]
pub(super) enum Slot {
    /// A material.
    Created(Material),
    /// A material that was refused (it carries a keyword the client cannot load): the name
    /// stays taken, so that no later definition stands in for it.
    Refused,
}

/// What one step of a parent chain applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Body {
    /// The defines of a list entry: where a parentless material starts.
    Base(u32),
    /// The members of the material `entry` of the file `file` (indices into
    /// `Materials::files` and its `materials`).
    Def {
        /// The file.
        file: u32,
        /// The material in it.
        entry: u32,
    },
}

/// One step of a parent chain: a list entry's defines, or one definition applied onto the
/// material an earlier step left.
///
/// The variant set a step leaves is the set of the step before it, emptied if the step
/// `replaced` it, with the names the step `added`: a function of the steps alone. Two groups
/// with the same steps therefore give every material the same variants.
///
/// The key rules a step's members are read with are those of its own definition, not of the
/// material it ends in: a `+name` key of a wrapped file leaves a material with the versioned
/// rules whatever the form of the file that created it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Node {
    /// The step before it.
    pub(super) parent: Option<u32>,
    /// What it applies.
    pub(super) body: Body,
    /// The key rules its definition is read with; `None` for a list entry's defines.
    key_rules: Option<KeyRules>,
    /// What it did to the variants of the step before it.
    variants: OwnVariants,
}

/// What a step of a parent chain is made of, as [`MaterialGroup::push_node`] takes it.
struct Step {
    /// What it applies.
    body: Body,
    /// The key rules its definition is read with.
    key_rules: Option<KeyRules>,
    /// What it did to the variants of the step before it.
    variants: OwnVariants,
}

/// The materials one list creates.
///
/// A name is taken by the first file that defines it: a later file's definition of the same
/// name is ignored unless its key starts with `+`, which applies it on top.
#[derive(Debug)]
#[non_exhaustive]
pub struct MaterialGroup {
    /// Which group.
    pub kind: MaterialGroupKind,
    /// The names in creation order.
    pub(super) slots: IndexMap<String, Slot>,
    /// The steps of every parent chain.
    pub(super) nodes: Vec<Node>,
    /// The defines of each list entry that loaded a file.
    pub(super) bases: Vec<Vec<String>>,
    /// The fingerprint and the length of every material's name: what a part of a queried name
    /// is tested against before it is looked up. Ordered, so that two loads print the same.
    pub(super) fingerprints: BTreeSet<(u64, usize)>,
}

/// The fingerprint of no text: the FNV-1a offset basis.
const FINGERPRINT_START: u64 = 0xcbf2_9ce4_8422_2325;

/// The fingerprint of a text one byte longer than the text whose fingerprint is `before`
/// (FNV-1a): the fingerprint of every prefix of a name comes out of one pass over the name.
fn fingerprint_with(before: u64, byte: u8) -> u64 {
    (before ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
}

/// The fingerprint of `text`.
fn fingerprint(text: &str) -> u64 {
    text.bytes().fold(FINGERPRINT_START, fingerprint_with)
}

/// One merged file as a group loads it.
pub(super) struct FileSource<'a> {
    /// The file's index in `Materials::files`.
    pub(super) index: u32,
    /// The file's path.
    pub(super) path: &'a PackPath,
    /// The merged file.
    pub(super) file: &'a MaterialFile,
    /// Per material: whether it is refused.
    pub(super) refused: &'a [bool],
}

/// The same materials created by the same steps from the same list entries.
///
/// Each material is compared once, as a link (see [`Material`]), and each step of the
/// creation once, with the key rules its definition is read with and what it did to the
/// variants it started from: which names it added and whether it dropped the inherited ones.
/// No variant set is read, so the cost is the size of the two groups, whatever the depth of
/// their chains.
///
/// Equal groups have the same [`names`](Self::names), variants included: a material's variants
/// are those of the step it was created from with what its own steps did, so equal steps give
/// equal sets. That a parent has the same variants in the end is not enough: a child copies
/// them when it is created, and a `+name` key of a later file adds to the parent alone.
///
/// Group equality covers the names, the inheritance structure (the step each step and each
/// material starts from), what each step did to the variants and the key rules each step is
/// read with; it does not cover the members of the definitions, of which a group holds only
/// the place in [`Materials::files`](super::Materials::files): to compare what the materials
/// are, compare the files of [`Materials::files`](super::Materials::files) as well. The
/// materials of two equal groups whose definitions have the same members at the same places
/// resolve alike ([`Materials::resolve_all`](super::Materials::resolve_all)), whichever form,
/// wrapped or flat, each file is written in.
impl PartialEq for MaterialGroup {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.bases == other.bases
            && self.nodes == other.nodes
            && self.slots.len() == other.slots.len()
            && self.slots.iter().eq(other.slots.iter())
    }
}

impl MaterialGroup {
    /// An empty group.
    pub(super) fn new(kind: MaterialGroupKind) -> Self {
        Self {
            kind,
            slots: IndexMap::new(),
            nodes: Vec::new(),
            bases: Vec::new(),
            fingerprints: BTreeSet::new(),
        }
    }

    /// Puts `slot` under `name`.
    fn insert(&mut self, name: &str, slot: Slot) {
        if matches!(slot, Slot::Created(_)) {
            self.fingerprints.insert((fingerprint(name), name.len()));
        }
        self.slots.insert(name.to_owned(), slot);
    }

    /// The material `name`.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Material> {
        match self.slots.get(name)? {
            Slot::Created(material) => Some(material),
            Slot::Refused => None,
        }
    }

    /// Whether `name` is a name of the group: a material, or `<material>.<variant>`.
    ///
    /// A material's name may hold dots itself, so the name is split at each of its dots. The
    /// cost is the length of `name`: its part before a dot is looked up only when its
    /// fingerprint, carried along the name byte by byte, and its length are those of a
    /// material. The lookup that follows is what decides: a part that has the fingerprint and
    /// the length of a material without being one costs its own length once more.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        if self.get(name).is_some() {
            return true;
        }
        let mut before = FINGERPRINT_START;
        for (at, byte) in name.bytes().enumerate() {
            if byte == b'.'
                && self.fingerprints.contains(&(before, at))
                && self
                    .get(&name[..at])
                    .is_some_and(|material| material.has_variant(&name[at + 1..]))
            {
                return true;
            }
            before = fingerprint_with(before, byte);
        }
        false
    }

    /// The materials in creation order.
    pub fn iter(&self) -> impl Iterator<Item = &Material> + '_ {
        self.slots.values().filter_map(|slot| match slot {
            Slot::Created(material) => Some(material),
            Slot::Refused => None,
        })
    }

    /// How many materials the group has, variants not counted.
    #[must_use]
    pub fn len(&self) -> usize {
        self.iter().count()
    }

    /// Whether the group has no material.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }

    /// Every name of the group: each material in creation order, followed by its
    /// `<name>.<variant>` names. The cost is the number of names, which a long chain of
    /// materials that each add a variant makes quadratic in the chain.
    pub fn names(&self) -> impl Iterator<Item = String> + '_ {
        self.iter().flat_map(|material| {
            let variants = material
                .variants()
                .map(|variant| format!("{}.{variant}", material.name));
            std::iter::once(material.name.clone()).chain(variants)
        })
    }

    /// Creates the materials of one merged file, parent first, starting the parentless ones
    /// from `base_defines`. The findings come back with the index of the material each is
    /// about.
    ///
    /// The file's keys form a tree by parent name (a `+name` key stands for `name`); it is
    /// walked breadth first from the keys without a parent, the keys of a level bytewise, as
    /// the client's sorted map holds them. A key the walk does not reach, because its parent
    /// is no key of this file or because it sits on a cycle, creates nothing. The cost
    /// is linear in the file, times the logarithm of a variant set.
    pub(super) fn load_file(
        &mut self,
        source: &FileSource<'_>,
        base_defines: Vec<String>,
        out: &mut Vec<(usize, Diagnostic)>,
    ) {
        let base = self.push_node(
            None,
            Step {
                body: Body::Base(index_of(self.bases.len())),
                key_rules: None,
                variants: OwnVariants::default(),
            },
        );
        self.bases.push(base_defines);
        let file = source.file;
        if let MaterialFileForm::Single { .. } = file.form {
            self.load_single(source, base);
            return;
        }
        let texts: Vec<String> = file.materials.keys().map(ToString::to_string).collect();
        let mut order: Vec<usize> = (0..texts.len()).collect();
        order.sort_unstable_by(|a, b| texts[*a].cmp(&texts[*b]));
        let mut children: HashMap<&str, Vec<usize>> = HashMap::new();
        let mut queue = VecDeque::new();
        for &index in &order {
            let Some((key, _)) = file.materials.get_index(index) else {
                continue;
            };
            match &key.parent {
                None => queue.push_back(index),
                Some(parent) => children.entry(parent.as_str()).or_default().push(index),
            }
        }
        let mut visited = vec![false; texts.len()];
        // The names this file created: a second key of one name in the file is a duplicate.
        let mut created: HashSet<&str> = HashSet::new();
        while let Some(index) = queue.pop_front() {
            let Some((key, def)) = file.materials.get_index(index) else {
                continue;
            };
            visited[index] = true;
            let visit = Visit {
                source,
                index,
                key,
                def,
                text: &texts[index],
                base,
            };
            if self.visit(&visit, created.contains(key.name.as_str()), out) {
                created.insert(key.name.as_str());
            }
            // The children hang under the name, whichever key of that name comes first.
            if let Some(kids) = children.remove(key.name.as_str()) {
                queue.extend(kids);
            }
        }
        for index in order {
            if visited[index] {
                continue;
            }
            let Some((key, _)) = file.materials.get_index(index) else {
                continue;
            };
            let parent = key.parent.as_deref().unwrap_or_default();
            out.push((
                index,
                unresolved(source.path, &[texts[index].as_str()], MATERIAL, parent),
            ));
        }
    }

    /// Appends a step and gives its index.
    fn push_node(&mut self, parent: Option<u32>, step: Step) -> u32 {
        let index = index_of(self.nodes.len());
        self.nodes.push(Node {
            parent,
            body: step.body,
            key_rules: step.key_rules,
            variants: step.variants,
        });
        index
    }

    /// The one material of a single-material file: created from the list entry's defines and
    /// stored under the file's name, replacing any material of that name.
    fn load_single(&mut self, source: &FileSource<'_>, base: u32) {
        let Some((key, def)) = source.file.materials.get_index(0) else {
            return;
        };
        let slot = if source.refused.first().copied().unwrap_or(false) {
            Slot::Refused
        } else {
            let body = Body::Def {
                file: source.index,
                entry: 0,
            };
            let key_rules = def.key_rules.unwrap_or(KeyRules::Legacy);
            let step = Step {
                body,
                key_rules: Some(key_rules),
                variants: OwnVariants::default(),
            };
            Slot::Created(Material {
                name: key.name.clone(),
                parent: None,
                file: source.path.clone(),
                key_rules,
                variants: NameSet::default(),
                node: self.push_node(Some(base), step),
                own: OwnVariants::default(),
            })
        };
        self.insert(&key.name, slot);
    }

    /// Visits one key of a file's tree; whether the file created or extended the material.
    /// `again` says that this file already did so under the same name.
    fn visit(
        &mut self,
        visit: &Visit<'_>,
        again: bool,
        out: &mut Vec<(usize, Diagnostic)>,
    ) -> bool {
        let Visit {
            source,
            index,
            key,
            def,
            text,
            base,
        } = *visit;
        let warn = |code| at_key(source.path, text, code, Severity::Info);
        // A child copies the material the group has under its parent's name.
        let (parent_node, inherited) = match &key.parent {
            None => (base, NameSet::default()),
            Some(name) => {
                let Some(parent) = self.get(name) else {
                    out.push((index, unresolved(source.path, &[text], MATERIAL, name)));
                    return false;
                };
                (parent.node, parent.variants.clone())
            }
        };
        let refused = source.refused.get(index).copied().unwrap_or(false);
        let body = Body::Def {
            file: source.index,
            entry: index_of(index),
        };
        let key_rules = def
            .key_rules
            .unwrap_or_else(|| source.file.form.key_rules());
        match self.slots.get(key.name.as_str()) {
            // The name is taken by a refused material: nothing stands in for it.
            Some(Slot::Refused) => {
                let finding = if key.extend {
                    unresolved(source.path, &[text], MATERIAL, &key.name)
                } else {
                    warn(DiagnosticCode::DuplicateIdentifier { text: None })
                };
                out.push((index, finding));
                false
            }
            // The first definition of a name stays; the game ignores the later one in
            // silence.
            Some(Slot::Created(_)) if !key.extend => {
                if !again {
                    let code = DiagnosticCode::DuplicateIdentifier { text: None };
                    out.push((index, warn(code)));
                }
                false
            }
            // A refused body creates nothing and extends nothing.
            Some(Slot::Created(_)) if refused => false,
            // `+name`: the body is applied onto the material the group has.
            Some(Slot::Created(existing)) => {
                let (node, variants) = (existing.node, existing.variants.clone());
                let mut own = existing.own.clone();
                let mut did = OwnVariants::default();
                let variants =
                    apply_variants(variants, &mut did, def, source.path, text, index, out);
                own.then(&did);
                let step = Step {
                    body,
                    key_rules: Some(key_rules),
                    variants: did,
                };
                let node = self.push_node(Some(node), step);
                if let Some(Slot::Created(material)) = self.slots.get_mut(key.name.as_str()) {
                    material.node = node;
                    material.variants = variants;
                    material.own = own;
                    material.file = source.path.clone();
                    material.key_rules = key_rules;
                }
                true
            }
            None if refused => {
                self.insert(&key.name, Slot::Refused);
                false
            }
            None => {
                let mut own = OwnVariants::default();
                let variants =
                    apply_variants(inherited, &mut own, def, source.path, text, index, out);
                let step = Step {
                    body,
                    key_rules: Some(key_rules),
                    variants: own.clone(),
                };
                let material = Material {
                    name: key.name.clone(),
                    parent: key.parent.clone(),
                    file: source.path.clone(),
                    key_rules,
                    variants,
                    node: self.push_node(Some(parent_node), step),
                    own,
                };
                self.insert(&key.name, Slot::Created(material));
                true
            }
        }
    }
}

/// One key of a file's tree as [`MaterialGroup::visit`] sees it.
#[derive(Clone, Copy)]
struct Visit<'a> {
    source: &'a FileSource<'a>,
    index: usize,
    key: &'a MaterialKey,
    def: &'a MaterialDef,
    text: &'a str,
    base: u32,
}

/// The kind a reference to a material is reported with.
const MATERIAL: &str = "material";
/// The kind a reference to a variant is reported with.
const VARIANT: &str = "material variant";

/// The client's engine-log text for a variant that replaces one of the same name.
pub(super) const VARIANT_REPLACED: &str = "Variant: {} was replaced with a new version";

/// An index as the group stores it.
fn index_of(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

/// A finding at `segments` of the file `path`.
fn at(
    path: &PackPath,
    segments: &[Segment<'_>],
    code: DiagnosticCode,
    severity: Severity,
) -> Diagnostic {
    let mut json_path = JsonPath::new();
    for segment in segments {
        match segment {
            Segment::Key(key) => json_path.push(key),
            Segment::Index(index) => json_path.push_index(*index),
        }
    }
    let location = Location::new(None, Some(path.clone()), json_path, None);
    Diagnostic::new(code, severity, Phase::Validate, location)
}

/// One step of a place in a file.
enum Segment<'a> {
    Key(&'a str),
    Index(usize),
}

/// A finding at the key `text` of the file `path`.
fn at_key(path: &PackPath, text: &str, code: DiagnosticCode, severity: Severity) -> Diagnostic {
    at(path, &[Segment::Key(text)], code, severity)
}

/// `id`, a `kind`, resolves to nothing; reported at the keys `keys`.
fn unresolved(path: &PackPath, keys: &[&str], kind: &'static str, id: &str) -> Diagnostic {
    let segments: Vec<Segment<'_>> = keys.iter().map(|key| Segment::Key(key)).collect();
    let code = DiagnosticCode::UnresolvedReference { text: None };
    let mut finding = at(path, &segments, code, Severity::Info);
    finding.detail = Some(Box::new(DiagnosticDetail::Reference {
        kind,
        id: id.to_owned(),
        hint: None,
    }));
    finding
}

/// The variants a material has once `def` is applied onto the `inherited` ones: `variants`
/// empties them first, `+variants` adds to them, and `variants` is the one read when both are
/// written. A variant of a name that is already there replaces it; a dotted name whose
/// part before the last dot is no variant yet is not created. `own` is told what was
/// done.
fn apply_variants(
    inherited: NameSet,
    own: &mut OwnVariants,
    def: &MaterialDef,
    path: &PackPath,
    text: &str,
    index: usize,
    out: &mut Vec<(usize, Diagnostic)>,
) -> NameSet {
    let (mut variants, list, key) = match (&def.variants.replace, &def.variants.add) {
        (Some(list), _) => {
            *own = OwnVariants {
                replaced: true,
                added: Vec::new(),
            };
            (NameSet::default(), list, "variants")
        }
        (None, Some(list)) => (inherited, list, "+variants"),
        (None, None) => return inherited,
    };
    for (element, variant) in list.iter().enumerate() {
        for name in variant.0.keys() {
            let place = [
                Segment::Key(text),
                Segment::Key(key),
                Segment::Index(element),
                Segment::Key(name),
            ];
            if let Some((outer, _)) = name.rsplit_once('.')
                && !variants.contains(outer)
            {
                let code = DiagnosticCode::UnresolvedReference { text: None };
                let mut finding = at(path, &place, code, Severity::Info);
                finding.detail = Some(Box::new(DiagnosticDetail::Reference {
                    kind: VARIANT,
                    id: outer.to_owned(),
                    hint: None,
                }));
                out.push((index, finding));
                continue;
            }
            let (with, was_there) = variants.with(name);
            if was_there {
                let text = VanillaText::Formatted {
                    template: VARIANT_REPLACED,
                    args: vec![name.clone()],
                };
                let code = DiagnosticCode::ShadowedDefinition { text: Some(text) };
                out.push((index, at(path, &place, code, Severity::Info)));
            } else {
                own.added.push(name.clone());
            }
            variants = with;
        }
    }
    variants
}
