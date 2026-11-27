//! The `facet` impl of [`MolangList`]: a proxy over every JSON shape, the technique of `json`'s
//! strict wrappers.
//!
//! An untagged `One(Molang) | Many(Vec<MolangList>)` cannot be derived directly: facet's untagged
//! solver does not descend into a proxied newtype variant. So the list reads a [`ListForm`] and
//! converts it the way the Molang value proxy converts a scalar.

// The untagged derive fails with "unexpected token: got f64, expected matching untagged variant
// for scalar" under facet-json 0.50.0-rc.7. The scalar conversions are those of the Molang
// value proxy.

use facet::Facet;
use indexmap::IndexMap;

use super::{Molang, MolangList, MolangSource};
use crate::json::Value;

/// Every JSON shape a list element can be. The shapes a list does not take exist so the
/// conversion can reject them: without them the untagged solver would coerce `null` into a
/// number and a number into a string. `String` comes first so `"1"` is never read as a number.
#[derive(Debug, Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(dead_code, reason = "the object payload exists to be rejected")]
pub(crate) enum ListForm {
    /// `null`.
    Null,
    /// A string: an expression.
    String(String),
    /// A bool.
    Bool(bool),
    /// A non-negative integer token.
    UInt(u64),
    /// A negative integer token.
    Int(i64),
    /// A real token.
    Number(f64),
    /// An array: each element is read the same way.
    Array(Vec<ListForm>),
    /// An object: never a list element.
    Object(IndexMap<String, Value>),
}

/// [`MolangList`] as JSON.
#[derive(Debug, Facet)]
#[facet(transparent)]
pub(crate) struct ListProxy(ListForm);

/// Why the `facet` impl of [`MolangList`] rejected a JSON value or could not write one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum ListRejected {
    /// `null` inside an array (at the top, the field's `Option` takes it).
    #[error("a Molang list element cannot be null")]
    Null,
    /// An object, including the `{"expression", "version"}` form.
    #[error("a Molang list element cannot be an object")]
    Object,
    /// A number that is not finite as an `f32`, read or written.
    #[error("a Molang constant must be a finite f32")]
    NonFinite,
    /// A value of a kind a newer molang added, which this list cannot write.
    #[error("a Molang value of an unknown kind")]
    Unsupported,
}

impl TryFrom<ListProxy> for MolangList {
    type Error = ListRejected;

    fn try_from(proxy: ListProxy) -> Result<Self, ListRejected> {
        from_form(proxy.0)
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)] // the nearest f32 is the reading
fn from_form(form: ListForm) -> Result<MolangList, ListRejected> {
    let constant = |x: f32| {
        if x.is_finite() {
            Ok(MolangList::One(Molang::Const(x)))
        } else {
            Err(ListRejected::NonFinite)
        }
    };
    match form {
        ListForm::Null => Err(ListRejected::Null),
        ListForm::String(text) => Ok(MolangList::One(Molang::Expr(
            MolangSource::string_without_context(text),
        ))),
        ListForm::Bool(b) => Ok(MolangList::One(Molang::Bool(b))),
        ListForm::UInt(x) => constant(x as f32),
        ListForm::Int(x) => constant(x as f32),
        ListForm::Number(x) => constant(x as f32),
        ListForm::Array(elements) => elements
            .into_iter()
            .map(from_form)
            .collect::<Result<_, _>>()
            .map(MolangList::Many),
        ListForm::Object(_) => Err(ListRejected::Object),
    }
}

impl TryFrom<&MolangList> for ListProxy {
    type Error = ListRejected;

    fn try_from(list: &MolangList) -> Result<Self, ListRejected> {
        to_form(list).map(ListProxy)
    }
}

fn to_form(list: &MolangList) -> Result<ListForm, ListRejected> {
    Ok(match list {
        MolangList::One(Molang::Const(x)) if !x.is_finite() => return Err(ListRejected::NonFinite),
        MolangList::One(Molang::Const(x)) => ListForm::Number(f64::from(*x)),
        MolangList::One(Molang::Bool(b)) => ListForm::Bool(*b),
        // A list never reads the object form; one built by hand is written as its text.
        MolangList::One(Molang::Expr(source)) => ListForm::String(source.as_str().to_owned()),
        MolangList::One(_) => return Err(ListRejected::Unsupported),
        MolangList::Many(elements) => {
            ListForm::Array(elements.iter().map(to_form).collect::<Result<_, _>>()?)
        }
    })
}
