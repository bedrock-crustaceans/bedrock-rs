//! Validated, sorted intermediate form between the raw biome dump JSON (`input.rs`) and
//! the generated Rust (`emit.rs`) -- same shape and purpose as `codegen::palette::model`,
//! kept as its own smaller pipeline because the biome dump is a single flat id/name table,
//! not the two-table (properties + blocks) shape the block dump has.

use std::collections::BTreeMap;

use super::input::RawBiomeDump;
use crate::codegen::palette::GeneratorError;

#[derive(Debug, Clone, PartialEq)]
pub struct BiomeSpec {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionBiomes {
    pub tag: String,
    pub game_version: (u8, u8, u8),
    /// Sorted by [`BiomeSpec::id`].
    pub biomes: Vec<BiomeSpec>,
}

/// Validates and sorts one biome dump: every id and every name must be unique within the
/// dump (neither has been observed to collide in a real one; both are exactly the
/// "unrecognized/inconsistent input" this generator is required to fail loudly on).
pub fn build(
    tag: &str,
    game_version: (u8, u8, u8),
    raw: RawBiomeDump,
) -> Result<VersionBiomes, GeneratorError> {
    let mut by_id: BTreeMap<i32, String> = BTreeMap::new();
    let mut seen_names: BTreeMap<String, ()> = BTreeMap::new();

    for item in raw.data_items {
        if item.name.is_empty() {
            return Err(GeneratorError::EmptyBiomeName {
                tag: tag.to_string(),
            });
        }
        if seen_names.insert(item.name.clone(), ()).is_some() {
            return Err(GeneratorError::DuplicateBiomeName {
                tag: tag.to_string(),
                name: item.name,
            });
        }
        if by_id.insert(item.id, item.name.clone()).is_some() {
            return Err(GeneratorError::DuplicateBiomeId {
                tag: tag.to_string(),
                id: item.id,
            });
        }
    }

    Ok(VersionBiomes {
        tag: tag.to_string(),
        game_version,
        biomes: by_id
            .into_iter()
            .map(|(id, name)| BiomeSpec { id, name })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::biome::input::RawBiomeItem;

    fn dump(items: Vec<(i32, &str)>) -> RawBiomeDump {
        RawBiomeDump {
            data_items: items
                .into_iter()
                .map(|(id, name)| RawBiomeItem {
                    id,
                    name: name.to_string(),
                })
                .collect(),
            module_type: "vanilla_data".to_string(),
            name: "mojang-biome".to_string(),
            vanilla_data_type: Some("biome".to_string()),
        }
    }

    #[test]
    fn sorts_by_id() {
        let raw = dump(vec![(5, "minecraft:z"), (1, "minecraft:a")]);
        let biomes = build("test", (1, 0, 0), raw).unwrap();
        assert_eq!(biomes.biomes[0].id, 1);
        assert_eq!(biomes.biomes[1].id, 5);
    }

    #[test]
    fn rejects_duplicate_id() {
        let raw = dump(vec![(1, "minecraft:a"), (1, "minecraft:b")]);
        let err = build("test", (1, 0, 0), raw).unwrap_err();
        assert!(matches!(err, GeneratorError::DuplicateBiomeId { .. }));
    }

    #[test]
    fn rejects_duplicate_name() {
        let raw = dump(vec![(1, "minecraft:a"), (2, "minecraft:a")]);
        let err = build("test", (1, 0, 0), raw).unwrap_err();
        assert!(matches!(err, GeneratorError::DuplicateBiomeName { .. }));
    }

    #[test]
    fn rejects_empty_name() {
        let raw = dump(vec![(1, "")]);
        let err = build("test", (1, 0, 0), raw).unwrap_err();
        assert!(matches!(err, GeneratorError::EmptyBiomeName { .. }));
    }
}
