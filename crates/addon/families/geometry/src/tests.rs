//! Unit tests of the pieces the integration tests cannot reach: the chain of inherited bones.

use std::sync::Arc;

use super::{Bone, Geometry, Inherited, ItemDisplayTransform, ItemDisplayTransforms};

/// A chain of `depth` ancestors, the nearest one named `p0`.
fn chain(depth: usize) -> Arc<Inherited> {
    let mut parent = None;
    for level in (0..depth).rev() {
        parent = Some(Arc::new(Inherited::new(
            format!("geometry.p{level}"),
            vec![Bone::new(format!("bone{level}"))],
            parent,
        )));
    }
    parent.expect("depth is at least one")
}

#[test]
fn a_bone_is_found_on_the_geometry_before_its_parents_and_nearest_parent_first() {
    let far = Arc::new(Inherited::new(
        "geometry.far",
        vec![Bone::new("head"), Bone::new("tail")],
        None,
    ));
    let mut near_head = Bone::new("Head");
    near_head.inflate = Some(2.0);
    let near = Arc::new(Inherited::new("geometry.near", vec![near_head], Some(far)));
    let mut own = Bone::new("body");
    own.inflate = Some(1.0);
    let geometry = Geometry {
        bones: Some(vec![own]),
        inherited: Some(near),
        ..Geometry::new("geometry.child")
    };
    assert_eq!(geometry.bone("BODY").and_then(|b| b.inflate), Some(1.0));
    assert_eq!(geometry.bone("head").and_then(|b| b.inflate), Some(2.0));
    assert_eq!(geometry.bone("tail").map(|b| b.name.as_str()), Some("tail"));
    assert!(geometry.bone("wing").is_none());
}

#[test]
fn a_chain_of_a_hundred_thousand_parents_is_compared_printed_and_dropped_without_recursion() {
    const DEPTH: usize = 100_000;
    let a = chain(DEPTH);
    let b = chain(DEPTH);
    assert_eq!(a.chain().count(), DEPTH);
    assert_eq!(*a, *b, "two equal chains built apart");
    let shorter = chain(DEPTH - 1);
    assert_ne!(*a, *shorter);
    // A link prints itself, not the links above it.
    let printed = format!("{a:?}");
    assert_eq!(
        printed,
        r#"Inherited { identifier: "geometry.p0", bones: 1, chain: 100000 }"#
    );
    let geometry = Geometry {
        inherited: Some(a.clone()),
        ..Geometry::new("geometry.child")
    };
    assert_eq!(
        geometry.bone("bone99999").map(|b| b.name.as_str()),
        Some("bone99999")
    );
    drop(geometry);
    drop(a);
    drop(b);
    drop(shorter);
}

#[test]
fn a_shared_tail_survives_the_drop_of_one_of_its_heads() {
    let tail = chain(3);
    let left = Arc::new(Inherited::new(
        "geometry.left",
        Vec::new(),
        Some(tail.clone()),
    ));
    let right = Arc::new(Inherited::new(
        "geometry.right",
        Vec::new(),
        Some(tail.clone()),
    ));
    drop(left);
    assert_eq!(right.chain().count(), 4);
    assert_eq!(tail.chain().count(), 3);
    assert_eq!(tail.identifier(), "geometry.p0");
    assert_eq!(tail.bones().len(), 1);
}

/// Two chains are equal when they name the same geometries in the same order: the identifiers
/// along the chain, not the bones (which the geometries themselves compare).
#[test]
fn two_chains_are_equal_when_they_name_the_same_geometries() {
    let link = |identifier: &str, bone: &str, parent: Option<Arc<Inherited>>| {
        Arc::new(Inherited::new(identifier, vec![Bone::new(bone)], parent))
    };
    let base = link("geometry.a", "head", None);
    assert_eq!(*base, *base.clone(), "one link is itself");
    assert_eq!(*base, *link("geometry.a", "head", None));
    assert_eq!(
        *base,
        *link("geometry.a", "tail", None),
        "bones are not compared"
    );
    assert_ne!(
        *base,
        *link("geometry.b", "head", None),
        "another identifier"
    );
    let longer = link("geometry.child", "hat", Some(base.clone()));
    assert_ne!(
        *longer,
        *link("geometry.child", "hat", None),
        "one ends first"
    );
    assert_ne!(*link("geometry.child", "hat", None), *longer);
    assert_ne!(
        *longer,
        *link(
            "geometry.child",
            "hat",
            Some(link("geometry.b", "head", None))
        ),
        "another identifier further up"
    );
    assert_eq!(
        *longer,
        *link(
            "geometry.child",
            "hat",
            Some(link("geometry.a", "tail", None))
        )
    );
    assert_eq!(longer.chain().count(), 2);
    assert_eq!(longer.depth(), 2);
    assert_eq!(base.depth(), 1);
}

#[test]
fn bone_new_takes_an_owned_string_and_lowercases_it() {
    assert_eq!(Bone::new(String::from("Head")).name, "head");
    assert_eq!(Bone::new("TAIL").name, "tail");
}

#[test]
fn item_display_transforms_iterate_by_reference_in_vanilla_order() {
    let transforms = ItemDisplayTransforms {
        head: Some(ItemDisplayTransform::default()),
        gui: Some(ItemDisplayTransform::default()),
        ..ItemDisplayTransforms::default()
    };
    let mut names = Vec::new();
    for (name, _) in &transforms {
        names.push(name);
    }
    assert_eq!(names, ["gui", "head"]);
    assert_eq!(names, transforms.iter().map(|(n, _)| n).collect::<Vec<_>>());
}
