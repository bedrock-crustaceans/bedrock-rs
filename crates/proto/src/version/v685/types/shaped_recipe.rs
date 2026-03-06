use crate::version::versions::ProtoVersion;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecVAR};
use std::io::{Read, Write};
use std::mem::size_of;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ShapedRecipe<V: ProtoVersion> {
    pub recipe_unique_id: String,
    pub ingredient_grid: Vec<Vec<V::RecipeIngredient>>,
    pub production_list: Vec<V::NetworkItemInstanceDescriptor>,
    pub recipe_id: Uuid,
    pub recipe_tag: String,
    pub priority: i32,
    pub assume_symmetry: bool,
    pub unlocking_requirement: V::RecipeUnlockingRequirement,
    pub network_id: u32,
}

impl<V: ProtoVersion> ProtoCodec for ShapedRecipe<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.recipe_unique_id.serialize(stream)?;

        let x_len: u32 = self.ingredient_grid.len().try_into()?;
        let y_len: u32 = self.ingredient_grid[0].len().try_into()?;
        <u32 as ProtoCodecVAR>::serialize(&x_len, stream)?;
        <u32 as ProtoCodecVAR>::serialize(&y_len, stream)?;
        for y in &self.ingredient_grid {
            for recipe in y {
                recipe.serialize(stream)?;
            }
        }

        <u32 as ProtoCodecVAR>::serialize(&self.production_list.len().try_into()?, stream)?;
        for p in &self.production_list {
            p.serialize(stream)?;
        }

        self.recipe_id.serialize(stream)?;
        self.recipe_tag.serialize(stream)?;
        <i32 as ProtoCodecVAR>::serialize(&self.priority, stream)?;
        self.assume_symmetry.serialize(stream)?;
        self.unlocking_requirement.serialize(stream)?;
        <u32 as ProtoCodecVAR>::serialize(&self.network_id, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let recipe_unique_id = String::deserialize(stream)?;

        let ingredient_grid = {
            let x_len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
            let y_len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
            let mut x_vec = Vec::with_capacity(x_len.try_into()?);
            for _ in 0..x_len {
                let mut y_vec = Vec::with_capacity(y_len.try_into()?);
                for _ in 0..y_len {
                    y_vec.push(V::RecipeIngredient::deserialize(stream)?);
                }
                x_vec.push(y_vec);
            }
            x_vec
        };

        let production_list = {
            let len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
            let mut vec = Vec::with_capacity(len.try_into()?);
            for _ in 0..len {
                vec.push(V::NetworkItemInstanceDescriptor::deserialize(stream)?);
            }
            vec
        };

        let recipe_id = Uuid::deserialize(stream)?;
        let recipe_tag = String::deserialize(stream)?;
        let priority = <i32 as ProtoCodecVAR>::deserialize(stream)?;
        let assume_symmetry = bool::deserialize(stream)?;
        let unlocking_requirement = V::RecipeUnlockingRequirement::deserialize(stream)?;
        let network_id = <u32 as ProtoCodecVAR>::deserialize(stream)?;

        Ok(Self {
            recipe_unique_id,
            ingredient_grid,
            production_list,
            recipe_id,
            recipe_tag,
            priority,
            network_id,
            assume_symmetry,
            unlocking_requirement,
        })
    }

    fn size_hint(&self) -> usize {
        self.recipe_unique_id.size_hint()
            + size_of::<u32>()
            + size_of::<u32>()
            + self
                .ingredient_grid
                .iter()
                .map(|y| y.iter().map(|i| i.size_hint()).sum::<usize>())
                .sum::<usize>()
            + size_of::<u32>()
            + self
                .production_list
                .iter()
                .map(|y| y.size_hint())
                .sum::<usize>()
            + self.recipe_id.size_hint()
            + self.recipe_tag.size_hint()
            + self.priority.size_hint()
            + self.assume_symmetry.size_hint()
            + self.unlocking_requirement.size_hint()
            + self.network_id.size_hint()
    }
}

// TODO: verify ProtoCodec impl
