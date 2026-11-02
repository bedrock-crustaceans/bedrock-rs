//! Reporting while a merged sound document is read: findings at the JSON path the reader is
//! at, and the members no reader names.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;

/// Collects the findings of a read, each at the path the reader stands on.
#[derive(Debug)]
pub struct Reporter<'a> {
    path: JsonPath,
    diagnostics: &'a mut Diagnostics,
}

impl<'a> Reporter<'a> {
    /// A reporter at the document root.
    pub fn new(diagnostics: &'a mut Diagnostics) -> Self {
        Self {
            path: JsonPath::new(),
            diagnostics,
        }
    }

    /// Steps into the member `key`.
    pub fn enter(&mut self, key: &str) {
        self.path.push(key);
    }

    /// Steps into the element `index`.
    pub fn enter_index(&mut self, index: usize) {
        self.path.push_index(index);
    }

    /// Steps back out of the last member or element.
    pub fn leave(&mut self) {
        self.path.pop();
    }

    /// A finding at the current path.
    pub fn here(&mut self, code: DiagnosticCode, severity: Severity) {
        let location = Location::new(None, None, self.path.clone(), None);
        self.diagnostics
            .push(Diagnostic::new(code, severity, Phase::Validate, location));
    }

    /// A finding that carries one of vanilla's texts, at the current path.
    pub fn vanilla(&mut self, code: DiagnosticCode, severity: Severity) {
        self.here(code, severity);
    }

    /// A content-log error of vanilla's after which the file loads: a kept error.
    pub fn kept(&mut self, code: DiagnosticCode) {
        let location = Location::new(None, None, self.path.clone(), None);
        self.diagnostics.push_kept_error(Diagnostic::new(
            code,
            Severity::Error,
            Phase::Validate,
            location,
        ));
    }

    /// A finding at the member `key` of the current path.
    pub fn at(&mut self, key: &str, code: DiagnosticCode, severity: Severity) {
        self.enter(key);
        self.here(code, severity);
        self.leave();
    }
}

/// The members of the object `value` that no reader names: each is kept, and reported as
/// `UnrecognizedField` unless it is one of `ignored`, the members vanilla's own files carry
/// and its parser never reads. Nothing for a value that is no object.
pub fn check_unknown(
    value: &Value,
    known: &[&str],
    ignored: &[&str],
    report: &mut Reporter<'_>,
) -> Extra {
    let mut extra = Extra::default();
    let Some(object) = value.as_object() else {
        return extra;
    };
    for (key, member) in object {
        let key = key.as_str();
        if known.contains(&key) {
            continue;
        }
        if !ignored.contains(&key) {
            // The crate's own information: the game's parsers read the members they know and ignore
            // the rest.
            report.at(
                key,
                DiagnosticCode::UnrecognizedField { text: None },
                Severity::Info,
            );
        }
        extra.insert(key.to_owned(), member.clone());
    }
    extra
}
