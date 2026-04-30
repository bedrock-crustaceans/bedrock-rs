#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Campfire {
    pub item_time1: i32,
    pub item_time2: i32,
    pub item_time3: i32,
    pub item_time4: i32,
}
