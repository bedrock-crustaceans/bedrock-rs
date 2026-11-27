//! The cave carvers: `cave_carver_feature`, `nether_cave_carver_feature` and
//! `underwater_cave_carver_feature`, which share their members.

use facet::Facet;

use super::Description;
use super::molang::WORLD_GEN;
use super::values::{CarverRange, JsonInt};
use super::{V1_13_0, strict_struct};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_shared_shapes::BlockSpecifier;

/// Declares one carver struct with the common members (and `replace_air_with` where given).
macro_rules! carver {
    ($(#[$meta:meta])* $name:ident { $($(#[$extra_meta:meta])* $extra:ident),* }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Facet, Schema)]
        pub struct $name {
            /// JSON `description`.
            pub description: Description,
            /// The block the carved space is filled with (JSON `fill_with`).
            pub fill_with: Option<BlockSpecifier>,
            /// A Molang expression over the `world_gen` queries scaling the tunnel width (JSON
            /// `width_modifier`).
            #[facet(schema::molang = WORLD_GEN)]
            pub width_modifier: Option<Molang>,
            /// One in how many chunks is skipped, at least 1 (JSON `skip_carve_chance`).
            #[facet(schema::min = 1)]
            pub skip_carve_chance: Option<JsonInt>,
            /// JSON `height_limit`.
            pub height_limit: Option<JsonInt>,
            /// JSON `y_scale`.
            pub y_scale: Option<CarverRange>,
            /// JSON `horizontal_radius_multiplier`.
            pub horizontal_radius_multiplier: Option<CarverRange>,
            /// JSON `vertical_radius_multiplier`.
            pub vertical_radius_multiplier: Option<CarverRange>,
            /// JSON `floor_level`.
            pub floor_level: Option<CarverRange>,
            $(
                $(#[$extra_meta])*
                pub $extra: Option<BlockSpecifier>,
            )*
        }

        strict_struct!($name, V1_13_0);

        // The blocks a carver names.
        impl super::molang::Refer for $name {
            fn refer<'a>(&'a self, out: &mut super::molang::References<'a>) {
                out.member("fill_with", &self.fill_with);
                $( out.member(stringify!($extra), &self.$extra); )*
            }
        }
    };
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
