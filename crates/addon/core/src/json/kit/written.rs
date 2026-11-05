//! [`Written`]: the root a value keeps as its author wrote it, beside the fields.

use super::Value;

/// The JSON a value keeps as its author wrote it, beside the typed fields that are the game's
/// reading of it, and left out of equality: a holder that derives `PartialEq` compares its
/// fields alone, so a loaded value equals a hand-built one with the same fields.
///
/// It is the slot of the `written: Option<Value>` families (`loading_messages`, font's two
/// types, splashes, credits; skin's is its own): those that keep the root as written where
/// their fields cannot say it (an element the game skips, a value of a type it ignores) and
/// write it back while the fields still read the same. The protocol that fills and writes the
/// slot is `bedrock_addon_support::fixed_path::Authored` (`from_value` keeps the root,
/// `to_value` writes it while a re-read of it equals the value); this is the type of the slot
/// it fills. It dereferences to the `Option`, so `Authored::written_mut` keeps its signature:
/// `&mut self.written` is the `&mut Option<Value>` it returns.
///
/// Replacing a family's public `written: Option<Value>` field with it is behaviour-bearing,
/// and lands per family: the field's type is public API, and equality changes, both the
/// holder's `PartialEq` (today a loaded value that kept its root differs from a hand-built one)
/// and `Authored::to_value`'s re-read comparison, which then compares the fields alone.
///
/// ```
/// use bedrock_addon_core::json::kit::Written;
/// use bedrock_addon_core::json::Value;
///
/// #[derive(Clone, Debug, Default, PartialEq)]
/// struct Splashes {
///     splashes: Vec<String>,
///     written: Written,
/// }
///
/// let built = Splashes { splashes: vec!["Hi!".to_owned()], ..Splashes::default() };
/// let mut loaded = built.clone();
/// *loaded.written = Some(Value::from("as the author wrote it"));
/// assert_eq!(loaded, built);
/// assert!(loaded.written.is_some());
/// ```
#[derive(Clone, Debug)]
pub struct Written<T = Value>(pub Option<T>);

/// Every authored text is equal to every other: the holder's fields decide.
impl<T> PartialEq for Written<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T> Eq for Written<T> {}

impl<T> Default for Written<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> From<Option<T>> for Written<T> {
    fn from(written: Option<T>) -> Self {
        Self(written)
    }
}

impl<T> std::ops::Deref for Written<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Option<T> {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Written<T> {
    fn deref_mut(&mut self) -> &mut Option<T> {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A value of a `written` family: the fields, and the root they were read from.
    #[derive(Clone, Debug, Default, PartialEq)]
    struct Credits {
        sections: Vec<String>,
        written: Written,
    }

    fn root(text: &str) -> Value {
        crate::json::from_str_jsonc(text).expect("JSON")
    }

    #[test]
    fn the_authored_text_is_left_out_of_equality() {
        let built = Credits {
            sections: vec!["a".to_owned()],
            written: Written::default(),
        };
        let loaded = Credits {
            sections: vec!["a".to_owned()],
            written: Written(Some(root(r#"[{"section": "a"}, 5]"#))),
        };
        assert_eq!(loaded, built);
        assert_eq!(Written(Some(root("1"))), Written::<Value>(Some(root("2"))));
        // The fields still decide.
        let edited = Credits {
            sections: vec!["b".to_owned()],
            ..loaded.clone()
        };
        assert_ne!(edited, loaded);
    }

    #[test]
    fn the_authored_text_is_an_option_to_read_set_and_take() {
        let mut credits = Credits::default();
        assert!(credits.written.is_none());
        // What `fixed_path::Authored::written_mut` hands out.
        let slot: &mut Option<Value> = &mut credits.written;
        *slot = Some(root("[]"));
        assert_eq!(credits.written.as_ref(), Some(&root("[]")));
        assert_eq!(credits.written.take(), Some(root("[]")));
        assert_eq!(credits.written.0, None);
        assert_eq!(Written::from(Some(3)).0, Some(3));
    }

    #[test]
    fn debug_shows_the_authored_text() {
        let written = Written(Some(root("[1]")));
        assert!(
            format!("{written:?}").starts_with("Written(Some("),
            "{written:?}"
        );
    }
}
