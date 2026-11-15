//! `credits/credits.json`, the game's own credits, and the files of the game's data that a
//! resource pack carries in vain.

use super::{Credits, CreditsDiscipline, CreditsTitle, codes};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{CodeDef, Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::build::{Object, array};
use bedrock_addon_core::json::kit::fixed_path::{Authored, reader, to_text};
use bedrock_addon_core::json::kit::{Node, Read, Written};
use bedrock_addon_core::schema::{FromJson, read_document};

/// A finding of the crate's own, an information, at the value `at`.
fn shape(read: &mut Read<'_>, code: &'static CodeDef<0>, at: &Node<'_>) {
    read.push(Diagnostic::of(code).at(at.path().clone()));
}

/// A heading or a name as the game reads it: any value as a string, a boolean as `true` /
/// `false` and anything else empty, without a word (`schema::read_with`).
#[allow(
    clippy::unnecessary_wraps,
    reason = "the shape of a `read_with` function"
)]
pub(super) fn text(node: &Node<'_>, _: &mut Read<'_>) -> Option<String> {
    Some(to_text(node.value()))
}

/// The codes of a list's two shapes the game skips: the member is no array, an element is no
/// object with a member.
type Skipped = (&'static CodeDef<0>, &'static CodeDef<0>);

/// The elements of the list `node` that are non-empty objects, each read as `T`; the others
/// are told (`skipped.1`) before any is read, as the game filters a list before it reads it,
/// and a member that is no array is told (`skipped.0`) and has no element.
fn objects<T: FromJson>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    (not_array, empty): Skipped,
) -> Option<Vec<T>> {
    if !node.value().is_array() {
        shape(read, not_array, node);
        return Some(Vec::new());
    }
    let elements = read.array(node)?;
    let mut kept = Vec::new();
    for element in elements.iter() {
        if element.value().as_object().is_some_and(|o| !o.is_empty()) {
            kept.push(element);
        } else {
            shape(read, empty, &element);
        }
    }
    kept.iter()
        .map(|element| T::from_json(element, read))
        .collect()
}

/// The `disciplines` of a section (`schema::read_with`).
pub(super) fn disciplines(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<CreditsDiscipline>> {
    objects(
        node,
        read,
        (&codes::DISCIPLINES_NOT_ARRAY, &codes::DISCIPLINE_EMPTY),
    )
}

/// The `titles` of a discipline (`schema::read_with`).
pub(super) fn titles(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<CreditsTitle>> {
    objects(node, read, (&codes::TITLES_NOT_ARRAY, &codes::TITLE_EMPTY))
}

/// The `names` of a title: each element as the game reads a string; a member that is no array
/// is told and has none (`schema::read_with`).
pub(super) fn names(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    if !node.value().is_array() {
        shape(read, &codes::NAMES_NOT_ARRAY, node);
        return Some(Vec::new());
    }
    let elements = read.array(node)?;
    Some(elements.iter().map(|name| to_text(name.value())).collect())
}

/// The root: an array of sections; a root that is no array has none, and is told.
impl FromJson for Credits {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let sections = objects(node, read, (&codes::ROOT_NOT_ARRAY, &codes::SECTION_EMPTY))?;
        Some(Self {
            sections,
            written: Written::default(),
        })
    }
}

impl Authored for Credits {
    fn read(root: &Value, diagnostics: &mut Diagnostics) -> Self {
        let mut read = reader(diagnostics);
        let node = Node::new(root, JsonPath::new());
        read_document(&node, &mut read).unwrap_or_default()
    }

    fn fields_value(&self) -> Value {
        array(self.sections.iter().map(|section| {
            let mut object = Object::default();
            object.put("section", section.section.as_deref().map(Value::from));
            object.put(
                "disciplines",
                section
                    .disciplines
                    .as_ref()
                    .map(|list| array(list.iter().map(discipline_value))),
            );
            object.extra(&section.extra);
            object.finish()
        }))
    }

    fn written_mut(&mut self) -> &mut Option<Value> {
        &mut self.written
    }
}

fn discipline_value(discipline: &CreditsDiscipline) -> Value {
    let mut object = Object::default();
    object.put(
        "discipline",
        discipline.discipline.as_deref().map(Value::from),
    );
    object.put(
        "titles",
        discipline.titles.as_ref().map(|titles| {
            array(titles.iter().map(|title| {
                let mut object = Object::default();
                object.put("title", title.title.as_deref().map(Value::from));
                object.put(
                    "names",
                    title
                        .names
                        .as_ref()
                        .map(|names| array(names.iter().map(|name| Value::from(name.as_str())))),
                );
                object.extra(&title.extra);
                object.finish()
            }))
        }),
    );
    object.extra(&discipline.extra);
    object.finish()
}

/// Checks a `StarterSeeds.txt` on its own: one seed per line; a blank line is a seed the game
/// cannot use when it picks it (an information at the line's index). Any other text is a
/// seed: the game hashes what is no number.
// Parsing a seed string fails on an empty text without a log line, and only the developer
// assert "Failed to parse the starter seed from file" follows, so the crate reports it as an
// information. A non-numeric line is hashed into a seed.
pub fn check_starter_seeds(bytes: &[u8]) -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    let text = String::from_utf8_lossy(bytes);
    // The last line has no line break in the game's own files; a trailing break adds none.
    let text = text.strip_suffix('\n').unwrap_or(&text);
    for (line, seed) in text.split('\n').enumerate() {
        if seed.trim().is_empty() {
            diagnostics.push(
                Diagnostic::of(&core_codes::INVALID_VALUE)
                    .severity(Severity::Info)
                    .at(JsonPath::new().index(line)),
            );
        }
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::json::jsoncpp::read;
    use bedrock_addon_core::test_cost::assert_linear_at;

    /// The credits are read in time linear in their sections.
    #[test]
    fn the_credits_are_read_in_linear_time() {
        let build = |entries: usize| {
            let sections: Vec<String> = (0..entries)
                .map(|i| {
                    format!(
                        r#"{{ "section": "s{i}", "disciplines": [{{ "discipline": "d", "titles": [{{ "title": "t", "names": ["n{i}", {i}] }}, 3] }}] }}"#
                    )
                })
                .collect();
            read(format!("[{}]", sections.join(",")).as_bytes()).expect("JSON")
        };
        assert_linear_at("Credits::from_value", 5_000, build, |root| {
            let mut diagnostics = Diagnostics::default();
            let credits = Credits::from_value(root, &mut diagnostics);
            assert_eq!(
                Some(credits.sections.len()),
                root.as_array().map(|a| a.iter().count())
            );
            assert_eq!(diagnostics.len(), credits.sections.len());
        });
    }
}

#[cfg(test)]
mod kept_root_tests {
    use super::*;
    use crate::CreditsSection;
    use bedrock_addon_core::json::kit::fixed_path::testing::{
        assert_kept_root_written_linearly, list,
    };

    /// The root kept beside the fields is left out of equality: a loaded value that kept one
    /// equals the value built by hand with the same fields, and is still written as authored.
    #[test]
    fn a_kept_root_is_left_out_of_equality() {
        let root =
            bedrock_addon_core::json::jsoncpp::read(br#"[{"section": "A"}, 5]"#).expect("JSON");
        let loaded = Credits::from_value(&root, &mut Diagnostics::default());
        assert_eq!(*loaded.written, Some(root.clone()));
        let built = Credits {
            sections: vec![CreditsSection {
                section: Some("A".to_owned()),
                ..CreditsSection::default()
            }],
            ..Credits::default()
        };
        assert_eq!(loaded, built);
        assert_eq!(loaded.to_value(), root);
        assert_ne!(built.to_value(), root);
    }

    /// A write of a value that keeps its root is linear in its entries, with one entry the
    /// game skips.
    #[test]
    fn a_kept_root_is_written_in_linear_time() {
        assert_kept_root_written_linearly::<Credits>("Credits::to_value", 1000, |entries| {
            let sections = list(entries, |i| {
                format!(
                    r#"{{ "section": "s{i}", "disciplines": [{{ "discipline": "d", "titles": [{{ "title": "t", "names": ["n{i}"] }}] }}] }}"#
                )
            });
            format!("[{sections}, 5]")
        });
    }
}
