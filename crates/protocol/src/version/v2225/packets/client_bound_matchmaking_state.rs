use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 353)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundMatchmakingStatePacket {
    pub state: MatchmakingState,
    pub destination_name: String,
    pub options: Option<MatchmakingStateOptions>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct MatchmakingStateOptions {
    pub triggering_player_name: Option<String>,
    pub triggered_by_local_player: Option<bool>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum MatchmakingState {
    Idle = 0,
    Matchmaking = 1,
    MatchFound = 2,
    Canceled = 3,
    PlayerLeftParty = 4,
    PlayerLeftServer = 5,
    ServerShutdown = 6,
    TimedOut = 7,
    RequeueAsParty = 8,
}
