use facet::Facet;


#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct SignText {
    pub hide_glow_outline: bool,
    pub ignore_lighting: bool,
    pub persist_formatting: bool,
    pub sign_text_color: i32,
    pub filtered_text: String,
    pub text: String,
    pub text_owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Sign {
    pub back_text: SignText,
    pub front_text: SignText,
    pub is_waxed: bool,
}
