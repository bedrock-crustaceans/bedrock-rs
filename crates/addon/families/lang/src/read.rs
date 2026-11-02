//! The `.lang` reader: vanilla's `Localization::appendTranslations`, a byte-level state machine
//! (`support::text::lang_scan`), and our lints on what it read.

use std::collections::HashMap;

use super::{KeyFilter, LangFile, LangLayout, Language, Line, Translations};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity,
    Span, VanillaText,
};
use bedrock_addon_support::text::lang_scan::{BOM, End, Kind, scan};

/// Bytes as text: vanilla copies bytes without decoding them.
fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The key as the table stores it: ASCII `A`–`Z` lower-cased, every other byte unchanged.
// `c - 'A' < 26` → `c | 0x20`.
fn stored_key(key: &str) -> String {
    key.to_ascii_lowercase()
}

/// A finding of ours about the entry `key` on the line at `span`.
fn finding(code: DiagnosticCode, severity: Severity, key: &str, span: Span) -> Diagnostic {
    let mut at = JsonPath::new();
    at.push(key);
    Diagnostic::new(
        code,
        severity,
        Phase::Parse,
        Location::new(None, None, at, Some(span)),
    )
}

/// Vanilla's two warnings for a line that ends while its key is read (V1, V2): the 1-based
/// line number and the key read so far, lower-cased.
// `Line: %zu - Invalid lang file format. New line character was found while parsing key:
// '%s'.` and `… Null character …`, both
// `ContentLogHelper::_contentLog(true, LogLevel::Warning, LogArea::Localization, …)`.
fn no_equals(line: usize, key: &str, end: End, span: Span) -> Diagnostic {
    let template = match end {
        End::LineFeed => {
            "Line: {} - Invalid lang file format. New line character was found while parsing key: '{}'."
        }
        End::Terminator => {
            "Line: {} - Invalid lang file format. Null character was found while parsing key: '{}'."
        }
    };
    finding(
        DiagnosticCode::MalformedInput {
            message: "a line without `=`".to_owned(),
            text: Some(VanillaText::Formatted {
                template,
                args: vec![line.to_string(), key.to_owned()],
            }),
        },
        Severity::Warning,
        key,
        span,
    )
}

/// L13: the key was defined on line `earlier` already. The last definition wins, as in
/// vanilla, which says nothing.
// `vanilla = "silent, last wins"`: `try_emplace` then an assign
fn duplicate(key: &str, earlier: usize, span: Span) -> Diagnostic {
    let mut finding = finding(
        DiagnosticCode::DuplicateIdentifier { text: None },
        Severity::Info,
        key,
        span,
    );
    finding.detail = Some(Box::new(DiagnosticDetail::Reference {
        kind: "line",
        id: earlier.to_string(),
        hint: None,
    }));
    finding
}

/// L4: a tab ended the value and something other than a comment follows it. Vanilla ignores
/// that text without a word.
// `vanilla = "silent"`. Blanks between the tab and the `#` are not text
fn tab_cuts_text(trailing: &[u8]) -> bool {
    // What follows a value starts with the tab or the `##` that ended it; past the tab and
    // any blanks, a `#` is a comment either way.
    let rest = trailing.trim_ascii_start();
    !rest.is_empty() && !rest.starts_with(b"#")
}

/// L11: the value holds bytes the lookup-time expansion treats in a way an author rarely
/// means: a `$s` or `$d` outside a `%N$s` form (erased wherever it stands), `%%` (not an
/// escape: both bytes stay), `%0` or an index of two digits (only `%1` to `%9` are indices).
// `vanilla = "silent"`; `replaceTokens`.
fn odd_placeholder(value: &[u8]) -> bool {
    value.iter().enumerate().any(|(at, byte)| {
        let next = value.get(at + 1);
        match byte {
            b'$' if matches!(next, Some(b's' | b'd')) => {
                let positional =
                    at >= 2 && value[at - 2] == b'%' && (b'1'..=b'9').contains(&value[at - 1]);
                !positional
            }
            b'%' => match next {
                Some(b'%' | b'0') => true,
                Some(digit) if digit.is_ascii_digit() => {
                    value.get(at + 2).is_some_and(u8::is_ascii_digit)
                }
                _ => false,
            },
            _ => false,
        }
    })
}

impl LangFile {
    /// Reads the bytes of a `.lang` file of `language`, with what the reader found.
    ///
    /// A file never fails as a whole: a line vanilla drops is kept as [`Line::Dropped`], with
    /// a warning when vanilla logs one.
    #[must_use]
    pub fn from_bytes(language: Language, bytes: &[u8]) -> (LangFile, Diagnostics) {
        let mut lines = Vec::new();
        let mut diagnostics = Diagnostics::default();
        // The line of the latest definition of each stored key.
        let mut defined: HashMap<String, usize> = HashMap::new();
        for segment in scan(bytes) {
            match segment.kind {
                Kind::Blank => lines.push(Line::Blank),
                Kind::Empty(raw) => lines.push(Line::Dropped(text(&raw))),
                Kind::Comment(comment) => lines.push(Line::Comment(text(&comment))),
                Kind::NoEquals { key, raw, end } => {
                    let key = stored_key(&text(&key));
                    diagnostics.push(no_equals(segment.line, &key, end, segment.span));
                    lines.push(Line::Dropped(text(&raw)));
                }
                Kind::Entry {
                    key,
                    value,
                    trailing,
                } => {
                    let decoded =
                        std::str::from_utf8(&key).is_ok() && std::str::from_utf8(&value).is_ok();
                    let key = text(&key);
                    let stored = stored_key(&key);
                    // L14: vanilla copies the bytes without decoding them; the table here
                    // holds text, so an invalid sequence is read as U+FFFD.
                    // `vanilla = "bytes copied, no decoding"`.
                    if !decoded {
                        diagnostics.push(finding(
                            DiagnosticCode::MalformedInput {
                                message: "a key or value that is not UTF-8".to_owned(),
                                text: None,
                            },
                            Severity::Info,
                            &stored,
                            segment.span,
                        ));
                    }
                    // L3: `=` as the first byte of the line is a key character in vanilla
                    // (`=a=b` is the key `=a`), which is rarely what was meant.
                    // `vanilla = "silent; '=' becomes a key character"`
                    if key.starts_with('=') {
                        diagnostics.push(finding(
                            DiagnosticCode::InvalidValue { text: None },
                            Severity::Info,
                            &stored,
                            segment.span,
                        ));
                    }
                    // L2: nothing is trimmed, so `key = value` is the key `"key "`.
                    // `vanilla = "silent; key stored with the trailing
                    // space"`. The game's only lang-parse texts are
                    // the newline and NUL ones: the lint is ours, an
                    // information.
                    if key.ends_with(' ') || value.starts_with(b" ") {
                        diagnostics.push(finding(
                            DiagnosticCode::LangKeyTrailingSpace,
                            Severity::Info,
                            &stored,
                            segment.span,
                        ));
                    }
                    if let Some(earlier) = defined.insert(stored.clone(), segment.line) {
                        diagnostics.push(duplicate(&stored, earlier, segment.span));
                    }
                    if tab_cuts_text(&trailing) {
                        diagnostics.push(finding(
                            DiagnosticCode::IgnoredMember { text: None },
                            Severity::Info,
                            &stored,
                            segment.span,
                        ));
                    }
                    if odd_placeholder(&value) {
                        diagnostics.push(finding(
                            DiagnosticCode::InvalidValue { text: None },
                            Severity::Info,
                            &stored,
                            segment.span,
                        ));
                    }
                    lines.push(Line::Entry {
                        key,
                        value: text(&value),
                        trailing: text(&trailing),
                    });
                }
            }
        }
        let bom = bytes.starts_with(BOM);
        // A last line without its line feed is written back without one. A last line of only
        // blanks is the exception: its blanks are not kept, so it needs the line feed to stay
        // a line.
        // NULs and carriage returns after the last line feed start no line.
        let body = bytes.strip_prefix(BOM).unwrap_or(bytes);
        let end = body
            .iter()
            .rposition(|byte| !matches!(byte, 0 | b'\r'))
            .map_or(0, |last| last + 1);
        let body = &body[..end];
        let ended = body.is_empty() || body.ends_with(b"\n");
        let final_line_feed = ended || matches!(lines.last(), None | Some(Line::Blank));
        let file = LangFile {
            language,
            lines,
            layout: LangLayout {
                bom,
                final_line_feed,
            },
        };
        (file, diagnostics)
    }

    /// What the stack loader makes of the file: every entry but the pack keywords, the key
    /// lower-cased, the last definition of a key winning.
    #[must_use]
    pub fn translations(&self) -> Translations {
        self.select(&KeyFilter::GLOBAL)
    }

    /// The entries `filter` lets through, stored under its prefix: vanilla's reader with its
    /// exclude list, include list and key prefix.
    #[must_use]
    pub fn select(&self, filter: &KeyFilter<'_>) -> Translations {
        let mut table = Translations::new();
        for line in &self.lines {
            let Line::Entry { key, value, .. } = line else {
                continue;
            };
            // A `~` in a value sets the flag as the value is read, before the lists are
            // looked at.
            table.may_contain_identifier |= value.contains('~');
            let key = stored_key(key);
            // Both lists are matched byte for byte against the lower-cased key without the
            // prefix.
            let listed = |list: &[&str]| list.contains(&key.as_str());
            if listed(filter.exclude) || (!filter.include.is_empty() && !listed(filter.include)) {
                continue;
            }
            // `try_emplace` followed by an assign: the last definition in the file wins
            table
                .strings
                .insert(format!("{}{key}", filter.prefix), value.clone());
        }
        table
    }
}
