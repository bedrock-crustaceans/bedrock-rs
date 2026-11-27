//! Format 1.21.80: the struct and the step to 1.21.89.

use crate::codes;
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::finding;

/// An entity document as written in format 1.21.80 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 21, 80>;

/// 1.21.80 → 1.21.89.
impl LegacyStep<1, 21, 80> for Payload {
    type Next = super::v1_21_89::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // C, the legacy upgrade callbacks of 1.21.89: a `null` group is dropped. Events that
        // name it are left alone (vanilla ignores unknown group names at run time).
        if let Some(groups) = &mut payload.component_groups {
            let null: Vec<String> = groups
                .iter()
                .filter(|(_, group)| group.is_null())
                .map(|(name, _)| name.clone())
                .collect();
            for name in null {
                groups.shift_remove(&name);
                finding(
                    cx,
                    codes::NULL_GROUP_REMOVED.with([name.as_str()]),
                    &JsonPath::new().with(&["component_groups", &name]),
                );
            }
        }
        Legacy(payload)
    }
}
