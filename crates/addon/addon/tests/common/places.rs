//! What the probes of `writable.rs` and `writable_legacy.rs` share: reading a text through a
//! family's public entry, the values of a JSON text with the place of each token, and the
//! comparison of written values.

#![allow(dead_code, reason = "each probe uses its part")]

use bedrock_addon::diagnostic::Diagnostic;
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

// --- reading ---------------------------------------------------------------------------------

/// What a family made of one text.
pub struct Outcome {
    pub loaded: bool,
    /// The value written back, or the writer's error.
    pub written: Result<String, String>,
    pub findings: Vec<Diagnostic>,
}

/// `write` is given the document, not its value: the document writes its unknown members
/// (`Document::to_json`), which the value alone does not hold.
pub fn outcome<T>(
    result: Result<Document<T>, Rejected>,
    write: impl Fn(&Document<T>) -> Result<String, String>,
) -> Outcome {
    match result {
        Ok(document) => Outcome {
            loaded: true,
            written: write(&document),
            findings: document.diagnostics().iter().cloned().collect(),
        },
        Err(rejected) => Outcome {
            loaded: false,
            written: Err("not loaded".to_owned()),
            findings: rejected.diagnostics.iter().cloned().collect(),
        },
    }
}

/// The family's public entry for one file.
pub fn content_in<T: Content>(path: &PackPath, text: &str, cx: &LoadContext) -> Outcome {
    outcome(T::parse(path, text.as_bytes(), cx), |value| {
        value.to_json().map_err(|error| format!("{error:?}"))
    })
}

// --- the values of a text --------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Number,
    String,
    Other,
    Object,
    Array,
}

/// A value of a text: its path, what it is and where its token lies.
#[derive(Clone, Debug)]
pub struct Node {
    pub path: Vec<String>,
    /// Whether each segment of the path is an index.
    pub index: Vec<bool>,
    pub kind: Kind,
    pub start: usize,
    pub end: usize,
}

impl Node {
    pub fn shown(&self) -> String {
        self.path.join("/")
    }

    /// The path with every index as `*`.
    pub fn starred(&self) -> Vec<&str> {
        self.path
            .iter()
            .zip(&self.index)
            .map(|(segment, index)| if *index { "*" } else { segment.as_str() })
            .collect()
    }

    /// The last two member names of the path (indices as `*`).
    pub fn signature(&self) -> String {
        let starred = self.starred();
        let mut keys = 0;
        let mut from = starred.len();
        while from > 0 && keys < 2 {
            from -= 1;
            keys += usize::from(starred[from] != "*");
        }
        starred[from..].join("/")
    }

    /// The member's name: the last key of the path, with `/*` per index after it.
    pub fn name(&self) -> String {
        let starred = self.starred();
        let last_key = starred.iter().rposition(|segment| *segment != "*");
        starred[last_key.unwrap_or(0)..].join("/")
    }
}

/// Every value of `text` (plain JSON: the small documents, and what `plain` makes of a file).
pub fn nodes(text: &str) -> Vec<Node> {
    fn skip(bytes: &[u8], mut at: usize) -> usize {
        while bytes[at].is_ascii_whitespace() {
            at += 1;
        }
        at
    }
    fn string(bytes: &[u8], at: usize) -> usize {
        assert_eq!(bytes[at], b'"');
        let mut end = at + 1;
        while bytes[end] != b'"' {
            end += if bytes[end] == b'\\' { 2 } else { 1 };
        }
        end + 1
    }
    fn value(
        text: &str,
        at: usize,
        path: &mut Vec<String>,
        index: &mut Vec<bool>,
        out: &mut Vec<Node>,
    ) -> usize {
        let bytes = text.as_bytes();
        let at = skip(bytes, at);
        let mut node = Node {
            path: path.clone(),
            index: index.clone(),
            kind: Kind::Other,
            start: at,
            end: at,
        };
        match bytes[at] {
            b'{' => {
                let mut next = skip(bytes, at + 1);
                while bytes[next] != b'}' {
                    let end = string(bytes, next);
                    path.push(text[next + 1..end - 1].to_owned());
                    index.push(false);
                    next = skip(bytes, end);
                    assert_eq!(bytes[next], b':');
                    next = skip(bytes, value(text, next + 1, path, index, out));
                    path.pop();
                    index.pop();
                    if bytes[next] == b',' {
                        next = skip(bytes, next + 1);
                    }
                }
                node.kind = Kind::Object;
                node.end = next + 1;
            }
            b'[' => {
                let mut next = skip(bytes, at + 1);
                let mut element = 0;
                while bytes[next] != b']' {
                    path.push(element.to_string());
                    index.push(true);
                    next = skip(bytes, value(text, next, path, index, out));
                    path.pop();
                    index.pop();
                    element += 1;
                    if bytes[next] == b',' {
                        next = skip(bytes, next + 1);
                    }
                }
                node.kind = Kind::Array;
                node.end = next + 1;
            }
            b'"' => {
                node.kind = Kind::String;
                node.end = string(bytes, at);
            }
            first => {
                let mut end = at;
                while !matches!(bytes.get(end), None | Some(b',' | b'}' | b']'))
                    && !bytes[end].is_ascii_whitespace()
                {
                    end += 1;
                }
                if first == b'-' || first.is_ascii_digit() {
                    node.kind = Kind::Number;
                }
                node.end = end;
            }
        }
        let end = node.end;
        out.push(node);
        end
    }
    let mut out = Vec::new();
    value(
        &format!("{text} "),
        0,
        &mut Vec::new(),
        &mut Vec::new(),
        &mut out,
    );
    out.sort_by_key(|node| node.start);
    out
}

/// `key` as a JSON string (`None` for a key with a character JSON escapes).
fn quoted(key: &str) -> Option<String> {
    (!key.contains(['"', '\\']) && !key.chars().any(char::is_control)).then(|| format!("\"{key}\""))
}

/// `text` without the member or element at `node` (`None` when the text has no such token
/// boundaries: the root).
pub fn without(text: &str, node: &Node) -> Option<String> {
    let bytes = text.as_bytes();
    let last = node.index.last()?;
    let mut start = node.start;
    if !*last {
        // The member's key and colon.
        start = text[..start].rfind(':')?;
        start = text[..start].rfind('"')?;
        let key = node.path.last()?;
        let quoted = quoted(key)?;
        start = (start + 1).checked_sub(quoted.len())?;
        text[start..].strip_prefix(quoted.as_str())?;
    }
    let mut end = node.end;
    while bytes.get(end).is_some_and(u8::is_ascii_whitespace) {
        end += 1;
    }
    if bytes.get(end) == Some(&b',') {
        end += 1;
    } else {
        // The last member: the comma before it goes.
        let before = text[..start].trim_end();
        if before.ends_with(',') {
            start = before.len() - 1;
        }
    }
    Some(format!("{}{}", &text[..start], &text[end..]))
}

pub fn parse(text: &str) -> Option<Value> {
    bedrock_addon::json::from_str_jsonc::<Value>(text).ok()
}

/// The value of `json` at `path`.
pub fn lookup<'v>(json: &'v Value, path: &[String]) -> Option<&'v Value> {
    let mut at = json;
    for segment in path {
        at = if let Some(object) = at.as_object() {
            object.get(segment)?
        } else {
            at.as_array()?.get(segment.parse::<usize>().ok()?)?
        };
    }
    Some(at)
}

/// A scalar as a text that is equal for equal values: a number by the number it spells.
pub fn scalar(value: &Value) -> Option<String> {
    if value.is_null() {
        Some("null".to_owned())
    } else if let Some(flag) = value.as_bool() {
        Some(format!("b:{flag}"))
    } else if let Some(number) = value.as_number() {
        Some(format!("n:{:?}", number.to_f64_lossy() + 0.0))
    } else {
        value.as_string().map(|text| format!("s:{}", text.as_str()))
    }
}

/// The number `value` is.
pub fn number(value: &Value) -> Option<f64> {
    value.as_number().map(facet_value::VNumber::to_f64_lossy)
}

/// Whether `held` is the `f32` nearest to the number `token` spells.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the nearest f32 is the question"
)]
#[expect(
    clippy::float_cmp,
    reason = "the question is whether the two f32 are the same value"
)]
pub fn same_float(held: f64, token: &str) -> bool {
    token
        .parse::<f64>()
        .is_ok_and(|spelled| (held as f32) == (spelled as f32) && (held as f32).is_finite())
}

/// Every scalar of `value`, as [`scalar`] spells it.
pub fn leaves(value: &Value, out: &mut Vec<String>) {
    if let Some(object) = value.as_object() {
        for (_, member) in object {
            leaves(member, out);
        }
    } else if let Some(array) = value.as_array() {
        for element in array.as_slice() {
            leaves(element, out);
        }
    } else if let Some(leaf) = scalar(value) {
        out.push(leaf);
    }
}

/// The findings as `code at path`.
pub fn kinds(findings: &[Diagnostic]) -> Vec<String> {
    findings
        .iter()
        .map(|d| format!("{} at `{}`", code(d), d.location.json_path))
        .collect()
}

/// The variant name of a finding's code.
pub fn code(finding: &Diagnostic) -> String {
    format!("{:?}", finding.code)
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .collect()
}

/// The path of a finding, a segment at a time.
pub fn path_of(finding: &Diagnostic) -> Vec<String> {
    use bedrock_addon::diagnostic::JsonPathSegment;
    finding
        .location
        .json_path
        .segments()
        .map(|segment| match segment {
            JsonPathSegment::Key(key) => key.to_owned(),
            JsonPathSegment::Index(index) => index.to_string(),
        })
        .collect()
}

/// What the non-strict readers make of `replacement` in a member of any scalar type: the
/// number itself (a float member), `true` as 1 and `false` as 0, and for each integer width
/// the low bits of the integer (a real truncated toward zero first; a negative real as the
/// negated truncation of its magnitude, saturating at the bounds of the conversion: the 8,
/// 16 and 32 bit readers convert through `i32`, the 64 bit ones through `i64` and `u64`).
pub fn non_strict_readings(replacement: &str) -> Vec<f64> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn through(x: f64) -> [u64; 3] {
        let via = |t: fn(f64) -> u64| if x >= 0.0 { t(x) } else { t(-x).wrapping_neg() };
        [
            via(|v| u64::from(v as i32 as u32)),
            via(|v| v as i64 as u64),
            via(|v| v as u64),
        ]
    }
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
    fn widths(bits: u64) -> Vec<f64> {
        let mut out = Vec::new();
        for width in [8u32, 16, 32, 64] {
            let mask = if width == 64 {
                u64::MAX
            } else {
                (1 << width) - 1
            };
            let low = bits & mask;
            out.push(low as f64);
            let signed = if width == 64 {
                low as i64
            } else {
                ((low << (64 - width)) as i64) >> (64 - width)
            };
            out.push(signed as f64);
        }
        out
    }
    match replacement {
        "true" => return vec![1.0],
        "false" => return vec![0.0],
        _ => {}
    }
    let mut out = Vec::new();
    if let Ok(integer) = replacement.parse::<i64>() {
        out.extend(widths(integer.cast_unsigned()));
    } else if let Ok(integer) = replacement.parse::<u64>() {
        out.extend(widths(integer));
    } else if let Ok(real) = replacement.parse::<f64>()
        && real.is_finite()
    {
        out.push(real);
        for bits in through(real) {
            out.extend(widths(bits));
        }
    }
    out
}
