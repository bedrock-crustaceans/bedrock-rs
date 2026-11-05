//! Format 1.21.80: the struct and the step to 1.21.89.

super::legacy_struct!(1, 21, 80);

use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, VanillaText};
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::finding;

/// 1.21.80 → 1.21.89.
impl Upgrade for Entity {
    type Next = super::v1_21_89::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
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
                    DiagnosticCode::DroppedComponent {
                        text: Some(VanillaText::NullGroupRemoved {
                            group: name.clone(),
                        }),
                    },
                    &JsonPath::new().with(&["component_groups", &name]),
                );
            }
        }
        super::v1_21_89::Entity(payload)
    }
}
