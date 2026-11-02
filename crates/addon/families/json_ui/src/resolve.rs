//! From the merged files to the definition table: one entry per member, each `@base` chain
//! flattened in, the first definition of a namespace and name kept.
//!
//! Every definition is resolved once, and nothing is copied: a definition keeps its own
//! members and a shared link to what its base hands on ([`Inherited`]), so a chain costs its
//! length in time and in memory, whatever each definition adds.

use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;

use super::chain::{Inherited, Link, Ring};
use super::{Control, DefKey, DefRef, GlobalVariables, UiFile};
use bedrock_addon_core::json::Value;

/// What resolving has to say about one definition.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ResolveEvent {
    /// A later definition of a namespace and name an earlier one already holds.
    Duplicate,
    /// The base chain names a definition that does not exist; the reference.
    MissingBase(String),
    /// The base chain comes back to a definition it already visited.
    Cycle,
}

/// A [`ResolveEvent`] and the definition it is about.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Resolved {
    pub file: String,
    pub namespace: String,
    pub key: DefKey,
    pub event: ResolveEvent,
}

/// One definition as collected from a file.
struct Entry<'a> {
    file: &'a str,
    namespace: &'a str,
    key: &'a DefKey,
    body: &'a Value,
}

impl Entry<'_> {
    /// The namespace and name the definition is found under.
    fn id(&self) -> (String, String) {
        (self.namespace.to_owned(), self.key.name.clone())
    }
}

/// The base a key names, with its `$variables` replaced: `(namespace, name)`. `None` when the
/// key has no base or a variable of it has no string value here.
// The game replaces the variables of a reference when it loads the definitions. The ones it
// sees at that point are the definition's own `$name` members, then the global variables;
// the variables frames of the controls around a definition exist only when a screen builds it.
pub(crate) fn base_of(
    key: &DefKey,
    body: &Value,
    namespace: &str,
    globals: &GlobalVariables,
) -> Option<(String, String)> {
    let base = key.base.as_ref()?;
    let written = base.to_string();
    let reference = if written.contains('$') {
        DefRef::parse(&substitute(&written, body, globals)?)
    } else {
        base.clone()
    };
    Some((
        reference
            .namespace
            .filter(|namespace| !namespace.is_empty())
            .unwrap_or_else(|| namespace.to_owned()),
        reference.name,
    ))
}

/// `text` with every `$name` replaced by the string the definition or the globals give it.
fn substitute(text: &str, body: &Value, globals: &GlobalVariables) -> Option<String> {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find('$') {
        out.push_str(&rest[..at]);
        let length = 1 + rest[at + 1..]
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
            .count();
        let name = &rest[at..at + length];
        let own = body.as_object().and_then(|object| {
            object
                .get(name)
                .or_else(|| object.get(&format!("{name}|default")))
        });
        let value = own.or_else(|| globals.get(name))?.as_string()?;
        out.push_str(value.as_str());
        rest = &rest[at + length..];
    }
    out.push_str(rest);
    Some(out)
}

/// Where a definition stands in the walk.
#[derive(Clone, Copy)]
enum State {
    /// Not reached yet.
    New,
    /// On the stack, at this position: its base chain is being walked.
    Open(usize),
    /// Resolved.
    Done,
}

/// What the walk has resolved so far, per entry.
struct Table {
    state: Vec<State>,
    /// What a resolved definition inherits.
    inherited: Vec<Inherited>,
    /// What a definition hands on to the ones that inherit from it: made when the first of
    /// them asks, and shared by all of them.
    handed_on: Vec<Option<Inherited>>,
    events: Vec<Option<ResolveEvent>>,
}

impl Table {
    /// What the resolved entry `index` hands on: its own members that are not `null` over
    /// what it inherits. A `null` member of its own is not inherited, and what its own base
    /// has under that key is.
    fn inheritable(&mut self, index: usize, entries: &[Entry<'_>]) -> Inherited {
        let inherited = &self.inherited[index];
        self.handed_on[index]
            .get_or_insert_with(|| {
                let entry = &entries[index];
                Inherited::Chain(Arc::new(Link::new(
                    entry.id(),
                    entry.body.clone(),
                    inherited.clone(),
                )))
            })
            .clone()
    }

    /// Resolves `index`: it inherits what `base` hands on.
    fn finish(&mut self, index: usize, base: Option<usize>, entries: &[Entry<'_>]) {
        if let Some(base) = base {
            self.inherited[index] = self.inheritable(base, entries);
        }
        self.state[index] = State::Done;
    }

    /// Resolves the definitions of a cycle, given in base order (each one's base is the next,
    /// the last one's the first). Each inherits around the cycle once: per member the nearest
    /// definition that has it wins.
    fn close_cycle(&mut self, cycle: &[usize], entries: &[Entry<'_>]) {
        let ring = Arc::new(Ring::new(
            cycle
                .iter()
                .map(|index| (entries[*index].id(), entries[*index].body.clone()))
                .collect(),
        ));
        for (position, index) in cycle.iter().enumerate() {
            // It inherits what the next one hands on, and hands on from its own place.
            self.inherited[*index] =
                Inherited::Ring(Arc::clone(&ring), (position + 1) % cycle.len());
            self.handed_on[*index] = Some(Inherited::Ring(Arc::clone(&ring), position));
            self.events[*index] = Some(ResolveEvent::Cycle);
            self.state[*index] = State::Done;
        }
    }
}

/// The definition table of the merged `files` (in list order) and what was found on the way.
// The game resolves the base of every entry that has one and then enters each entry into its
// namespace's map, so the first entry of a name wins and a later one is dropped.
pub(crate) fn resolve(
    files: &IndexMap<String, UiFile>,
    globals: &GlobalVariables,
) -> (IndexMap<String, IndexMap<String, Control>>, Vec<Resolved>) {
    let mut entries: Vec<Entry<'_>> = Vec::new();
    for (path, file) in files {
        for (key, body) in file.definitions() {
            entries.push(Entry {
                file: path,
                namespace: file.namespace_or_root(),
                key,
                body,
            });
        }
    }
    // The winning entry of each namespace and name: the first collected.
    let mut winners: HashMap<(&str, &str), usize> = HashMap::new();
    for (index, entry) in entries.iter().enumerate() {
        winners
            .entry((entry.namespace, entry.key.name.as_str()))
            .or_insert(index);
    }
    let is_winner = |index: usize| {
        let entry = &entries[index];
        winners.get(&(entry.namespace, entry.key.name.as_str())) == Some(&index)
    };
    let mut table = Table {
        state: vec![State::New; entries.len()],
        inherited: vec![Inherited::Nothing; entries.len()],
        handed_on: vec![None; entries.len()],
        events: vec![None; entries.len()],
    };
    for start in 0..entries.len() {
        if !is_winner(start) {
            table.events[start] = Some(ResolveEvent::Duplicate);
            continue;
        }
        if !matches!(table.state[start], State::New) {
            continue;
        }
        // The chain below `start`, walked with an explicit stack: its length is the input's.
        let mut stack = vec![start];
        table.state[start] = State::Open(0);
        while let Some(&top) = stack.last() {
            let entry = &entries[top];
            let reference = if entry.body.is_object() {
                base_of(entry.key, entry.body, entry.namespace, globals)
            } else {
                None
            };
            let mut base = None;
            if let Some((namespace, name)) = reference {
                match winners.get(&(namespace.as_str(), name.as_str())).copied() {
                    // A missing base is silent in vanilla; the walk stops there.
                    None => {
                        table.events[top] =
                            Some(ResolveEvent::MissingBase(format!("{namespace}.{name}")));
                    }
                    Some(target) => match table.state[target] {
                        State::New => {
                            table.state[target] = State::Open(stack.len());
                            stack.push(target);
                            continue;
                        }
                        // The chain came back to a definition it is walking: a cycle.
                        State::Open(position) => {
                            table.close_cycle(&stack[position..], &entries);
                            stack.truncate(position);
                            continue;
                        }
                        State::Done => {
                            // What stopped the base's walk stopped this one's too.
                            if let Some(
                                event @ (ResolveEvent::MissingBase(_) | ResolveEvent::Cycle),
                            ) = &table.events[target]
                            {
                                table.events[top] = Some(event.clone());
                            }
                            base = Some(target);
                        }
                    },
                }
            }
            table.finish(top, base, &entries);
            stack.pop();
        }
    }
    collect(&entries, table)
}

/// The table and the findings, in the order the definitions were collected.
fn collect(
    entries: &[Entry<'_>],
    mut table: Table,
) -> (IndexMap<String, IndexMap<String, Control>>, Vec<Resolved>) {
    let mut found = Vec::new();
    let mut namespaces: IndexMap<String, IndexMap<String, Control>> = IndexMap::new();
    for (index, entry) in entries.iter().enumerate() {
        if let Some(event) = table.events[index].take() {
            found.push(Resolved {
                file: entry.file.to_owned(),
                namespace: entry.namespace.to_owned(),
                key: entry.key.clone(),
                event,
            });
        }
        // A later definition of a name an earlier one holds is not in the table.
        if !matches!(table.state[index], State::Done) {
            continue;
        }
        namespaces
            .entry(entry.namespace.to_owned())
            .or_default()
            .insert(
                entry.key.name.clone(),
                Control {
                    key: entry.key.clone(),
                    namespace: entry.namespace.to_owned(),
                    source_file: entry.file.to_owned(),
                    raw: entry.body.clone(),
                    inherited: std::mem::take(&mut table.inherited[index]),
                },
            );
    }
    (namespaces, found)
}
