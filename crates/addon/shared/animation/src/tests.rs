//! Unit tests of the crate-private parts; the fixture rows are in `tests/animation.rs` and
//! `tests/animation_controller.rs`.

use super::walk::RULES;
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::node_tree::{Kind, Node, Walk};

#[test]
fn a_root_with_comments_is_read_at_its_value() {
    let root = Node::root("// note\n { \"a\": 1 }").unwrap();
    assert_eq!(root.kind(), Kind::Object);
    let walk = Walk::new(RULES, &JsonPath::new());
    assert_eq!(walk.members(root).unwrap().len(), 1);
}
