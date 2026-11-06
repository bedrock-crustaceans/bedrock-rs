//! The link rules of the client visuals.
//!
//! A client entity, an attachable and an atomic client entity share one description
//! ([`user::User`]); the checks of its maps are written once per kind of target
//! (`user_<target>.rs`) and called by the file of each pair of modules (`entity_<target>.rs`,
//! `item_<target>.rs`). The rules that go the other way, from an animation or a render
//! controller to the descriptions that use it, are `animation_entity.rs` and
//! `render_user.rs` (fed by `render_entity.rs` and `render_item.rs`).
//!
//! The findings come rule by rule, in the order of [`run`], and within a rule in the order of
//! the documents.

#[cfg(any(
    feature = "client_entity",
    feature = "atomic_client_entity",
    feature = "attachable"
))]
mod user;

#[cfg(all(
    feature = "animation",
    feature = "animation_controller",
    any(
        feature = "client_entity",
        feature = "atomic_client_entity",
        feature = "attachable"
    )
))]
mod user_animation;
#[cfg(all(
    feature = "geometry",
    any(
        feature = "client_entity",
        feature = "atomic_client_entity",
        feature = "attachable"
    )
))]
mod user_geometry;
#[cfg(all(
    feature = "particle",
    any(
        feature = "client_entity",
        feature = "atomic_client_entity",
        feature = "attachable"
    )
))]
mod user_particle;
#[cfg(all(
    feature = "render_controller",
    feature = "material",
    any(
        feature = "client_entity",
        feature = "atomic_client_entity",
        feature = "attachable"
    )
))]
mod user_render;
#[cfg(all(
    feature = "sound_definitions",
    any(feature = "client_entity", feature = "atomic_client_entity")
))]
mod user_sound;
#[cfg(all(
    feature = "texture_atlas",
    feature = "texture_file",
    feature = "texture_set",
    any(
        feature = "client_entity",
        feature = "atomic_client_entity",
        feature = "attachable"
    )
))]
mod user_texture;

#[cfg(all(feature = "client_entity", feature = "entity"))]
mod entity;
#[cfg(all(
    any(feature = "client_entity", feature = "atomic_client_entity"),
    feature = "animation",
    feature = "animation_controller"
))]
mod entity_animation;
#[cfg(all(
    any(feature = "client_entity", feature = "atomic_client_entity"),
    feature = "geometry"
))]
mod entity_geometry;
#[cfg(all(
    any(feature = "client_entity", feature = "atomic_client_entity"),
    feature = "particle"
))]
mod entity_particle;
#[cfg(all(
    any(feature = "client_entity", feature = "atomic_client_entity"),
    feature = "render_controller",
    feature = "material"
))]
mod entity_render;
#[cfg(all(
    any(feature = "client_entity", feature = "atomic_client_entity"),
    feature = "sound_definitions"
))]
mod entity_sound;
#[cfg(all(
    any(feature = "client_entity", feature = "atomic_client_entity"),
    feature = "texture_atlas",
    feature = "texture_file",
    feature = "texture_set"
))]
mod entity_texture;

#[cfg(all(feature = "attachable", feature = "item"))]
mod item;
#[cfg(all(
    feature = "attachable",
    feature = "animation",
    feature = "animation_controller"
))]
mod item_animation;
#[cfg(all(feature = "attachable", feature = "client_entity"))]
mod item_entity;
#[cfg(all(feature = "attachable", feature = "geometry"))]
mod item_geometry;
#[cfg(all(feature = "attachable", feature = "particle"))]
mod item_particle;
#[cfg(all(
    feature = "attachable",
    feature = "render_controller",
    feature = "material"
))]
mod item_render;
#[cfg(all(
    feature = "attachable",
    feature = "texture_atlas",
    feature = "texture_file",
    feature = "texture_set"
))]
mod item_texture;

#[cfg(all(
    feature = "animation",
    feature = "animation_controller",
    any(feature = "client_entity", feature = "atomic_client_entity")
))]
mod animation_entity;
#[cfg(all(
    feature = "render_controller",
    feature = "material",
    any(feature = "client_entity", feature = "atomic_client_entity")
))]
mod render_entity;
#[cfg(all(
    feature = "render_controller",
    feature = "material",
    feature = "attachable"
))]
mod render_item;
#[cfg(all(
    feature = "render_controller",
    feature = "material",
    any(
        feature = "client_entity",
        feature = "atomic_client_entity",
        feature = "attachable"
    )
))]
mod render_user;

#[cfg(test)]
mod tests;

#[allow(
    unused_variables,
    reason = "without a client entity family and the `attachable` feature no rule of the group is compiled"
)]
pub(super) fn run(cx: &super::LinkCx<'_>, out: &mut super::Findings) {
    #[cfg(any(feature = "client_entity", feature = "atomic_client_entity"))]
    let entities = entities(cx);
    #[cfg(feature = "attachable")]
    let attachables = attachables(cx);

    // The references of a client entity and of an atomic client entity.
    #[cfg(all(
        any(feature = "client_entity", feature = "atomic_client_entity"),
        feature = "render_controller",
        feature = "material"
    ))]
    entity_render::run(cx, out, &entities);
    #[cfg(all(
        any(feature = "client_entity", feature = "atomic_client_entity"),
        feature = "particle"
    ))]
    entity_particle::run(cx, out, &entities);
    #[cfg(all(
        any(feature = "client_entity", feature = "atomic_client_entity"),
        feature = "geometry"
    ))]
    entity_geometry::run(cx, out, &entities);
    #[cfg(all(
        any(feature = "client_entity", feature = "atomic_client_entity"),
        feature = "animation",
        feature = "animation_controller"
    ))]
    entity_animation::run(cx, out, &entities);
    #[cfg(all(
        any(feature = "client_entity", feature = "atomic_client_entity"),
        feature = "texture_atlas",
        feature = "texture_file",
        feature = "texture_set"
    ))]
    entity_texture::run(cx, out, &entities);
    #[cfg(all(
        any(feature = "client_entity", feature = "atomic_client_entity"),
        feature = "sound_definitions"
    ))]
    entity_sound::run(cx, out, &entities);
    #[cfg(all(feature = "client_entity", feature = "entity"))]
    entity::run(cx, out);

    // The references of an attachable and its identifier.
    #[cfg(all(
        feature = "attachable",
        feature = "render_controller",
        feature = "material"
    ))]
    item_render::run(cx, out, &attachables);
    #[cfg(all(feature = "attachable", feature = "particle"))]
    item_particle::run(cx, out, &attachables);
    #[cfg(all(feature = "attachable", feature = "geometry"))]
    item_geometry::run(cx, out, &attachables);
    #[cfg(all(
        feature = "attachable",
        feature = "animation",
        feature = "animation_controller"
    ))]
    item_animation::run(cx, out, &attachables);
    #[cfg(all(
        feature = "attachable",
        feature = "texture_atlas",
        feature = "texture_file",
        feature = "texture_set"
    ))]
    item_texture::run(cx, out, &attachables);
    #[cfg(all(feature = "attachable", feature = "item"))]
    item::run(cx, out);
    #[cfg(all(feature = "attachable", feature = "client_entity"))]
    item_entity::run(cx, out);

    // From an animation and from a render controller to the descriptions that use it.
    #[cfg(all(
        feature = "animation",
        feature = "animation_controller",
        any(feature = "client_entity", feature = "atomic_client_entity")
    ))]
    animation_entity::run(cx, out, &entities);
    #[cfg(all(
        feature = "render_controller",
        feature = "material",
        any(
            feature = "client_entity",
            feature = "atomic_client_entity",
            feature = "attachable"
        )
    ))]
    {
        let mut uses = render_user::Uses::default();
        #[cfg(any(feature = "client_entity", feature = "atomic_client_entity"))]
        render_entity::uses(&entities, &mut uses);
        #[cfg(feature = "attachable")]
        render_item::uses(&attachables, &mut uses);
        render_user::run(cx, out, &uses);
    }
}

/// The descriptions of the stack's client entities, then of its atomic client entities.
#[cfg(any(feature = "client_entity", feature = "atomic_client_entity"))]
fn entities<'a>(cx: &super::LinkCx<'a>) -> Vec<user::User<'a>> {
    #[cfg_attr(
        not(all(feature = "client_entity", feature = "atomic_client_entity")),
        allow(
            unused_imports,
            unused_mut,
            reason = "only one of the two documents is compiled"
        )
    )]
    use user::{Family, User};
    #[allow(unused_mut, reason = "only one of the two documents is compiled")]
    let mut users = Vec::new();
    #[cfg(feature = "client_entity")]
    for (_, document) in cx.resource().client_entities() {
        let description = &document.value().description.slots;
        // The `animation_controllers` list of a document older than 1.10.0 became
        // `scripts.animate`; a merged definition has other documents' entries in it.
        let before_1_10 = document.version().parsed_as
            < bedrock_addon_core::version::FormatVersion::new(1, 10, 0)
            && document.origin().contributors.is_empty();
        let made_controllers = description
            .scripts
            .as_ref()
            .filter(|_| before_1_10)
            .and_then(|scripts| scripts.animate.as_deref());
        users.push(User {
            origin: document.origin(),
            family: Family::ClientEntity,
            materials: Some(&description.materials),
            textures: Some(&description.textures),
            geometry: Some(&description.geometry),
            animations: description.animations.as_ref(),
            particle_effects: description.particle_effects.as_ref(),
            sound_effects: description.sound_effects.as_ref(),
            render_controllers: description
                .render_controllers
                .as_deref()
                .unwrap_or_default(),
            made_controllers,
        });
    }
    #[cfg(feature = "atomic_client_entity")]
    for (_, document) in cx.resource().atomic_client_entities() {
        let description = &document.value().description;
        users.push(User {
            origin: document.origin(),
            family: Family::Atomic,
            materials: description.materials.as_ref(),
            textures: description.textures.as_ref(),
            geometry: description.geometry.as_ref(),
            animations: description.animations.as_ref(),
            particle_effects: description.particle_effects.as_ref(),
            sound_effects: description.sound_effects.as_ref(),
            render_controllers: description
                .render_controllers
                .as_deref()
                .unwrap_or_default(),
            made_controllers: None,
        });
    }
    users
}

/// The descriptions of the stack's attachables.
#[cfg(feature = "attachable")]
fn attachables<'a>(cx: &super::LinkCx<'a>) -> Vec<user::User<'a>> {
    use user::{Family, User};
    let mut users = Vec::new();
    for (_, document) in cx.resource().attachables() {
        let description = &document.value().description.slots;
        let before_1_10 = document.version().parsed_as
            < bedrock_addon_core::version::FormatVersion::new(1, 10, 0)
            && document.origin().contributors.is_empty();
        let made_controllers = description
            .scripts
            .as_ref()
            .filter(|_| before_1_10)
            .and_then(|scripts| scripts.animate.as_deref());
        users.push(User {
            origin: document.origin(),
            family: Family::Attachable,
            materials: Some(&description.materials),
            textures: Some(&description.textures),
            geometry: Some(&description.geometry),
            animations: description.animations.as_ref(),
            particle_effects: description.particle_effects.as_ref(),
            sound_effects: description.sound_effects.as_ref(),
            render_controllers: description
                .render_controllers
                .as_deref()
                .unwrap_or_default(),
            made_controllers,
        });
    }
    users
}

/// The slot a render-controller expression names when it is `<prefix><name>` and nothing
/// else (`geometry.default`), in lower case; `None` for any longer expression. The prefix is
/// compared without case: the game folds a controller file, and Molang names have no case.
#[cfg(all(
    feature = "render_controller",
    feature = "material",
    any(
        feature = "client_entity",
        feature = "atomic_client_entity",
        feature = "attachable"
    )
))]
fn plain_reference(text: &str, prefix: &str) -> Option<String> {
    let text = text.trim();
    let head = text.get(..prefix.len())?;
    let name = text.get(prefix.len()..)?;
    let name_byte = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'.';
    (head.eq_ignore_ascii_case(prefix) && !name.is_empty() && name.bytes().all(name_byte))
        .then(|| name.to_ascii_lowercase())
}
