//! Reporting while a merged `sounds.json` is read: findings at the JSON path the reader is at,
//! the members no reader names, and the two scalars the reader takes by `asString` and `asInt`
//! with the warning when the member has another JSON type.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Phase, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::jsoncpp::fork::{self, Stored};

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

    /// Steps back out of the last member or element.
    pub fn leave(&mut self) {
        self.path.pop();
    }

    /// A finding at the current path.
    pub fn here(&mut self, code: Code, severity: Severity) {
        self.diagnostics.push(
            Diagnostic::error(code)
                .severity(severity)
                .at(self.path.clone())
                .phase(Phase::Validate),
        );
    }

    /// A content-log error of vanilla's after which the file loads: a kept error.
    pub fn kept(&mut self, code: Code) {
        self.diagnostics.push_kept_error(
            Diagnostic::error(code)
                .at(self.path.clone())
                .phase(Phase::Validate),
        );
    }

    /// A finding at the member `key` of the current path.
    pub fn at(&mut self, key: &str, code: Code, severity: Severity) {
        self.enter(key);
        self.here(code, severity);
        self.leave();
    }
}

/// Reports the members of the object `value` that no reader names, as an information, unless
/// one is of `ignored`: the members vanilla's own files carry and its parser never reads.
pub fn check_unknown(value: &Value, known: &[&str], ignored: &[&str], report: &mut Reporter<'_>) {
    let object = value.as_object().into_iter().flatten();
    for key in object.map(|(key, _)| key.as_str()) {
        if !known.contains(&key) && !ignored.contains(&key) {
            // The crate's own information: the game's parsers read the members they know and
            // ignore the rest.
            let code = Code::from(&core_codes::UNRECOGNIZED_FIELD);
            report.at(key, code, Severity::Info);
        }
    }
}

/// An integer member read with `asInt(default)`; any other type but `null` reads by the
/// reader's rule with a warning.
pub fn integer(value: &Value, key: &str, default: i32, report: &mut Reporter<'_>) -> i32 {
    let member = fork::member(value, key);
    if !matches!(fork::stored(member), Stored::Null | Stored::Integer) {
        report.at(key, Code::from(&core_codes::INVALID_VALUE), Severity::Info);
    }
    fork::as_int(member, default)
}

/// A string member read with `asString()`; a number, bool, array or object reads by the
/// reader's rule with a warning.
pub fn string(value: &Value, key: &str, report: &mut Reporter<'_>) -> String {
    let member = fork::member(value, key);
    if !matches!(fork::stored(member), Stored::Null | Stored::String) {
        report.at(key, Code::from(&core_codes::INVALID_VALUE), Severity::Info);
    }
    fork::to_text(member)
}
#[cfg(test)]
mod tests {
    #[test]
    fn public_types_implement_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<super::Reporter<'static>>();
    }
}
