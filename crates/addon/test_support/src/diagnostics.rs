//! Findings as text and by code: what a test prints when it fails and what it asserts.

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};

/// One finding as a line: severity, code id, JSON path and text, each after one space.
pub fn line(f: &Diagnostic) -> String {
    format!(
        "{:?} {} {} {}",
        f.severity,
        f.code.id(),
        f.location.json_path,
        f.code
    )
}

/// The findings, one line each ([`line`]).
pub fn lines(d: &Diagnostics) -> Vec<String> {
    d.iter().map(line).collect()
}

/// Requires no finding at warning level or above.
#[track_caller]
pub fn clean(d: &Diagnostics) {
    let found: Vec<_> = d.at_least(Severity::Warning).collect();
    assert!(found.is_empty(), "{}", lines(d).join("\n"));
}

/// The findings at error level.
pub fn errors(d: &Diagnostics) -> Vec<&Diagnostic> {
    d.at_least(Severity::Error).collect()
}

/// The findings whose code id is `id`.
pub fn with_code<'a>(d: &'a Diagnostics, id: &str) -> Vec<&'a Diagnostic> {
    d.iter().filter(|f| f.code.id() == id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::codes;

    /// An empty document's finding and a shadowed definition, in that order.
    fn empty_and_shadowed() -> Diagnostics {
        let mut d = Diagnostics::default();
        d.push(Diagnostic::info(&codes::EMPTY_DOCUMENT));
        d.push(Diagnostic::warn(&codes::SHADOWED_DEFINITION));
        d
    }

    /// The format the visual families' tests assert (`atomic_client_entity/rows.rs` in
    /// `bedrock_addon`): an empty document's finding has no path, so two spaces.
    #[test]
    fn lines_prints_severity_id_path_and_text() {
        let mut d = Diagnostics::default();
        d.push(Diagnostic::info(&codes::EMPTY_DOCUMENT));
        assert_eq!(lines(&d), ["Info core/empty_document  empty document"]);
    }

    #[test]
    fn with_code_keeps_one_code() {
        let d = empty_and_shadowed();
        let found = with_code(&d, "core/empty_document");
        assert_eq!(found.len(), 1, "{:?}", lines(&d));
        assert!(found[0].is(&codes::EMPTY_DOCUMENT), "{:?}", found[0]);
    }
}
