//! A deprecated member or enum value, one of cereal's result codes.
//!
//! The game's reader logs `member "{}" is deprecated` for a member its type marks deprecated
//! and skips it without reading the value, and logs `enum value is deprecated` for an enum
//! name its map marks deprecated. The code is in the error mask of both the strict and the
//! unstrict readers, so the load result is invalid and the document is rejected. A family
//! reports one with [`Diagnostic::deprecated_member`] or [`Diagnostic::deprecated_enum_value`]
//! and does not read the value.

use super::{Code, Diagnostic, JsonPath, Location, Phase, Severity, codes};

/// Vanilla's text for a deprecated enum value.
pub const ENUM_VALUE_DEPRECATED: &str = "enum value is deprecated";

impl Diagnostic {
    /// `member "<member>" is deprecated` at `at` (the member's path): an error, which rejects
    /// the document; the member's value is not read.
    pub fn deprecated_member(member: &str, at: JsonPath) -> Self {
        Self::new(
            codes::MEMBER_DEPRECATED.with([member]),
            Severity::Error,
            Phase::Parse,
            Location::new(None, None, at, None),
        )
    }

    /// `enum value is deprecated` at `at` (the member holding the value): an error, which
    /// rejects the document.
    pub fn deprecated_enum_value(at: JsonPath) -> Self {
        Self::new(
            Code::from(&codes::ENUM_VALUE_DEPRECATED),
            Severity::Error,
            Phase::Parse,
            Location::new(None, None, at, None),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(key: &str) -> JsonPath {
        let mut path = JsonPath::new();
        path.push(key);
        path
    }

    #[test]
    fn a_deprecated_member_is_an_error_with_cereals_text() {
        let finding = Diagnostic::deprecated_member("__DEP_fnctns", at("__DEP_fnctns"));
        assert_eq!(
            finding.severity,
            Severity::Error,
            "0x200 is in both error masks"
        );
        assert!(finding.is(&codes::MEMBER_DEPRECATED));
        assert_eq!(
            finding.code.to_string(),
            r#"member "__DEP_fnctns" is deprecated"#
        );
        assert_eq!(finding.location.json_path.to_string(), "__DEP_fnctns");
    }

    #[test]
    fn a_deprecated_enum_value_is_an_error_with_cereals_text() {
        let finding = Diagnostic::deprecated_enum_value(at("mode"));
        assert_eq!(
            finding.severity,
            Severity::Error,
            "0x200 is in both error masks"
        );
        assert_eq!(finding.code.to_string(), "enum value is deprecated");
    }
}
