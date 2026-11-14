use crate::compression::Compression;
use crate::error::LoginError;

/// The compression a server announces in its network settings.
#[derive(Clone, Debug, PartialEq)]
pub struct NetworkSettings {
    pub compression: Compression,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            compression: Compression::Zlib {
                threshold: 256,
                compression_level: 6,
            },
        }
    }
}

impl NetworkSettings {
    pub fn new(compression: Compression) -> Self {
        Self { compression }
    }

    /// Builds the settings from the algorithm id and threshold a network settings packet carries.
    ///
    /// # Errors
    ///
    /// Returns [`LoginError::Compression`] when `algorithm` is unknown.
    pub fn from_wire(algorithm: u16, threshold: u16) -> Result<Self, LoginError> {
        Ok(Self {
            compression: Compression::from_network_settings(algorithm, threshold)?,
        })
    }

    /// The algorithm id a network settings packet carries.
    pub fn algorithm(&self) -> u16 {
        self.compression.id_u16()
    }

    pub fn threshold(&self) -> u16 {
        self.compression.threshold()
    }
}

/// Requires the protocol a client announced to be the one the server supports.
///
/// # Errors
///
/// Returns [`LoginError::ProtocolMismatch`] when `client` differs from `server`.
pub fn check_protocol(client: i32, server: i32) -> Result<(), LoginError> {
    if client == server {
        Ok(())
    } else {
        Err(LoginError::ProtocolMismatch { client, server })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_protocols_pass() {
        assert!(check_protocol(2225, 2225).is_ok());
    }

    #[test]
    fn older_and_newer_clients_are_a_mismatch_naming_both_sides() {
        for client in [2224, 2226] {
            assert!(matches!(
                check_protocol(client, 2225),
                Err(LoginError::ProtocolMismatch { client: found, server: 2225 }) if found == client
            ));
        }
    }

    #[test]
    fn wire_values_round_trip_through_the_settings() {
        for compression in [
            Compression::Zlib {
                threshold: 256,
                compression_level: 6,
            },
            Compression::Snappy { threshold: 64 },
            Compression::None,
        ] {
            let settings = NetworkSettings::new(compression);
            let rebuilt =
                NetworkSettings::from_wire(settings.algorithm(), settings.threshold()).unwrap();
            assert_eq!(rebuilt.algorithm(), settings.algorithm());
            assert_eq!(rebuilt.threshold(), settings.threshold());
        }
    }

    #[test]
    fn unknown_algorithm_is_an_error() {
        assert!(matches!(
            NetworkSettings::from_wire(7, 1),
            Err(LoginError::Compression(_))
        ));
    }

    #[test]
    fn default_settings_compress_with_zlib() {
        assert!(matches!(
            NetworkSettings::default().compression,
            Compression::Zlib { .. }
        ));
    }
}
