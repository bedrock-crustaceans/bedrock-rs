use crate::ProtoVersion;
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub enum ItemStackRequestActionType<V: ProtoVersion> {
    Take {
        amount: i8,
        source: V::ItemStackRequestSlotInfo,
        destination: V::ItemStackRequestSlotInfo,
    },
    Place {
        amount: i8,
        source: V::ItemStackRequestSlotInfo,
        destination: V::ItemStackRequestSlotInfo,
    },
    Swap {
        source: V::ItemStackRequestSlotInfo,
        destination: V::ItemStackRequestSlotInfo,
    },
    Drop {
        amount: i8,
        source: V::ItemStackRequestSlotInfo,
        randomly: bool,
    },
    Destroy {
        amount: i8,
        source: V::ItemStackRequestSlotInfo,
    },
    Consume {
        amount: i8,
        source: V::ItemStackRequestSlotInfo,
    },
    Create {
        slot: i8,
    },
    ScreenLabTableCombine,
    ScreenBeaconPayment {
        primary_effect: i32,
        secondary_effect: i32,
    },
    ScreenHUDMineBlock {
        hotbar_slot: i32,
        predicted_durability: i32,
        stack_network_id: i32,
    },
    CraftRecipe {
        recipe_network_id: u32,
        number_of_requested_crafts: i8,
    },
    CraftRecipeAuto {
        recipe_network_id: u32,
        number_of_requested_crafts: i8,
        ingredients: Vec<V::RecipeIngredient>,
    },
    CraftCreative {
        creative_item_network_id: u32,
        number_of_requested_crafts: i8,
    },
    CraftRecipeOptional {
        recipe_network_id: u32,
        filtered_strings_index: i32,
    },
    CraftRepairAndDisenchant {
        recipe_network_id: u32,
        number_of_requested_crafts: i8,
        repair_cost: i32,
    },
    CraftLoom {
        pattern_id: String,
        number_of_requested_crafts: i8,
    },
    #[deprecated = "Ask Tylaing"]
    CraftNonImplemented,
    #[deprecated = "Ask Tylaing"]
    CraftResults {
        result_items: Vec<V::ItemStackRequestNetworkItemInstanceDescriptor>,
        times_crafted: i8,
    },
}

#[allow(deprecated)]
impl<V: ProtoVersion> ItemStackRequestActionType<V> {
    fn type_id(&self) -> u32 {
        match self {
            Self::Take { .. } => 0,
            Self::Place { .. } => 1,
            Self::Swap { .. } => 2,
            Self::Drop { .. } => 3,
            Self::Destroy { .. } => 4,
            Self::Consume { .. } => 5,
            Self::Create { .. } => 6,
            Self::ScreenLabTableCombine => 7,
            Self::ScreenBeaconPayment { .. } => 8,
            Self::ScreenHUDMineBlock { .. } => 9,
            Self::CraftRecipe { .. } => 10,
            Self::CraftRecipeAuto { .. } => 11,
            Self::CraftCreative { .. } => 12,
            Self::CraftRecipeOptional { .. } => 13,
            Self::CraftRepairAndDisenchant { .. } => 14,
            Self::CraftLoom { .. } => 15,
            Self::CraftNonImplemented => 16,
            Self::CraftResults { .. } => 17,
        }
    }

    fn legacy_type_id(&self) -> u8 {
        match self {
            Self::Take { .. } => 0,
            Self::Place { .. } => 1,
            Self::Swap { .. } => 2,
            Self::Drop { .. } => 3,
            Self::Destroy { .. } => 4,
            Self::Consume { .. } => 5,
            Self::Create { .. } => 6,
            Self::ScreenLabTableCombine => 9,
            Self::ScreenBeaconPayment { .. } => 10,
            Self::ScreenHUDMineBlock { .. } => 11,
            Self::CraftRecipe { .. } => 12,
            Self::CraftRecipeAuto { .. } => 13,
            Self::CraftCreative { .. } => 14,
            Self::CraftRecipeOptional { .. } => 15,
            Self::CraftRepairAndDisenchant { .. } => 16,
            Self::CraftLoom { .. } => 17,
            Self::CraftNonImplemented => 18,
            Self::CraftResults { .. } => 19,
        }
    }
}

#[allow(deprecated)]
impl<V: ProtoVersion> ProtoCodec for ItemStackRequestActionType<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <u32 as ProtoCodecVAR>::serialize(&self.type_id(), stream)?;
        <u8 as ProtoCodec>::serialize(&self.legacy_type_id(), stream)?;

        match self {
            Self::Take {
                amount,
                source,
                destination,
            }
            | Self::Place {
                amount,
                source,
                destination,
            } => {
                <i8 as ProtoCodec>::serialize(amount, stream)?;
                source.serialize(stream)?;
                destination.serialize(stream)?;
            }
            Self::Swap {
                source,
                destination,
            } => {
                source.serialize(stream)?;
                destination.serialize(stream)?;
            }
            Self::Drop {
                amount,
                source,
                randomly,
            } => {
                <i8 as ProtoCodec>::serialize(amount, stream)?;
                source.serialize(stream)?;
                <bool as ProtoCodec>::serialize(randomly, stream)?;
            }
            Self::Destroy { amount, source } | Self::Consume { amount, source } => {
                <i8 as ProtoCodec>::serialize(amount, stream)?;
                source.serialize(stream)?;
            }
            Self::Create { slot } => {
                <i8 as ProtoCodec>::serialize(slot, stream)?;
            }
            Self::ScreenLabTableCombine => {}
            Self::ScreenBeaconPayment {
                primary_effect,
                secondary_effect,
            } => {
                <i32 as ProtoCodecVAR>::serialize(primary_effect, stream)?;
                <i32 as ProtoCodecVAR>::serialize(secondary_effect, stream)?;
            }
            Self::ScreenHUDMineBlock {
                hotbar_slot,
                predicted_durability,
                stack_network_id,
            } => {
                <i32 as ProtoCodecVAR>::serialize(hotbar_slot, stream)?;
                <i32 as ProtoCodecVAR>::serialize(predicted_durability, stream)?;
                <i32 as ProtoCodecVAR>::serialize(stack_network_id, stream)?;
            }
            Self::CraftRecipe {
                recipe_network_id,
                number_of_requested_crafts,
            } => {
                <u32 as ProtoCodecVAR>::serialize(recipe_network_id, stream)?;
                <i8 as ProtoCodec>::serialize(number_of_requested_crafts, stream)?;
            }
            Self::CraftRecipeAuto {
                recipe_network_id,
                number_of_requested_crafts,
                ingredients,
            } => {
                <u32 as ProtoCodecVAR>::serialize(recipe_network_id, stream)?;
                <i8 as ProtoCodec>::serialize(number_of_requested_crafts, stream)?;
                <Vec<V::RecipeIngredient> as ProtoCodec>::serialize(ingredients, stream)?;
            }
            Self::CraftCreative {
                creative_item_network_id,
                number_of_requested_crafts,
            } => {
                <u32 as ProtoCodecVAR>::serialize(creative_item_network_id, stream)?;
                <i8 as ProtoCodec>::serialize(number_of_requested_crafts, stream)?;
            }
            Self::CraftRecipeOptional {
                recipe_network_id,
                filtered_strings_index,
            } => {
                <u32 as ProtoCodecVAR>::serialize(recipe_network_id, stream)?;
                <i32 as ProtoCodecLE>::serialize(filtered_strings_index, stream)?;
            }
            Self::CraftRepairAndDisenchant {
                recipe_network_id,
                number_of_requested_crafts,
                repair_cost,
            } => {
                <u32 as ProtoCodecVAR>::serialize(recipe_network_id, stream)?;
                <i8 as ProtoCodec>::serialize(number_of_requested_crafts, stream)?;
                <i32 as ProtoCodecVAR>::serialize(repair_cost, stream)?;
            }
            Self::CraftLoom {
                pattern_id,
                number_of_requested_crafts,
            } => {
                <String as ProtoCodec>::serialize(pattern_id, stream)?;
                <i8 as ProtoCodec>::serialize(number_of_requested_crafts, stream)?;
            }
            Self::CraftNonImplemented => {}
            Self::CraftResults {
                result_items,
                times_crafted,
            } => {
                <Vec<V::ItemStackRequestNetworkItemInstanceDescriptor> as ProtoCodec>::serialize(
                    result_items,
                    stream,
                )?;
                <i8 as ProtoCodec>::serialize(times_crafted, stream)?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let type_id = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        let _legacy_type_id = <u8 as ProtoCodec>::deserialize(stream)?;

        Ok(match type_id {
            0 => Self::Take {
                amount: <i8 as ProtoCodec>::deserialize(stream)?,
                source: <V::ItemStackRequestSlotInfo as ProtoCodec>::deserialize(stream)?,
                destination: <V::ItemStackRequestSlotInfo as ProtoCodec>::deserialize(stream)?,
            },
            1 => Self::Place {
                amount: <i8 as ProtoCodec>::deserialize(stream)?,
                source: <V::ItemStackRequestSlotInfo as ProtoCodec>::deserialize(stream)?,
                destination: <V::ItemStackRequestSlotInfo as ProtoCodec>::deserialize(stream)?,
            },
            2 => Self::Swap {
                source: <V::ItemStackRequestSlotInfo as ProtoCodec>::deserialize(stream)?,
                destination: <V::ItemStackRequestSlotInfo as ProtoCodec>::deserialize(stream)?,
            },
            3 => Self::Drop {
                amount: <i8 as ProtoCodec>::deserialize(stream)?,
                source: <V::ItemStackRequestSlotInfo as ProtoCodec>::deserialize(stream)?,
                randomly: <bool as ProtoCodec>::deserialize(stream)?,
            },
            4 => Self::Destroy {
                amount: <i8 as ProtoCodec>::deserialize(stream)?,
                source: <V::ItemStackRequestSlotInfo as ProtoCodec>::deserialize(stream)?,
            },
            5 => Self::Consume {
                amount: <i8 as ProtoCodec>::deserialize(stream)?,
                source: <V::ItemStackRequestSlotInfo as ProtoCodec>::deserialize(stream)?,
            },
            6 => Self::Create {
                slot: <i8 as ProtoCodec>::deserialize(stream)?,
            },
            7 => Self::ScreenLabTableCombine,
            8 => Self::ScreenBeaconPayment {
                primary_effect: <i32 as ProtoCodecVAR>::deserialize(stream)?,
                secondary_effect: <i32 as ProtoCodecVAR>::deserialize(stream)?,
            },
            9 => Self::ScreenHUDMineBlock {
                hotbar_slot: <i32 as ProtoCodecVAR>::deserialize(stream)?,
                predicted_durability: <i32 as ProtoCodecVAR>::deserialize(stream)?,
                stack_network_id: <i32 as ProtoCodecVAR>::deserialize(stream)?,
            },
            10 => Self::CraftRecipe {
                recipe_network_id: <u32 as ProtoCodecVAR>::deserialize(stream)?,
                number_of_requested_crafts: <i8 as ProtoCodec>::deserialize(stream)?,
            },
            11 => Self::CraftRecipeAuto {
                recipe_network_id: <u32 as ProtoCodecVAR>::deserialize(stream)?,
                number_of_requested_crafts: <i8 as ProtoCodec>::deserialize(stream)?,
                ingredients: <Vec<V::RecipeIngredient> as ProtoCodec>::deserialize(stream)?,
            },
            12 => Self::CraftCreative {
                creative_item_network_id: <u32 as ProtoCodecVAR>::deserialize(stream)?,
                number_of_requested_crafts: <i8 as ProtoCodec>::deserialize(stream)?,
            },
            13 => Self::CraftRecipeOptional {
                recipe_network_id: <u32 as ProtoCodecVAR>::deserialize(stream)?,
                filtered_strings_index: <i32 as ProtoCodecLE>::deserialize(stream)?,
            },
            14 => Self::CraftRepairAndDisenchant {
                recipe_network_id: <u32 as ProtoCodecVAR>::deserialize(stream)?,
                number_of_requested_crafts: <i8 as ProtoCodec>::deserialize(stream)?,
                repair_cost: <i32 as ProtoCodecVAR>::deserialize(stream)?,
            },
            15 => Self::CraftLoom {
                pattern_id: <String as ProtoCodec>::deserialize(stream)?,
                number_of_requested_crafts: <i8 as ProtoCodec>::deserialize(stream)?,
            },
            16 => Self::CraftNonImplemented,
            17 => Self::CraftResults {
                result_items:
                    <Vec<V::ItemStackRequestNetworkItemInstanceDescriptor> as ProtoCodec>::deserialize(
                        stream,
                    )?,
                times_crafted: <i8 as ProtoCodec>::deserialize(stream)?,
            },
            other => {
                return Err(ProtoCodecError::InvalidEnumID(
                    other.to_string(),
                    "ItemStackRequestActionType",
                ));
            }
        })
    }

    fn size_hint(&self) -> usize {
        <u32 as ProtoCodecVAR>::size_hint(&self.type_id())
            + size_of::<u8>()
            + match self {
                Self::Take {
                    amount,
                    source,
                    destination,
                }
                | Self::Place {
                    amount,
                    source,
                    destination,
                } => {
                    <i8 as ProtoCodec>::size_hint(amount)
                        + source.size_hint()
                        + destination.size_hint()
                }
                Self::Swap {
                    source,
                    destination,
                } => source.size_hint() + destination.size_hint(),
                Self::Drop {
                    amount,
                    source,
                    randomly,
                } => {
                    <i8 as ProtoCodec>::size_hint(amount)
                        + source.size_hint()
                        + <bool as ProtoCodec>::size_hint(randomly)
                }
                Self::Destroy { amount, source } | Self::Consume { amount, source } => {
                    <i8 as ProtoCodec>::size_hint(amount) + source.size_hint()
                }
                Self::Create { slot } => <i8 as ProtoCodec>::size_hint(slot),
                Self::ScreenLabTableCombine => 0,
                Self::ScreenBeaconPayment {
                    primary_effect,
                    secondary_effect,
                } => {
                    <i32 as ProtoCodecVAR>::size_hint(primary_effect)
                        + <i32 as ProtoCodecVAR>::size_hint(secondary_effect)
                }
                Self::ScreenHUDMineBlock {
                    hotbar_slot,
                    predicted_durability,
                    stack_network_id,
                } => {
                    <i32 as ProtoCodecVAR>::size_hint(hotbar_slot)
                        + <i32 as ProtoCodecVAR>::size_hint(predicted_durability)
                        + <i32 as ProtoCodecVAR>::size_hint(stack_network_id)
                }
                Self::CraftRecipe {
                    recipe_network_id,
                    number_of_requested_crafts,
                } => {
                    <u32 as ProtoCodecVAR>::size_hint(recipe_network_id)
                        + <i8 as ProtoCodec>::size_hint(number_of_requested_crafts)
                }
                Self::CraftRecipeAuto {
                    recipe_network_id,
                    number_of_requested_crafts,
                    ingredients,
                } => {
                    <u32 as ProtoCodecVAR>::size_hint(recipe_network_id)
                        + <i8 as ProtoCodec>::size_hint(number_of_requested_crafts)
                        + <Vec<V::RecipeIngredient> as ProtoCodec>::size_hint(ingredients)
                }
                Self::CraftCreative {
                    creative_item_network_id,
                    number_of_requested_crafts,
                } => {
                    <u32 as ProtoCodecVAR>::size_hint(creative_item_network_id)
                        + <i8 as ProtoCodec>::size_hint(number_of_requested_crafts)
                }
                Self::CraftRecipeOptional {
                    recipe_network_id,
                    filtered_strings_index,
                } => {
                    <u32 as ProtoCodecVAR>::size_hint(recipe_network_id)
                        + <i32 as ProtoCodecLE>::size_hint(filtered_strings_index)
                }
                Self::CraftRepairAndDisenchant {
                    recipe_network_id,
                    number_of_requested_crafts,
                    repair_cost,
                } => {
                    <u32 as ProtoCodecVAR>::size_hint(recipe_network_id)
                        + <i8 as ProtoCodec>::size_hint(number_of_requested_crafts)
                        + <i32 as ProtoCodecVAR>::size_hint(repair_cost)
                }
                Self::CraftLoom {
                    pattern_id,
                    number_of_requested_crafts,
                } => {
                    <String as ProtoCodec>::size_hint(pattern_id)
                        + <i8 as ProtoCodec>::size_hint(number_of_requested_crafts)
                }
                Self::CraftNonImplemented => 0,
                Self::CraftResults {
                    result_items,
                    times_crafted,
                } => {
                    <Vec<V::ItemStackRequestNetworkItemInstanceDescriptor> as ProtoCodec>::size_hint(
                        result_items,
                    ) + <i8 as ProtoCodec>::size_hint(times_crafted)
                }
            }
    }
}
