//! `splashes.json`: reading one copy, the game's reading of its members, and the step one copy
//! takes on the pool.

use super::{ConditionalSplashes, SplashClient, SplashRequires, Splashes};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::build::{Object, array};
use bedrock_addon_core::json::kit::fixed_path::{
    Authored, iterated, kept_unknown, raw_member, reader, to_text,
};
use bedrock_addon_core::json::kit::{Disposition, Members, Node, Read, Written};

/// `InvalidValue` without a vanilla text: a finding of the crate's own.
const INVALID: DiagnosticCode = DiagnosticCode::InvalidValue { text: None };

const ROOT_KEYS: [&str; 3] = ["canMerge", "splashes", "conditional"];
const ENTRY_KEYS: [&str; 2] = ["requires", "splashes"];
const REQUIRES_KEYS: [&str; 4] = ["platforms", "treatments", "stores", "trialMode"];
/// The requirement lists, in the game's category order.
const CATEGORIES: [&str; 3] = ["platforms", "treatments", "stores"];

/// A splash list as the game reads it: every iterated element read as a string with an empty
/// default. The game iterates the list without testing its type.
fn texts(list: Option<&Value>) -> Vec<String> {
    list.map(|list| {
        iterated(list, &JsonPath::new())
            .into_iter()
            .map(|(_, element)| to_text(element))
            .collect()
    })
    .unwrap_or_default()
}

/// The findings of a splash list: a member that is no array (the crate's own), and every
/// element that is no string.
fn check_texts(read: &mut Read<'_>, list: Option<Node<'_>>) {
    let Some(list) = list.filter(|list| !list.value().is_null()) else {
        return;
    };
    if !list.value().is_array() {
        read.wrong_type(&list);
    }
    for (at, element) in iterated(list.value(), list.path()) {
        if !element.is_string() {
            read.push(Diagnostic::info(INVALID).at(at));
        }
    }
}

/// The members of `node` when it is an object; nothing is told.
fn members<'v>(read: &mut Read<'_>, node: &Node<'v>) -> Option<Members<'v>> {
    node.value()
        .is_object()
        .then(|| read.object(node))
        .flatten()
}

/// Whether a requirement list holds for a client that answers to `have`.
/// A null, a non-array and an empty array hold; otherwise the list holds when any string
/// element is one the client meets, and non-strings are skipped.
fn requirement_holds(list: Option<&Value>, have: &[String]) -> bool {
    let Some(elements) = list.and_then(Value::as_array) else {
        return true;
    };
    elements.is_empty()
        || elements
            .iter()
            .filter_map(Value::as_string)
            .any(|wanted| have.iter().any(|name| name == wanted.as_str()))
}

impl Authored for Splashes {
    /// Reads a copy's root. A root that is no object reads as an empty copy.
    fn read(root: &Value, diagnostics: &mut Diagnostics) -> Self {
        let at = JsonPath::new();
        let mut read = reader(diagnostics);
        let Some(members) = members(&mut read, &Node::new(root, at.clone())) else {
            // The first member lookup turns the root into `{}`.
            read.push(Diagnostic::info(INVALID).at(at));
            return Self::default();
        };
        let member = |key: &str| members.get(key).map(|node| node.value().clone());
        let splashes = Self {
            can_merge: member("canMerge"),
            splashes: member("splashes"),
            conditional: member("conditional"),
            extra: UnknownMembers::default(),
            written: Written::default(),
        };
        if splashes.can_merge.as_ref().is_some_and(|v| !v.is_bool()) {
            read.push(Diagnostic::info(INVALID).at(at.child("canMerge")));
        }
        check_texts(&mut read, members.get("splashes"));
        let extra = kept_unknown(&mut read, &members, &ROOT_KEYS);
        if let Some(conditional) = members.get("conditional") {
            check_conditional(&mut read, &conditional);
        }
        Self { extra, ..splashes }
    }

    /// The copy's members as the JSON object they are written as.
    fn fields_value(&self) -> Value {
        let mut object = Object::default();
        object.put("canMerge", self.can_merge.clone());
        object.put("splashes", self.splashes.clone());
        object.put("conditional", self.conditional.clone());
        object.extra(&self.extra);
        object.finish()
    }

    fn written_mut(&mut self) -> &mut Option<Value> {
        &mut self.written
    }
}

/// Checks the `conditional` entries. Their members are kept as written in `conditional`,
/// so the unknown ones are told and nothing more.
fn check_conditional(read: &mut Read<'_>, conditional: &Node<'_>) {
    if conditional.value().is_null() {
        return;
    }
    if !conditional.value().is_array() {
        read.push(Diagnostic::info(INVALID).at(conditional.path().clone()));
        return;
    }
    let Some(entries) = read.array(conditional) else {
        return;
    };
    for entry in entries.iter() {
        let entry_members = members(read, &entry);
        let requires = entry_members
            .as_ref()
            .and_then(|members| members.get("requires"))
            .filter(|requires| requires.value().is_object());
        let (Some(entry_members), Some(requires)) = (entry_members, requires) else {
            read.push(Diagnostic::info(INVALID).at(entry.path().clone()));
            continue;
        };
        check_texts(read, entry_members.get("splashes"));
        read.unknown(&entry_members, &ENTRY_KEYS, Disposition::Keep);
        let Some(requires) = members(read, &requires) else {
            continue;
        };
        for category in CATEGORIES {
            let only_others = requires
                .get(category)
                .and_then(|list| list.value().as_array())
                .is_some_and(|list| !list.is_empty() && !list.iter().any(Value::is_string));
            if only_others {
                read.push(Diagnostic::info(INVALID).at(requires.node().path().child(category)));
            }
        }
        read.unknown(&requires, &REQUIRES_KEYS, Disposition::Keep);
    }
}

impl Splashes {
    /// Whether the copy adds to the lower packs' splashes: only when `canMerge` is the
    /// boolean `true`. Any other copy replaces them.
    pub fn merges(&self) -> bool {
        self.can_merge.as_ref().and_then(Value::as_bool) == Some(true)
    }

    /// The copy's unconditional splashes as the game reads them: a string as written, a
    /// boolean as `true` / `false`, anything else as the empty text (which can be shown).
    pub fn texts(&self) -> Vec<String> {
        texts(self.splashes.as_ref())
    }

    /// The entries of `conditional`, when it is an array; an element that is no object reads
    /// as an entry without requirements, which is never used.
    pub fn conditional_entries(&self) -> Vec<ConditionalSplashes> {
        let Some(entries) = self.conditional.as_ref().and_then(Value::as_array) else {
            return Vec::new();
        };
        entries
            .iter()
            .map(|entry| {
                let mut extra = UnknownMembers::default();
                if let Some(object) = entry.as_object() {
                    for (name, member) in object {
                        if !ENTRY_KEYS.contains(&name.as_str()) {
                            extra.insert(name.as_str().to_owned(), member.clone());
                        }
                    }
                }
                ConditionalSplashes {
                    requires: raw_member(entry, "requires").cloned(),
                    splashes: raw_member(entry, "splashes").cloned(),
                    extra,
                }
            })
            .collect()
    }

    /// Takes the step this copy takes on the pool the lower packs collected, for `client`.
    ///
    /// For the full game, a copy that does not [merge](Self::merges) empties the pool, and the
    /// copy's splashes are added. For every client, the splashes of each conditional entry
    /// whose requirements hold are added after them. A trial neither empties the pool nor adds
    /// the unconditional splashes.
    pub fn apply(&self, pool: &mut Vec<String>, client: &SplashClient) {
        let (clears, added) = self.step(client);
        if clears {
            pool.clear();
        }
        pool.extend(added);
    }

    /// The step of [`apply`](Self::apply): whether the pool is emptied first, and the splashes
    /// then added.
    pub(crate) fn step(&self, client: &SplashClient) -> (bool, Vec<String>) {
        let mut added = Vec::new();
        let clears = !client.trial && !self.merges();
        if !client.trial {
            added = self.texts();
        }
        if let Some(entries) = self.conditional.as_ref().and_then(Value::as_array) {
            for entry in entries {
                let passes = raw_member(entry, "requires")
                    .filter(|requires| requires.is_object())
                    .is_some_and(|requires| requires_pass(requires, client));
                if passes {
                    added.extend(texts(raw_member(entry, "splashes")));
                }
            }
        }
        (clears, added)
    }

    /// A copy holding `texts` as its unconditional splashes and nothing else.
    pub(crate) fn of_texts(texts: &[String]) -> Self {
        Self {
            splashes: Some(array(texts.iter().map(|text| Value::from(text.as_str())))),
            ..Self::default()
        }
    }
}

/// Whether `client` meets a `requires` object.
// trialMode, then the three categories.
fn requires_pass(requires: &Value, client: &SplashClient) -> bool {
    let trial = raw_member(requires, "trialMode").and_then(Value::as_bool);
    trial.is_none_or(|wanted| wanted == client.trial)
        && requirement_holds(raw_member(requires, "platforms"), &client.platforms)
        && requirement_holds(raw_member(requires, "treatments"), &client.treatments)
        && requirement_holds(raw_member(requires, "stores"), &client.stores)
}

impl ConditionalSplashes {
    /// The requirements, when `requires` is an object.
    pub fn requirements(&self) -> Option<SplashRequires> {
        let requires = self.requires.as_ref().filter(|v| v.is_object())?;
        let mut extra = UnknownMembers::default();
        for (name, member) in requires.as_object()? {
            if !REQUIRES_KEYS.contains(&name.as_str()) {
                extra.insert(name.as_str().to_owned(), member.clone());
            }
        }
        Some(SplashRequires {
            platforms: raw_member(requires, "platforms").cloned(),
            treatments: raw_member(requires, "treatments").cloned(),
            stores: raw_member(requires, "stores").cloned(),
            trial_mode: raw_member(requires, "trialMode").cloned(),
            extra,
        })
    }

    /// The entry's splashes as the game reads them ([`Splashes::texts`]).
    pub fn texts(&self) -> Vec<String> {
        texts(self.splashes.as_ref())
    }
}

impl SplashRequires {
    /// Whether `client` meets every requirement.
    pub fn passes(&self, client: &SplashClient) -> bool {
        let trial = self.trial_mode.as_ref().and_then(Value::as_bool);
        trial.is_none_or(|wanted| wanted == client.trial)
            && requirement_holds(self.platforms.as_ref(), &client.platforms)
            && requirement_holds(self.treatments.as_ref(), &client.treatments)
            && requirement_holds(self.stores.as_ref(), &client.stores)
    }
}

#[cfg(test)]
mod kept_root_tests {
    use super::*;
    use bedrock_addon_core::json::kit::fixed_path::testing::{
        assert_kept_root_written_linearly, list,
    };

    /// The root kept beside the fields is left out of equality: a loaded value that kept one
    /// equals the value built by hand with the same fields, and is still written as authored.
    #[test]
    fn a_kept_root_is_left_out_of_equality() {
        let root = bedrock_addon_core::json::jsoncpp::read(b"[1]").expect("JSON");
        let loaded = Splashes::from_value(&root, &mut Diagnostics::default());
        assert_eq!(*loaded.written, Some(root.clone()));
        assert_eq!(loaded, Splashes::default());
        assert_eq!(loaded.to_value(), root);
        assert_ne!(Splashes::default().to_value(), root);
    }

    /// A write of a value that keeps its root is linear in its entries, with one entry the
    /// game skips.
    #[test]
    fn a_kept_root_is_written_in_linear_time() {
        assert_kept_root_written_linearly::<Splashes>("Splashes::to_value", 20000, |entries| {
            let splashes = list(entries, |i| format!("\"splash {i}\""));
            format!("[{splashes}]")
        });
    }
}
