//! The link rules of the client visuals.
//!
//! A client entity, an attachable and an atomic client entity share one description
//! ([`user::User`]); the checks of its maps are written once per kind of target
//! (`user_<target>.rs`), and [`run`] calls each over the descriptions of each family that can
//! name the target. The rules that go the other way, from an animation or a render
//! controller to the descriptions that use it, are `animation_entity.rs` and
//! `render_user.rs`.
//!
//! The findings come check by check, in the order of [`run`]: each check over every client
//! and atomic client entity, then each over every attachable; within a check in the order of
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

#[cfg(all(feature = "attachable", feature = "client_entity"))]
mod attachable_client_entity;
#[cfg(all(feature = "attachable", feature = "item"))]
mod attachable_item;

#[cfg(all(
    feature = "animation",
    feature = "animation_controller",
    any(feature = "client_entity", feature = "atomic_client_entity")
))]
mod animation_entity;
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
    #[cfg(any(feature = "client_entity", feature = "atomic_client_entity"))]
    each(cx, out, ENTITY_CHECKS, &entities);
    #[cfg(all(feature = "client_entity", feature = "entity"))]
    entity::run(cx, out);

    // The references of an attachable and its identifier.
    #[cfg(feature = "attachable")]
    each(cx, out, ATTACHABLE_CHECKS, &attachables);
    #[cfg(all(feature = "attachable", feature = "item"))]
    attachable_item::run(cx, out);
    #[cfg(all(feature = "attachable", feature = "client_entity"))]
    attachable_client_entity::run(cx, out);

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
        // A client entity is a user of each render controller it names; an atomic client
        // entity is not: its controllers are judged by what the game says when it draws the
        // entity (`user_render::atomic_controllers`).
        #[cfg(any(feature = "client_entity", feature = "atomic_client_entity"))]
        for user in entities
            .iter()
            .filter(|user| user.family == user::Family::ClientEntity)
        {
            uses.add(user);
        }
        #[cfg(feature = "attachable")]
        for user in &attachables {
            uses.add(user);
        }
        render_user::run(cx, out, &uses);
    }
}

/// A check of one description, as `user_<target>.rs` writes it.
#[cfg(any(
    feature = "client_entity",
    feature = "atomic_client_entity",
    feature = "attachable"
))]
type Check = fn(&super::LinkCx<'_>, &mut super::Findings, &user::User<'_>);

/// The checks of a client entity and of an atomic client entity, in the order they run.
#[cfg(any(feature = "client_entity", feature = "atomic_client_entity"))]
const ENTITY_CHECKS: &[Check] = &[
    #[cfg(all(feature = "render_controller", feature = "material"))]
    user_render::materials,
    #[cfg(all(feature = "render_controller", feature = "material"))]
    user_render::material_shaders,
    #[cfg(all(feature = "render_controller", feature = "material"))]
    user_render::render_controllers,
    #[cfg(all(feature = "render_controller", feature = "material"))]
    user_render::atomic_controllers,
    #[cfg(feature = "particle")]
    user_particle::check,
    #[cfg(feature = "geometry")]
    user_geometry::check,
    #[cfg(all(feature = "animation", feature = "animation_controller"))]
    user_animation::check,
    #[cfg(all(
        feature = "texture_atlas",
        feature = "texture_file",
        feature = "texture_set"
    ))]
    user_texture::check,
    #[cfg(feature = "sound_definitions")]
    user_sound::check,
];

/// The checks of an attachable, in the order they run: those of a client entity but the
/// atomic one and sounds (an attachable's `sound_effects` are not checked).
#[cfg(feature = "attachable")]
const ATTACHABLE_CHECKS: &[Check] = &[
    #[cfg(all(feature = "render_controller", feature = "material"))]
    user_render::materials,
    #[cfg(all(feature = "render_controller", feature = "material"))]
    user_render::material_shaders,
    #[cfg(all(feature = "render_controller", feature = "material"))]
    user_render::render_controllers,
    #[cfg(feature = "particle")]
    user_particle::check,
    #[cfg(feature = "geometry")]
    user_geometry::check,
    #[cfg(all(feature = "animation", feature = "animation_controller"))]
    user_animation::check,
    #[cfg(all(
        feature = "texture_atlas",
        feature = "texture_file",
        feature = "texture_set"
    ))]
    user_texture::check,
];

/// Each of `checks` over every one of `users`, check by check.
#[cfg(any(
    feature = "client_entity",
    feature = "atomic_client_entity",
    feature = "attachable"
))]
fn each(
    cx: &super::LinkCx<'_>,
    out: &mut super::Findings,
    checks: &[Check],
    users: &[user::User<'_>],
) {
    for check in checks {
        for user in users {
            check(cx, out, user);
        }
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
