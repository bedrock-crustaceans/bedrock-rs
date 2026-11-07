//! What a finding carries beyond its code: [`DiagnosticDetail`], its [`ReferenceHint`], the
//! [`NotFoundHint`] and the [`ShadowReason`].

use std::fmt;

use crate::pack_identity::PackIdVersion;
use crate::version::FormatVersion;

/// Why an import specifier likely resolved to no file: the near miss a lint found.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NotFoundHint {
    /// The specifier leaves the pack's `scripts/` root.
    EscapesRoot,
    /// `.` or `..` segments in the middle of the path.
    MidPathDots,
    /// A file matches when case is ignored.
    CaseMismatch,
    /// A directory import (`./dir` for `./dir/index.js`).
    DirectoryIndex,
    /// The file exists with `.js` appended.
    AppendedJs,
    /// A JSON file imported as a module.
    ImportedAsJson,
    /// A bare package name (`lodash`), which no bundle resolves.
    BarePackage,
    /// The specifier repeats the `scripts/` prefix.
    ScriptsPrefix,
}

impl fmt::Display for NotFoundHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EscapesRoot => "the path leaves the scripts root",
            Self::MidPathDots => "the path has `.` or `..` in the middle",
            Self::CaseMismatch => "a file matches when case is ignored",
            Self::DirectoryIndex => "directory imports are not resolved",
            Self::AppendedJs => "the file exists with `.js` appended",
            Self::ImportedAsJson => "a JSON file cannot be imported",
            Self::BarePackage => "bare package names are not resolved",
            Self::ScriptsPrefix => "the path repeats the `scripts/` prefix",
        })
    }
}

/// Why the shared identifier map of client entities and attachables ignored a document
/// (`visual::MergeStep::Ignore`): the reason of [`DiagnosticDetail::Shadow`] and of the
/// `ClientEntityShadowed` and `AttachableShadowed` codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ShadowReason {
    /// Its `min_engine_version` is above the highest pack `min_engine_version` of the stack.
    MinEngineVersionAboveStack,
    /// Its `min_engine_version` is below that of the definition already loaded.
    OlderMinEngineVersion,
}

/// An attachment of a [`Diagnostic`](super::Diagnostic): what the code alone does not say.
///
/// A [`CodeDef`](super::CodeDef) that [`was`](super::CodeDef::was) a legacy code with a typed payload (a
/// `ShadowReason`, a `PackIdVersion`, an `io::ErrorKind`, a UUID, a `NotFoundHint`, a
/// `FormatVersion`) puts that payload here: the definition's arguments are the text the line
/// says, the detail is the data a caller matches on. Those payload variants are data only and
/// add nothing to the content-log line.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum DiagnosticDetail {
    /// The Molang compiler's diagnostic behind an `InvalidMolang` / `MolangLint`: its code,
    /// parser message and span in the expression.
    #[cfg(feature = "molang")]
    #[cfg_attr(docsrs, doc(cfg(feature = "molang")))]
    Molang(molang::diag::Diagnostic),
    /// The name behind an `UnresolvedReference` / `InvalidReference`: what kind of thing it
    /// names, the name as written, and a near miss when one exists.
    Reference {
        /// The kind of thing the name refers to (`"geometry"`, `"material"`, …).
        kind: &'static str,
        /// The name as written.
        id: String,
        /// A near miss, when one exists.
        hint: Option<ReferenceHint>,
    },
    /// The paths a file lookup tried and found in no pack, behind a `FontFileMissing`: what
    /// kind of file was looked for and each path, in the order the game tries them.
    TriedPaths {
        /// The kind of file looked for (`"font file"`).
        kind: &'static str,
        /// Each path tried, in the game's order.
        paths: Vec<String>,
    },
    /// What a finding of this crate says beyond its code, in fixed words (the game has no
    /// text for it): which member a written document carries a value as, or why an item has
    /// no text.
    Note(&'static str),
    /// Why the shared identifier map ignored a document (`ClientEntityShadowed`'s and
    /// `AttachableShadowed`'s `reason`). Data only.
    Shadow(ShadowReason),
    /// The packs of a stack finding: the pack, and the dependency it names when it names one
    /// (`MissingDependency`, `DependencyAutoAdded`, `PackNotFound`). Data only.
    Packs {
        /// The pack the finding is about.
        pack: PackIdVersion,
        /// The dependency it names, when it names one.
        dependency: Option<PackIdVersion>,
    },
    /// What the host reported for a file it could not read (`ReadError`'s `kind`). Data only.
    Io(std::io::ErrorKind),
    /// The UUID a finding names (`UnmatchedConfigFolder`'s `expected`): a finding with none
    /// has no such detail. Data only.
    Uuid(uuid::Uuid),
    /// Why an import likely resolved to no file (`ScriptImportNotFound`'s `hint`). Data only.
    NotFound(NotFoundHint),
    /// The format version a finding names (`BiomeFormatHint`'s `version`, `FieldSinceEngine`'s
    /// `since`). Data only.
    FormatVersion(FormatVersion),
    /// The reader's own message behind a syntax finding whose line is the game's sentence
    /// (`MalformedInput`'s `message` where its `text` is given): what the parser said, which
    /// the content-log line does not print. Data only.
    Syntax(String),
}

/// The suffix a detail adds to the diagnostic's message: ` | <compiler message>` for Molang,
/// `: <kind> '<id>'` and the hint for a reference, `: <kind> '<path>, <path>'` for tried paths,
/// `: <note>` for a note; nothing for a typed payload.
impl fmt::Display for DiagnosticDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "molang")]
            Self::Molang(diagnostic) => write!(f, " | {diagnostic}"),
            Self::Note(note) => write!(f, ": {note}"),
            Self::Reference { kind, id, hint } => {
                write!(f, ": {kind} '{id}'")?;
                match hint {
                    None => Ok(()),
                    Some(ReferenceHint::CaseMismatch(target)) => {
                        write!(f, " (did you mean '{target}'?)")
                    }
                    Some(ReferenceHint::MissingNamespace) => f.write_str(" (namespace missing)"),
                    Some(ReferenceHint::Shadowed) => {
                        f.write_str(" (shadowed by a higher-priority definition)")
                    }
                }
            }
            Self::TriedPaths { kind, paths } => write!(f, ": {kind} '{}'", paths.join(", ")),
            // Data only: the code's arguments carry what the line says.
            Self::Shadow(_)
            | Self::Packs { .. }
            | Self::Io(_)
            | Self::Uuid(_)
            | Self::NotFound(_)
            | Self::FormatVersion(_)
            | Self::Syntax(_) => Ok(()),
        }
    }
}

/// A near miss for a reference that did not resolve as written.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ReferenceHint {
    /// A target exists whose name differs only in case: the target's name.
    CaseMismatch(String),
    /// The name lacks a namespace that the target has.
    MissingNamespace,
    /// The target exists but is shadowed by a higher-priority definition.
    Shadowed,
}

#[cfg(test)]
mod tests {
    use super::super::{Diagnostic, Severity, codes};
    use super::*;
    use crate::pack_identity::PackType;

    /// Each typed payload a legacy code carries, as a detail.
    fn typed_details() -> Vec<DiagnosticDetail> {
        let pack = |n: u128| {
            PackIdVersion::new(
                uuid::Uuid::from_u128(n),
                crate::version::SemVersion::new(1, 0, 0),
                PackType::Resource,
            )
        };
        vec![
            DiagnosticDetail::Shadow(ShadowReason::OlderMinEngineVersion),
            DiagnosticDetail::Packs {
                pack: pack(1),
                dependency: Some(pack(2)),
            },
            DiagnosticDetail::Packs {
                pack: pack(1),
                dependency: None,
            },
            DiagnosticDetail::Io(std::io::ErrorKind::NotFound),
            DiagnosticDetail::Uuid(uuid::Uuid::from_u128(3)),
            DiagnosticDetail::NotFound(NotFoundHint::CaseMismatch),
            DiagnosticDetail::FormatVersion(crate::version::FormatVersion::new(1, 21, 0)),
        ]
    }

    /// A typed payload rides on the finding as data: compared, cloned, and not in the line.
    #[test]
    fn a_typed_payload_is_data_on_the_finding_and_not_in_its_line() {
        for detail in typed_details() {
            let bare = Diagnostic::of(&codes::INTERNAL_ERROR).severity(Severity::Warning);
            let with = bare.clone().detail(detail.clone());
            assert_eq!(with.detail.as_deref(), Some(&detail));
            assert_eq!(with.clone(), with);
            assert_ne!(with, bare);
            assert_eq!(with.to_string(), bare.to_string(), "{detail:?}");
        }
        let details = typed_details();
        for (i, a) in details.iter().enumerate() {
            for (j, b) in details.iter().enumerate() {
                assert_eq!(a == b, i == j, "{a:?} vs {b:?}");
            }
        }
    }
}
