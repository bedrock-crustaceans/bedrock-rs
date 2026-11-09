//! A loading-screen tip is a localisation key some language of the stack translates.
//!
//! The game hands each tip to its translator: a text that is a key of the client's language
//! is shown translated, any other text is shown as it is written. So a tip that is no key is
//! either a text the author meant to show as it is or a key no `.lang` file has, and only the
//! author can tell which. Told here as an information at the tip; a lint, left out when the
//! lints are off. The finding is an `UnresolvedLangKey` information: the game shows the raw
//! key when no language has it (the translator falls back to the text itself).

use std::collections::HashMap;

use bedrock_addon_core::diagnostic::{JsonPath, Severity};

use crate::families::loading_messages::{LoadingMessage, LoadingMessages, MessageList};
use crate::json::Value;
use crate::pack::link::prelude::*;

/// The three lists a stage of the game is answered from.
const STAGES: [MessageList; 3] = [
    MessageList::Beginner,
    MessageList::MidGame,
    MessageList::LateGame,
];

/// Every tip the game can show from the stack's `loading_messages.json` is a key
/// of some language of either stack (compared without ASCII case), or one the host says the
/// game has. One that is none is an `UnresolvedLangKey` information at the element, or at its
/// `message` when the element is an object. An element of a list the file writes as an
/// object is named by its key there.
///
/// The tips are the ones `LoadingMessages::tips` gives: the stage lists are not read while
/// `loading_messages` has an element, and an element the game skips or never shows is no
/// tip. An empty text is no key. A text is looked up once however many lists hold it; each
/// member that holds it is told.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    if !cx.lints() {
        return;
    }
    let messages = cx.resource().loading_messages();
    let legacy = messages.value().uses_legacy_list();
    // Whether each text is missing: one lookup per distinct text.
    let mut missing: HashMap<&str, bool> = HashMap::new();
    for list in MessageList::ALL {
        if legacy && STAGES.contains(&list) {
            continue;
        }
        let (Some(elements), Some(origin)) = (
            messages.value().list(list),
            messages.origin(&list.key().to_owned()),
        ) else {
            continue;
        };
        // How the file writes the list: the key of each element of an object, in the order read.
        let written = written_as(messages.value(), list, elements.len());
        for (index, element) in elements.iter().enumerate() {
            let Some(text) = element.shown().filter(|text| !text.is_empty()) else {
                continue;
            };
            let unknown = *missing
                .entry(text)
                .or_insert_with(|| cx.lang_key(text).is_missing());
            if unknown {
                let path = element_path(list, &written, index, element);
                let at = At::new(origin, path, Severity::Info);
                out.push(unresolved_lang_key(at, text));
            }
        }
    }
}

/// How the file writes a list of tips.
#[derive(Debug)]
enum Written<'a> {
    /// As an array, or not known (a value built or changed by hand has no file).
    Array,
    /// As an object: the key of each element held, in the order held.
    Keys(Vec<&'a str>),
    /// As an object whose members are not the elements held: another number of them.
    Other,
}

/// How the file writes `list`, of `held` elements. When it writes the list as an object, the
/// game iterates the member values in key order, and the elements held are in that order.
fn written_as(messages: &LoadingMessages, list: MessageList, held: usize) -> Written<'_> {
    let Some(members) = messages
        .written
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|root| root.get(list.key()))
        .and_then(Value::as_object)
    else {
        return Written::Array;
    };
    let mut keys: Vec<&str> = members.keys().map(facet_value::VString::as_str).collect();
    // jsoncpp's objects are a `std::map`: byte order, as the reader of the list sorts them.
    keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    if keys.len() == held {
        Written::Keys(keys)
    } else {
        Written::Other
    }
}

/// The path the file has the element `index` of `list` at, or its `message` when the element
/// is an object; the list itself when the file's object does not hold the elements held.
fn element_path(
    list: MessageList,
    written: &Written<'_>,
    index: usize,
    element: &LoadingMessage,
) -> JsonPath {
    let mut path = JsonPath::new();
    path.push(list.key());
    match written {
        Written::Keys(keys) if index < keys.len() => path.push(keys[index]),
        Written::Keys(_) | Written::Other => return path,
        Written::Array => path.push_index(index),
    }
    if matches!(element, LoadingMessage::Conditional(_)) {
        path.push("message");
    }
    path
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::diagnostic::Diagnostics;

    use crate::pack::StackSet;
    use crate::pack::link::tests::support::set;
    use crate::pack::link::{LinkOptions, link_with_rules};

    /// A world whose tips file has `n` tips, every second one a key of the one language's
    /// `n` keys and the others no key of it, with everything the rule reads loaded.
    fn world(n: usize) -> StackSet {
        use std::fmt::Write as _;
        let mut tips = String::from(r#"{"beginner_loading_messages": ["#);
        let mut lang = String::new();
        for i in 0..n {
            let separator = if i == 0 { "" } else { ", " };
            let name = if i % 2 == 0 { "tips.key" } else { "tips.none" };
            let _ = write!(tips, r#"{separator}"{name}_{i}""#);
            let _ = writeln!(lang, "tips.key_{i}=Tip {i}");
        }
        tips.push_str("]}");
        let set = set(
            &[],
            &[
                ("loading_messages.json", tips.as_str()),
                ("texts/languages.json", r#"["en_US"]"#),
                ("texts/en_US.lang", lang.as_str()),
            ],
        );
        // The first pass loads the tips and the language: the timed ones only look up.
        assert_eq!(pass(&set).len(), n / 2);
        set
    }

    /// A list the file writes as an object whose members are not the elements held (a
    /// value whose lists were changed after the load) names no element by an index of the
    /// object: the tip is told at the list.
    #[test]
    fn a_tip_of_an_object_of_other_members_is_told_at_the_list() {
        use super::{Written, element_path, written_as};
        use crate::families::loading_messages::{LoadingMessage, LoadingMessages, MessageList};
        let written = crate::json::from_str_jsonc::<crate::json::Value>(
            r#"{"beginner_loading_messages": {"b": "tips.b", "a": "tips.a"}}"#,
        )
        .expect("JSON");
        let tip = LoadingMessage::Text("tips.a".to_owned());
        let messages = LoadingMessages {
            beginner_loading_messages: Some(vec![tip.clone()]),
            written: Some(written).into(),
            ..LoadingMessages::default()
        };
        let list = MessageList::Beginner;
        assert!(
            matches!(written_as(&messages, list, 2), Written::Keys(ref keys) if keys == &["a", "b"])
        );
        let other = written_as(&messages, list, 1);
        assert!(matches!(other, Written::Other));
        assert_eq!(
            element_path(list, &other, 0, &tip).to_string(),
            "beginner_loading_messages"
        );
        assert_eq!(
            element_path(list, &Written::Array, 0, &tip).to_string(),
            "beginner_loading_messages/0"
        );
    }

    fn pass(set: &StackSet) -> Diagnostics {
        link_with_rules(set, LinkOptions::default(), &[super::run])
    }

    /// The rule calls `LinkCx::lang_key` once per distinct tip, and a miss asks every listed
    /// language: `n` tips among `n` keys cost `n`.
    #[test]
    fn n_tips_among_n_keys_cost_n() {
        crate::test_cost::assert_linear_at(
            "link: n tips among n keys",
            20_000,
            world,
            |set: &StackSet| {
                let found = pass(set);
                assert!(!found.is_empty());
                found
            },
        );
    }
}
