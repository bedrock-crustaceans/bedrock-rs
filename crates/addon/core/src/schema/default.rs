//! The test `#[derive(Schema)]` emits beside the `Default` it emits: the default is the value
//! read from an empty object, and it reads back unchanged from what the writer makes of it.

/// How a type's `Default` failed [`default_round_trip`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DefaultFault {
    /// `{}` does not read as the type; the reader's error.
    #[error("the empty object does not read as the type")]
    EmptyObjectUnread(#[source] facet_json::DeserializeError),
    /// `{}` reads, as a value other than the default.
    #[error("the empty object reads as a value other than the default")]
    NotTheEmptyObject,
    /// The writer refuses the default; the writer's error.
    #[error("the default cannot be written")]
    Unwritten(#[source] crate::json::EncodeError),
    /// What the writer made of the default does not read; the reader's error.
    #[error("the written default does not read")]
    WrittenUnread(#[source] facet_json::DeserializeError),
    /// What the writer made of the default reads, as a value other than the default.
    #[error("the written default reads as another value: {written}")]
    NotTheSame {
        /// The text the writer made of the default.
        written: String,
    },
}

/// Checks `T::default()` against the door's reader and writer: with `empty_object`, that `{}`
/// reads as the default (a `#[facet(transparent)]` newtype is no object, and passes `false`),
/// then that the default written and read back is the default.
///
/// # Errors
///
/// Returns the first [`DefaultFault`] found, in the order above.
pub fn default_round_trip<T>(empty_object: bool) -> Result<(), DefaultFault>
where
    T: Default + PartialEq + core::fmt::Debug + facet::Facet<'static>,
{
    let default = T::default();
    if empty_object {
        let parsed: T = crate::json::from_str("{}").map_err(DefaultFault::EmptyObjectUnread)?;
        if parsed != default {
            return Err(DefaultFault::NotTheEmptyObject);
        }
    }
    let written = crate::json::to_json(&default).map_err(DefaultFault::Unwritten)?;
    let again: T = crate::json::from_str(&written).map_err(DefaultFault::WrittenUnread)?;
    if again != default {
        return Err(DefaultFault::NotTheSame { written });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{self, Schema};
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

    /// A struct whose `Default` the schema derive emits: an optional, a filled and a skipped
    /// field.
    #[derive(Debug, PartialEq, Facet, Schema)]
    struct Derived {
        #[facet(schema::fallback = 1)]
        priority: Option<i32>,
        #[facet(default = 0.5)]
        speed: f32,
        #[facet(skip, rename = "\u{0}seen")]
        seen: bool,
    }

    /// A hand `Default` that sets a skipped field: the writer leaves the field out, so the
    /// written default reads back with the field's own default instead.
    #[derive(Debug, PartialEq, Facet)]
    struct Unkept {
        level: i32,
        #[facet(skip, rename = "\u{0}seen")]
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

    #[test]
    fn a_transparent_default_round_trips_without_the_empty_object() {
        let checked = default_round_trip::<Bare>(false);
        assert!(checked.is_ok(), "{checked:?}");
        let fault = default_round_trip::<Bare>(true);
        assert!(
            matches!(fault, Err(DefaultFault::EmptyObjectUnread(_))),
            "{fault:?}"
        );
    }
}
