use super::super::types::{NetworkItemInstanceDescriptor, RecipeIngredient, RecipeUnlockingRequirement};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecVAR};
use std::io::Cursor;
use std::mem::size_of;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ShapedRecipe {
    pub recipe_unique_id: String,
    pub ingredient_grid: Vec<Vec<RecipeIngredient>>,
    pub production_list: Vec<NetworkItemInstanceDescriptor>,
    pub recipe_id: Uuid,
    pub recipe_tag: String,
    pub priority: i32,
    pub assume_symmetry: bool,
    pub unlocking_requirement: RecipeUnlockingRequirement
}

impl ProtoCodec for ShapedRecipe {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        self.recipe_unique_id.proto_serialize(stream)?;

        let x_len: u32 = self.ingredient_grid.len().try_into()?;
        let y_len: u32 = self.ingredient_grid[0].len().try_into()?;
        <u32 as ProtoCodecVAR>::proto_serialize(&x_len, stream)?;
        <u32 as ProtoCodecVAR>::proto_serialize(&y_len, stream)?;
        for y in &self.ingredient_grid {
            for recipe in y {
                recipe.proto_serialize(stream)?;
            }
        }

        <u32 as ProtoCodecVAR>::proto_serialize(&self.production_list.len().try_into()?, stream)?;
        for p in &self.production_list {
            p.proto_serialize(stream)?;
        }

        self.recipe_id.proto_serialize(stream)?;
        self.recipe_tag.proto_serialize(stream)?;
        <i32 as ProtoCodecVAR>::proto_serialize(&self.priority, stream)?;
        self.assume_symmetry.proto_serialize(stream)?;
        self.unlocking_requirement.proto_serialize(stream)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let recipe_unique_id = String::proto_deserialize(stream)?;

        let ingredient_grid = {
            let x_len = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
            let y_len = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
            let mut x_vec = Vec::with_capacity(x_len.try_into()?);
            for _ in 0..x_len {
                let mut y_vec = Vec::with_capacity(y_len.try_into()?);
                for _ in 0..y_len {
                    y_vec.push(RecipeIngredient::proto_deserialize(stream)?);
                }
                x_vec.push(y_vec);
            }
            x_vec
        };

        let production_list = {
            let len = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
            let mut vec = Vec::with_capacity(len.try_into()?);
            for _ in 0..len {
                vec.push(NetworkItemInstanceDescriptor::proto_deserialize(stream)?);
            }
            vec
        };

        let recipe_id = Uuid::proto_deserialize(stream)?;
        let recipe_tag = String::proto_deserialize(stream)?;
        let priority = <i32 as ProtoCodecVAR>::proto_deserialize(stream)?;
        let assume_symmetry = bool::proto_deserialize(stream)?;
        let unlocking_requirement = RecipeUnlockingRequirement::proto_deserialize(stream)?;

        Ok(Self {
            recipe_unique_id,
            ingredient_grid,
            production_list,
            recipe_id,
            recipe_tag,
            priority,
            assume_symmetry,
            unlocking_requirement,
        })
    }

    fn get_size_prediction(&self) -> usize {
        self.recipe_unique_id.get_size_prediction()
            + size_of::<u32>()
            + size_of::<u32>()
            + self
            .ingredient_grid
            .iter()
            .map(|y| y
                .iter()
                .map(|i|
                    i.get_size_prediction())
                .sum::<usize>())
            .sum::<usize>()
            + size_of::<u32>()
            + self.production_list.iter().map(|y| y.get_size_prediction()).sum::<usize>()
            + self.recipe_id.get_size_prediction()
            + self.recipe_tag.get_size_prediction()
            + self.priority.get_size_prediction()
            + size_of::<bool>()
    }
}

// VERIFY: ProtoCodec impl