//! Version resolution (vanilla's stage 3): the requests for one native module and the one
//! version that serves them.
//!
//! This is vanilla's dependency resolver in the mode plugin contexts use, where a request is
//! promoted to the newest version that serves it: first the table of requests is built, one
//! per dependency, then the requests for each module are resolved to one version.
// The table is built request by request in manifest order, and only then is each module's
// column resolved, so a later request for a module can change which version serves an earlier
// one.

use std::collections::{HashMap, HashSet};

use super::ScriptVersion;
use bedrock_addon_core::manifest::NativeModuleVersion;

/// One request for a version of a module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Request {
    /// The version asked for.
    pub(super) version: ScriptVersion,
    /// Who asks, as vanilla prints it (`<name> - <version>`).
    pub(super) by: String,
    /// The plugin itself asks (its manifest names the module); otherwise a module does.
    pub(super) explicit: bool,
}

/// One row of a module's request table.
// Who made the request: the plugin itself or one of the modules it depends on.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Entry {
    /// The version.
    pub(super) version: ScriptVersion,
    /// Who asks.
    pub(super) by: String,
    /// The plugin itself asks.
    pub(super) explicit: bool,
    /// The version is the newest that serves itself.
    pub(super) supported: bool,
    /// No version of the catalog serves the request.
    pub(super) missing: bool,
    /// The row is the newest version serving a request, not the request.
    pub(super) promotion: bool,
}

/// What the requests for one module come to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum VersionResolution {
    /// One version serves every request.
    Resolved {
        /// The version.
        version: ScriptVersion,
        /// Who asked for it.
        by: String,
        /// The requests served by a version newer than they would get alone.
        promoted: Vec<Entry>,
    },
    /// No version does.
    Failed {
        /// Pairs of requests that no one version serves: each later request with one
        /// earlier request it conflicts with.
        conflicts: Vec<(Entry, Entry)>,
        /// The requests no version of the catalog serves.
        missing: Vec<Entry>,
    },
}

/// The version a dependency asks for: its own, or for a selector the newest version of
/// `supported` with that pre-release tag (`0.0.0-<tag>` when there is none).
pub(super) fn requested(
    version: &NativeModuleVersion,
    supported: &[ScriptVersion],
) -> ScriptVersion {
    // A selector (`alpha`, `beta`, `internal`) asks for the newest version of that channel;
    // a written version asks for exactly that version.
    let tag = match version {
        NativeModuleVersion::Version(version) => return ScriptVersion::from(version),
        NativeModuleVersion::LatestAlpha => "alpha",
        NativeModuleVersion::LatestBeta => "beta",
        NativeModuleVersion::LatestInternal => "internal",
        // `NativeModuleVersion` is `#[non_exhaustive]` in `bedrock_addon_core`: a selector this
        // reader does not know asks for the newest release.
        _ => "",
    };
    supported
        .iter()
        .filter(|candidate| candidate.pre == tag)
        .max()
        .cloned()
        .unwrap_or_else(|| ScriptVersion::with_pre(0, 0, 0, tag))
}

/// The newest version of `supported` that strictly serves `request`.
pub(super) fn newest_serving<'a>(
    request: &ScriptVersion,
    supported: &'a [ScriptVersion],
) -> Option<&'a ScriptVersion> {
    supported
        .iter()
        .filter(|candidate| candidate.satisfies(request, true))
        .max()
}

/// The request table of one module: for each request, the newest version serving it when
/// that is not the request itself (a promotion row), then the request. A request equal to an
/// earlier one of the same kind of asker adds nothing.
pub(super) fn table(requests: &[Request], supported: &[ScriptVersion]) -> Vec<Entry> {
    let mut seen = HashSet::new();
    let mut rows = Vec::new();
    for request in requests {
        if !seen.insert((&request.version, request.explicit)) {
            continue;
        }
        // The request is supported when it is the newest version that strictly serves it.
        let newest = newest_serving(&request.version, supported);
        let supported_itself = newest == Some(&request.version);
        if let (false, Some(newest)) = (supported_itself, newest) {
            rows.push(Entry {
                version: newest.clone(),
                by: request.by.clone(),
                explicit: request.explicit,
                supported: true,
                missing: false,
                promotion: true,
            });
        }
        rows.push(Entry {
            version: request.version.clone(),
            by: request.by.clone(),
            explicit: request.explicit,
            supported: supported_itself,
            // Vanilla asks whether the engine has the version, and its
            // engine has every release of a major. A catalog may list only the newest, so
            // "missing" here is that nothing serves the request: both catalogs then agree.
            missing: newest.is_none(),
            promotion: false,
        });
    }
    rows
}

/// Whether the higher row of a pair is checked strictly against the lower: it is a module's
/// request, and the table's first row of the plugin's own asks for less (or there is none).
fn strict_as_higher(row: &Entry, first_explicit: Option<&ScriptVersion>) -> bool {
    // Only a request a module made is held to the stricter rule, and only when it asks for
    // more than the plugin's own request does.
    !row.explicit && first_explicit.is_none_or(|first| *first < row.version)
}

/// The rule the conflict search is tested against.
#[cfg(test)]
/// Whether no one version serves both rows of a table whose first explicit row asks for
/// `first_explicit`: the higher row does not serve the lower.
pub(super) fn pair_conflicts(a: &Entry, b: &Entry, first_explicit: Option<&ScriptVersion>) -> bool {
    let (higher, lower) = if a.version < b.version {
        (b, a)
    } else {
        (a, b)
    };
    !higher
        .version
        .satisfies(&lower.version, strict_as_higher(higher, first_explicit))
}

/// What the earlier request rows of one major look like, as far as a later row's conflicts
/// depend on them.
#[derive(Default)]
struct MajorRows {
    /// The first row.
    first: Option<usize>,
    /// The first row whose version is not the first row's.
    other_version: Option<usize>,
    /// The first row without a pre-release tag.
    first_release: Option<usize>,
    /// The row without a tag of the newest version.
    newest_release: Option<usize>,
    /// The first row with a tag.
    first_tagged: Option<usize>,
    /// The first row with a tag whose version is not the first tagged row's.
    other_tagged: Option<usize>,
    /// The row with a tag of the lowest version.
    lowest_tagged: Option<usize>,
    /// The first row with a tag that is checked strictly as the higher of a pair.
    first_strict_tagged: Option<usize>,
}

/// The conflicts of a table: for each request row that conflicts with an earlier request
/// row, the indices of one such earlier row (the earliest this search knows) and of the row.
///
/// Vanilla checks every pair of rows. This walks the table once and keeps, per major, the
/// few rows a later one can conflict with, so a manifest with many requests costs its
/// length; [`pair_conflicts`] is the rule it must agree with.
pub(super) fn conflicts(table: &[Entry]) -> Vec<(usize, usize)> {
    let first_explicit = table
        .iter()
        .find(|row| row.explicit)
        .map(|row| &row.version);
    let mut found = Vec::new();
    // The first request row, and the first one of another major than it.
    let mut first: Option<usize> = None;
    let mut first_other_major: Option<usize> = None;
    let mut majors: HashMap<u16, MajorRows> = HashMap::new();
    for (index, row) in table.iter().enumerate() {
        if row.promotion {
            continue;
        }
        let version = &row.version;
        let strict = strict_as_higher(row, first_explicit);
        let differs = |other: usize| table[other].version != *version;
        let mut partners: [Option<usize>; 5] = [None; 5];
        // Rows of two majors are never served by one version.
        partners[0] = match first {
            Some(first) if table[first].version.major != version.major => Some(first),
            _ => first_other_major,
        };
        let major = majors.entry(version.major).or_default();
        if version.major == 0 {
            // Major 0 serves only an equal request.
            partners[1] = major
                .first
                .filter(|&first| differs(first))
                .or(major.other_version);
        } else if version.pre.is_empty() {
            // A release above a tagged row never serves it; below one, it is served unless
            // that row is strict.
            partners[1] = major.first_strict_tagged;
            partners[2] = major
                .lowest_tagged
                .filter(|&lowest| table[lowest].version < *version);
        } else {
            // A tagged row never serves a release above it, and serves one below it unless
            // the tagged row is strict.
            partners[1] = if strict {
                major.first_release
            } else {
                major
                    .newest_release
                    .filter(|&newest| table[newest].version > *version)
            };
            // Two tagged rows of different versions never serve each other.
            partners[2] = major
                .first_tagged
                .filter(|&tagged| differs(tagged))
                .or(major.other_tagged);
        }
        if let Some(partner) = partners.into_iter().flatten().min() {
            found.push((partner, index));
        }

        match first {
            None => first = Some(index),
            Some(first) => {
                if first_other_major.is_none() && table[first].version.major != version.major {
                    first_other_major = Some(index);
                }
            }
        }
        match major.first {
            None => major.first = Some(index),
            Some(first) => {
                if major.other_version.is_none() && table[first].version != *version {
                    major.other_version = Some(index);
                }
            }
        }
        if version.pre.is_empty() {
            major.first_release.get_or_insert(index);
            if major
                .newest_release
                .is_none_or(|newest| table[newest].version < *version)
            {
                major.newest_release = Some(index);
            }
        } else {
            match major.first_tagged {
                None => major.first_tagged = Some(index),
                Some(tagged) => {
                    if major.other_tagged.is_none() && table[tagged].version != *version {
                        major.other_tagged = Some(index);
                    }
                }
            }
            if major
                .lowest_tagged
                .is_none_or(|lowest| table[lowest].version > *version)
            {
                major.lowest_tagged = Some(index);
            }
            if strict {
                major.first_strict_tagged.get_or_insert(index);
            }
        }
    }
    found
}

/// Resolves the requests for one module against the versions the catalog serves for it.
pub(super) fn resolve(requests: &[Request], supported: &[ScriptVersion]) -> VersionResolution {
    let rows = table(requests, supported);
    let missing =
        |rows: &[Entry]| -> Vec<Entry> { rows.iter().filter(|row| row.missing).cloned().collect() };
    let found = conflicts(&rows);
    if !found.is_empty() {
        return VersionResolution::Failed {
            conflicts: found
                .into_iter()
                .map(|(a, b)| (rows[a].clone(), rows[b].clone()))
                .collect(),
            missing: missing(&rows),
        };
    }
    let Some(first) = rows.first() else {
        return VersionResolution::Failed {
            conflicts: Vec::new(),
            missing: Vec::new(),
        };
    };
    // A later row takes over when it serves the best version so far.
    let best = rows.iter().skip(1).fold(first, |best, row| {
        if row.version.satisfies(&best.version, false) {
            row
        } else {
            best
        }
    });
    if !best.supported {
        return VersionResolution::Failed {
            conflicts: Vec::new(),
            missing: missing(&rows),
        };
    }
    // A request is promoted when another request makes it get a newer version than the
    // newest that serves it alone (its promotion row, right before it).
    let mut promoted = Vec::new();
    let mut alone: Option<&ScriptVersion> = None;
    for row in &rows {
        if row.promotion {
            alone = Some(&row.version);
            continue;
        }
        let own = alone.take().unwrap_or(&row.version);
        if *own != best.version {
            promoted.push(row.clone());
        }
    }
    VersionResolution::Resolved {
        version: best.version.clone(),
        by: best.by.clone(),
        promoted,
    }
}
