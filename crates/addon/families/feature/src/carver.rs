//! The cave carvers: `cave_carver_feature`, `nether_cave_carver_feature` and
//! `underwater_cave_carver_feature`, which share their members.

use facet::Facet;

use super::Description;
use super::validate::Check;
use super::values::{CarverRange, JsonInt};
use super::walk::{WORLD_GEN, Walk};
use super::{Rules, V1_13_0, strict_struct};
use bedrock_addon_core::diagnostic::{DiagnosticCode, VanillaText};
use bedrock_addon_core::molang::{Molang, MolangFields, MolangVisitor};
use bedrock_addon_core::shared::BlockSpecifier;
use bedrock_addon_core::versioning::ValidateCx;

/// The client's text for a float range array of another length than two.
const FLOAT_RANGE_ARRAY: &str =
    "Float range array was not parsed. Float range arrays should have 2 values.";

/// Declares one carver struct with the common members (and `replace_air_with` where given).
macro_rules! carver {
    ($(#[$meta:meta])* $name:ident { $($(#[$extra_meta:meta])* $extra:ident),* }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Facet)]
        pub struct $name {
            /// JSON `description`.
            pub description: Description,
            /// The block the carved space is filled with (JSON `fill_with`).
            #[facet(default)]
            pub fill_with: Option<BlockSpecifier>,
            /// A Molang expression over the `world_gen` queries scaling the tunnel width (JSON
            /// `width_modifier`).
            #[facet(default)]
            pub width_modifier: Option<Molang>,
            /// One in how many chunks is skipped, at least 1 (JSON `skip_carve_chance`).
            #[facet(default)]
            pub skip_carve_chance: Option<JsonInt>,
            /// JSON `height_limit`.
            #[facet(default)]
            pub height_limit: Option<JsonInt>,
            /// JSON `y_scale`.
            #[facet(default)]
            pub y_scale: Option<CarverRange>,
            /// JSON `horizontal_radius_multiplier`.
            #[facet(default)]
            pub horizontal_radius_multiplier: Option<CarverRange>,
            /// JSON `vertical_radius_multiplier`.
            #[facet(default)]
            pub vertical_radius_multiplier: Option<CarverRange>,
            /// JSON `floor_level`.
            #[facet(default)]
            pub floor_level: Option<CarverRange>,
            $(
                $(#[$extra_meta])*
                #[facet(default)]
                pub $extra: Option<BlockSpecifier>,
            )*
        }

        strict_struct!($name, V1_13_0);

        impl Walk for $name {
            fn walk(&mut self, v: &mut MolangVisitor<'_>) {
                v.field("width_modifier", &WORLD_GEN, &mut self.width_modifier);
            }
        }

        impl MolangFields for $name {
            fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
                self.walk(v);
            }
        }

        // The blocks a carver names.
        impl super::walk::Refer for $name {
            fn refer<'a>(&'a self, out: &mut super::walk::References<'a>) {
                out.member("fill_with", &self.fill_with);
                $( out.member(stringify!($extra), &self.$extra); )*
            }
        }

        impl Rules for $name {
            fn rules(&mut self, check: &mut ValidateCx<'_>) {
                if let Some(block) = &self.fill_with {
                    let at = check.path("fill_with");
                    check.diagnostics.extend(block.check(&at));
                }
                $(
                    if let Some(block) = &self.$extra {
                        let at = check.path(stringify!($extra));
                        check.diagnostics.extend(block.check(&at));
                    }
                )*
                check.int_min("skip_carve_chance", self.skip_carve_chance, 1);
                for (key, range) in [
                    ("y_scale", &self.y_scale),
                    ("horizontal_radius_multiplier", &self.horizontal_radius_multiplier),
                    ("vertical_radius_multiplier", &self.vertical_radius_multiplier),
                    ("floor_level", &self.floor_level),
                ] {
                    range_rules(check, key, range.as_ref());
                }
            }
        }
    };
}

/// A carver float range: its own finding, or vanilla's text for an unparsed array. The array
/// is an error that refuses the feature: the client logs "Float range array was not parsed",
/// the range then fails its validation, and a failed member fails the whole schema.
fn range_rules(check: &mut ValidateCx<'_>, key: &str, range: Option<&CarverRange>) {
    match range {
        Some(CarverRange::Range(range)) => {
            let at = check.path(key);
            check.diagnostics.extend(range.check(&at));
        }
        Some(CarverRange::Unparsed(_)) => check.error(
            key,
            DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed(FLOAT_RANGE_ARRAY)),
            },
        ),
        None => {}
    }
}

carver!(
    /// `minecraft:cave_carver_feature`: carves caves.
    CaveCarverFeature {}
);

carver!(
    /// `minecraft:nether_cave_carver_feature`: carves nether caves.
    NetherCaveCarverFeature {}
);

carver!(
    /// `minecraft:underwater_cave_carver_feature`: carves caves below water.
    UnderwaterCaveCarverFeature {
        /// The block air is replaced with (JSON `replace_air_with`).
        replace_air_with
    }
);
