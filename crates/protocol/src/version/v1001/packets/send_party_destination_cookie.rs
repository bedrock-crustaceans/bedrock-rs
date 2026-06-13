use bedrock_macros::{ProtoCodec, packet};
use strum_macros::{Display, EnumString};

#[packet(id = 349)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SendPartyDestinationCookiePacket {
    pub cookie: String,
    #[str]
    pub intent: PartyDestinationCookieIntent,
    pub destination_name: String,
}

#[derive(Clone, Debug, EnumString, Display)]
pub enum PartyDestinationCookieIntent {
    #[strum(serialize = "notify")]
    Notify,
    #[strum(serialize = "optin")]
    OptIn,
    #[strum(serialize = "optout")]
    OptOut,
}
