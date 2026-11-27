//! [`Era`]: a range of format versions, the documents one of the game's readers reads, which a
//! field's `schema::era(ERA, ..)` keys hold in.

use crate::version::FormatVersion;

/// A range of format versions: the documents one of the game's readers reads (particle's legacy
/// parser below 1.20.80, its strict reader from it). A field's `schema::era(ERA, key = value, ..)`
/// keys hold in a document whose format is in the range, in place of the field's own: its codes
/// (`wrong_type`, `missing`, `invalid`), told at their own severity, and its Molang
/// spec (`molang`). The read asks the format it knows
/// ([`Read::known_format`](crate::json::kit::Read::known_format)), the Molang walk the one it is
/// given ([`MolangVisitor::format`](crate::molang::MolangVisitor::format)); a read or a walk that
/// knows none is in no era.
///
/// ```
/// use bedrock_addon_core::schema::Era;
/// use bedrock_addon_core::version::FormatVersion;
/// const CEREAL: FormatVersion = FormatVersion::new(1, 20, 80);
/// const LEGACY: Era = Era::below(CEREAL);
/// assert!(LEGACY.holds(Some(FormatVersion::new(1, 10, 0))));
/// assert!(!LEGACY.holds(Some(CEREAL)));
/// assert!(Era::since(CEREAL).holds(Some(CEREAL)));
/// assert!(!LEGACY.holds(None));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Era {
    /// The first version of the range, `None` for no lower end.
    pub since: Option<FormatVersion>,
    /// The first version past the range, `None` for no upper end.
    pub until: Option<FormatVersion>,
}

impl Era {
    /// The documents below `version`.
    #[must_use]
    pub const fn below(version: FormatVersion) -> Self {
        Self {
            since: None,
            until: Some(version),
        }
    }

    /// The documents at `version` and later.
    #[must_use]
    pub const fn since(version: FormatVersion) -> Self {
        Self {
            since: Some(version),
            until: None,
        }
    }

    /// The documents from `since` up to, not including, `until`.
    #[must_use]
    pub const fn between(since: FormatVersion, until: FormatVersion) -> Self {
        Self {
            since: Some(since),
            until: Some(until),
        }
    }

    /// Whether a document of `format` is in the era; one of no known format is in none.
    #[must_use]
    pub fn holds(self, format: Option<FormatVersion>) -> bool {
        format.is_some_and(|format| {
            self.since.is_none_or(|since| format >= since)
                && self.until.is_none_or(|until| format < until)
        })
    }
}

#[cfg(test)]
mod tests {
    use facet::Facet;

    use super::Era;
    use crate::diagnostic::{Class, CodeDef, Diagnostics, JsonPath, Kind};
    use crate::json::kit::{Node, Profile, Read};
    use crate::json::{JsonSlice, parse};
    use crate::schema::{self, FromJson, Schema};
    use crate::version::FormatVersion;
    use crate::versioning::{MemberFailure, NullHandling, Reading};

    const STRICT: FormatVersion = FormatVersion::new(1, 20, 80);
    const OLD: Era = Era::below(STRICT);

    static OLD_WRONG: CodeDef<0> = CodeDef::new(
        "test/old_wrong",
        Kind::InvalidValue,
        Class::KeptError,
        "not a number",
    );
    static OLD_MISSING: CodeDef<0> = CodeDef::new(
        "test/old_missing",
        Kind::MissingRequired,
        Class::KeptError,
        "required field does not exist",
    );

    /// Reads `text` as `T` in a document of `format`, the old reader's reading below
    /// [`STRICT`] (a failed member its default) and the strict one from it.
    fn read_at<T: FromJson>(text: &str, format: FormatVersion) -> (Option<T>, Vec<String>) {
        let parsed = parse::parse_slice(JsonSlice::new(text), parse::Options::default()).unwrap();
        let node = Node::parsed(&parsed, JsonPath::of_keys(["root"]));
        let reading = if OLD.holds(Some(format)) {
            Reading::OWN.with_on_member_failure(MemberFailure::DefaultAndTell)
        } else {
            Reading::DOCUMENT
        }
        .with_null(NullHandling::AsValue);
        let mut found = Diagnostics::default();
        let mut read = Read::new(reading, &mut found)
            .profile(Profile::DECLARED_ORDER)
            .format(Some(format));
        let value = T::from_json(&node, &mut read);
        read.finish();
        let told = found
            .iter()
            .map(|f| {
                let kept = if f.kept { " kept" } else { "" };
                format!(
                    "{:?}{kept} {} {}",
                    f.severity,
                    f.location.json_path,
                    f.code.id()
                )
            })
            .collect();
        (value, told)
    }

    const LEGACY: FormatVersion = FormatVersion::new(1, 10, 0);

    #[derive(Clone, Debug, PartialEq, Facet, Schema)]
    struct Spark {
        #[facet(default = 2.0, schema::era(OLD, wrong_type = OLD_WRONG, missing = OLD_MISSING))]
        radius: f32,
        #[facet(schema::era(OLD, wrong_type = OLD_WRONG))]
        rate: Option<f32>,
    }

    #[test]
    fn an_era_s_codes_are_its_reader_s_lines_and_the_field_s_own_hold_in_the_others() {
        let (spark, told) = read_at::<Spark>(r#"{"radius": "x", "rate": true}"#, LEGACY);
        assert_eq!(
            spark,
            Some(Spark {
                radius: 2.0,
                rate: None
            })
        );
        // Each line at the code's own severity: kept errors, which reject nothing.
        assert_eq!(
            told,
            [
                "Error kept root/radius test/old_wrong",
                "Error kept root/rate test/old_wrong"
            ]
        );
        let (spark, told) = read_at::<Spark>(r#"{"radius": "x", "rate": true}"#, STRICT);
        assert_eq!(spark, None);
        assert_eq!(
            told,
            [
                "Error root/radius core/invalid_type",
                "Error root/rate core/invalid_type"
            ]
        );
    }

    /// The old reader writes its line where the member is absent and goes on with the default;
    /// the strict one has a default and says nothing.
    #[test]
    fn an_era_s_missing_code_is_told_where_the_member_is_absent_and_the_default_stands() {
        let (spark, told) = read_at::<Spark>("{}", LEGACY);
        assert_eq!(
            spark,
            Some(Spark {
                radius: 2.0,
                rate: None
            })
        );
        assert_eq!(told, ["Error kept root/radius test/old_missing"]);
        let (spark, told) = read_at::<Spark>("{}", STRICT);
        assert_eq!(
            spark,
            Some(Spark {
                radius: 2.0,
                rate: None
            })
        );
        assert!(told.is_empty(), "{told:?}");
    }

    /// What the old reader makes of a value it cannot compile: one.
    const REPLACED: crate::molang::MolangFieldSpec = crate::molang::MolangFieldSpec::KEPT
        .on_failure(crate::molang::MolangFailure::ReplaceWith(1.0));

    /// A leaf that reads the spec of its member's era, for the read's half of the test.
    #[allow(clippy::unnecessary_wraps, reason = "a `schema::read_with` fn")]
    fn spec_default(node: &Node<'_>, read: &mut Read<'_>) -> Option<crate::molang::Molang> {
        let replaced = match read.member_molang(node).map(|spec| spec.on_failure) {
            Some(crate::molang::MolangFailure::ReplaceWith(x)) => x,
            _ => 0.0,
        };
        Some(crate::molang::Molang::Const(replaced))
    }

    #[derive(Clone, Debug, PartialEq, Facet, Schema)]
    struct Glow {
        #[facet(schema::era(OLD, molang = REPLACED))]
        level: Option<crate::molang::Molang>,
        #[facet(schema::read_with = spec_default, schema::era(OLD, molang = REPLACED))]
        read_level: Option<crate::molang::Molang>,
    }

    #[test]
    fn an_era_s_molang_spec_is_the_walk_s_in_a_document_of_the_era_and_the_read_s() {
        use crate::molang::{Molang, MolangFailure, MolangFieldSpec, visit_fields_at};
        let mut glow = Glow {
            level: Some(Molang::Const(0.5)),
            read_level: None,
        };
        let mut walked = |format| {
            let mut specs = Vec::new();
            visit_fields_at(&mut glow, &JsonPath::new(), format, |field| {
                specs.push((field.path.to_string(), field.spec.on_failure));
            });
            specs
        };
        let replaced = MolangFailure::ReplaceWith(1.0);
        assert_eq!(walked(Some(LEGACY)), [("level".to_owned(), replaced)]);
        let default = MolangFieldSpec::DEFAULT.on_failure;
        assert_eq!(walked(Some(STRICT)), [("level".to_owned(), default)]);
        assert_eq!(walked(None), [("level".to_owned(), default)]);
        let read = |format| {
            read_at::<Glow>(r#"{"read_level": "q.x"}"#, format)
                .0
                .and_then(|glow| glow.read_level)
        };
        assert_eq!(read(LEGACY), Some(Molang::Const(1.0)));
        assert_eq!(read(STRICT), Some(Molang::Const(0.0)));
    }

    /// A document that holds a part of another era's, walked by a hand walk of its own.
    struct Holder {
        old: Glow,
        new: Glow,
    }

    impl crate::molang::MolangFields for Holder {
        fn visit_molang(&mut self, visitor: &mut crate::molang::MolangVisitor<'_>) {
            visitor.at_format(Some(LEGACY), |visitor| visitor.nested("old", &mut self.old));
            visitor.nested("new", &mut self.new);
        }
    }

    #[test]
    fn a_walk_may_walk_a_part_of_the_document_in_another_era() {
        use crate::molang::{Molang, MolangFailure, MolangFieldSpec, visit_fields};
        let glow = || Glow {
            level: Some(Molang::Const(0.5)),
            read_level: None,
        };
        let mut holder = Holder {
            old: glow(),
            new: glow(),
        };
        let mut specs = Vec::new();
        visit_fields(&mut holder, &JsonPath::new(), |field| {
            specs.push((field.path.to_string(), field.spec.on_failure));
        });
        assert_eq!(
            specs,
            [
                ("old/level".to_owned(), MolangFailure::ReplaceWith(1.0)),
                ("new/level".to_owned(), MolangFieldSpec::DEFAULT.on_failure),
            ]
        );
    }
}
