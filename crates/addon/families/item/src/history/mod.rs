//! The older item formats, one struct per format version, and the chain between them.
//!
//! `v1_10_0` is the legacy item format (and every document that is not component based);
//! `v1_16_100` … `v1_26_0` are the component formats, each an instance of
//! [`ItemDocument`](crate::history::ItemDocument); `v1_26_30` is the first modern format
//! (its sound members bind level sound events); the newest format, 1.26.50, is
//! [`crate::Item`]. [`Any`] is the struct a file's
//! `format_version` selected, from `Item::parse_as_written`.

mod component;
mod document;
mod gates;
mod steps;

pub use component::{
    Armor, Chargeable, Component, Components, CustomComponentEntry, CustomComponentList,
    CustomComponents, LegacyDestroySpeed, LegacyDigger, LegacyDurability, LegacyFoodComponent,
    LegacyIcon, LegacyRepairable, LegacyStorageItem, OnUse, OnUseOn, RenderOffsets, UseDuration,
    Weapon,
};
pub use document::{Description, ItemDocument, pack, unpack};

pub(crate) use document::{
    SideCarPlace, binding, into_component_map, parse_parts, write_parts, written_components,
};
#[cfg(test)]
pub(crate) use gates::{GateRow, gate_with, release_threshold};
pub(crate) use gates::{HISTORY_ROWS, gate};

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::FormatVersionInput;
use bedrock_addon_core::versioning::{self, Family, Loaded};

use crate::Item;

pub mod v1_10_0;
pub mod v1_26_30;

/// Declares the module of one component format.
macro_rules! format_module {
    ($(#[$meta:meta])* $module:ident = ($major:literal, $minor:literal, $patch:literal)) => {
        $(#[$meta])*
        pub mod $module {
            /// The item document of this format version.
            pub type Item = super::ItemDocument<{ super::pack($major, $minor, $patch) }>;
        }
    };
}

format_module! {
    /// 1.16.100 (and 1.17.0–1.19.82): the dynamic map of every registered component, with the
    /// pre-1.18 `food`, `durability` and `repairable` shapes.
    v1_16_100 = (1, 16, 100)
}
format_module! {
    /// 1.19.83: `food`, `durability` and `repairable` in their post-1.18 shape.
    v1_19_83 = (1, 19, 83)
}
format_module! {
    /// 1.20.0: the typed `description` with `category`.
    v1_20_0 = (1, 20, 0)
}
format_module! {
    /// 1.20.20: `description.menu_category`, `glint`; `armor` removed.
    v1_20_20 = (1, 20, 20)
}
format_module! {
    /// 1.20.30: the fixed component set; `food` without `on_consume`.
    v1_20_30 = (1, 20, 30)
}
format_module! {
    /// 1.20.40: `on_use`, `on_use_on`, `weapon`, `render_offsets` removed.
    v1_20_40 = (1, 20, 40)
}
format_module! {
    /// 1.20.50: `use_modifiers` and `tags`; `chargeable` and `use_duration` removed.
    v1_20_50 = (1, 20, 50)
}
format_module! {
    /// 1.20.60: `icon.textures`; unknown members are reported from here on.
    v1_20_60 = (1, 20, 60)
}
format_module! {
    /// 1.20.80: `custom_components` as a list of names.
    v1_20_80 = (1, 20, 80)
}
format_module! {
    /// 1.21.10: `damage_absorption`, `durability_sensor`.
    v1_21_10 = (1, 21, 10)
}
format_module! {
    /// 1.21.30: `bundle_interaction`, `dyeable`, `rarity`, `storage_item`.
    v1_21_30 = (1, 21, 30)
}
format_module! {
    /// 1.21.40: `block_placer.replace_block_item`.
    v1_21_40 = (1, 21, 40)
}
format_module! {
    /// 1.21.50: `compostable`.
    v1_21_50 = (1, 21, 50)
}
format_module! {
    /// 1.21.60: namespaced `menu_category.group`, `custom_components` entries, the storage
    /// weight components.
    v1_21_60 = (1, 21, 60)
}
format_module! {
    /// 1.21.80: `icon` rebound.
    v1_21_80 = (1, 21, 80)
}
format_module! {
    /// 1.21.90: `kinetic_weapon`, `piercing_weapon`, `fire_resistant`, `swing_duration`,
    /// `swing_sounds`, `wearable.hides_player_location`; keyed custom components.
    v1_21_90 = (1, 21, 90)
}
format_module! {
    /// 1.21.110: `menu_category.category` retyped (default `none`).
    v1_21_110 = (1, 21, 110)
}
format_module! {
    /// 1.26.0: `damage` 0–32767, `block_placer.aligned_placement`.
    v1_26_0 = (1, 26, 0)
}

/// The item family of the version-chain machinery (hidden: only the loaders name it).
#[doc(hidden)]
pub struct ItemFamily;

bedrock_addon_core::version_chain! {
    family = ItemFamily;
    /// The struct an item file's `format_version` selected (the legacy item format for a
    /// document that is not component based).
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// The legacy item format.
        V1_10_0(v1_10_0::Item),
        /// The 1.16.100 format.
        V1_16_100(v1_16_100::Item),
        /// The 1.19.83 format.
        V1_19_83(v1_19_83::Item),
        /// The 1.20.0 format.
        V1_20_0(v1_20_0::Item),
        /// The 1.20.20 format.
        V1_20_20(v1_20_20::Item),
        /// The 1.20.30 format.
        V1_20_30(v1_20_30::Item),
        /// The 1.20.40 format.
        V1_20_40(v1_20_40::Item),
        /// The 1.20.50 format.
        V1_20_50(v1_20_50::Item),
        /// The 1.20.60 format.
        V1_20_60(v1_20_60::Item),
        /// The 1.20.80 format.
        V1_20_80(v1_20_80::Item),
        /// The 1.21.10 format.
        V1_21_10(v1_21_10::Item),
        /// The 1.21.30 format.
        V1_21_30(v1_21_30::Item),
        /// The 1.21.40 format.
        V1_21_40(v1_21_40::Item),
        /// The 1.21.50 format.
        V1_21_50(v1_21_50::Item),
        /// The 1.21.60 format.
        V1_21_60(v1_21_60::Item),
        /// The 1.21.80 format.
        V1_21_80(v1_21_80::Item),
        /// The 1.21.90 format.
        V1_21_90(v1_21_90::Item),
        /// The 1.21.110 format.
        V1_21_110(v1_21_110::Item),
        /// The 1.26.0 format.
        V1_26_0(v1_26_0::Item),
        /// The 1.26.30 format.
        V1_26_30(v1_26_30::Item),
        /// The newest format, 1.26.50.
        V1_26_50(crate::Item),
    }
    beta = /// A beta document.
    Beta(crate::Beta);
}

impl Any {
    /// Runs the remaining upgrade steps on a document
    /// [`Item::parse_as_written`](bedrock_addon_core::document::JsonDocument::parse_as_written)
    /// gave: the chain's checks `Item::parse` runs, without its resource-pack rule (an item at
    /// 1.16.100 or later refused in a resource pack), which `parse_as_written` does not apply
    /// either. A beta document is rejected here; it has no stable form.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the document is a beta one, or a remaining upgrade step leaves an
    /// error finding.
    pub fn upgrade(document: Document<Any>, cx: &LoadContext) -> Result<Document<Item>, Rejected> {
        let (any, origin, version, diagnostics) = document.into_parsed_parts();
        let identity = versioning::Identity {
            identifier: Some(any.identifier().to_owned()),
            runtime_identifier: None,
        };
        let written = FormatVersionInput {
            version: version.written.or(Some(version.effective)),
            beta_literal: version.beta,
            prerelease: false,
        };
        let at = JsonPath::new().child(ItemFamily::PAYLOAD_KEY);
        let parsed = versioning::Parsed::<ItemFamily>::from_value(
            Some(any),
            version,
            written,
            at,
            diagnostics,
        )
        .with_identity(identity);
        let outcome = versioning::finish::<ItemFamily>(parsed, cx);
        let diagnostics = outcome.diagnostics.located(&origin);
        match outcome.value {
            Some(Loaded::Stable(item)) => {
                Ok(Document::new(item, origin, outcome.version, diagnostics))
            }
            _ => Err(Rejected::new(origin, diagnostics)),
        }
    }

    /// The identifier the document names.
    pub fn identifier(&self) -> &str {
        match self {
            Any::V1_10_0(v) => &v.description.identifier,
            Any::V1_16_100(v) => &v.description.identifier,
            Any::V1_19_83(v) => &v.description.identifier,
            Any::V1_20_0(v) => &v.description.identifier,
            Any::V1_20_20(v) => &v.description.identifier,
            Any::V1_20_30(v) => &v.description.identifier,
            Any::V1_20_40(v) => &v.description.identifier,
            Any::V1_20_50(v) => &v.description.identifier,
            Any::V1_20_60(v) => &v.description.identifier,
            Any::V1_20_80(v) => &v.description.identifier,
            Any::V1_21_10(v) => &v.description.identifier,
            Any::V1_21_30(v) => &v.description.identifier,
            Any::V1_21_40(v) => &v.description.identifier,
            Any::V1_21_50(v) => &v.description.identifier,
            Any::V1_21_60(v) => &v.description.identifier,
            Any::V1_21_80(v) => &v.description.identifier,
            Any::V1_21_90(v) => &v.description.identifier,
            Any::V1_21_110(v) => &v.description.identifier,
            Any::V1_26_0(v) => &v.description.identifier,
            Any::V1_26_30(v) => v.description.identifier.as_str(),
            Any::V1_26_50(v) => v.description.identifier.as_str(),
            Any::Beta(v) => v.description.identifier.as_str(),
        }
    }
}
