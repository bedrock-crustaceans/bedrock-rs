use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use std::io::Cursor;

#[derive(Clone, Debug)]
pub enum CommandOriginType {
    Player = 0,
    CommandBlock = 1,
    MinecartCommandBlock = 2,
    DevConsole = 3,
    Test = 4,
    AutomationPlayer = 5,
    ClientAutomation = 6,
    DedicatedServer = 7,
    Entity = 8,
    Virtual = 9,
    GameArgument = 10,
    EntityServer = 11,
    Precompiled = 12,
    GameDirectorEntityServer = 13,
    Scripting = 14,
    ExecuteContext = 15,
}

impl ProtoCodec for CommandOriginType {
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

impl TryFrom<String> for CommandOriginType {
    type Error = ProtoCodecError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "player" => Ok(CommandOriginType::Player),
            "commandblock" => Ok(CommandOriginType::CommandBlock),
            "minecartcommandblock" => Ok(CommandOriginType::MinecartCommandBlock),
            "devconsole" => Ok(CommandOriginType::DevConsole),
            "test" => Ok(CommandOriginType::Test),
            "automationplayer" => Ok(CommandOriginType::AutomationPlayer),
            "clientautomation" => Ok(CommandOriginType::ClientAutomation),
            "dedicatedserver" => Ok(CommandOriginType::DedicatedServer),
            "entity" => Ok(CommandOriginType::Entity),
            "virtual" => Ok(CommandOriginType::Virtual),
            "gameargument" => Ok(CommandOriginType::GameArgument),
            "entityserver" => Ok(CommandOriginType::EntityServer),
            "precompiled" => Ok(CommandOriginType::Precompiled),
            "gamedirectorentityserver" => Ok(CommandOriginType::GameDirectorEntityServer),
            "scripting" => Ok(CommandOriginType::Scripting),
            "executecontext" => Ok(CommandOriginType::ExecuteContext),
            invalid => Err(ProtoCodecError::InvalidEnumID(
                invalid.to_string(),
                "CommandOriginType",
            )),
        }
    }
}

impl From<CommandOriginType> for String {
    fn from(value: CommandOriginType) -> Self {
        match value {
            CommandOriginType::Player => "player",
            CommandOriginType::CommandBlock => "commandblock",
            CommandOriginType::MinecartCommandBlock => "minecartcommandblock",
            CommandOriginType::DevConsole => "devconsole",
            CommandOriginType::Test => "test",
            CommandOriginType::AutomationPlayer => "automationplayer",
            CommandOriginType::ClientAutomation => "clientautomation",
            CommandOriginType::DedicatedServer => "dedicatedserver",
            CommandOriginType::Entity => "entity",
            CommandOriginType::Virtual => "virtual",
            CommandOriginType::GameArgument => "gameargument",
            CommandOriginType::EntityServer => "entityserver",
            CommandOriginType::Precompiled => "precompiled",
            CommandOriginType::GameDirectorEntityServer => "gamedirectorentityserver",
            CommandOriginType::Scripting => "scripting",
            CommandOriginType::ExecuteContext => "executecontext",
        }
        .to_string()
    }
}
