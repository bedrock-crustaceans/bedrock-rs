//! The `minecraft` namespace is the base game's. Rows vs05, vs06 (vs01
//! and vs07 are the accepted sides).

use super::common::*;

const IDENTIFIER: &str = "minecraft:voxel_shape/description/identifier";

/// The lower half of the block, named `identifier`.
fn named(identifier: &str) -> String {
    shape(identifier, &[("[0, 0, 0]", "[16, 8, 16]")])
}

/// V2's one finding for `identifier`.
fn not_valid_for_custom(identifier: &str) -> [(&'static str, String, String); 1] {
    [error(
        "voxel_shape/namespace_not_valid",
        &format!("The `minecraft` namespace is not valid for custom voxel shape: `{identifier}`"),
        IDENTIFIER,
    )]
}

/// V1's one finding for `identifier`.
fn required_for_vanilla(identifier: &str) -> [(&'static str, String, String); 1] {
    [error(
        "voxel_shape/namespace_required",
        &format!("The `minecraft` namespace is required for Vanilla shape: `{identifier}`"),
        IDENTIFIER,
    )]
}

#[test]
fn vs05_a_user_pack_may_not_use_the_minecraft_namespace() {
    let d = rejects(&named("minecraft:invalid_namespace"));
    assert_eq!(
        errors(&d),
        not_valid_for_custom("minecraft:invalid_namespace")
    );
    // The vanilla anvil of vs01, from a pack that is not the base game's.
    let anvil = fixture("vs01_anvil_base_game/shapes/anvil.json");
    assert_eq!(
        errors(&rejects(&anvil)),
        not_valid_for_custom("minecraft:anvil")
    );
}

#[test]
fn v2_only_a_namespace_that_is_exactly_minecraft_is_vanilla_s() {
    // The text before the first colon, compared as written.
    for identifier in [
        "minecraft",
        "Minecraft:shape",
        "MINECRAFT:shape",
        "minecraftx:shape",
        "xminecraft:shape",
        "my:minecraft:shape",
        "minecraft_shape",
        ":minecraft",
    ] {
        let shape = loads(&named(identifier));
        assert_eq!(shape.description.identifier.as_str(), identifier);
    }
    for identifier in ["minecraft:", "minecraft:a:b", "minecraft:Shape With Spaces"] {
        assert_eq!(
            errors(&rejects(&named(identifier))),
            not_valid_for_custom(identifier),
            "{identifier}"
        );
    }
}

#[test]
fn vs06_a_base_game_pack_must_use_the_minecraft_namespace_deviation() {
    // Vanilla: a debug assert, then the same skip (`vanilla = "handleAssert, then skipped"`).
    let d = rejects_in(&named("custom:shape"), &base_game());
    assert_eq!(errors(&d), required_for_vanilla("custom:shape"));
}

#[test]
fn v1_a_bare_name_is_not_in_the_minecraft_namespace() {
    for identifier in [
        "anvil",
        "minecraft",
        "Minecraft:anvil",
        "minecraftx:anvil",
        "",
    ] {
        assert_eq!(
            errors(&rejects_in(&named(identifier), &base_game())),
            required_for_vanilla(identifier),
            "{identifier}"
        );
    }
    for identifier in ["minecraft:anvil", "minecraft:", "minecraft:a:b"] {
        let shape = loads_in(&named(identifier), &base_game());
        assert_eq!(shape.description.identifier.as_str(), identifier);
    }
}
