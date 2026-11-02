//! `splashes.json`: reading one copy, the game's reading of its members, and the step one copy
//! takes on the pool.

use super::{ConditionalSplashes, SplashClient, SplashRequires, Splashes};
use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;
use bedrock_addon_support::fixed_path::{
    Authored, Object, array, finding, index, invalid, iterated, key, others, raw_member, to_text,
};

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

/// The findings of a splash list at `at`: a member that is no array (the crate's
/// own), and every element that is no string.
fn check_texts(list: Option<&Value>, at: &JsonPath, diagnostics: &mut Diagnostics) {
    let Some(list) = list.filter(|list| !list.is_null()) else {
        return;
    };
    if !list.is_array() {
        diagnostics.push(finding(
            DiagnosticCode::InvalidType { text: None },
            Severity::Info,
            at.clone(),
        ));
    }
    for (at, element) in iterated(list, at) {
        if !element.is_string() {
            diagnostics.push(finding(invalid(), Severity::Info, at));
        }
    }
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
        if !root.is_object() {
            // The first member lookup turns the root into `{}`.
            diagnostics.push(finding(invalid(), Severity::Info, at));
            return Self::default();
        }
        let splashes = Self {
            can_merge: raw_member(root, "canMerge").cloned(),
            splashes: raw_member(root, "splashes").cloned(),
            conditional: raw_member(root, "conditional").cloned(),
            extra: Extra::default(),
            written: None,
        };
        if splashes.can_merge.as_ref().is_some_and(|v| !v.is_bool()) {
            diagnostics.push(finding(invalid(), Severity::Info, key(&at, "canMerge")));
        }
        check_texts(
            splashes.splashes.as_ref(),
            &key(&at, "splashes"),
            diagnostics,
        );
        let extra = others(root, &ROOT_KEYS, &at, diagnostics);
        splashes.check_conditional(&key(&at, "conditional"), diagnostics);
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

impl Splashes {
    /// Checks the `conditional` entries.
    fn check_conditional(&self, at: &JsonPath, diagnostics: &mut Diagnostics) {
        let Some(conditional) = self.conditional.as_ref().filter(|v| !v.is_null()) else {
            return;
        };
        let Some(entries) = conditional.as_array() else {
            diagnostics.push(finding(invalid(), Severity::Info, at.clone()));
            return;
        };
        for (i, entry) in entries.iter().enumerate() {
            let at = index(at, i);
            let requires = raw_member(entry, "requires").filter(|v| v.is_object());
            let Some(requires) = requires else {
                diagnostics.push(finding(invalid(), Severity::Info, at));
                continue;
            };
            check_texts(
                raw_member(entry, "splashes"),
                &key(&at, "splashes"),
                diagnostics,
            );
            others(entry, &ENTRY_KEYS, &at, diagnostics);
            let at = key(&at, "requires");
            for category in CATEGORIES {
                let only_others = raw_member(requires, category)
                    .and_then(Value::as_array)
                    .is_some_and(|list| !list.is_empty() && !list.iter().any(Value::is_string));
                if only_others {
                    diagnostics.push(finding(invalid(), Severity::Info, key(&at, category)));
                }
            }
            others(requires, &REQUIRES_KEYS, &at, diagnostics);
        }
    }

    /// Whether the copy adds to the lower packs' splashes: only when `canMerge` is the
    /// boolean `true`. Any other copy replaces them.
    pub fn merges(&self) -> bool {
        self.can_merge.as_ref().and_then(Value::as_bool) == Some(true)
    }

    /// The copy's unconditional splashes as the game reads them: a string as written, a
    /// boolean as `true` / `false`, anything else as the empty text (which can be shown).
    #[must_use]
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
                let mut extra = Extra::default();
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
    #[must_use]
    pub fn requirements(&self) -> Option<SplashRequires> {
        let requires = self.requires.as_ref().filter(|v| v.is_object())?;
        let mut extra = Extra::default();
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
    #[must_use]
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
    use bedrock_addon_support::fixed_path::testing::{assert_kept_root_written_linearly, list};

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
