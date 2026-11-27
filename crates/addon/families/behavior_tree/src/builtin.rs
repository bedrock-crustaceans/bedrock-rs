//! The nine trees built into the game: JSON texts the game ships, loaded after the
//! pack trees through the same reader, so each replaces a pack tree of its name.

// `AutomationTrees` and `getAutomationBehaviorJSONMap`. The
// texts are the game's data strings, byte for byte: tab-indented, a leading line feed, no
// final one, and twelve spaces before the brace that closes a `minecraft:repeat_until_success` body. They
// are string literals with every line end and tab written out, so no checkout can change a
// byte of them.

/// `(minecraft:name, the text)` of every built-in tree, by name.
pub(super) const TREES: [(&str, &str); 9] = [
    (
        "activate_tool",
        "\n{\n\t\t\"minecraft:behavior_tree\" : \n\t{\n\t\t\"minecraft:name\": \"activate_tool\",\n\t\t\"minecraft:root\" : \n\t\t{\n\t\t\t\"minecraft:repeat_until_success\": {\n\t\t\t\t\"child\": {\n\t\t\t\t\t\"minecraft:activate_tool\": {}\n\t\t\t\t}\n            }\n\t\t}\n\t}\n}",
    ),
    (
        "break_block",
        "\n{\n\t\t\"minecraft:behavior_tree\" : \n\t{\n\t\t\"minecraft:name\": \"break_block\",\n\t\t\"minecraft:root\" : \n\t\t{\n\t\t\t\"minecraft:repeat_until_success\": {\n\t\t\t\t\"child\": {\n\t\t\t\t\t\"minecraft:break_block\": {}\n\t\t\t\t}\n            }\n\t\t}\n\t}\n}",
    ),
    (
        "find_block",
        "\n{\n\t\t\"minecraft:behavior_tree\" : \n\t{\n\t\t\"minecraft:name\": \"find_block\",\n\t\t\"minecraft:root\" : \n\t\t{\n\t\t\t\"minecraft:repeat_until_success\": {\n\t\t\t\t\"child\": {\n\t\t\t\t\t\"minecraft:find_block\": {}\n\t\t\t\t}\n            }\n\t\t}\n\t}\n}",
    ),
    (
        "fly",
        "\n{\n\t\t\"minecraft:behavior_tree\" : \n\t{\n\t\t\"minecraft:name\": \"fly\",\n\t\t\"minecraft:root\" : \n\t\t{\n\t\t\t\"minecraft:fly\": {}\n\t\t}\n\t}\n}",
    ),
    (
        "get_interaction_position_for_block",
        "\n{\n\t\t\"minecraft:behavior_tree\" : \n\t{\n\t\t\"minecraft:name\": \"get_interaction_position_for_block\",\n\t\t\"minecraft:root\" : \n\t\t{\n\t\t\t\"minecraft:get_interaction_position_for_block\": {}\n\t\t}\n\t}\n}",
    ),
    (
        "look_at",
        "\n{\n\t\t\"minecraft:behavior_tree\" : \n\t{\n\t\t\"minecraft:name\": \"look_at\",\n\t\t\"minecraft:root\" : \n\t\t{\n\t\t\t\"minecraft:look_at\": {}\n\t\t}\n\t}\n}",
    ),
    (
        "move",
        "\n{\n\t\t\"minecraft:behavior_tree\" : \n\t{\n\t\t\"minecraft:name\": \"move\",\n\t\t\"minecraft:root\" : \n\t\t{\n\t\t\t\"minecraft:move\": {}\n\t\t}\n\t}\n}",
    ),
    (
        "move_to",
        "\n{\n\t\t\"minecraft:behavior_tree\" : \n\t{\n\t\t\"minecraft:name\": \"move_to\",\n\t\t\"minecraft:root\" : \n\t\t{\n\t\t\t\"minecraft:move_to\": {}\n\t\t}\n\t}\n}",
    ),
    (
        "place_block",
        "\n{\n\t\t\"minecraft:behavior_tree\" : \n\t{\n\t\t\"minecraft:name\": \"place_block\",\n\t\t\"minecraft:root\" : \n\t\t{\n\t\t\t\"minecraft:repeat_until_success\": {\n\t\t\t\t\"child\": {\n\t\t\t\t\t\"minecraft:place_block\": {}\n\t\t\t\t}\n            }\n\t\t}\n\t}\n}",
    ),
];
