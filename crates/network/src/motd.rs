use crate::error::MotdError;
use std::str::FromStr;

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

impl BedrockMOTD {
    pub fn parse(raw: &[u8]) -> Result<Self, MotdError> {
        std::str::from_utf8(raw)?.parse()
    }
}

impl FromStr for BedrockMOTD {
    type Err = MotdError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut fields = text.split(';');
        let mut required = |name| fields.next().ok_or(MotdError::MissingField(name));

        let edition = required("edition")?.to_owned();
        let name = required("name")?.to_owned();
        let protocol = number(required("protocol")?, "protocol")?;
        let version = required("version")?.to_owned();
        let player_count = number(required("player_count")?, "player_count")?;
        let player_max = number(required("player_max")?, "player_max")?;

        let mut optional = |name| {
            fields
                .next()
                .filter(|field| !field.is_empty())
                .map(|field| (name, field))
        };
        let guid = optional("guid").map_or(Ok(0), |(name, field)| number(field, name))?;
        let sub_name = optional("sub_name").map_or_else(String::new, |(_, field)| field.to_owned());
        let game_mode =
            optional("game_mode").map_or_else(String::new, |(_, field)| field.to_owned());
        let nintendo_limited = optional("nintendo_limited").map(|(_, field)| field == "0");
        let port_v4 = optional("port_v4")
            .map(|(name, field)| number(field, name))
            .transpose()?;
        let port_v6 = optional("port_v6")
            .map(|(name, field)| number(field, name))
            .transpose()?;

        Ok(Self {
            edition,
            name,
            protocol,
            version,
            player_count,
            player_max,
            guid,
            sub_name,
            game_mode,
            nintendo_limited,
            port_v4,
            port_v6,
        })
    }
}

fn number<T: FromStr>(field: &str, name: &'static str) -> Result<T, MotdError> {
    field.parse().map_err(|_| MotdError::InvalidNumber(name))
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

#[cfg(test)]
mod tests {
    use super::*;

    const VANILLA_PONG: &str = "MCPE;Dedicated Server;390;1.14.60;0;10;13253860892328930865;Bedrock level;Survival;1;19132;19133;";

    fn vanilla_motd() -> BedrockMOTD {
        BedrockMOTD {
            edition: "MCPE".into(),
            name: "Dedicated Server".into(),
            protocol: 390,
            version: "1.14.60".into(),
            player_count: 0,
            player_max: 10,
            guid: 13253860892328930865,
            sub_name: "Bedrock level".into(),
            game_mode: "Survival".into(),
            nintendo_limited: Some(false),
            port_v4: Some(19132),
            port_v6: Some(19133),
        }
    }

    #[test]
    fn vanilla_pong_parses_to_the_golden_motd() {
        assert_eq!(VANILLA_PONG.parse::<BedrockMOTD>().unwrap(), vanilla_motd());
    }

    #[test]
    fn parsed_motd_serialises_back_to_the_pong() {
        let bytes: Box<[u8]> = VANILLA_PONG.parse::<BedrockMOTD>().unwrap().into();
        assert_eq!(&*bytes, VANILLA_PONG.as_bytes());
    }

    #[test]
    fn short_pong_leaves_the_trailing_fields_empty() {
        let motd = BedrockMOTD::parse(b"MCPE;Old Server;100;1.0.0;3;20").unwrap();
        assert_eq!(
            (motd.name.as_str(), motd.player_count, motd.player_max),
            ("Old Server", 3, 20)
        );
        assert_eq!(
            (motd.guid, motd.port_v4, motd.nintendo_limited),
            (0, None, None)
        );
    }

    #[test]
    fn pong_missing_the_player_counts_names_the_field() {
        assert!(matches!(
            "MCPE;Server;100;1.0.0".parse::<BedrockMOTD>(),
            Err(MotdError::MissingField("player_count"))
        ));
    }

    #[test]
    fn pong_with_a_bad_number_names_the_field() {
        assert!(matches!(
            "MCPE;Server;abc;1.0.0;0;10".parse::<BedrockMOTD>(),
            Err(MotdError::InvalidNumber("protocol"))
        ));
    }
}
