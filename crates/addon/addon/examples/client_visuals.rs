#![allow(clippy::pedantic, missing_docs)]
//! Walk the client entities of the resource stack and follow their geometry and texture
//! references through `HasVisuals`, the view client entities, attachables and atomic client
//! entities share.
//!
//! The stacks are built from bytes in memory, as in `in_memory_pack`.
//!
//! Run with `cargo run -p bedrock_addon --example client_visuals --no-default-features --features entity,geometry,texture_file,texture_set`.

mod common;

use bedrock_addon::pack::StackSet;
use bedrock_addon::visual::HasVisuals;

/// One line per thing found; the lines that start with `!` name something missing.
fn check_visuals(stacks: &StackSet) -> Vec<String> {
    let mut lines = Vec::new();
    let rp = &stacks.resource;
    for (id, client_entity) in rp.client_entities() {
        if !stacks.behavior.entities().contains(id) && !id.is_vanilla() {
            lines.push(format!(
                "! {id}: client entity without a behaviour definition"
            ));
        }
        let visuals = client_entity.visuals();
        // A slot map is `None` when the document has no such member.
        for (slot, geometry) in visuals.geometry.into_iter().flatten() {
            if rp.geometries().get(geometry).is_none() {
                lines.push(format!(
                    "! {id}: geometry slot {slot} names missing {geometry}"
                ));
            }
        }
        for (slot, texture) in visuals.textures.into_iter().flatten() {
            match rp
                .resolve_texture(texture)
                .and_then(|file| Some((file.pack, file.image?)))
            {
                Some((pack, image)) => lines.push(format!(
                    "{id}: {slot} → {}/{} ({:?})",
                    pack.name(),
                    image.path,
                    image.format
                )),
                None => lines.push(format!(
                    "! {id}: texture slot {slot} names missing {texture}"
                )),
            }
        }
    }
    lines
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stacks = common::stack_set()?;
    let lines = check_visuals(&stacks);
    for line in &lines {
        println!("{line}");
    }
    // The registry lists the client entities in the order their files were walked.
    assert_eq!(
        lines,
        [
            "! demo:ghost: client entity without a behaviour definition",
            "! demo:ghost: geometry slot default names missing geometry.ghost",
            "! demo:ghost: texture slot default names missing textures/entity/ghost",
            "demo:walker: default → Ruby resources/textures/entity/walker.png (Some(Png))",
        ]
    );
    assert!(
        !stacks.resource.report().has_errors(),
        "{}",
        stacks.resource.report()
    );
    Ok(())
}
