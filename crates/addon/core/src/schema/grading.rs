//! [`grade`]: what a member that fails to read or to check does under the slice's
//! [`MemberFailure`], the one grading both [`SchemaCx`](super::SchemaCx) and the reader kit
//! ([`json::kit::Read`](crate::json::kit::Read)) apply.

use crate::diagnostic::Severity;
use crate::versioning::MemberFailure;

/// What a failed member does under one [`MemberFailure`]: the severity of its finding, whether
/// what holds it has failed, and whether the member takes its default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Grading {
    /// The severity of the finding.
    pub severity: Severity,
    /// The owner has failed: the document under `Refuse`, the object holding the member under
    /// `DropOwner`, the array element under `DropElement`. The caller drops what the rule
    /// drops; the member keeps its value.
    pub fails_owner: bool,
    /// The member takes its default and the owner stands.
    pub resets: bool,
}

/// The grading of a failed member under `failure`:
///
/// | `failure` | severity | owner fails | member resets |
/// |---|---|---|---|
/// | `Refuse` | `Error` | yes | no |
/// | `DropOwner` | `Error` | yes | no |
/// | `DropElement` | `Error` | yes | no |
/// | `DefaultAndTell` | `Info` | no | yes |
///
/// `DefaultAndTell` tells at `Info`, as today's lenient readers do: the game writes no line
/// for a member it reads past.
pub const fn grade(failure: MemberFailure) -> Grading {
    match failure {
        MemberFailure::Refuse | MemberFailure::DropOwner | MemberFailure::DropElement => Grading {
            severity: Severity::Error,
            fails_owner: true,
            resets: false,
        },
        MemberFailure::DefaultAndTell => Grading {
            severity: Severity::Info,
            fails_owner: false,
            resets: true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rule_but_default_and_tell_fails_the_owner_with_an_error() {
        for failure in [
            MemberFailure::Refuse,
            MemberFailure::DropOwner,
            MemberFailure::DropElement,
        ] {
            assert_eq!(
                grade(failure),
                Grading {
                    severity: Severity::Error,
                    fails_owner: true,
                    resets: false
                },
                "{failure:?}"
            );
        }
        assert_eq!(
            grade(MemberFailure::DefaultAndTell),
            Grading {
                severity: Severity::Info,
                fails_owner: false,
                resets: true
            }
        );
    }
}
