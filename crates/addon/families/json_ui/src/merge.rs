//! The cross-pack merge of one UI file, as the game merges the copies of a file across packs.
//!
//! The lowest-priority copy is the base; each higher copy is merged per top-level control,
//! its `modifications` lists are applied, and the copy is committed only when it raised no
//! error.

use indexmap::IndexMap;

use super::value::{element_name, first_key, member, short_name, sorted_members, string_or_empty};
use super::{UiPackError, UiPackErrorKind};
use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::json::{MAX_JSON_DEPTH, Value};

/// One pack's copy of a file, lowest priority first in a file's stack.
#[derive(Debug)]
pub(crate) struct FileCopy {
    /// Which pack: an index the caller gives meaning to.
    pub slot: usize,
    /// What the copy holds.
    pub content: CopyContent,
}

/// The content of one copy.
#[derive(Clone, Debug)]
pub(crate) enum CopyContent {
    /// No bytes at all: an overlay copy like this is skipped.
    Empty,
    /// The parsed JSON.
    Json(Value),
    /// The reader's message for a copy that is not JSON.
    Malformed(String),
}

/// Something the merge has to say about one copy.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Event {
    /// The base copy does not parse; the file contributes nothing.
    BaseMalformed(String),
    /// Empty or duplicate child names in the base copy.
    BaseInvalidNames(Vec<String>),
    /// A `uiPackError`.
    Pack(UiPackError),
    /// A top-level overlay member whose value is not an object.
    IgnoredValue(String),
    /// `modifications` below a control's top level; the overlay key and the member path.
    NestedModifications(String),
    /// An overlay `name@base` key whose short name the base already has.
    Rebased(String),
    /// A `where` / `target` condition that matched nothing; the overlay key.
    Unmatched(String),
    /// An array-valued `where`; the overlay key.
    WhereArray(String),
}

/// An [`Event`] and the copy it is about.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Raised {
    /// The copy's slot.
    pub slot: usize,
    /// What happened.
    pub event: Event,
}

/// The result of merging the copies of one file.
#[derive(Debug)]
pub(crate) struct MergedFile {
    /// The merged root; `None` when the base copy did not parse.
    pub root: Option<Value>,
    /// The slots whose copy is part of the result, lowest first (the base, then every
    /// committed overlay).
    pub committed: Vec<usize>,
    /// Everything raised, in order.
    pub raised: Vec<Raised>,
}

/// Merges the copies of file `name`, lowest priority first.
pub(crate) fn merge_file(name: &str, copies: Vec<FileCopy>) -> MergedFile {
    let mut out = MergedFile {
        root: None,
        committed: Vec::new(),
        raised: Vec::new(),
    };
    let mut copies = copies.into_iter();
    let Some(base) = copies.next() else {
        return out;
    };
    let mut root = match base.content {
        CopyContent::Json(json) => json,
        // An empty base is not JSON either: jsoncpp's reader fails on an empty document.
        CopyContent::Empty => {
            out.raise(base.slot, Event::BaseMalformed("empty document".to_owned()));
            return out;
        }
        CopyContent::Malformed(message) => {
            out.raise(base.slot, Event::BaseMalformed(message));
            return out;
        }
    };
    out.committed.push(base.slot);
    let base_invalid = invalid_names(&root);
    if !base_invalid.is_empty() {
        out.raise(base.slot, Event::BaseInvalidNames(base_invalid.clone()));
    }
    // The short-name map is built from the base copy only.
    let mut short_names: IndexMap<String, String> = IndexMap::new();
    for (key, value) in sorted_members(&root) {
        if value.is_object() {
            short_names
                .entry(short_name(key).to_owned())
                .or_insert_with(|| key.to_owned());
        }
    }
    for (index, copy) in copies.enumerate() {
        let overlay = match copy.content {
            CopyContent::Empty => continue,
            CopyContent::Malformed(message) => {
                out.error(
                    copy.slot,
                    UiPackErrorKind::ParseError,
                    vec![name.to_owned(), message],
                );
                continue;
            }
            CopyContent::Json(json) => json,
        };
        let mut pass = Pass {
            file: name,
            slot: copy.slot,
            // The overlay index of `texture:<i>`: 1 for the first copy above the base.
            index: index + 1,
            short_names: &short_names,
            raised: Vec::new(),
            errors: 0,
        };
        let mut working = root.clone();
        let mut lists = pass.merge_controls(&mut working, &overlay);
        // Deepest path first; equal depths in the overlay's member order.
        lists.sort_by_key(|list| std::cmp::Reverse(list.depth));
        for list in &lists {
            pass.apply_list(&mut working, list);
        }
        // A path key grafts a copy's levels below the ones the file already has. The merged
        // file is held to the limit of a single copy, so that every walk over it is bounded;
        // the copy that crosses it is refused as a copy that is too deep itself is. The game has no
        // such limit; it is this crate's own bound.
        let levels = depth(&working);
        let introduced: Vec<String> = if levels > MAX_JSON_DEPTH {
            pass.error(
                UiPackErrorKind::ParseError,
                vec![format!(
                    "nesting too deep after the merge: {levels} levels, at most {MAX_JSON_DEPTH}"
                )],
            );
            Vec::new()
        } else {
            invalid_names(&working)
                .into_iter()
                .filter(|name| !base_invalid.contains(name))
                .collect()
        };
        if !introduced.is_empty() {
            // The game reports empty or repeated child names the overlay introduced as a pack
            // warning, not in the content log, and keeps the copy. The warning reaches the
            // player: it is merged into the pack's report, which the pack list and the pack
            // validation screens show. A warning the player sees stays a warning here.
            pass.raised.push(Raised {
                slot: copy.slot,
                event: Event::Pack(UiPackError {
                    kind: UiPackErrorKind::InvalidChildNames,
                    args: vec![name.to_owned(), names_argument(&introduced)],
                    severity: Severity::Warning,
                }),
            });
        }
        let Pass { raised, errors, .. } = pass;
        out.raised.extend(raised);
        // Commit or roll back: one error discards the pack's whole copy of the file.
        if errors == 0 {
            root = working;
            out.committed.push(copy.slot);
        }
    }
    out.root = Some(root);
    out
}

impl MergedFile {
    fn raise(&mut self, slot: usize, event: Event) {
        self.raised.push(Raised { slot, event });
    }

    fn error(&mut self, slot: usize, kind: UiPackErrorKind, args: Vec<String>) {
        self.raise(
            slot,
            Event::Pack(UiPackError {
                kind,
                args,
                severity: Severity::Error,
            }),
        );
    }
}

/// How many arrays and objects nest in `value`, counted without recursion.
fn depth(value: &Value) -> usize {
    let mut deepest = 0;
    let mut pending = vec![(value, 1)];
    while let Some((value, level)) = pending.pop() {
        if let Some(object) = value.as_object() {
            deepest = deepest.max(level);
            pending.extend(object.values().map(|member| (member, level + 1)));
        } else if let Some(array) = value.as_array() {
            deepest = deepest.max(level);
            pending.extend(array.as_slice().iter().map(|element| (element, level + 1)));
        }
    }
    deepest
}

/// The names as the `invalidChildNames` text takes them: one per line.
// The text runs on without a space after the argument ("…names: %sIn the future…"), so the
// argument ends its own line.
fn names_argument(names: &[String]) -> String {
    names.iter().fold(String::new(), |mut all, name| {
        all.push_str(name);
        all.push('\n');
        all
    })
}

/// The child names that are empty or repeated among their siblings, in every `controls` array
/// below the top-level members of `root`, each once, in the order found.
pub(crate) fn invalid_names(root: &Value) -> Vec<String> {
    fn walk(control: &Value, found: &mut Vec<String>) {
        let Some(children) = member(control, "controls").and_then(Value::as_array) else {
            return;
        };
        let mut seen: Vec<&str> = Vec::new();
        for child in children.as_slice() {
            let Some(key) = first_key(child) else {
                continue;
            };
            let name = short_name(key);
            if (name.is_empty() || seen.contains(&name)) && !found.iter().any(|f| f == name) {
                found.push(name.to_owned());
            }
            seen.push(name);
            if let Some(body) = child.as_object().and_then(|object| object.get(key)) {
                walk(body, found);
            }
        }
    }
    let mut found = Vec::new();
    for (_, control) in sorted_members(root) {
        walk(control, &mut found);
    }
    found
}

/// One overlay member's `modifications` array, waiting to be applied: the control it edits,
/// the path to it and the depth of the overlay key decide the order the lists run in.
struct ModificationList {
    /// The member of the working root the control lives in.
    top: String,
    /// The element index in each `controls` array on the way down to the control.
    path: Vec<usize>,
    /// The `modifications` value.
    list: Value,
    /// The overlay key, as written.
    key: String,
    /// The number of `/` in the key.
    depth: usize,
}

/// The merge of one overlay copy.
struct Pass<'a> {
    file: &'a str,
    slot: usize,
    index: usize,
    short_names: &'a IndexMap<String, String>,
    raised: Vec<Raised>,
    errors: usize,
}

impl Pass<'_> {
    fn raise(&mut self, event: Event) {
        self.raised.push(Raised {
            slot: self.slot,
            event,
        });
    }

    /// Reports a `uiPackError` as an error: the copy will be rolled back.
    fn error(&mut self, kind: UiPackErrorKind, mut args: Vec<String>) {
        args.insert(0, self.file.to_owned());
        self.errors += 1;
        self.raise(Event::Pack(UiPackError {
            kind,
            args,
            severity: Severity::Error,
        }));
    }

    /// Merges every member of the overlay onto the working root.
    fn merge_controls(&mut self, working: &mut Value, overlay: &Value) -> Vec<ModificationList> {
        let mut lists = Vec::new();
        for (key, value) in sorted_members(overlay) {
            if value.is_null() {
                continue;
            }
            let (top, path) = if key.contains('/') {
                let Some(found) = self.descend(working, key) else {
                    self.error(UiPackErrorKind::MissingControl, vec![key.to_owned()]);
                    continue;
                };
                found
            } else {
                let short = short_name(key);
                let top = self.short_names.get(key).map_or(key, String::as_str);
                if key.contains('@') && self.short_names.get(short).is_some_and(|full| full != key)
                {
                    self.raise(Event::Rebased(key.to_owned()));
                }
                (top.to_owned(), Vec::new())
            };
            // Only object values have an effect.
            if !value.is_object() {
                self.raise(Event::IgnoredValue(key.to_owned()));
                continue;
            }
            let depth = path.len();
            let Some(target) = locate_or_create(working, &top, &path) else {
                continue;
            };
            if target.is_object() {
                let mut found = None;
                self.recursive_merge(target, value, key, &mut Some(&mut found));
                if let Some(list) = found {
                    lists.push(ModificationList {
                        top,
                        path,
                        list,
                        key: key.to_owned(),
                        depth,
                    });
                }
            } else {
                // Null or not an object: replaced by the overlay object (a new control).
                *target = value.clone();
            }
        }
        lists
    }

    /// Resolves a `root/child/grandchild` key: the member of the working root and the element
    /// index in each `controls` array below it.
    fn descend(&self, working: &Value, key: &str) -> Option<(String, Vec<usize>)> {
        let mut segments = key.split('/');
        let top = self.short_names.get(segments.next()?)?.clone();
        let mut node = working.as_object()?.get(&top)?;
        let mut path = Vec::new();
        for segment in segments {
            let children = member(node, "controls")?.as_array()?;
            let index = children
                .as_slice()
                .iter()
                .position(|child| element_name(child) == Some(segment))?;
            let child = children.get(index)?;
            node = child.as_object()?.get(first_key(child)?)?;
            path.push(index);
        }
        Some((top, path))
    }

    /// Merges `src` onto `dest` member by member. `top` is `Some` at the top
    /// level of a control, where a `modifications` member is collected instead of dropped.
    fn recursive_merge(
        &mut self,
        dest: &mut Value,
        src: &Value,
        key: &str,
        top: &mut Option<&mut Option<Value>>,
    ) {
        let has_uv = member(src, "uv").is_some() || member(src, "uv_size").is_some();
        let Some(dest) = dest.as_object_mut() else {
            return;
        };
        for (name, value) in sorted_members(src) {
            if value.is_null() {
                continue;
            }
            if name == "modifications" {
                match top {
                    Some(found) => **found = Some(value.clone()),
                    None => self.raise(Event::NestedModifications(key.to_owned())),
                }
                continue;
            }
            if name == "texture" && has_uv {
                // Keeps the overriding texture with the UV rectangle of its own pack.
                dest.insert(format!("texture:{}", self.index), value.clone());
                continue;
            }
            match dest.get_mut(name) {
                Some(existing) if existing.is_object() && value.is_object() => {
                    self.recursive_merge(existing, value, key, &mut None);
                }
                _ => {
                    dest.insert(name, value.clone());
                }
            }
        }
    }

    /// Applies one `modifications` list to its control, rebuilding each edited array once.
    fn apply_list(&mut self, working: &mut Value, list: &ModificationList) {
        // `modifications` must be an array; otherwise the list is ignored.
        let Some(entries) = list.list.as_array() else {
            return;
        };
        let Some(control) = locate(working, &list.top, &list.path) else {
            return;
        };
        // One edit plan per array, in the order the arrays were first named.
        let mut edits: IndexMap<String, ArrayEdit> = IndexMap::new();
        for entry in entries.as_slice() {
            self.apply_entry(control, entry, &list.key, &mut edits);
        }
        let Some(control) = control.as_object_mut() else {
            return;
        };
        for (array_name, edit) in edits {
            if let Some(rebuilt) = edit.rebuild() {
                control.insert(array_name, rebuilt);
            }
        }
    }

    /// One element of a `modifications` array.
    fn apply_entry(
        &mut self,
        control: &Value,
        entry: &Value,
        key: &str,
        edits: &mut IndexMap<String, ArrayEdit>,
    ) {
        let get = |name: &str| member(entry, name);
        let here = |extra: &[&str]| -> Vec<String> {
            std::iter::once(key)
                .chain(extra.iter().copied())
                .map(str::to_owned)
                .collect()
        };
        // A string `control_name` is a name condition and `where` is then not read.
        let (condition, array_name) = match get("control_name").and_then(Value::as_string) {
            Some(name) => {
                let array = string_or_empty(get("array_name"));
                let array = if array.is_empty() { "controls" } else { array };
                (Condition::Name(name.as_str()), array)
            }
            None => (
                get("where").map_or(Condition::None, Condition::Where),
                string_or_empty(get("array_name")),
            ),
        };
        if array_name.is_empty() {
            return self.error(UiPackErrorKind::MissingArrayName, here(&[]));
        }
        let operation_name = string_or_empty(get("operation"));
        if operation_name.is_empty() {
            return self.error(UiPackErrorKind::MissingOperation, here(&[]));
        }
        let Some(operation) = Operation::from_name(operation_name) else {
            return self.error(
                UiPackErrorKind::InvalidOperationName,
                here(&[operation_name]),
            );
        };
        let value = get("value");
        if operation.needs_value() && value.is_none() {
            return self.error(UiPackErrorKind::MissingValue, here(&[operation.name()]));
        }
        let edit = edits.entry(array_name.to_owned()).or_insert_with(|| {
            ArrayEdit::new(
                member(control, array_name)
                    .and_then(Value::as_array)
                    .map_or(Vec::new(), |array| array.as_slice().to_vec()),
            )
        });
        let inserted = || Slot::Inserted(value.cloned().unwrap_or(Value::NULL));
        match operation {
            Operation::InsertBack => return edit.slots.push(inserted()),
            Operation::InsertFront => return edit.slots.insert(0, inserted()),
            _ => {}
        }
        let Some(index) = self.find(edit, &condition, key, operation) else {
            return;
        };
        let other = if operation.needs_target() {
            let target = match get("target_control").and_then(Value::as_string) {
                Some(name) => Condition::Name(name.as_str()),
                None => get("target").map_or(Condition::None, Condition::Where),
            };
            match self.find(edit, &target, key, operation) {
                Some(other) => other,
                None => return,
            }
        } else {
            index
        };
        let (Some(at), Some(other_at)) = (edit.position(index), edit.position(other)) else {
            // An element the list already removed can be matched but no longer edited.
            return;
        };
        match operation {
            Operation::InsertAfter => edit.slots.insert(at + 1, inserted()),
            Operation::InsertBefore => edit.slots.insert(at, inserted()),
            Operation::MoveBack => {
                let slot = edit.slots.remove(at);
                edit.slots.push(slot);
            }
            Operation::MoveFront => {
                let slot = edit.slots.remove(at);
                edit.slots.insert(0, slot);
            }
            Operation::MoveAfter | Operation::MoveBefore => {
                if at != other_at {
                    let slot = edit.slots.remove(at);
                    let anchor = edit.position(other).unwrap_or(0);
                    let to = anchor + usize::from(operation == Operation::MoveAfter);
                    edit.slots.insert(to, slot);
                }
            }
            Operation::Swap => edit.slots.swap(at, other_at),
            Operation::Remove => {
                edit.slots.remove(at);
            }
            Operation::Replace => edit.slots[at] = inserted(),
            Operation::InsertBack | Operation::InsertFront => {}
        }
    }

    /// The original index `condition` selects, reporting why
    /// it selects none.
    fn find(
        &mut self,
        edit: &ArrayEdit,
        condition: &Condition<'_>,
        key: &str,
        operation: Operation,
    ) -> Option<usize> {
        let operation = operation.name();
        let found = match condition {
            Condition::None | Condition::Name("") => {
                self.error(
                    UiPackErrorKind::MissingCondition,
                    vec![key.to_owned(), operation.to_owned()],
                );
                return None;
            }
            Condition::Name(name) => {
                let found = edit
                    .original
                    .iter()
                    .position(|element| element_name(element) == Some(name));
                if found.is_none() {
                    self.error(
                        UiPackErrorKind::MissingControlTarget,
                        vec![key.to_owned(), (*name).to_owned(), operation.to_owned()],
                    );
                    return None;
                }
                found
            }
            Condition::Where(condition) => {
                if condition.is_array() {
                    // Vanilla's inner loop continues on both branches: never a match.
                    self.raise(Event::WhereArray(key.to_owned()));
                    None
                } else if condition.is_object() {
                    let wanted = sorted_members(condition);
                    edit.original.iter().position(|element| {
                        !element.is_null()
                            && wanted.iter().any(|(name, value)| {
                                element.as_object().and_then(|object| object.get(name))
                                    == Some(value)
                            })
                    })
                } else {
                    // Any other non-null type: index 0, the first element.
                    (!edit.original.is_empty()).then_some(0)
                }
            }
        };
        if found.is_none() {
            // A `where` / `target` miss is silent in vanilla: the operation is skipped.
            self.raise(Event::Unmatched(key.to_owned()));
        }
        found
    }
}

/// What selects the element an operation works on.
enum Condition<'a> {
    /// Absent.
    None,
    /// A control name: the short name of an element's first member.
    Name(&'a str),
    /// A `where` / `target` value.
    Where(&'a Value),
}

/// The control at `path` below member `top` of the working root.
fn locate<'a>(working: &'a mut Value, top: &str, path: &[usize]) -> Option<&'a mut Value> {
    let mut node = working.as_object_mut()?.get_mut(top)?;
    for &index in path {
        let child = member_mut(node, "controls")?
            .as_array_mut()?
            .get_mut(index)?;
        let key = first_key(child)?.to_owned();
        node = child.as_object_mut()?.get_mut(&key)?;
    }
    Some(node)
}

/// [`locate`], creating the top-level member when the root lacks it.
fn locate_or_create<'a>(
    working: &'a mut Value,
    top: &str,
    path: &[usize],
) -> Option<&'a mut Value> {
    let root = working.as_object_mut()?;
    if !root.contains_key(top) {
        root.insert(top, Value::NULL);
    }
    locate(working, top, path)
}

fn member_mut<'a>(value: &'a mut Value, key: &str) -> Option<&'a mut Value> {
    value.as_object_mut()?.get_mut(key)
}

/// The edits of one `modifications` list to one array: the original elements and where each
/// stands now.
// Conditions see the original array; the edits rebuild it once, when all are known.
struct ArrayEdit {
    original: Vec<Value>,
    slots: Vec<Slot>,
}

enum Slot {
    /// The original element at this index.
    Original(usize),
    /// An inserted `value`: an array is spliced in, anything else is one element.
    Inserted(Value),
}

impl ArrayEdit {
    fn new(original: Vec<Value>) -> Self {
        let slots = (0..original.len()).map(Slot::Original).collect();
        Self { original, slots }
    }

    /// Where the original element `index` stands now, unless it was removed or replaced.
    fn position(&self, index: usize) -> Option<usize> {
        self.slots
            .iter()
            .position(|slot| matches!(slot, Slot::Original(at) if *at == index))
    }

    /// The array after the edits; `None` when there was no array and nothing was inserted.
    fn rebuild(mut self) -> Option<Value> {
        if self.original.is_empty() && self.slots.is_empty() {
            return None;
        }
        let mut rebuilt = facet_value::VArray::new();
        for slot in self.slots {
            match slot {
                Slot::Original(index) => rebuilt.push(std::mem::take(&mut self.original[index])),
                Slot::Inserted(value) => match value.as_array() {
                    Some(spliced) => {
                        for element in spliced.as_slice() {
                            rebuilt.push(element.clone());
                        }
                    }
                    None => rebuilt.push(value),
                },
            }
        }
        Some(rebuilt.into_value())
    }
}

/// A modification operation, numbered by the game from `insert_back` (1) to `replace` (11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operation {
    InsertBack,
    InsertFront,
    InsertAfter,
    InsertBefore,
    MoveBack,
    MoveFront,
    MoveAfter,
    MoveBefore,
    Swap,
    Remove,
    Replace,
}

impl Operation {
    const ALL: [(Self, &'static str); 11] = [
        (Self::InsertBack, "insert_back"),
        (Self::InsertFront, "insert_front"),
        (Self::InsertAfter, "insert_after"),
        (Self::InsertBefore, "insert_before"),
        (Self::MoveBack, "move_back"),
        (Self::MoveFront, "move_front"),
        (Self::MoveAfter, "move_after"),
        (Self::MoveBefore, "move_before"),
        (Self::Swap, "swap"),
        (Self::Remove, "remove"),
        (Self::Replace, "replace"),
    ];

    fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .find(|(_, text)| *text == name)
            .map(|(operation, _)| *operation)
    }

    /// The operation's name: the inverse of the name lookup.
    fn name(self) -> &'static str {
        Self::ALL
            .iter()
            .find(|(operation, _)| *operation == self)
            .map_or("", |(_, text)| text)
    }

    /// Every `insert_*` and `replace` needs a `value`.
    fn needs_value(self) -> bool {
        matches!(
            self,
            Self::InsertBack
                | Self::InsertFront
                | Self::InsertAfter
                | Self::InsertBefore
                | Self::Replace
        )
    }

    /// `move_after`, `move_before` and `swap` need a second element.
    fn needs_target(self) -> bool {
        matches!(self, Self::MoveAfter | Self::MoveBefore | Self::Swap)
    }
}
