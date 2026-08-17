//! Validated, sorted intermediate form between the raw dump JSON (`input.rs`) and the
//! generated Rust (`emit.rs`).
//!
//! Sorting happens here, once, rather than being left to emission: every list in a
//! [`VersionPalette`] is ordered by name before anything downstream sees it, which is
//! what makes two generator runs over the same input byte-identical (the determinism the
//! schema-generator task requires) and what makes a diff between two versions' generated
//! files a diff of actual content rather than incidental reordering.

use std::collections::BTreeMap;

use super::GeneratorError;
use super::input::{RawDataItem, RawDump, RawValue};

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyKind {
    Bool,
    /// Sorted ascending, deduplicated.
    Int(Vec<i64>),
    /// Sorted, deduplicated.
    Str(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertySpec {
    pub name: String,
    pub kind: PropertyKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockSpec {
    pub name: String,
    /// Sorted, deduplicated. Names into the owning [`VersionPalette::properties`].
    pub properties: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionPalette {
    pub tag: String,
    pub game_version: (u8, u8, u8),
    /// Sorted by [`PropertySpec::name`].
    pub properties: Vec<PropertySpec>,
    /// Sorted by [`BlockSpec::name`].
    pub blocks: Vec<BlockSpec>,
}

impl VersionPalette {
    /// Linear-search lookup by name -- fine for the generator's own tooling and tests,
    /// which never run over more than one version's worth of blocks at a time. The
    /// generated, runtime-facing form (`bedrock_vanilla_data::palette::VersionPalette`)
    /// has its own binary-search accessor over the emitted `&'static` slices.
    pub fn block(&self, name: &str) -> Option<&BlockSpec> {
        self.blocks.iter().find(|b| b.name == name)
    }

    /// See [`Self::block`].
    pub fn property(&self, name: &str) -> Option<&PropertySpec> {
        self.properties.iter().find(|p| p.name == name)
    }
}

/// Validates and sorts one dump into a [`VersionPalette`], failing on anything the dump
/// asserts that does not actually hold -- a property whose declared `type` disagrees with
/// its own `values`, a block referencing a property name absent from `block_properties`,
/// a duplicate name anywhere a name is meant to be unique. None of these has been observed
/// in a real dump; they are exactly the "unrecognized input" the generator is required to
/// fail loudly on rather than silently work around.
pub fn build(
    tag: &str,
    game_version: (u8, u8, u8),
    raw: RawDump,
) -> Result<VersionPalette, GeneratorError> {
    let mut properties = BTreeMap::new();
    for prop in raw.block_properties {
        let kind = property_kind(tag, &prop.name, &prop.ty, &prop.values)?;
        if properties.insert(prop.name.clone(), kind).is_some() {
            return Err(GeneratorError::DuplicateProperty {
                tag: tag.to_string(),
                property: prop.name,
            });
        }
    }

    let mut blocks = BTreeMap::new();
    for item in raw.data_items {
        let RawDataItem {
            name,
            properties: refs,
            ..
        } = item;
        if name.is_empty() {
            return Err(GeneratorError::EmptyBlockName {
                tag: tag.to_string(),
            });
        }

        let mut names = Vec::with_capacity(refs.len());
        for r in refs {
            if !properties.contains_key(&r.name) {
                return Err(GeneratorError::UnknownPropertyRef {
                    tag: tag.to_string(),
                    block: name.clone(),
                    property: r.name,
                });
            }
            names.push(r.name);
        }
        let before = names.len();
        names.sort();
        names.dedup();
        if names.len() != before {
            return Err(GeneratorError::DuplicatePropertyOnBlock {
                tag: tag.to_string(),
                block: name,
            });
        }

        if blocks.insert(name.clone(), names).is_some() {
            return Err(GeneratorError::DuplicateBlock {
                tag: tag.to_string(),
                block: name,
            });
        }
    }

    Ok(VersionPalette {
        tag: tag.to_string(),
        game_version,
        properties: properties
            .into_iter()
            .map(|(name, kind)| PropertySpec { name, kind })
            .collect(),
        blocks: blocks
            .into_iter()
            .map(|(name, properties)| BlockSpec { name, properties })
            .collect(),
    })
}

fn property_kind(
    tag: &str,
    name: &str,
    ty: &str,
    values: &[super::input::RawPropertyValue],
) -> Result<PropertyKind, GeneratorError> {
    match ty {
        "bool" => {
            for v in values {
                if !matches!(v.value, RawValue::Bool(_)) {
                    return Err(mismatch(tag, name, ty));
                }
            }
            Ok(PropertyKind::Bool)
        }
        "int" => {
            let mut ints = Vec::with_capacity(values.len());
            for v in values {
                match v.value {
                    RawValue::Int(i) => ints.push(i),
                    _ => return Err(mismatch(tag, name, ty)),
                }
            }
            ints.sort_unstable();
            ints.dedup();
            Ok(PropertyKind::Int(ints))
        }
        "string" => {
            let mut strs = Vec::with_capacity(values.len());
            for v in values {
                match &v.value {
                    RawValue::Str(s) => strs.push(s.clone()),
                    _ => return Err(mismatch(tag, name, ty)),
                }
            }
            strs.sort();
            strs.dedup();
            Ok(PropertyKind::Str(strs))
        }
        other => Err(GeneratorError::UnknownPropertyType {
            tag: tag.to_string(),
            property: name.to_string(),
            ty: other.to_string(),
        }),
    }
}

fn mismatch(tag: &str, property: &str, ty: &str) -> GeneratorError {
    GeneratorError::PropertyValueTypeMismatch {
        tag: tag.to_string(),
        property: property.to_string(),
        ty: ty.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::palette::input::{
        RawBlockProperty, RawDataItem, RawPropertyRef, RawPropertyValue,
    };

    fn dump(properties: Vec<RawBlockProperty>, items: Vec<RawDataItem>) -> RawDump {
        RawDump {
            block_properties: properties,
            data_items: items,
            minecraft_version: None,
            module_type: "vanilla_data".to_string(),
            name: "mojang-block".to_string(),
            vanilla_data_type: Some("block".to_string()),
        }
    }

    fn bool_prop(name: &str) -> RawBlockProperty {
        RawBlockProperty {
            name: name.to_string(),
            ty: "bool".to_string(),
            values: vec![
                RawPropertyValue {
                    value: RawValue::Bool(false),
                },
                RawPropertyValue {
                    value: RawValue::Bool(true),
                },
            ],
        }
    }

    fn item(name: &str, properties: &[&str]) -> RawDataItem {
        RawDataItem {
            name: name.to_string(),
            properties: properties
                .iter()
                .map(|p| RawPropertyRef {
                    name: p.to_string(),
                })
                .collect(),
            raw_id: None,
            serialization_id: None,
        }
    }

    #[test]
    fn sorts_properties_and_blocks_by_name() {
        let raw = dump(
            vec![bool_prop("z_prop"), bool_prop("a_prop")],
            vec![
                item("minecraft:z_block", &[]),
                item("minecraft:a_block", &[]),
            ],
        );
        let palette = build("test", (1, 0, 0), raw).unwrap();

        let prop_names: Vec<_> = palette.properties.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(prop_names, vec!["a_prop", "z_prop"]);

        let block_names: Vec<_> = palette.blocks.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(block_names, vec!["minecraft:a_block", "minecraft:z_block"]);
    }

    #[test]
    fn rejects_a_block_referencing_an_undeclared_property() {
        let raw = dump(vec![], vec![item("minecraft:stone", &["nonexistent"])]);
        let err = build("test", (1, 0, 0), raw).unwrap_err();
        assert!(
            matches!(err, GeneratorError::UnknownPropertyRef { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn rejects_a_property_whose_declared_type_disagrees_with_its_values() {
        let raw = dump(
            vec![RawBlockProperty {
                name: "broken".to_string(),
                ty: "bool".to_string(),
                values: vec![RawPropertyValue {
                    value: RawValue::Int(1),
                }],
            }],
            vec![],
        );
        let err = build("test", (1, 0, 0), raw).unwrap_err();
        assert!(
            matches!(err, GeneratorError::PropertyValueTypeMismatch { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn rejects_an_unrecognized_property_type() {
        let raw = dump(
            vec![RawBlockProperty {
                name: "weird".to_string(),
                ty: "float".to_string(),
                values: vec![],
            }],
            vec![],
        );
        let err = build("test", (1, 0, 0), raw).unwrap_err();
        assert!(
            matches!(err, GeneratorError::UnknownPropertyType { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn rejects_a_duplicate_block_name() {
        let raw = dump(
            vec![],
            vec![item("minecraft:stone", &[]), item("minecraft:stone", &[])],
        );
        let err = build("test", (1, 0, 0), raw).unwrap_err();
        assert!(
            matches!(err, GeneratorError::DuplicateBlock { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn rejects_a_block_that_lists_the_same_property_twice() {
        let raw = dump(
            vec![bool_prop("p")],
            vec![item("minecraft:stone", &["p", "p"])],
        );
        let err = build("test", (1, 0, 0), raw).unwrap_err();
        assert!(
            matches!(err, GeneratorError::DuplicatePropertyOnBlock { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn int_and_string_value_domains_are_sorted_and_deduplicated() {
        let raw = dump(
            vec![RawBlockProperty {
                name: "age".to_string(),
                ty: "int".to_string(),
                values: vec![
                    RawPropertyValue {
                        value: RawValue::Int(3),
                    },
                    RawPropertyValue {
                        value: RawValue::Int(1),
                    },
                    RawPropertyValue {
                        value: RawValue::Int(3),
                    },
                ],
            }],
            vec![],
        );
        let palette = build("test", (1, 0, 0), raw).unwrap();
        assert_eq!(palette.properties[0].kind, PropertyKind::Int(vec![1, 3]));
    }
}
