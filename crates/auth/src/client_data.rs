use crate::error::ClientDataError;
use base64::Engine;
use base64::alphabet;
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(default, rename_all = "PascalCase")]
pub struct ClientData {
    pub animated_image_data: Vec<SkinAnimation>,
    pub cape_data: String,
    pub cape_id: String,
    pub cape_image_height: i32,
    pub cape_image_width: i32,
    pub cape_on_classic_skin: bool,
    pub client_random_id: i64,
    pub current_input_mode: i32,
    pub default_input_mode: i32,
    pub device_model: String,
    #[serde(rename = "DeviceOS")]
    pub device_os: i32,
    pub device_id: String,
    pub game_version: String,
    pub gui_scale: i32,
    pub filter_profanity: bool,
    pub client_editor_connection_intent: i32,
    pub client_is_editor_capable: bool,
    pub language_code: String,
    pub persona_skin: bool,
    pub platform_offline_id: String,
    pub platform_online_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub platform_user_id: String,
    pub premium_skin: bool,
    pub self_signed_id: String,
    pub server_address: String,
    pub skin_animation_data: String,
    pub skin_data: String,
    #[serde(rename = "SkinGeometryData")]
    pub skin_geometry: String,
    #[serde(rename = "SkinGeometryDataEngineVersion")]
    pub skin_geometry_version: String,
    pub skin_id: String,
    pub play_fab_id: String,
    pub skin_image_height: i32,
    pub skin_image_width: i32,
    pub skin_resource_patch: String,
    #[serde(rename = "SkinColor")]
    pub skin_colour: String,
    pub arm_size: String,
    pub persona_pieces: Vec<PersonaPiece>,
    #[serde(rename = "PieceTintColors")]
    pub piece_tint_colours: Vec<PersonaPieceTintColour>,
    pub third_party_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub third_party_name_only: Option<bool>,
    #[serde(rename = "UIProfile")]
    pub ui_profile: i32,
    pub trusted_skin: bool,
    pub override_skin: bool,
    pub compatible_with_client_side_chunk_gen: bool,
    pub max_view_distance: i32,
    pub memory_tier: i32,
    pub platform_type: i32,
    pub graphics_mode: i32,
    pub party_id: String,
    #[serde(rename = "IsPartyLeader")]
    pub party_leader: bool,
    pub profile_hash: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub nonce: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(default, rename_all = "PascalCase")]
pub struct SkinAnimation {
    pub frames: f64,
    pub image: String,
    pub image_height: i32,
    pub image_width: i32,
    #[serde(rename = "Type")]
    pub animation_type: i32,
    pub animation_expression: i32,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(default, rename_all = "PascalCase")]
pub struct PersonaPiece {
    #[serde(rename = "IsDefault")]
    pub default: bool,
    pub pack_id: String,
    pub piece_id: String,
    pub piece_type: String,
    pub product_id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(default, rename_all = "PascalCase")]
pub struct PersonaPieceTintColour {
    #[serde(rename = "Colors")]
    pub colours: [String; 4],
    pub piece_type: String,
}

impl ClientData {
    pub fn validate(&self) -> Result<(), ClientDataError> {
        if !(1..=15).contains(&self.device_os) {
            return Err(ClientDataError::DeviceOs(self.device_os));
        }
        if !is_version(&self.game_version) {
            return Err(ClientDataError::GameVersion(self.game_version.clone()));
        }
        if !is_language_tag(&self.language_code.replacen('_', "-", 1)) {
            return Err(ClientDataError::LanguageCode(self.language_code.clone()));
        }
        if !is_empty_or_uuid(&self.platform_offline_id) {
            return Err(ClientDataError::PlatformOfflineId(
                self.platform_offline_id.clone(),
            ));
        }
        if !self.platform_online_id.is_empty() && !is_u64(&self.platform_online_id) {
            return Err(ClientDataError::PlatformOnlineId(
                self.platform_online_id.clone(),
            ));
        }
        if !is_empty_or_uuid(&self.self_signed_id) {
            return Err(ClientDataError::SelfSignedId(self.self_signed_id.clone()));
        }
        if !is_server_address(&self.server_address) {
            return Err(ClientDataError::ServerAddress(self.server_address.clone()));
        }
        check_image(
            "SkinData",
            &self.skin_data,
            self.skin_image_height,
            self.skin_image_width,
        )?;
        check_image(
            "CapeData",
            &self.cape_data,
            self.cape_image_height,
            self.cape_image_width,
        )?;
        if !self
            .play_fab_id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(ClientDataError::PlayFabId(self.play_fab_id.clone()));
        }
        for animation in &self.animated_image_data {
            check_image(
                "AnimatedImageData",
                &animation.image,
                animation.image_height,
                animation.image_width,
            )?;
            if !(0..=3).contains(&animation.animation_type) {
                return Err(ClientDataError::AnimationType(animation.animation_type));
            }
        }
        let geometry = decode_base64("SkinGeometryData", &self.skin_geometry)?;
        if !geometry.is_empty() {
            check_json_object("SkinGeometryData", &geometry)?;
        }
        let patch = decode_base64("SkinResourcePatch", &self.skin_resource_patch)?;
        check_json_object("SkinResourcePatch", &patch)?;
        if self.skin_id.is_empty() {
            return Err(ClientDataError::EmptySkinId);
        }
        if !(0..=2).contains(&self.ui_profile) {
            return Err(ClientDataError::UiProfile(self.ui_profile));
        }
        Ok(())
    }
}

const LENIENT_STANDARD: GeneralPurpose = GeneralPurpose::new(
    &alphabet::STANDARD,
    GeneralPurposeConfig::new().with_decode_allow_trailing_bits(true),
);

fn decode_base64(field: &'static str, data: &str) -> Result<Vec<u8>, ClientDataError> {
    LENIENT_STANDARD
        .decode(data)
        .map_err(|source| ClientDataError::Base64 { field, source })
}

fn check_image(
    field: &'static str,
    data: &str,
    height: i32,
    width: i32,
) -> Result<(), ClientDataError> {
    let expected = i64::from(height) * i64::from(width) * 4;
    let actual = decode_base64(field, data)?.len();
    if i64::try_from(actual) != Ok(expected) {
        return Err(ClientDataError::ImageSize {
            field,
            expected,
            actual,
        });
    }
    Ok(())
}

fn check_json_object(field: &'static str, json: &[u8]) -> Result<(), ClientDataError> {
    serde_json::from_slice::<Option<Map<String, Value>>>(json)
        .map(drop)
        .map_err(|_| ClientDataError::NotJsonObject { field })
}

fn is_version(version: &str) -> bool {
    !version.is_empty() && version.bytes().all(|b| b.is_ascii_digit() || b == b'.')
}

fn is_u64(digits: &str) -> bool {
    digits.bytes().all(|b| b.is_ascii_digit()) && digits.parse::<u64>().is_ok()
}

fn is_empty_or_uuid(id: &str) -> bool {
    id.is_empty() || Uuid::try_parse(id).is_ok()
}

fn is_language_tag(tag: &str) -> bool {
    let mut subtags = tag.split('-');
    let primary = subtags.next().unwrap_or_default();
    (2..=3).contains(&primary.len())
        && primary.bytes().all(|b| b.is_ascii_alphabetic())
        && subtags.all(|subtag| {
            (1..=8).contains(&subtag.len()) && subtag.bytes().all(|b| b.is_ascii_alphanumeric())
        })
}

fn is_server_address(address: &str) -> bool {
    match address.split_once("://") {
        Some((scheme, rest)) => is_nethernet_address(scheme, rest),
        None => split_host_port(address).is_some_and(|(_, port)| is_port(port)),
    }
}

fn is_nethernet_address(scheme: &str, rest: &str) -> bool {
    let Some((url, repeated_port)) = rest.rsplit_once(':') else {
        return false;
    };
    let authority = url.split(['/', '?', '#']).next().unwrap_or_default();
    let host_port = authority.rsplit_once('@').map_or(authority, |(_, hp)| hp);
    matches!(scheme, "http" | "https")
        && split_host_port(host_port)
            .is_some_and(|(host, port)| !host.is_empty() && is_port(port) && port == repeated_port)
}

fn split_host_port(address: &str) -> Option<(&str, &str)> {
    let (host, port) = match address.strip_prefix('[') {
        Some(bracketed) => bracketed.split_once("]:")?,
        None => address.rsplit_once(':')?,
    };
    (!host.contains(['[', ']'])).then_some((host, port))
}

fn is_port(port: &str) -> bool {
    port.bytes().all(|b| b.is_ascii_digit()) && port.parse::<u16>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::realistic_client_data;
    use base64::engine::general_purpose::STANDARD;
    use serde_json::json;

    fn round_trip(json: &Value) -> ClientData {
        let data: ClientData = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(
            &serde_json::to_value(&data).unwrap(),
            json,
            "encoding differs from the client's document"
        );
        data
    }

    #[test]
    fn client_data_round_trips_a_realistic_document() {
        let data = round_trip(&realistic_client_data());

        assert_eq!(data.device_os, 7);
        assert_eq!(data.device_id, "a4f365bb1e04459bbe4cb3cbf9d546e0");
        assert_eq!(data.game_version, "1.21.120");
        assert_eq!(data.language_code, "en_US");
        assert_eq!(data.server_address, "play.example.com:19132");
        assert_eq!(data.skin_id, "c18e65aa-7b21-4637-9b63-8ad63622ef01_Alex");
        assert_eq!(data.skin_colour, "#0");
        assert_eq!(data.play_fab_id, "8a7f3c2e1d5b9046");
        assert_eq!(data.ui_profile, 0);
        assert_eq!(data.third_party_name_only, Some(false));
        assert!(!data.party_leader);
        assert_eq!(data.animated_image_data[0].animation_type, 1);
        assert!(data.persona_pieces[0].default);
        assert_eq!(data.piece_tint_colours[0].colours[3], "#0");
    }

    #[test]
    fn client_data_keeps_fields_it_does_not_know() {
        let data = round_trip(&realistic_client_data());

        assert_eq!(data.extra.get("IsEditorMode"), Some(&Value::Bool(false)));
        assert_eq!(data.extra.len(), 1, "{:?}", data.extra.keys());
    }

    #[test]
    fn client_data_omits_optional_fields_the_client_left_out() {
        let mut json = realistic_client_data();
        json.as_object_mut().unwrap().remove("ThirdPartyNameOnly");

        let data = round_trip(&json);

        assert_eq!(data.third_party_name_only, None);
        assert!(data.platform_user_id.is_empty());
        assert!(data.nonce.is_empty());
    }

    fn client_data_with(field: &str, value: Value) -> ClientData {
        let mut json = realistic_client_data();
        json[field] = value;
        serde_json::from_value(json).unwrap()
    }

    fn validate_with(field: &str, value: Value) -> Result<(), ClientDataError> {
        client_data_with(field, value).validate()
    }

    #[test]
    fn realistic_client_data_is_valid() {
        let data: ClientData = serde_json::from_value(realistic_client_data()).unwrap();

        data.validate().unwrap();
    }

    #[test]
    fn device_os_outside_known_range_is_rejected() {
        for os in [0, 16] {
            let result = validate_with("DeviceOS", json!(os));

            assert!(
                matches!(result, Err(ClientDataError::DeviceOs(got)) if got == os),
                "{result:?}"
            );
        }
    }

    #[test]
    fn game_version_with_letters_is_rejected() {
        let result = validate_with("GameVersion", json!("1.21.120-beta"));

        assert!(
            matches!(result, Err(ClientDataError::GameVersion(_))),
            "{result:?}"
        );
    }

    #[test]
    fn language_code_accepts_underscored_and_multi_part_tags() {
        for code in ["en_GB", "zh_Hans-CN", "es-419", "de"] {
            let result = validate_with("LanguageCode", json!(code));

            assert!(result.is_ok(), "{code}: {result:?}");
        }
    }

    #[test]
    fn language_code_that_is_not_a_tag_is_rejected() {
        for code in ["", "english", "en_US_POSIX", "e", "en-"] {
            let result = validate_with("LanguageCode", json!(code));

            assert!(
                matches!(result, Err(ClientDataError::LanguageCode(_))),
                "{code}: {result:?}"
            );
        }
    }

    #[test]
    fn platform_ids_may_be_empty_or_well_formed() {
        let mut data = client_data_with("PlatformOfflineId", json!(""));
        data.platform_online_id = "2535400000000001".into();
        data.self_signed_id = String::new();
        data.validate().unwrap();

        data.platform_offline_id = "05601fd2-9c71-30b1-b174-5fa11b9de09f".into();
        data.validate().unwrap();
    }

    #[test]
    fn platform_offline_id_that_is_not_a_uuid_is_rejected() {
        let result = validate_with("PlatformOfflineId", json!("steve"));

        assert!(
            matches!(result, Err(ClientDataError::PlatformOfflineId(_))),
            "{result:?}"
        );
    }

    #[test]
    fn platform_online_id_that_is_not_an_integer_is_rejected() {
        for id in ["-1", "+1", "xuid"] {
            let result = validate_with("PlatformOnlineId", json!(id));

            assert!(
                matches!(result, Err(ClientDataError::PlatformOnlineId(_))),
                "{id}: {result:?}"
            );
        }
    }

    #[test]
    fn self_signed_id_that_is_not_a_uuid_is_rejected() {
        let result = validate_with("SelfSignedId", json!("not-a-uuid"));

        assert!(
            matches!(result, Err(ClientDataError::SelfSignedId(_))),
            "{result:?}"
        );
    }

    #[test]
    fn server_address_accepts_hosts_ips_and_nethernet_urls() {
        for address in [
            "127.0.0.1:19132",
            "[::1]:19132",
            "2001:db8::1:19132",
            ":19132",
            "https://example.com:7551:7551",
            "http://192.168.1.2:7551:7551",
        ] {
            let result = validate_with("ServerAddress", json!(address));

            assert!(result.is_ok(), "{address}: {result:?}");
        }
    }

    #[test]
    fn malformed_server_address_is_rejected() {
        for address in [
            "example.com",
            "example.com:port",
            "example.com:65536",
            "ftp://example.com:21:21",
            "https://example.com:7551:7552",
            "https://:7551:7551",
            "https://example.com",
        ] {
            let result = validate_with("ServerAddress", json!(address));

            assert!(
                matches!(result, Err(ClientDataError::ServerAddress(_))),
                "{address}: {result:?}"
            );
        }
    }

    #[test]
    fn skin_data_must_match_its_dimensions() {
        let result = validate_with("SkinImageHeight", json!(32));

        assert!(
            matches!(
                result,
                Err(ClientDataError::ImageSize {
                    field: "SkinData",
                    expected: 8192,
                    actual: 16384
                })
            ),
            "{result:?}"
        );
    }

    #[test]
    fn skin_data_that_is_not_base64_is_rejected() {
        let result = validate_with("SkinData", json!("not base64!"));

        assert!(
            matches!(
                result,
                Err(ClientDataError::Base64 {
                    field: "SkinData",
                    ..
                })
            ),
            "{result:?}"
        );
    }

    #[test]
    fn cape_data_must_match_its_dimensions() {
        let result = validate_with("CapeData", json!(STANDARD.encode([0u8; 4])));

        assert!(
            matches!(
                result,
                Err(ClientDataError::ImageSize {
                    field: "CapeData",
                    ..
                })
            ),
            "{result:?}"
        );
    }

    #[test]
    fn play_fab_id_that_is_not_lowercase_hex_is_rejected() {
        let result = validate_with("PlayFabId", json!("8A7F3C2E1D5B9046"));

        assert!(
            matches!(result, Err(ClientDataError::PlayFabId(_))),
            "{result:?}"
        );
    }

    #[test]
    fn animated_image_must_match_its_dimensions() {
        let mut data = client_data_with("SkinId", json!("steve"));
        data.animated_image_data[0].image_width = 64;

        let result = data.validate();

        assert!(
            matches!(
                result,
                Err(ClientDataError::ImageSize {
                    field: "AnimatedImageData",
                    ..
                })
            ),
            "{result:?}"
        );
    }

    #[test]
    fn unknown_animation_type_is_rejected() {
        let mut data = client_data_with("SkinId", json!("steve"));
        data.animated_image_data[0].animation_type = 4;

        let result = data.validate();

        assert!(
            matches!(result, Err(ClientDataError::AnimationType(4))),
            "{result:?}"
        );
    }

    #[test]
    fn skin_geometry_may_be_empty_but_not_non_object_json() {
        validate_with("SkinGeometryData", json!("")).unwrap();

        let result = validate_with("SkinGeometryData", json!(STANDARD.encode("[]")));

        assert!(
            matches!(
                result,
                Err(ClientDataError::NotJsonObject {
                    field: "SkinGeometryData"
                })
            ),
            "{result:?}"
        );
    }

    #[test]
    fn skin_resource_patch_must_be_a_json_object() {
        for patch in ["", "bm90IGpzb24="] {
            let result = validate_with("SkinResourcePatch", json!(patch));

            assert!(
                matches!(
                    result,
                    Err(ClientDataError::NotJsonObject {
                        field: "SkinResourcePatch"
                    })
                ),
                "{patch}: {result:?}"
            );
        }
    }

    #[test]
    fn empty_skin_id_is_rejected() {
        let result = validate_with("SkinId", json!(""));

        assert!(
            matches!(result, Err(ClientDataError::EmptySkinId)),
            "{result:?}"
        );
    }

    #[test]
    fn ui_profile_outside_known_range_is_rejected() {
        let result = validate_with("UIProfile", json!(3));

        assert!(
            matches!(result, Err(ClientDataError::UiProfile(3))),
            "{result:?}"
        );
    }
}
