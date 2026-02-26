use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MolangVariableMap {
    pub serialized_variable_map: String,
}