use bedrock_level::serde_helpers::deserialize_bool;

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct EndGateway {
    pub age: i32,
    pub exit_portal: [i32; 3],
    #[serde(deserialize_with = "deserialize_bool")]
    pub end_gateway_bad_pos_checked: bool,
}
