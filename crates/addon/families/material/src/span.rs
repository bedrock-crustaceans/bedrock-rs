//! Where a finding stands in the text of the copy that wrote what it is about.
//!
//! The readers of this family work on parsed values, which have no offsets, and the copies of
//! a material file are merged before they are read. A finding is placed in the copy that
//! wrote its member, at the path the member has there; this module gives it the span of that
//! value in that copy's text.

use std::collections::HashMap;

use bytes::Bytes;
use facet_json::RawJson;
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, JsonPathSegment, Span};
use bedrock_addon_core::json::{JsonSlice, parse_at, strip_bom};

/// The spans of the values `paths` name in the text of one file, each `None` when a step of
/// its path is not in the text. Offsets count from the start of the file, a byte-order mark
/// included. Of a key written twice the last value is the one found, as the readers keep it.
///
/// Every level of the text that a path goes through is split once, whatever the number of
/// paths through it, so the cost is the size of the text times the depth of the paths, never
/// the size of the text per path.
pub(super) fn spans_of(bytes: &[u8], paths: &[&JsonPath]) -> Vec<Option<Span>> {
    let mut out = vec![None; paths.len()];
    let stripped = strip_bom(bytes);
    let Ok(text) = std::str::from_utf8(stripped) else {
        return out;
    };
    let root = JsonSlice {
        text,
        offset: bytes.len() - stripped.len(),
    };
    let segments: Vec<Vec<JsonPathSegment<'_>>> =
        paths.iter().map(|path| path.segments().collect()).collect();
    let wanted = segments
        .iter()
        .enumerate()
        .map(|(index, path)| (index, path.as_slice()))
        .collect();
    descend(root, wanted, &mut out);
    out
}

/// One path still to follow below a slice: the place of its span in the output, and the steps
/// left.
type Wanted<'p, 'k> = (usize, &'p [JsonPathSegment<'k>]);

/// Follows every path of `wanted` below `slice`. The recursion is as deep as the longest
/// path, and a path is no longer than the reader's depth limit lets a value nest.
fn descend<'a>(slice: JsonSlice<'a>, wanted: Vec<Wanted<'_, '_>>, out: &mut [Option<Span>]) {
    let mut keys: HashMap<&str, Vec<Wanted<'_, '_>>> = HashMap::new();
    let mut indices: HashMap<usize, Vec<Wanted<'_, '_>>> = HashMap::new();
    for (place, path) in wanted {
        match path.split_first() {
            None => {
                if let Some(span) = out.get_mut(place) {
                    *span = bedrock_addon_core::json::span_at(slice, &JsonPath::new());
                }
            }
            Some((JsonPathSegment::Key(key), rest)) => {
                keys.entry(key).or_default().push((place, rest));
            }
            Some((JsonPathSegment::Index(index), rest)) => {
                indices.entry(*index).or_default().push((place, rest));
            }
        }
    }
    if !keys.is_empty()
        && let Ok(members) = parse_at::<IndexMap<String, RawJson<'a>>>(slice)
    {
        for (key, below) in keys {
            if let Some(member) = members.get(key).and_then(|raw| slice.nested(raw)) {
                descend(member, below, out);
            }
        }
    }
    if !indices.is_empty()
        && let Ok(elements) = parse_at::<Vec<RawJson<'a>>>(slice)
    {
        for (index, below) in indices {
            if let Some(element) = elements.get(index).and_then(|raw| slice.nested(raw)) {
                descend(element, below, out);
            }
        }
    }
}

/// Gives every finding of `findings` that has a path and no span the span of the value its
/// path names in `bytes`, the text of the file the findings are placed in. A finding about
/// the file as a whole (no path) and one whose path names nothing in the text (a member that
/// is missing) stay without a span.
pub(super) fn fill(bytes: &[u8], findings: &mut [&mut Diagnostic]) {
    let open: Vec<usize> = (0..findings.len())
        .filter(|&at| {
            let location = &findings[at].location;
            location.span.is_none() && !location.json_path.is_empty()
        })
        .collect();
    if open.is_empty() {
        return;
    }
    let spans = {
        let paths: Vec<&JsonPath> = open
            .iter()
            .map(|&at| &findings[at].location.json_path)
            .collect();
        spans_of(bytes, &paths)
    };
    for (at, span) in open.into_iter().zip(spans) {
        findings[at].location.span = span;
    }
}

/// [`fill`] for a whole list of findings of one file.
pub(super) fn fill_all(bytes: &[u8], diagnostics: &mut Diagnostics) {
    let mut findings: Vec<Diagnostic> = std::mem::take(diagnostics).into_iter().collect();
    fill(bytes, &mut findings.iter_mut().collect::<Vec<_>>());
    diagnostics.extend(findings);
}

/// The texts of the copies a merged file was read from, by the place each copy has among
/// them, and the findings waiting for a span in each.
#[derive(Debug, Default)]
pub(super) struct Texts {
    texts: Vec<Bytes>,
}

impl Texts {
    /// Adds a copy's text; its place is the number of texts before it.
    pub(super) fn push(&mut self, bytes: Bytes) {
        self.texts.push(bytes);
    }

    /// Gives each finding of `placed` the span its path has in the text of the copy it was
    /// placed in (`None`: it was placed in no copy and keeps what it has). Each copy's text is
    /// gone through once for all its findings.
    pub(super) fn fill(&self, placed: &mut [(Option<usize>, Diagnostic)]) {
        let mut by_copy: IndexMap<usize, Vec<&mut Diagnostic>> = IndexMap::new();
        for (copy, finding) in placed.iter_mut() {
            if let Some(copy) = copy {
                by_copy.entry(*copy).or_default().push(finding);
            }
        }
        for (copy, mut findings) in by_copy {
            if let Some(bytes) = self.texts.get(copy) {
                fill(bytes, &mut findings);
            }
        }
    }
}
