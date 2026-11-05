//! The skin pack types and the game's reading of their members.

use std::borrow::Cow;

use bedrock_addon_core::pack_identity::PackOrigin;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;

use super::BUILT_IN_GEOMETRIES;

/// One `skins.json`: the skins of a skin pack.
///
/// JSON: `{"serialize_name": "MyPack", "localization_name": "MyPack", "skins":
/// [{"localization_name": "Hero", "geometry": "geometry.humanoid.custom", "texture":
/// "hero.png", "type": "free"}]}`. The game reads every string member the way it reads any
/// value as a string; where a member is written as something else, the typed field is `None`
/// and the member is kept in `extra` as written (the methods give the game's reading).
///
/// Hand-written: a skin pack is on no stack, so there is no walk or `LOADER`; [`SkinPack::load`]
/// is handed the pack's files, and its origins come from core's outside-the-stack kit
/// (`document::unversioned`), which persona's share.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "skins.json")]
pub struct SkinPack {
    /// JSON `serialize_name`. The game does not read it; creator tools want it equal to
    /// `localization_name`.
    pub serialize_name: Option<String>,
    /// JSON `localization_name`: the pack's name key is `skinpack.<name>`.
    pub localization_name: Option<String>,
    /// JSON `skins`: the entries that are objects, in order (of an object `skins`, its member
    /// values in key order). `None` when the member is absent.
    pub skins: Option<Vec<Skin>>,
    /// The other members, and the members above as written when the field cannot say them.
    pub extra: Extra,
    /// The root as the author wrote it, kept when the fields cannot say it (an entry of `skins`
    /// that is no object, a `skins` that is no array).
    /// `to_json` writes it for as long as the fields are still the game's reading of it; a
    /// value that was changed, or built by hand, writes its fields. The entries the game
    /// skipped are in this root alone, so one edit of any field drops every one of them from
    /// the written file, not only those beside the edited member. `PartialEq` and `Debug`
    /// include the root: a loaded value that kept one differs from a hand-built value with
    /// the same fields.
    pub written: Option<Value>,
}

/// One skin of a skin pack.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Skin {
    /// JSON `localization_name`: the skin's name key is `skin.<pack>.<name>`.
    pub localization_name: Option<String>,
    /// JSON `geometry`: one of [`BUILT_IN_GEOMETRIES`] or a model of the pack's
    /// `geometry.json`.
    pub geometry: Option<String>,
    /// JSON `texture`: the skin image, a file at the pack's root.
    pub texture: Option<String>,
    /// JSON `type`: `free` or `paid` by convention; the game compares it with nothing.
    pub type_: Option<String>,
    /// JSON `cape`: the cape image, a file of the pack.
    pub cape: Option<String>,
    /// JSON `animations`, as written: copied into the skin's resource patch.
    pub animations: Option<Value>,
    /// JSON `enable_attachables`, as written.
    pub enable_attachables: Option<Value>,
    /// JSON `held_item_ignores_lighting`, as written.
    pub held_item_ignores_lighting: Option<Value>,
    /// JSON `hide_armor`, as written.
    pub hide_armor: Option<Value>,
    /// The other members, and the string members above as written when they are no strings.
    pub extra: Extra,
}

/// A string member as the game reads it: the typed field, else the written value through
/// `asString("")` (a boolean is `true` / `false`, anything else empty).
fn text<'a>(typed: Option<&'a String>, extra: &'a Extra, name: &str) -> Cow<'a, str> {
    match typed {
        Some(text) => Cow::Borrowed(text),
        None => match extra.get(name).and_then(Value::as_bool) {
            Some(flag) => Cow::Owned(flag.to_string()),
            None => Cow::Borrowed(""),
        },
    }
}

impl SkinPack {
    /// The pack's name as the game reads `localization_name`; empty when absent.
    pub fn name(&self) -> Cow<'_, str> {
        text(
            self.localization_name.as_ref(),
            &self.extra,
            "localization_name",
        )
    }
}

impl Skin {
    /// The skin's name as the game reads `localization_name`; empty when absent.
    pub fn name(&self) -> Cow<'_, str> {
        text(
            self.localization_name.as_ref(),
            &self.extra,
            "localization_name",
        )
    }

    /// The model name as the game reads `geometry`.
    pub fn geometry_name(&self) -> Cow<'_, str> {
        text(self.geometry.as_ref(), &self.extra, "geometry")
    }

    /// The path of the skin image as the game reads `texture`. A path too long for the
    /// game's buffer (1024 bytes and more) reads as empty.
    pub fn texture_path(&self) -> Cow<'_, str> {
        fits(text(self.texture.as_ref(), &self.extra, "texture"))
    }

    /// The path of the cape image as the game reads `cape`; `None` when the skin has no
    /// `cape` member or its reading is empty.
    pub fn cape_path(&self) -> Option<Cow<'_, str>> {
        let written = self.cape.is_some() || self.extra.get("cape").is_some();
        let path = fits(text(self.cape.as_ref(), &self.extra, "cape"));
        (written && !path.is_empty()).then_some(path)
    }

    /// The skin's type for a pack that came from `origin`: `type` as the game reads it, and
    /// `free` whatever is written for a pack the user or a developer supplied.
    pub fn effective_type(&self, origin: PackOrigin) -> Cow<'_, str> {
        if matches!(origin, PackOrigin::User | PackOrigin::Dev) {
            Cow::Borrowed("free")
        } else {
            text(self.type_.as_ref(), &self.extra, "type")
        }
    }

    /// Whether the skin uses one of the game's own two models.
    pub fn uses_built_in_geometry(&self) -> bool {
        BUILT_IN_GEOMETRIES.contains(&self.geometry_name().as_ref())
    }
}

/// A path as the game's fixed buffer holds it: empty when it is 1024 bytes or longer.
fn fits(path: Cow<'_, str>) -> Cow<'_, str> {
    if path.len() > super::read::MAX_PATH {
        Cow::Borrowed("")
    } else {
        path
    }
}
