//! [`Dialect`]: the rules one of the game's readers of a document reads it by, and
//! [`Dialects`]: the readers of a document outside the version chain, chosen by its format
//! version.
//!
//! A family in the version chain reads each slice by the slice's [`Reading`]
//! (`Family::reading`), and its wording is the family's. A document the game reads with more
//! than one parser, picked by its `format_version` (the pack manifest: a legacy parser for
//! formats 0 to 2, the comprehensive document for 3), has more to switch than a reading: the
//! codes of a wrong type and of a missing member, the unknown members' severity and what is kept,
//! the `null` rule, and the shape of the read. A dialect says all of it once; the dialects say
//! which format reads by which.

use crate::diagnostic::Diagnostics;
use crate::version::FormatVersion;
use crate::versioning::Reading;

use super::{Profile, Read, Texts};

/// The rules one of the game's readers of a document reads it by: the [`Reading`] (the `null`
/// rule, the scalars, the unknown members' severity, whether they are kept, what a failed member
/// does), the [`Profile`] (the shape of the read) and the [`Texts`] (the codes it words the kit's
/// findings with).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Dialect {
    /// The slice rules.
    pub reading: Reading,
    /// The shape of the read.
    pub profile: Profile,
    /// The words of the kit's findings.
    pub texts: Texts,
}

impl Dialect {
    /// A read of a document in this dialect, telling into `findings`.
    #[must_use]
    pub fn read<'d>(&self, findings: &'d mut Diagnostics) -> Read<'d> {
        Read::new(self.reading, findings)
            .profile(self.profile)
            .texts(self.texts)
    }
}

/// The dialects of a document, each with the format version it reads from, oldest first: the
/// game's parsers of the document and the formats it hands each.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Dialects(pub &'static [(FormatVersion, Dialect)]);

impl Dialects {
    /// The dialect that reads a document written at `format`: the last one whose version is at
    /// or below it. `None` below the first.
    #[must_use]
    pub fn of(&self, format: FormatVersion) -> Option<&'static Dialect> {
        self.0
            .iter()
            .rev()
            .find(|(since, _)| *since <= format)
            .map(|(_, dialect)| dialect)
    }

    /// A read of a document written at `format` in its dialect ([`of`](Self::of)), knowing the
    /// format ([`Read::format`]: a member read from a later format is an unknown one), telling
    /// into `findings`. `None` below the first dialect.
    #[must_use]
    pub fn read<'d>(
        &self,
        format: FormatVersion,
        findings: &'d mut Diagnostics,
    ) -> Option<Read<'d>> {
        Some(self.of(format)?.read(findings).format(Some(format)))
    }
}

#[cfg(test)]
mod tests {
    use facet::Facet;

    use super::*;
    use crate::diagnostic::JsonPath;
    use crate::diagnostic::codes as core_codes;
    use crate::diagnostic::{Code, Severity};
    use crate::json::kit::{Node, WrongAt};
    use crate::schema::{Schema, read_document};
    use crate::versioning::NullHandling;

    #[derive(Debug, PartialEq, Facet, Schema)]
    struct Pack {
        name: String,
        size: Option<u8>,
    }

    fn wrong_type_property(_: &JsonPath) -> Code {
        Code::from(&core_codes::WRONG_TYPE_PROPERTY)
    }

    /// A legacy parser from 0.0.0 (a `null` absent, unknown members an information, the
    /// report's codes) and a strict document from 3.0.0.
    const DIALECTS: Dialects = Dialects(&[
        (
            FormatVersion::new(0, 0, 0),
            Dialect {
                reading: Reading::OWN
                    .with_null(NullHandling::AbsentSilently)
                    .with_unknown(Severity::Info),
                profile: Profile::DECLARED_ORDER,
                texts: Texts {
                    wrong_at: Some(WrongAt(wrong_type_property)),
                    missing_code: Some(&core_codes::MISSING_PROPERTY),
                    ..Texts::CODES
                },
            },
        ),
        (
            FormatVersion::new(3, 0, 0),
            Dialect {
                reading: Reading::DOCUMENT,
                profile: Profile::DECLARED_ORDER,
                texts: Texts::CODES,
            },
        ),
    ]);

    /// What a document at `format` tells: `severity path code` per finding.
    fn told(format: FormatVersion, text: &str) -> Vec<String> {
        let value = crate::json::from_str_jsonc(text).expect("JSON");
        let mut found = Diagnostics::default();
        let mut read = DIALECTS.read(format, &mut found).expect("a dialect");
        assert_eq!(read.known_format(), Some(format));
        let _ = read_document::<Pack>(&Node::new(&value, JsonPath::new()), &mut read);
        drop(read);
        found
            .iter()
            .map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code.id()))
            .collect()
    }

    #[test]
    fn a_document_is_read_in_the_dialect_of_its_format_version() {
        let legacy = FormatVersion::new(2, 0, 0);
        let comprehensive = FormatVersion::new(3, 0, 0);
        let missing = r#"{"size": null, "x": 1}"#;
        assert_eq!(
            told(legacy, missing),
            [
                "Info x core/unrecognized_field",
                "Error name core/missing_property"
            ]
        );
        assert_eq!(
            told(comprehensive, missing),
            [
                "Error x core/unrecognized_field",
                "Error name core/missing_required_field",
                "Error size core/invalid_type",
            ]
        );
        let wrong = r#"{"name": 5}"#;
        assert_eq!(told(legacy, wrong), ["Error name core/wrong_type_property"]);
        assert_eq!(told(comprehensive, wrong), ["Error name core/invalid_type"]);
    }

    #[test]
    fn the_dialect_of_a_format_is_the_last_from_at_or_below_it() {
        let of = |major, minor| DIALECTS.of(FormatVersion::new(major, minor, 0));
        assert_eq!(of(2, 9), Some(&DIALECTS.0[0].1));
        assert_eq!(of(3, 0), Some(&DIALECTS.0[1].1));
        assert_eq!(of(4, 1), Some(&DIALECTS.0[1].1));
        assert_eq!(
            Dialects(&DIALECTS.0[1..]).of(FormatVersion::new(2, 0, 0)),
            None
        );
    }

    #[test]
    fn a_dialect_s_read_is_its_reading_profile_and_texts() {
        let (_, dialect) = DIALECTS.0[0];
        let mut found = Diagnostics::default();
        let read = dialect.read(&mut found);
        assert_eq!(read.reading(), dialect.reading);
        assert_eq!(read.telling(), dialect.profile);
        assert_eq!(read.unknown_grade(), Severity::Info);
    }
}
