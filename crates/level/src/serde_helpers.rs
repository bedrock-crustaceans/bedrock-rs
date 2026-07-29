/// Reads an NBT byte tag as a boolean, the encoding Bedrock uses for flags: zero is
/// false, any other value is true.
#[inline]
pub fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;

    match i8::deserialize(deserializer)? {
        0 => Ok(false),
        _ => Ok(true),
    }
}
