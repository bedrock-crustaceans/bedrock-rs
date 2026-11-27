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

rules! {
    mod user needs any["client_entity", "atomic_client_entity", "attachable"];

    mod user_animation needs [
        "animation",
        "animation_controller",
        any["client_entity", "atomic_client_entity", "attachable"],
    ];
    mod user_geometry needs [
        "geometry",
        any["client_entity", "atomic_client_entity", "attachable"],
    ];
    mod user_particle needs [
        "particle",
        any["client_entity", "atomic_client_entity", "attachable"],
    ];
    mod user_render needs [
        "render_controller",
        "material",
        any["client_entity", "atomic_client_entity", "attachable"],
    ];
    mod user_sound needs ["sound_definitions", any["client_entity", "atomic_client_entity"]];
    mod user_texture needs [
        "texture_atlas",
        "texture_file",
        "texture_set",
        any["client_entity", "atomic_client_entity", "attachable"],
    ];

    mod entity needs ["client_entity", "entity"];

    mod attachable_client_entity needs ["attachable", "client_entity"];
    mod attachable_item needs ["attachable", "item"];

    mod animation_entity needs [
        "animation",
        "animation_controller",
        any["client_entity", "atomic_client_entity"],
    ];
    mod render_user needs [
        "render_controller",
        "material",
        any["client_entity", "atomic_client_entity", "attachable"],
    ];

    /// The checks of a client entity and of an atomic client entity, check by check over
    /// every one of `users`.
    #[cfg(any(feature = "client_entity", feature = "atomic_client_entity"))]
    run entity_checks(users: &[user::User<'_>]) for user in users =
        user_render::materials,
        user_render::material_shaders,
        user_render::render_controllers,
        user_render::atomic_controllers,
        user_particle::check,
        user_geometry::check,
        user_animation::check,
        user_texture::check,
        user_sound::check;

    /// The checks of an attachable, check by check over every one of `users`: those of a
    /// client entity but the atomic one and sounds (an attachable's `sound_effects` are not
    /// checked).
    #[cfg(feature = "attachable")]
    run attachable_checks(users: &[user::User<'_>]) for user in users =
        user_render::materials,
        user_render::material_shaders,
        user_render::render_controllers,
        user_particle::check,
        user_geometry::check,
        user_animation::check,
        user_texture::check;
}

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
    entity_checks(cx, out, &entities);
    entity! { entity::run(cx, out); }

    // The references of an attachable and its identifier.
    #[cfg(feature = "attachable")]
    attachable_checks(cx, out, &attachables);
    attachable_item! { attachable_item::run(cx, out); }
    attachable_client_entity! { attachable_client_entity::run(cx, out); }

    // From an animation and from a render controller to the descriptions that use it.
    animation_entity! { animation_entity::run(cx, out, &entities); }
    render_user! {
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
