//! The test `#[derive(Schema)]` emits beside the `Default` it emits: the default is the value
//! read from an empty object, and it reads back unchanged from what the writer makes of it.

/// How a type's `Default` failed [`default_round_trip`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DefaultFault {
    /// `{}` does not read as the type.
    #[error("the empty object does not read as the type")]
    EmptyObjectUnread,
    /// `{}` reads, as a value other than the default.
    #[error("the empty object reads as a value other than the default")]
    NotTheEmptyObject,
    /// The writer refuses the default; the writer's error.
    #[error("the default cannot be written")]
    Unwritten(#[source] crate::json::EncodeError),
    /// What the writer made of the default is no JSON, or does not read as the type.
    #[error("the written default does not read")]
    WrittenUnread,
    /// A required option is missing from the default, and `{}` or the written default reads.
    #[error("the default reads although a required member is missing from it")]
    ReadsWithoutRequired,
    /// What the writer made of the default reads, as a value other than the default.
    #[error("the written default reads as another value: {written}")]
    NotTheSame {
        /// The text the writer made of the default.
        written: String,
    },
}

/// Checks `T::default()` against the reader and the writer: with `empty_object`, that `{}`
/// reads as the default (a `#[facet(transparent)]` newtype is no object, and passes `false`),
/// then that the default written and read back is the default.
///
/// # Errors
///
/// Returns the first [`DefaultFault`] found, in the order above.
pub fn default_round_trip<T>(empty_object: bool) -> Result<(), DefaultFault>
where
    T: Default + PartialEq + core::fmt::Debug + facet::Facet<'static> + super::FromJson,
{
    let default = T::default();
    if empty_object {
        let empty = crate::json::Value::from(facet_value::VObject::new());
        let parsed: T = super::read_value(&empty).ok_or(DefaultFault::EmptyObjectUnread)?;
        if parsed != default {
            return Err(DefaultFault::NotTheEmptyObject);
        }
    }
    let written = crate::json::to_json(&default).map_err(DefaultFault::Unwritten)?;
    let again: T = crate::json::from_str_jsonc(&written)
        .ok()
        .and_then(|json| super::read_value(&json))
        .ok_or(DefaultFault::WrittenUnread)?;
    if again != default {
        return Err(DefaultFault::NotTheSame { written });
    }
    Ok(())
}

/// Checks the `Default` of a type with a `schema::required` option, which no text spells: the
/// default is written, and neither `{}` nor what the writer made of it reads (the member is told
/// missing, as the game asks for it).
///
/// # Errors
///
/// [`DefaultFault::Unwritten`] when the writer refuses the default,
/// [`DefaultFault::ReadsWithoutRequired`] when either text reads.
pub fn default_refused<T>() -> Result<(), DefaultFault>
where
    T: Default + facet::Facet<'static> + super::FromJson,
{
    let written = crate::json::to_json(&T::default()).map_err(DefaultFault::Unwritten)?;
    let empty = crate::json::Value::from(facet_value::VObject::new());
    let reads = |json: &crate::json::Value| super::read_value::<T>(json).is_some();
    let written = crate::json::from_str_jsonc(&written).ok();
    if reads(&empty) || written.as_ref().is_some_and(reads) {
        return Err(DefaultFault::ReadsWithoutRequired);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::kit::{Node, Read};
    use crate::schema::{self, FromJson, Schema};
    use facet::Facet;

    /// A hand `Default` the empty object does not give back: facet reads an object holding
    /// none of a type's fields as the type's `Default`, and a NaN is not equal to itself.
    #[derive(Debug, PartialEq, Facet)]
    struct HandDefault {
        level: f32,
    }

    impl Default for HandDefault {
        fn default() -> Self {
            Self { level: f32::NAN }
        }
    }

    /// The empty object reads as the number 0, not as the hand default.
    impl FromJson for HandDefault {
        fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
            read.object(node)?;
            Some(Self { level: 0.0 })
        }
    }

    /// A struct whose `Default` the schema derive emits: an optional, a filled and a skipped
    /// field.
    #[derive(Debug, PartialEq, Facet, Schema)]
    struct Derived {
        #[facet(schema::fallback = 1)]
        priority: Option<i32>,
        #[facet(default = 0.5)]
        speed: f32,
        #[facet(skip)]
        seen: bool,
    }

    /// A hand `Default` that sets a skipped field: the writer leaves the field out, so the
    /// written default reads back with the field's own default instead.
    #[derive(Debug, PartialEq, Facet)]
    struct Unkept {
        level: i32,
        #[facet(skip)]
        seen: bool,
    }

    impl Default for Unkept {
        fn default() -> Self {
            Self {
                level: 1,
                seen: true,
            }
        }
    }

    /// The empty object reads as the default, as facet-json read a struct of no member; a `level`
    /// member is read, and the skipped field is its own default.
    impl FromJson for Unkept {
        fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
            let members = read.object(node)?;
            if members.is_empty() {
                return Some(Self::default());
            }
            let level = read.required(&members, "level")?;
            Some(Self {
                level: i32::from_json(&level, read)?,
                seen: false,
            })
        }
    }

    /// A transparent newtype: written as a bare number, so `{}` is not its text.
    #[derive(Debug, PartialEq, Facet, Schema)]
    #[facet(transparent)]
    struct Bare(#[facet(default = 0.4)] f32);

    #[test]
    fn a_default_the_empty_object_does_not_read_is_told() {
        let fault = default_round_trip::<HandDefault>(true);
        assert!(
            matches!(fault, Err(DefaultFault::NotTheEmptyObject)),
            "{fault:?}"
        );
    }

    #[test]
    fn a_default_the_writer_does_not_keep_is_told() {
        let fault = default_round_trip::<Unkept>(true);
        assert!(
            matches!(fault, Err(DefaultFault::NotTheSame { .. })),
            "{fault:?}"
        );
    }

    #[test]
    fn a_schema_default_reads_from_the_empty_object_and_round_trips() {
        let checked = default_round_trip::<Derived>(true);
        assert!(checked.is_ok(), "{checked:?}");
    }

    /// A required option: no text spells the default.
    #[derive(Debug, PartialEq, Facet, Schema)]
    struct Needs {
        #[facet(schema::required)]
        name: Option<String>,
    }

    #[test]
    fn a_default_with_a_required_option_is_refused_by_both_texts() {
        let checked = default_refused::<Needs>();
        assert!(checked.is_ok(), "{checked:?}");
        let fault = default_refused::<Derived>();
        assert!(
            matches!(fault, Err(DefaultFault::ReadsWithoutRequired)),
            "{fault:?}"
        );
    }

    #[test]
    fn a_transparent_default_round_trips_without_the_empty_object() {
        let checked = default_round_trip::<Bare>(false);
        assert!(checked.is_ok(), "{checked:?}");
        let fault = default_round_trip::<Bare>(true);
        assert!(
            matches!(fault, Err(DefaultFault::EmptyObjectUnread)),
            "{fault:?}"
        );
    }
}
