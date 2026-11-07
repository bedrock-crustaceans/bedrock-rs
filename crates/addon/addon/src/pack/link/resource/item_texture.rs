//! The `minecraft:icon` of each behaviour-pack item against the merged item
//! atlas of the resource stack, with the icon rules of the `item_texture` folder that need the
//! item and are told with it.
//!
//! An icon names tiles of `textures/item_texture.json`: the bare name, the 1.20.50 `texture`
//! member and every value of `textures`, except the value of a `<material>_palette` key, which
//! names a trim colour palette. The client looks a name up as written; the name of a tile the
//! atlas lacks shows the missing texture.

use bedrock_addon_core::diagnostic::builders;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::{ComponentMap, Origin};

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use crate::diagnostic::{Code, JsonPath, Severity, sentence};
use crate::families::item::component::{Icon, ItemComponent, Tags, Wearable, WearableSlot};
use crate::families::item::trim::TrimData;
use crate::families::item::{ItemFormat, LegacyItemData};
use crate::id::{AtlasKey, ItemId};
use crate::json::Value;

/// The linked-asset validator's line for an item-atlas name that is no tile: the breadcrumb
/// (the item's name, `-> components -> minecraft:icon: `), then the name.
const MISSING_ASSET: &str = "{} Missing referenced asset {}";

/// [`MISSING_ASSET`] for a name that a tile matches when case is ignored: the tile's name is
/// appended (the crate's own; the client's lookup is exact).
const MISSING_ASSET_BY_CASE: &str =
    "{} Missing referenced asset {} (an atlas key differs only in case: '{}')";

/// A `textures` map without `default`; the argument is the item's full name.
const NO_DEFAULT: &str =
    "{} : minecraft:icon component should include default key in textures map to set icon texture";

/// Trimmable armour without `minecraft:icon`; the argument is the item's full name.
const NO_ICON: &str = "Custom armor item '{}' needs an icon component";

/// Trimmable armour whose `minecraft:wearable` is worn in no armour slot; the argument is the
/// item's full name.
const NOT_ARMOR: &str = "Only armor wearable components are trimmable; '{}' is not armor.";

/// Trimmable armour without `minecraft:wearable`; the argument is the item's full name.
const NO_WEARABLE: &str = "Custom armor item '{}' needs a wearable component";

/// The keys of `textures` the game reads besides the palettes.
const READ_KEYS: [&str; 5] = [
    "default",
    "dyed",
    "icon_trim",
    "bundle_open_front",
    "bundle_open_back",
];

/// The item tag of the armour the client makes trimmed icons for.
const TRIMMABLE_ARMORS: &str = "minecraft:trimmable_armors";

/// Every item of the behaviour stack, stable then beta. The names of an icon are compared
/// when the world has a view of the item atlas: a pack of the resource stack has the atlas
/// file, or a host answers for the game's tiles. The rules about the item alone always run.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let items = cx.behavior().items();
    let beta = items.beta();
    if items.is_empty() && beta.is_none_or(bedrock_addon_core::document::Registry::is_empty) {
        return;
    }
    let atlas_in_view = cx.item_atlas_view(out);
    for (_, document) in items {
        let item = document.value();
        let subject = Subject {
            origin: document.origin(),
            id: &item.description.identifier,
            components: &item.components,
            legacy: &item.legacy,
            atlas_in_view,
        };
        subject.check(cx, out);
    }
    for (_, document) in beta.into_iter().flat_map(|beta| beta.iter()) {
        let item = document.value();
        let subject = Subject {
            origin: document.origin(),
            id: &item.description.identifier,
            components: &item.components,
            legacy: &item.legacy,
            atlas_in_view,
        };
        subject.check(cx, out);
    }
}

/// What the rules read of one item, stable or beta.
#[derive(Debug)]
struct Subject<'a> {
    origin: &'a Origin,
    id: &'a ItemId,
    components: &'a ComponentMap<ItemComponent>,
    legacy: &'a LegacyItemData,
    /// Whether the world has a view of the item atlas.
    atlas_in_view: super::super::View,
}

impl Subject<'_> {
    /// The path of `minecraft:icon` in the file, or of its member `keys` below it.
    fn icon_at(keys: &[&str]) -> JsonPath {
        let mut path = components_at();
        path.push("minecraft:icon");
        for key in keys {
            path.push(key);
        }
        path
    }

    /// The rules of one item.
    fn check(&self, cx: &LinkCx<'_>, out: &mut Findings) {
        if self.legacy.format == ItemFormat::Legacy {
            self.legacy_icon(cx, out);
            return;
        }
        let trimmable = self
            .components
            .get::<Tags>()
            .is_some_and(|tags| tags.tags.iter().any(|tag| tag == TRIMMABLE_ARMORS));
        let Some(icon) = self.components.get::<Icon>() else {
            if trimmable {
                // The icon is the first of the three things the game asks of trimmable armour.
                self.no_trimmed_icons(out, NO_ICON);
            }
            return;
        };
        if !icon.textures.is_empty() && !icon.textures.contains_key("default") {
            // The game sets the item's icon to the empty name.
            let at = At::new(self.origin, Self::icon_at(&["textures"]), Severity::Error);
            out.push(at.diagnostic(builders::worded(
                &core_codes::MISSING_REQUIRED_FIELD,
                &core_codes::MISSING_REQUIRED_FIELD_WORDED,
                formatted(NO_DEFAULT, &[self.id.as_str()]).as_deref(),
            )));
        }
        for (key, name) in &icon.textures {
            // The file may hold the `default` texture as the bare name, as `texture` or as
            // `textures.default`: the component is the place all three share.
            let path = if key == "default" {
                Self::icon_at(&[])
            } else {
                Self::icon_at(&["textures", key])
            };
            match role(key) {
                Role::Palette => continue,
                Role::Unread if cx.lints() => {
                    let at = At::new(self.origin, path.clone(), Severity::Info);
                    out.push(at.diagnostic(Code::from(&core_codes::NO_EFFECT)));
                }
                Role::Unread | Role::Read => {}
            }
            self.tile(cx, out, name, path);
        }
        if !trimmable {
            return;
        }
        match self.components.get::<Wearable>() {
            None => self.no_trimmed_icons(out, NO_WEARABLE),
            // The off hand is the one slot of the component that is no armour slot.
            Some(wearable) if !is_armor_slot(wearable.slot) => {
                let mut path = components_at();
                path.push("minecraft:wearable");
                path.push("slot");
                // The game logs this at the warning level: a warning.
                let at = At::new(self.origin, path, Severity::Warning);
                out.push(at.diagnostic(builders::worded(
                    &core_codes::INVALID_VALUE,
                    &core_codes::INVALID_VALUE_WORDED,
                    formatted(NOT_ARMOR, &[self.id.as_str()]).as_deref(),
                )));
            }
            Some(_) => {}
        }
    }

    /// A component trimmable armour lacks, told with the game's `text` at the item's
    /// components: the item gets no trimmed icons.
    // Both lines are logged at the warning level: warnings.
    fn no_trimmed_icons(&self, out: &mut Findings, text: &'static str) {
        let at = At::new(self.origin, components_at(), Severity::Warning);
        out.push(at.diagnostic(builders::worded(
            &core_codes::MISSING_REQUIRED_FIELD,
            &core_codes::MISSING_REQUIRED_FIELD_WORDED,
            formatted(text, &[self.id.as_str()]).as_deref(),
        )));
    }

    /// The icon of an item of the legacy item format: the bare name, or `texture`.
    fn legacy_icon(&self, cx: &LinkCx<'_>, out: &mut Findings) {
        let Some(icon) = self
            .legacy
            .format_item
            .as_ref()
            .and_then(|item| item.client.get("minecraft:icon"))
        else {
            return;
        };
        let name = icon.as_string().or_else(|| {
            icon.as_object()
                .and_then(|icon| icon.get("texture"))
                .and_then(Value::as_string)
        });
        if let Some(name) = name {
            self.tile(cx, out, name.as_str(), Self::icon_at(&[]));
        }
    }

    /// One item-atlas name. The empty name is no reference.
    fn tile(&self, cx: &LinkCx<'_>, out: &mut Findings, name: &str, path: JsonPath) {
        if name.is_empty() || !self.atlas_in_view.judges() {
            return;
        }
        let Target::Missing(hint) = cx.item_atlas_key(&AtlasKey::parse(name)) else {
            return;
        };
        let breadcrumb = format!("{} -> components -> minecraft:icon: ", self.id.as_str());
        let text = match &hint {
            Some(crate::diagnostic::ReferenceHint::CaseMismatch(tile)) => sentence(
                MISSING_ASSET_BY_CASE,
                &[breadcrumb, name.to_owned(), tile.clone()],
            ),
            _ => sentence(MISSING_ASSET, &[breadcrumb, name.to_owned()]),
        };
        // The linked-asset check's line is at the error level and nothing is dropped: a kept
        // error. With an atlas of an add-on's own alone the tile may be the game's, so that
        // case is an information.
        let severity = self.atlas_in_view.severity(Severity::Error);
        let at = At::new(self.origin, path, severity).with_text(text);
        let finding = unresolved(at, kinds::ATLAS_KEY, name, hint);
        out.push(if finding.severity == Severity::Error {
            finding.into_kept()
        } else {
            finding
        });
    }
}

/// Whether a wearable in `slot` is humanoid armour: worn on the head, chest, legs or feet.
// Of the component's five slots only the off hand is no armour slot.
fn is_armor_slot(slot: WearableSlot) -> bool {
    matches!(
        slot,
        WearableSlot::Head | WearableSlot::Chest | WearableSlot::Legs | WearableSlot::Feet
    )
}

/// The path of an item's `components` in its file.
fn components_at() -> JsonPath {
    let mut path = JsonPath::new();
    path.push("minecraft:item");
    path.push("components");
    path
}

/// A vanilla text with arguments.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every call site fills an `Option<String>` field"
)]
fn formatted(template: &'static str, args: &[&str]) -> Option<String> {
    Some(sentence(template, args))
}

/// What the game does with a key of `textures`.
#[derive(Debug)]
enum Role {
    /// One of the five keys the game reads: its value is a tile.
    Read,
    /// `<material>_palette` of a trim material: its value names a colour palette, no tile.
    Palette,
    /// Any other key: the game never reads it.
    Unread,
}

/// The role of `key`. A palette key is told by the text before its first `_palette`, which
/// must be one of the game's trim materials.
fn role(key: &str) -> Role {
    if READ_KEYS.contains(&key) {
        return Role::Read;
    }
    let material = key.find("_palette").and_then(|end| key.get(..end));
    if material.is_some_and(|material| TrimData::vanilla().material(material).is_some()) {
        return Role::Palette;
    }
    Role::Unread
}
