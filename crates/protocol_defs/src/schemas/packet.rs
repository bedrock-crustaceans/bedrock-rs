use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Packet {
    id: u32,
    ident: Box<str>,
    fields: Box<[Field]>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
    ident: Box<str>,
    #[serde(rename = "type")]
    ty: FieldType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FieldType {
    Bool,
    U8,
    I8,
    U16(Endianness),
    I16(Endianness),
    U32(Endianness),
    I32(Endianness),
    U64(Endianness),
    I64(Endianness),
    U128(Endianness),
    I128(Endianness),
    F32(Endianness),
    F64(Endianness),
    VarU32,
    VarI32,
    VarU64,
    VarI64,
    VarU128,
    VarI128,
    String,
    List(Box<FieldType>),
    Array(Box<FieldType>, usize),
    Option(Box<FieldType>),
    Type(Box<str>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Endianness {
    BigEndian,
    LittleEndian,
}

#[cfg(test)]
mod tests {
    use ron::from_str;
    use crate::ProtocolDefs;
    use super::*;
    
    #[test]
    fn packet() {
        let file = ProtocolDefs::get("unknown/packets/request_network_settings.ron").unwrap();
        let str = str::from_utf8(&file.data).unwrap();

        let packet = from_str::<Packet>(str).unwrap();

        println!("{:?}", packet);
    }
}