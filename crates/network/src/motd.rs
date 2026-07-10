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
        let mut parts = vec![
            motd.edition.clone(),
            motd.name.clone(),
            motd.protocol.to_string(),
            motd.version.clone(),
            motd.player_count.to_string(),
            motd.player_max.to_string(),
            motd.guid.to_string(),
            motd.sub_name.clone(),
            motd.game_mode.clone(),
            if motd.nintendo_limited.unwrap_or(false) {
                "0".into()
            } else {
                "1".into()
            },
        ];

        if let Some(port_v4) = motd.port_v4 {
            parts.push(port_v4.to_string());
        }

        if let Some(port_v6) = motd.port_v6 {
            parts.push(port_v6.to_string());
        }

        let str = parts.join(";") + ";";
        str.into_bytes().into_boxed_slice()
    }
}
