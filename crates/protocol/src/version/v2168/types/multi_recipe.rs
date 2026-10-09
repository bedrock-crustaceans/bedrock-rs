use bedrock_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MultiRecipe {
    pub multi_recipe_id: Uuid,
    #[endianness(var)]
    pub network_id: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_protocol_core::ProtoCodec;
    use std::io::Cursor;

    #[test]
    fn multi_recipe_network_id_is_an_unsigned_varint() {
        let mut bytes = [0u8; 17];
        bytes[16] = 0x01;
        let recipe = MultiRecipe::deserialize(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(recipe.network_id, 1);
        let mut encoded = Vec::new();
        recipe.serialize(&mut encoded).unwrap();
        assert_eq!(encoded, bytes);
    }
}
