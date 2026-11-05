//! `credits/credits.json`, the game's own credits, and the files of the game's data that a
//! resource pack carries in vain.

use super::{Credits, CreditsDiscipline, CreditsSection, CreditsTitle};
use bedrock_addon_core::diagnostic::{
    DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_support::fixed_path::{
    Authored, Object, array, finding, index, key, raw_member, to_text,
};

// The game's asserts on its own file are developer asserts and write no content-log line, so
// the crate reports them as informations with the game's texts.
/// The assert for an empty file.
pub(super) const EMPTY_FILE: &str = "File is not supposed to be empty";
/// The assert for a file that is not JSON.
pub(super) const NOT_JSON: &str = "Failed to load json file";
const ROOT_NOT_ARRAY: &str = "The root must be an array";
const SECTION_EMPTY: &str = "The section data should not be empty";
const DISCIPLINES_NOT_ARRAY: &str = "\"disciplines\" must be an array";
const DISCIPLINE_EMPTY: &str = "The discipline data should not be empty";
const TITLES_NOT_ARRAY: &str = "\"titles\" must be an array";
const TITLE_EMPTY: &str = "The title object should not be empty";
const NAMES_NOT_ARRAY: &str = "\"names\" must be an array";

fn shape(text: &'static str, at: &JsonPath, diagnostics: &mut Diagnostics) {
    diagnostics.push(finding(
        DiagnosticCode::InvalidType {
            text: Some(VanillaText::Fixed(text)),
        },
        Severity::Info,
        at.clone(),
    ));
}

/// The members of an object that are none of `read`.
fn rest(value: &Value, read: [&str; 2]) -> UnknownMembers {
    let mut extra = UnknownMembers::default();
    if let Some(object) = value.as_object() {
        for (name, member) in object {
            if !read.contains(&name.as_str()) {
                extra.insert(name.as_str().to_owned(), member.clone());
            }
        }
    }
    extra
}

/// The elements of the array member `name` of `parent` that are non-empty objects, each with
/// its path; `not_array` when the member is present and no array, `empty` per other element.
fn objects<'v>(
    parent: &'v Value,
    name: &str,
    at: &JsonPath,
    (not_array, empty): (&'static str, &'static str),
    diagnostics: &mut Diagnostics,
) -> Option<Vec<(JsonPath, &'v Value)>> {
    let member = raw_member(parent, name)?;
    let at = key(at, name);
    let Some(elements) = member.as_array() else {
        shape(not_array, &at, diagnostics);
        return Some(Vec::new());
    };
    Some(non_empty(elements.iter(), &at, empty, diagnostics))
}

fn non_empty<'v>(
    elements: impl Iterator<Item = &'v Value>,
    at: &JsonPath,
    empty: &'static str,
    diagnostics: &mut Diagnostics,
) -> Vec<(JsonPath, &'v Value)> {
    let mut kept = Vec::new();
    for (i, element) in elements.enumerate() {
        let at = index(at, i);
        if element.as_object().is_some_and(|object| !object.is_empty()) {
            kept.push((at, element));
        } else {
            shape(empty, &at, diagnostics);
        }
    }
    kept
}

impl Authored for Credits {
    /// Reads the file's root with the game's own checks, each an information: a root that is no
    /// array has no section; an element that is no object with a member is left out.
    fn read(root: &Value, diagnostics: &mut Diagnostics) -> Self {
        let at = JsonPath::new();
        let Some(sections) = root.as_array() else {
            shape(ROOT_NOT_ARRAY, &at, diagnostics);
            return Self::default();
        };
        let sections = non_empty(sections.iter(), &at, SECTION_EMPTY, diagnostics)
            .into_iter()
            .map(|(at, section)| CreditsSection {
                section: raw_member(section, "section").map(to_text),
                disciplines: objects(
                    section,
                    "disciplines",
                    &at,
                    (DISCIPLINES_NOT_ARRAY, DISCIPLINE_EMPTY),
                    diagnostics,
                )
                .map(|disciplines| {
                    disciplines
                        .into_iter()
                        .map(|(at, discipline)| read_discipline(discipline, &at, diagnostics))
                        .collect()
                }),
                extra: rest(section, ["section", "disciplines"]),
            })
            .collect();
        Self {
            sections,
            written: None,
        }
    }

    /// The sections as the JSON array they are written as.
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

fn read_discipline(
    discipline: &Value,
    at: &JsonPath,
    diagnostics: &mut Diagnostics,
) -> CreditsDiscipline {
    CreditsDiscipline {
        discipline: raw_member(discipline, "discipline").map(to_text),
        titles: objects(
            discipline,
            "titles",
            at,
            (TITLES_NOT_ARRAY, TITLE_EMPTY),
            diagnostics,
        )
        .map(|titles| {
            titles
                .into_iter()
                .map(|(at, title)| CreditsTitle {
                    title: raw_member(title, "title").map(to_text),
                    names: raw_member(title, "names").map(|names| {
                        if let Some(names) = names.as_array() {
                            names.iter().map(to_text).collect()
                        } else {
                            shape(NAMES_NOT_ARRAY, &key(&at, "names"), diagnostics);
                            Vec::new()
                        }
                    }),
                    extra: rest(title, ["title", "names"]),
                })
                .collect()
        }),
        extra: rest(discipline, ["discipline", "titles"]),
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
            diagnostics.push(finding(
                DiagnosticCode::InvalidValue { text: None },
                Severity::Info,
                index(&JsonPath::new(), line),
            ));
        }
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::test_cost::assert_linear_at;
    use bedrock_addon_support::jsoncpp::read;

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
    use bedrock_addon_support::fixed_path::testing::{assert_kept_root_written_linearly, list};

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
