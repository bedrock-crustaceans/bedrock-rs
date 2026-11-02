//! The deep merge vanilla runs over every pack's copy of an atlas file before it reads the
//! result.

use bedrock_addon_core::json::Value;

/// `patch` laid over `base`: two objects merge member by member, two arrays keep the patch's
/// elements and the base's tail past them, a `null` patch leaves the base, and anything else
/// is the patch.
// The same table for both atlases: not RFC 7386, since `null` deletes nothing and array elements are never merged.
pub(super) fn patch_json(base: &Value, patch: Value) -> Value {
    if patch.is_null() {
        return base.clone();
    }
    if let (Some(lower), Some(upper)) = (base.as_object(), patch.as_object()) {
        let mut merged = upper.clone();
        for (key, value) in lower {
            let combined = match upper.get(key.as_str()) {
                Some(over) => patch_json(value, over.clone()),
                None => value.clone(),
            };
            merged.insert(key.as_str(), combined);
        }
        return merged.into_value();
    }
    if let (Some(lower), Some(upper)) = (base.as_array(), patch.as_array()) {
        let mut merged = upper.clone();
        for value in lower.as_slice().iter().skip(upper.len()) {
            merged.push(value.clone());
        }
        return merged.into_value();
    }
    patch
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(text: &str) -> Value {
        bedrock_addon_core::json::from_str::<Value>(text).unwrap()
    }

    fn patched(base: &str, patch: &str) -> Value {
        patch_json(&value(base), value(patch))
    }

    #[test]
    fn objects_merge_member_by_member() {
        assert_eq!(
            patched(
                r#"{"a": 1, "b": {"x": 1, "y": 2}}"#,
                r#"{"b": {"x": 9}, "c": 3}"#
            ),
            value(r#"{"b": {"x": 9, "y": 2}, "c": 3, "a": 1}"#)
        );
    }

    #[test]
    fn arrays_keep_the_patch_and_the_base_tail() {
        assert_eq!(
            patched(r#"["a","b","c"]"#, r#"["m"]"#),
            value(r#"["m","b","c"]"#)
        );
        assert_eq!(patched(r#"["a"]"#, r#"["m","n"]"#), value(r#"["m","n"]"#));
        // Elements are not merged with each other.
        assert_eq!(
            patched(r#"[{"p": 1, "q": 2}]"#, r#"[{"p": 3}]"#),
            value(r#"[{"p": 3}]"#)
        );
    }

    #[test]
    fn a_null_patch_keeps_the_base_and_cannot_delete() {
        assert_eq!(patched(r#"{"a": 1}"#, "null"), value(r#"{"a": 1}"#));
        assert_eq!(
            patched(r#"{"a": 1}"#, r#"{"a": null}"#),
            value(r#"{"a": 1}"#)
        );
    }

    #[test]
    fn a_change_of_shape_takes_the_patch() {
        assert_eq!(patched(r#"["a","b"]"#, r#""m""#), value(r#""m""#));
        assert_eq!(patched(r#"{"a": 1}"#, "[]"), value("[]"));
        assert_eq!(patched("[]", r#"{"a": 1}"#), value(r#"{"a": 1}"#));
        assert_eq!(patched("null", r#"{"a": 1}"#), value(r#"{"a": 1}"#));
    }
}
