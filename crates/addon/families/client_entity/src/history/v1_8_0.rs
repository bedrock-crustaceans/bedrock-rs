//! Format versions 1.8.0 up to 1.10.0, and the step to 1.10.0 (the shared one: a client entity's
//! `animation_controllers` become animations and `scripts.animate` entries).

use bedrock_addon_core::id::EntityId;
use bedrock_addon_shared_visual::history::v1_8_0::Described;

use super::ClientEntityFamily;

/// A client entity document of format version 1.8.0 up to 1.10.0.
pub type ClientEntity = Described<ClientEntityFamily>;

/// `description` at format versions 1.8.0 up to 1.10.0: animation controllers are a list of
/// their own, render controllers are names only, and `queryable_geometry` and
/// `held_item_scale` do not exist.
pub type Description = bedrock_addon_shared_visual::history::v1_8_0::Description<EntityId>;

#[cfg(test)]
mod tests {
    use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::vanilla::ActorType;
    use bedrock_addon_core::version::FormatVersion;
    use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

    use super::*;

    /// `minecraft:client_entity/description`.
    fn description_at() -> JsonPath {
        JsonPath::of_keys(["minecraft:client_entity", "description"])
    }

    /// The findings of the 1.8 → 1.10 step on a description with `members`, with the moves
    /// the step recorded.
    fn stepped(members: &str) -> Diagnostics {
        let text = format!(r#"{{"identifier": "demo:ghost", {members}}}"#);
        let json = bedrock_addon_core::json::from_str_jsonc(&text).expect("JSON");
        let description =
            bedrock_addon_core::schema::read_value::<Description>(&json).expect("a description");
        let entity = ClientEntity {
            format_version: FormatVersion::new(1, 8, 0),
            description,
        };
        let load = LoadContext::detached();
        let mut diagnostics = Diagnostics::default();
        let _ = entity.upgrade(&mut UpgradeCx {
            load: &load,
            original_version: FormatVersion::new(1, 8, 0),
            identifier: "demo:ghost",
            actor_type: ActorType::Mob,
            skip_section_a: false,
            reading: load.reading,
            diagnostics: &mut diagnostics,
        });
        diagnostics
    }

    /// Where the file has the member the upgraded document has at `at` (below the
    /// description).
    fn in_the_file(told: &Diagnostics, at: &[&str]) -> String {
        told.source_of(&description_at().with(at)).to_string()
    }

    /// [`in_the_file`] for the entry `index` of `scripts.animate`.
    fn animate_in_the_file(told: &Diagnostics, index: usize) -> String {
        let mut at = description_at().with(&["scripts", "animate"]);
        at.push_index(index);
        told.source_of(&at).to_string()
    }

    /// What a later check tells of an animation or an
    /// `animate` entry the step made of a controller is told at the controller's element.
    #[test]
    fn the_animation_and_the_animate_entry_made_of_a_controller_are_where_the_file_has_it() {
        let told = stepped(
            r#""animations": {"walk": "animation.x.walk"}, "animation_controllers": [{"move": "controller.animation.x.a"}, {"zz": "controller.animation.x.z", "look": "controller.animation.x.b"}]"#,
        );
        let controllers = "minecraft:client_entity/description/animation_controllers";
        assert_eq!(
            in_the_file(&told, &["animations", "controller__move"]),
            format!("{controllers}/0/move")
        );
        assert_eq!(
            in_the_file(&told, &["animations", "controller__look"]),
            format!("{controllers}/1/look")
        );
        assert_eq!(
            animate_in_the_file(&told, 0),
            format!("{controllers}/0/move")
        );
        assert_eq!(
            animate_in_the_file(&told, 1),
            format!("{controllers}/1/look")
        );
        // An animation of the file is where it was.
        assert_eq!(
            in_the_file(&told, &["animations", "walk"]),
            "minecraft:client_entity/description/animations/walk"
        );
    }

    /// Of two elements of one key the later one made the animation that stands; the note
    /// of the earlier one and of an animation of the file stay where they are.
    #[test]
    fn an_animation_made_twice_is_where_the_later_controller_is() {
        let told = stepped(
            r#""animations": {"controller__move": "animation.x.own"}, "animation_controllers": [{"move": "controller.animation.x.a"}, {"move": "controller.animation.x.b"}]"#,
        );
        let description = "minecraft:client_entity/description";
        assert_eq!(
            in_the_file(&told, &["animations", "controller__move"]),
            format!("{description}/animation_controllers/1/move")
        );
        let mut told = told;
        told.name_as_the_file_has_it();
        let notes: Vec<String> = told
            .iter()
            .map(|f| format!("{} {:?} {}", f.code.id(), f.severity, f.location.json_path))
            .collect();
        assert_eq!(
            notes,
            [
                format!("core/dropped_field Info {description}/animations/controller__move"),
                format!("core/dropped_field Info {description}/animation_controllers/0/move"),
            ]
        );
    }
}
