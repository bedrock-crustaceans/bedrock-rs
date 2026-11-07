//! `font/emoticons.json`.

use std::collections::HashSet;

use indexmap::IndexMap;

use super::fields::{Fields, Written};
use super::{Emoticon, EmoticonList};
use bedrock_addon_core::diagnostic::builders::{invalid, malformed};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::build::array;
use bedrock_addon_core::json::kit::fixed_path::Authored;

/// A base-16 `strtol` narrowed to a 32-bit integer, as the game reads an emoticon code: white
/// space, a sign and `0x` are skipped, the hex digits that follow are the number and the rest is
/// ignored; `None` without a digit and for a number outside `i32`.
fn hex_int32(text: &str) -> Option<i32> {
    let bytes = text.as_bytes();
    let mut at = 0;
    while bytes
        .get(at)
        .is_some_and(|b| matches!(b, b' ' | b'\t'..=b'\r'))
    {
        at += 1;
    }
    let negative = match bytes.get(at) {
        Some(b'-') => {
            at += 1;
            true
        }
        Some(b'+') => {
            at += 1;
            false
        }
        _ => false,
    };
    let hex = |at: usize| bytes.get(at).and_then(|b| char::from(*b).to_digit(16));
    // `0x` is a prefix only before a digit; otherwise the `0` alone is the number.
    if bytes.get(at) == Some(&b'0')
        && matches!(bytes.get(at + 1), Some(b'x' | b'X'))
        && hex(at + 2).is_some()
    {
        at += 2;
    }
    let mut value: i64 = 0;
    let mut digits = 0;
    while let Some(digit) = hex(at) {
        // Past `i32` the entry is skipped whatever follows; stop growing.
        value = (value * 16 + i64::from(digit)).min(i64::from(u32::MAX) * 16);
        digits += 1;
        at += 1;
    }
    if digits == 0 {
        return None;
    }
    i32::try_from(if negative { -value } else { value }).ok()
}

/// Whether the game's text scan can match `name` between colons.
fn matchable(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

impl Emoticon {
    /// The character the emoticon stands for, as the game reads `code`: a base-16 number
    /// after optional white space, a sign and `0x`, with whatever follows the digits ignored.
    /// `None` when no digit is there or the number is outside a 32-bit integer: the game skips
    /// the emoticon.
    pub fn code_point(&self) -> Option<i32> {
        hex_int32(&self.code)
    }
}

impl Authored for EmoticonList {
    /// Reads the file's root. A root that is no array has no emoticon.
    fn read(root: &Value, diagnostics: &mut Diagnostics) -> Self {
        let at = JsonPath::new();
        let Some(elements) = root.as_array() else {
            // The game returns silently here, so the crate reports an information without the
            // "Malformed JSON" assert, which belongs to an empty file.
            diagnostics.push(
                Diagnostic::info(malformed("the root is not an array".to_owned(), None)).at(at),
            );
            return Self::default();
        };
        let mut seen: HashSet<&str> = HashSet::new();
        let mut list = Vec::new();
        for (i, element) in elements.iter().enumerate() {
            let at = at.index(i);
            let mut fields = Fields::new(element, at.clone());
            let (name, code) = (fields.raw("name"), fields.raw("code"));
            let (Some(name), Some(code)) = (
                name.and_then(Value::as_string),
                code.and_then(Value::as_string),
            ) else {
                diagnostics.push(Diagnostic::info(invalid()).at(at));
                continue;
            };
            let emoticon = Emoticon {
                name: name.as_str().to_owned(),
                code: code.as_str().to_owned(),
                extra: fields.finish(diagnostics),
            };
            if emoticon.code_point().is_none() {
                diagnostics.push(Diagnostic::info(invalid()).at(at.child("code")));
            } else if !seen.insert(name.as_str()) {
                // The first entry of a name stands.
                diagnostics
                    .push(Diagnostic::of(&core_codes::FONT_NAME_OVERRIDDEN).at(at.child("name")));
            }
            if !matchable(&emoticon.name) {
                // A name that can never be matched in text.
                diagnostics.push(Diagnostic::info(invalid()).at(at.child("name")));
            }
            list.push(emoticon);
        }
        Self {
            emoticons: list,
            written: bedrock_addon_core::json::kit::Written::default(),
        }
    }

    /// The emoticons as the JSON array they are written as.
    fn fields_value(&self) -> Value {
        array(self.emoticons.iter().map(|emoticon| {
            let mut object = Written::default();
            object.put("name", Some(Value::from(emoticon.name.as_str())));
            object.put("code", Some(Value::from(emoticon.code.as_str())));
            object.finish(&emoticon.extra)
        }))
    }

    fn written_mut(&mut self) -> &mut Option<Value> {
        &mut self.written
    }
}

impl EmoticonList {
    /// The emoticons the game registers: each name with its character, the first entry of a
    /// name standing and an entry whose `code` is no number left out. A name with a character
    /// other than a letter, a digit or `_` is registered and never matched in text.
    pub fn code_points(&self) -> IndexMap<&str, i32> {
        let mut registered = IndexMap::new();
        for emoticon in &self.emoticons {
            if let Some(code_point) = emoticon.code_point() {
                registered
                    .entry(emoticon.name.as_str())
                    .or_insert(code_point);
            }
        }
        registered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::Diagnostics;
    use bedrock_addon_core::json::jsoncpp::read;
    use bedrock_addon_core::test_cost::assert_linear_at;

    /// The emoticon list is read in time linear in its entries, every name its own.
    #[test]
    fn the_emoticon_list_is_read_in_linear_time() {
        let build = |entries: usize| {
            let emoticons: Vec<String> = (0..entries)
                .map(|i| format!(r#"{{ "name": "e{i}", "code": "0x{i:X}" }}"#))
                .collect();
            read(format!("[{}]", emoticons.join(",")).as_bytes()).expect("JSON")
        };
        assert_linear_at("EmoticonList::from_value", 10_000, build, |root| {
            let mut diagnostics = Diagnostics::default();
            let list = EmoticonList::from_value(root, &mut diagnostics);
            assert_eq!(
                Some(list.emoticons.len()),
                root.as_array().map(|a| a.iter().count())
            );
            assert!(diagnostics.is_empty());
        });
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
        let root =
            bedrock_addon_core::json::jsoncpp::read(br#"[{"name": "a", "code": "0xE000"}, 3]"#)
                .expect("JSON");
        let loaded = EmoticonList::from_value(&root, &mut Diagnostics::default());
        assert_eq!(*loaded.written, Some(root.clone()));
        let built = EmoticonList {
            written: bedrock_addon_core::json::kit::Written::default(),
            ..loaded.clone()
        };
        assert_eq!(loaded, built);
        assert_eq!(loaded.to_value(), root);
        assert_ne!(built.to_value(), root);
    }

    /// A write of a value that keeps its root is linear in its entries, with one entry the
    /// game skips.
    #[test]
    fn a_kept_root_is_written_in_linear_time() {
        assert_kept_root_written_linearly::<EmoticonList>(
            "EmoticonList::to_value",
            4000,
            |entries| {
                let emoticons = list(entries, |i| {
                    format!(r#"{{ "name": "e{i}", "code": "0x{i:X}" }}"#)
                });
                format!("[{emoticons}, 5]")
            },
        );
    }
}
