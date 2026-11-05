//! What `version_chain!`'s expansion calls. Not an API.

use super::payload::span_of;
use super::pipeline::{molang_context, unsupported};
use super::range::{Selection, select};
use super::{
    BETA_VERSION, EnvelopeCheck, Family, ParseCx, StructIndex, Upgrade, UpgradeCx, VersionChain,
    VersionStruct,
};
use crate::diagnostic::{Diagnostics, JsonPath, Location};
use crate::json::{EncodeError, JsonSlice, Value};
use crate::load::LoadContext;
use crate::molang::pass_one;
use crate::version::{FormatVersion, FormatVersionInput};

/// Parses `payload` as `T`, then runs Molang pass 1 over it with the document's version
/// (the pack's version is `LoadContext::molang_version`).
pub fn parse_with<T: VersionStruct>(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<T> {
    let mut value = T::parse(payload, cx, d)?;
    let molang = molang_context(&cx.written, cx.load);
    let base = Location::new(None, None, cx.at.clone(), Some(span_of(payload)));
    pass_one(&mut value, &molang, &base, d);
    Some(value)
}

/// One step of a generated chain's walk: the struct `take` finds in `any`, upgraded one version
/// and wrapped again by `wrap`, in the same box; `any` unchanged when it holds another struct
/// (which the walk never asks for; `take` boxes it so that its `Err` is small).
///
/// Never inlined, and the value passes boxed: the values of a step live in this frame and
/// leave the stack with it, so the walk needs the stack of one step whatever the length of the
/// chain.
#[inline(never)]
pub fn step<A, T: Upgrade>(
    mut any: Box<A>,
    take: fn(A) -> Result<T, Box<A>>,
    wrap: fn(T::Next) -> A,
    cx: &mut UpgradeCx<'_>,
) -> Box<A> {
    *any = match take(*any) {
        Ok(value) => wrap(value.upgrade(cx)),
        Err(other) => *other,
    };
    any
}

/// `VersionChain::parse` of a generated chain: the struct `v` selects under `F`'s range model,
/// or `UnsupportedVersion`. `payload` is the payload under `F`'s key (the whole document on the
/// raw-text path), so findings are below the payload key, as [`load`]'s are.
///
/// [`load`]: super::load()
pub fn parse_selected<F: Family>(
    payload: JsonSlice<'_>,
    v: FormatVersion,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<F::Any> {
    let written = FormatVersionInput {
        version: Some(v),
        beta_literal: false,
        prerelease: false,
    };
    let mut at = JsonPath::new();
    if !raw_text::<F>(v) {
        at.push(F::PAYLOAD_KEY);
    }
    let Selection::Struct(i) = select::<F>(&written) else {
        let mut version_at = JsonPath::new();
        version_at.push("format_version");
        d.push(unsupported(&v.to_string(), version_at));
        return None;
    };
    let cx = ParseCx {
        load: cx,
        written,
        original_version: v,
        at,
    };
    <F::Any as VersionChain<F>>::parse_struct(payload, StructIndex::Stable(i), &cx, d)
}

/// A whole document in the struct's own version: the envelope around `payload`
/// (`"format_version"` first, `"beta"` for the beta struct), or `payload` itself where the
/// family reads the root struct from the whole text.
pub fn document_json<F: Family>(
    version: FormatVersion,
    payload: &Value,
) -> Result<String, EncodeError> {
    if raw_text::<F>(version) {
        return crate::json::to_json(payload);
    }
    let written: Value = if version == BETA_VERSION {
        "beta".into()
    } else {
        version.to_string().into()
    };
    let mut root = facet_value::VObject::new();
    root.insert("format_version", written);
    root.insert(F::PAYLOAD_KEY, payload.clone());
    crate::json::to_json(&Value::from(root))
}

/// Whether a document of `version` is read on the raw-text path, where the payload is the
/// whole document and no envelope is checked.
fn raw_text<F: Family>(version: FormatVersion) -> bool {
    match F::ENVELOPE {
        EnvelopeCheck::Always | EnvelopeCheck::Uncounted => false,
        EnvelopeCheck::Never => true,
        EnvelopeCheck::From(minimum) => version < minimum,
    }
}
