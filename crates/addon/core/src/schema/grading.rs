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
    /// The owner has failed (the document under `Refuse`); the member keeps its value.
    pub fails_owner: bool,
    /// The member takes its default and the owner stands.
    pub resets: bool,
}

/// The grading of a failed member under `failure`:
///
/// | `failure` | severity | owner fails | member resets |
/// |---|---|---|---|
/// | `Refuse` | `Error` | yes | no |
/// | `DefaultAndTell` | `Info` | no | yes |
/// | `DefaultAndRefuse` | `Error` | no | yes |
///
/// `DefaultAndTell` tells at `Info`, as today's lenient readers do: the game writes no line
/// for a member it reads past.
pub const fn grade(failure: MemberFailure) -> Grading {
    match failure {
        MemberFailure::Refuse => Grading {
            severity: Severity::Error,
            fails_owner: true,
            resets: false,
        },
        MemberFailure::DefaultAndRefuse => Grading {
            severity: Severity::Error,
            fails_owner: false,
            resets: true,
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
        assert_eq!(
            grade(MemberFailure::Refuse),
            Grading {
                severity: Severity::Error,
                fails_owner: true,
                resets: false
            }
        );
        assert_eq!(
            grade(MemberFailure::DefaultAndRefuse),
            Grading {
                severity: Severity::Error,
                fails_owner: false,
                resets: true
            }
        );
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
