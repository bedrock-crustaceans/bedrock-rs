#[derive(Debug, Clone, PartialEq)]
pub struct BedrockMOTD {
    pub edition: String,
    pub name: String,
    pub protocol: u32,
    pub version: String,
    pub player_count: i32,
    pub player_max: i32,
    pub guid: u64,
    pub sub_name: String,
    pub game_mode: String,
    pub nintendo_limited: Option<bool>,
    pub port_v4: Option<u16>,
    pub port_v6: Option<u16>,
}

impl From<BedrockMOTD> for Box<[u8]> {
    fn from(motd: BedrockMOTD) -> Self {
        Self::from(&motd)
    }
}

impl From<&BedrockMOTD> for Box<[u8]> {
    fn from(motd: &BedrockMOTD) -> Self {
        let mut text = format!(
            "{};{};{};{};{};{};{};{};{};{};",
            motd.edition,
            motd.name,
            motd.protocol,
            motd.version,
            motd.player_count,
            motd.player_max,
            motd.guid,
            motd.sub_name,
            motd.game_mode,
            if motd.nintendo_limited.unwrap_or(false) {
                "0"
            } else {
                "1"
            },
        );

        for port in [motd.port_v4, motd.port_v6].into_iter().flatten() {
            text.push_str(&port.to_string());
            text.push(';');
        }

        text.into_bytes().into_boxed_slice()
    }
}
