//! Renders a validated [`VersionBiomes`](super::model::VersionBiomes) to the Rust source
//! `bedrock_vanilla_data` compiles -- same `prettyplease`-then-`rustfmt` handoff as
//! `codegen::palette::emit`, driven by the shared `write_pretty` in `codegen::palette`.

use proc_macro2::TokenStream;
use quote::quote;

use super::model::{BiomeSpec, VersionBiomes};

/// The generated file's module name for one version, e.g. `biome_v1_21_130` for
/// `(1, 21, 130)`. Prefixed `biome_` (unlike the block-palette modules' bare `v1_21_130`)
/// so the two pipelines' per-version files never collide in the shared
/// `crates/vanilla_data/src/generated/` directory. See `codegen::versioned_module_name`,
/// shared with `codegen::palette::emit`.
pub fn module_name(game_version: (u8, u8, u8)) -> String {
    super::super::versioned_module_name("biome_", game_version)
}

/// Renders one version's biome table to a full source file. The header text is built by
/// `codegen::generated_header`, shared with `codegen::palette::emit`.
pub fn render_version_file(biomes: &VersionBiomes) -> TokenStream {
    let tag = &biomes.tag;
    let (major, minor, patch) = biomes.game_version;

    let header = super::super::generated_header(
        tag,
        "metadata/vanilladata_modules/mojang-biomes.json",
        "xtask/src/codegen/biome",
    );

    let entries = biomes.biomes.iter().map(render_biome_spec);

    quote! {
        #![doc = #header]

        use crate::biome::{BiomeEntry, VersionBiomes};

        pub static BIOMES: VersionBiomes = VersionBiomes {
            tag: #tag,
            game_version: (#major, #minor, #patch),
            biomes: &[
                #(#entries),*
            ],
        };
    }
}

fn render_biome_spec(spec: &BiomeSpec) -> TokenStream {
    let id = spec.id;
    let name = &spec.name;
    quote! {
        BiomeEntry {
            id: #id,
            name: #name,
        }
    }
}
