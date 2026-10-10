use crate::motd::BedrockMOTD;
use nethernet::prelude::{IdentityError, ServerData, ServerIdentity};
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

pub struct IdentityStore {
    pem: String,
    issued: Instant,
}

impl IdentityStore {
    pub fn generate() -> Result<Self, IdentityError> {
        let pem = ServerIdentity::generate("", SystemTime::now())?.to_pem()?;
        Ok(Self::new(pem))
    }

    pub fn from_pem(pem: impl Into<String>) -> Result<Self, IdentityError> {
        let pem = pem.into();
        ServerIdentity::from_pem(&pem, "", SystemTime::now())?;
        Ok(Self::new(pem))
    }

    pub fn load_or_generate(path: impl AsRef<Path>) -> Result<Self, IdentityError> {
        let path = path.as_ref();
        if let Ok(pem) = std::fs::read_to_string(path) {
            match Self::from_pem(pem) {
                Ok(store) => return Ok(store),
                Err(error) => {
                    tracing::warn!("ignoring unreadable {}: {error}", path.display())
                }
            }
        }

        let store = Self::generate()?;
        if let Err(error) = std::fs::write(path, &store.pem) {
            tracing::warn!(
                "failed to save {}, clients will be asked to trust a new key next start: {error}",
                path.display()
            );
        }
        Ok(store)
    }

    pub fn pem(&self) -> &str {
        &self.pem
    }

    pub fn identity(&self) -> Result<ServerIdentity, IdentityError> {
        ServerIdentity::from_pem(&self.pem, "", SystemTime::now())
    }

    pub fn renew(&mut self, every: Duration) -> Option<Result<ServerIdentity, IdentityError>> {
        if self.issued.elapsed() < every {
            return None;
        }

        let renewed = self.identity();
        if renewed.is_ok() {
            self.issued = Instant::now();
        }
        Some(renewed)
    }

    fn new(pem: String) -> Self {
        Self {
            pem,
            issued: Instant::now(),
        }
    }
}

impl From<&BedrockMOTD> for ServerData {
    fn from(motd: &BedrockMOTD) -> Self {
        let mut data = ServerData::new(motd.name.clone(), motd.sub_name.clone());
        data.player_count = motd.player_count;
        data.max_player_count = motd.player_max;
        data.protocol_version = motd.protocol;
        data.game_version = motd.version.clone();
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_path() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nethernet.pem");
        (dir, path)
    }

    #[test]
    fn missing_file_generates_and_saves_an_identity() {
        let (_dir, path) = identity_path();

        let store = IdentityStore::load_or_generate(&path).unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), store.pem());
    }

    #[test]
    fn saved_identity_loads_back_unchanged() {
        let (_dir, path) = identity_path();
        let first = IdentityStore::load_or_generate(&path).unwrap();

        let second = IdentityStore::load_or_generate(&path).unwrap();

        assert_eq!(first.pem(), second.pem());
    }

    #[test]
    fn unreadable_file_is_replaced() {
        let (_dir, path) = identity_path();
        std::fs::write(&path, "not a key").unwrap();

        let store = IdentityStore::load_or_generate(&path).unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), store.pem());
        assert!(IdentityStore::from_pem(store.pem()).is_ok());
    }

    #[test]
    fn renew_waits_for_the_interval() {
        let mut store = IdentityStore::generate().unwrap();

        assert!(store.renew(Duration::from_secs(3600)).is_none());
        assert!(store.renew(Duration::ZERO).unwrap().is_ok());
    }

    #[test]
    fn server_data_mirrors_the_motd() {
        let motd = BedrockMOTD {
            edition: "MCPE".into(),
            name: "Name".into(),
            protocol: 975,
            version: "1.26.0".into(),
            player_count: 3,
            player_max: 20,
            guid: 1,
            sub_name: "World".into(),
            game_mode: "Survival".into(),
            nintendo_limited: None,
            port_v4: None,
            port_v6: None,
        };

        let data = ServerData::from(&motd);

        assert_eq!(
            (data.server_name.as_str(), data.level_name.as_str()),
            ("Name", "World")
        );
        assert_eq!((data.player_count, data.max_player_count), (3, 20));
        assert_eq!(
            (data.protocol_version, data.game_version.as_str()),
            (975, "1.26.0")
        );
    }
}
