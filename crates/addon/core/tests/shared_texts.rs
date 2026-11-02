#![allow(clippy::pedantic, missing_docs)]
//! The message rows the shared types add to `VanillaText`, rendered byte
//! for byte as vanilla's content log prints them, and the codes they ride on.

use bedrock_addon_core::diagnostic::{DiagnosticCode, VanillaText};

#[test]
fn shared_type_texts_render_as_vanilla_prints_them() {
    let rows = [
        (
            VanillaText::FilterTestNotFound {
                name: "is_purple".into(),
            },
            "Filter test not found -> is_purple",
        ),
        (
            VanillaText::ItemDescriptorShape { value: "5".into() },
            "ItemDescriptor requires a string, {\"item\": string}, or {\"tags\": string} or {\"item_tag\": string}. ItemDescriptor cannot parse 5\n",
        ),
        (
            VanillaText::BlockDescriptorShape { value: "5".into() },
            "BlockDescriptor requires a string, block object, or any_tag object. Cannot parse 5",
        ),
        (
            VanillaText::BlockDescriptorName { value: "7".into() },
            "Only strings are valid for name. The invalid name value: 7\n",
        ),
        (
            VanillaText::BlockDescriptorStateType {
                block: "minecraft:wool".into(),
                state: "color".into(),
                state_type: 3,
            },
            "Invalid state type during block parsing: block name minecraft:wool, state name color, state type 3. State type can only be bool, int, or string.",
        ),
        (
            VanillaText::BlockDescriptorTagsAndStates {
                block: "a:b".into(),
            },
            "Cannot have both a tag expression and block states in a block descriptor for block: a:b",
        ),
        (
            VanillaText::BlockDescriptorNameAndTags {
                block: "a:b".into(),
            },
            "Cannot have both a block name and tag expression in a block descriptor for block: a:b",
        ),
    ];
    for (text, expected) in rows {
        assert_eq!(text.to_string(), expected);
        let code = DiagnosticCode::InvalidValue { text: Some(text) };
        assert_eq!(code.to_string(), expected);
    }
}
