use bedrock_level::serde_helpers::deserialize_bool;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct SignText {
    #[serde(deserialize_with = "deserialize_bool")]
    pub hide_glow_outline: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub ignore_lighting: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub persist_formatting: bool,
    pub sign_text_color: i32,
    pub filtered_text: String,
    pub text: String,
    pub text_owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Sign {
    pub back_text: SignText,
    pub front_text: SignText,
    #[serde(deserialize_with = "deserialize_bool")]
    pub is_waxed: bool,
}
