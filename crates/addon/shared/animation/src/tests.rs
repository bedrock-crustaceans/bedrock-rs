//! Unit tests of the crate-private parts; the fixture rows are in `tests/animation.rs` and
//! `tests/animation_controller.rs`.

use super::walk::{Node, Walk};

#[test]
fn a_root_with_comments_is_read_at_its_value() {
    let root = Node::root("// note\n { \"a\": 1 }").unwrap();
    assert_eq!(root.kind(), super::walk::Kind::Object);
    let walk = Walk::new();
    assert_eq!(walk.members(root).unwrap().len(), 1);
}
