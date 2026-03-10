use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use std::io::Cursor;

#[derive(Clone, Debug)]
#[repr(i8)]
pub enum CommandOutputType {
    None = 0,
    LastOutput = 1,
    Silent = 2,
    AllOutput = 3,
    DataSet = 4,
}

impl ProtoCodec for CommandOutputType {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        String::proto_serialize(&String::from(self.clone()), stream)
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        Self::try_from(String::proto_deserialize(stream)?)
    }

    fn get_size_prediction(&self) -> usize {
        String::from(self.clone()).get_size_prediction()
    }
}

impl TryFrom<String> for CommandOutputType {
    type Error = ProtoCodecError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "none" => Ok(CommandOutputType::None),
            "lastoutput" => Ok(CommandOutputType::LastOutput),
            "silent" => Ok(CommandOutputType::Silent),
            "alloutput" => Ok(CommandOutputType::AllOutput),
            "dataset" => Ok(CommandOutputType::DataSet),
            invalid => Err(ProtoCodecError::InvalidEnumID(
                invalid.to_string(),
                "CommandOutputType",
            )),
        }
    }
}

impl From<CommandOutputType> for String {
    fn from(value: CommandOutputType) -> Self {
        match value {
            CommandOutputType::None => "none",
            CommandOutputType::LastOutput => "lastoutput",
            CommandOutputType::Silent => "silent",
            CommandOutputType::AllOutput => "alloutput",
            CommandOutputType::DataSet => "dataset",
        }
        .to_string()
    }
}
