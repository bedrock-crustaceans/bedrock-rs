#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignText {
    pub hide_glow_outline: bool,
    pub ignore_lighting: bool,
    pub persist_formatting: bool,
    pub sign_text_color: i32,
    pub text: String,
    pub text_owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Sign {
    pub back_text: SignText,
    pub front_text: SignText,
    pub is_waxed: bool,
}
