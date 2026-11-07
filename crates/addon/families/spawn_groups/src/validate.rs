//! Family rules on the loaded documents: the empty spawn group, and the Molang outcome of the
//! `tags` members.

use super::SpawnGroup;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::versioning::ValidateCx;
use bedrock_addon_shared_spawn::legacy::GROUPS_KEY;
use bedrock_addon_shared_spawn::read::finding;
use bedrock_addon_shared_spawn::validate::{molang, path};

/// A spawn group without conditions loads and is never registered (vanilla is silent about it, so
/// the crate reports an `EmptyDocument` information).
pub(crate) fn group(group: &mut SpawnGroup, cx: &mut ValidateCx<'_>) {
    if group.conditions.as_ref().is_none_or(Vec::is_empty) {
        finding(
            DiagnosticCode::from(&core_codes::EMPTY_DOCUMENT),
            Severity::Info,
            &path(&cx.at, &[GROUPS_KEY, "conditions"]),
            cx.diagnostics,
        );
    }
    molang(group, cx);
}
