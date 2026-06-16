use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub name: Option<Box<str>>,
    pub meta: Meta,
    #[serde(default)]
    pub packets: Box<[Change]>,
    #[serde(default)]
    pub types: Box<[Change]>,
    #[serde(default)]
    pub enums: Box<[Change]>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub protocol: u32,
    pub branch: Box<str>,
    pub version: Box<str>,
    pub raknet: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Change {
    Added(Box<str>),
    Removed(Box<str>),
    Modified(Box<str>),
}

#[cfg(test)]
mod tests {
    use crate::ProtocolDefs;
    use crate::schemas::version::Version;
    use ron::from_str;

    #[test]
    fn unknown_version() {
        let file = ProtocolDefs::get("unknown/version.ron").unwrap();
        let str = str::from_utf8(&file.data).unwrap();

        let version = from_str::<Version>(str).unwrap();

        println!("{:?}", version);
    }
}
