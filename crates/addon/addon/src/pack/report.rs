//! The stack report: a view over everything a stack (or a `StackSet`) has loaded.

use std::cmp::Ordering;
use std::fmt;

use bedrock_addon_core::diagnostic::{Diagnostic, Severity};
use bedrock_addon_core::pack_identity::{PackPath, PackRef};

/// A view over everything a stack (or `StackSet`) has loaded: the pack-selection findings, and
/// for each loaded collection what loading it found and the collection's own diagnostics. It
/// borrows the cached collections and does not copy diagnostics.
///
/// [`iter`](Self::iter) gives the findings in load order: the selection, then each collection
/// in accessor-list order (behaviour stack first). `Display` prints one line per finding
/// sorted by pack (stack position), path and offset.
#[derive(Clone)]
pub struct StackReport<'a> {
    /// Load order.
    findings: Vec<&'a Diagnostic>,
    /// Sorted by pack, path, offset; stable, so equal keys keep load order.
    sorted: Vec<&'a Diagnostic>,
}

impl<'a> StackReport<'a> {
    /// A report over `findings`, in load order.
    pub(crate) fn new(findings: Vec<&'a Diagnostic>) -> StackReport<'a> {
        let mut sorted = findings.clone();
        sorted.sort_by(|a, b| by_place(a, b));
        StackReport { findings, sorted }
    }
}

impl StackReport<'_> {
    /// Every finding.
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.findings.iter().copied()
    }

    /// Findings at or above `severity`.
    pub fn at_least(&self, severity: Severity) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.iter()
            .filter(move |diagnostic| diagnostic.severity >= severity)
    }

    /// At least one error-level finding that rejects: a kept error does not count, as in
    /// [`Diagnostics::has_errors`](bedrock_addon_core::diagnostic::Diagnostics::has_errors)
    /// and [`PackReport::has_errors`](bedrock_addon_core::diagnostic::PackReport::has_errors).
    /// A kept error is still in [`at_least`](Self::at_least)`(Severity::Error)`.
    pub fn has_errors(&self) -> bool {
        self.iter().any(Diagnostic::rejects)
    }

    /// Findings grouped by pack, in stack order; findings that name no pack are left out.
    pub fn by_pack(&self) -> impl Iterator<Item = (&PackRef, &[&Diagnostic])> + '_ {
        self.sorted
            .chunk_by(|a, b| a.location.pack == b.location.pack)
            .filter_map(|group| Some((group[0].location.pack.as_ref()?, group)))
    }

    /// Findings grouped by file (one group per pack and path, in `Display` order); findings
    /// that name no file are left out.
    pub fn by_file(&self) -> impl Iterator<Item = (&PackPath, &[&Diagnostic])> + '_ {
        self.sorted
            .chunk_by(|a, b| {
                a.location.pack == b.location.pack && a.location.path == b.location.path
            })
            .filter_map(|group| Some((group[0].location.path.as_ref()?, group)))
    }
}

/// The findings in load order, as [`StackReport::iter`] gives them.
impl<'r> IntoIterator for &'r StackReport<'_> {
    type Item = &'r Diagnostic;
    type IntoIter = std::iter::Copied<std::slice::Iter<'r, &'r Diagnostic>>;

    fn into_iter(self) -> Self::IntoIter {
        self.findings.iter().copied()
    }
}

/// One line per finding, sorted by pack, path, offset.
impl fmt::Display for StackReport<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diagnostic in &self.sorted {
            writeln!(f, "{diagnostic}")?;
        }
        Ok(())
    }
}

/// The findings in load order.
impl fmt::Debug for StackReport<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(&self.findings).finish()
    }
}

/// Pack (findings without one first, then by stack index, the app-data layer last, then
/// identity and name), path (none first), offset (none first).
fn by_place(a: &Diagnostic, b: &Diagnostic) -> Ordering {
    let pack = |d: &Diagnostic| {
        d.location.pack.as_ref().map(|pack| {
            (
                pack.index().unwrap_or(usize::MAX),
                pack.id().uuid,
                pack.id().version.to_string(),
                pack.name().to_owned(),
            )
        })
    };
    pack(a)
        .cmp(&pack(b))
        .then_with(|| a.location.path.cmp(&b.location.path))
        .then_with(|| {
            let offset = |d: &Diagnostic| d.location.span.map(|span| span.offset);
            offset(a).cmp(&offset(b))
        })
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::diagnostic::codes as core_codes;
    use bedrock_addon_core::diagnostic::{
        Code, Diagnostic, JsonPath, Location, Phase, Severity, Span,
    };
    use bedrock_addon_core::pack_identity::{PackIdVersion, PackPath, PackRef, PackType};
    use bedrock_addon_core::version::SemVersion;
    use uuid::Uuid;

    use super::StackReport;

    fn pack(index: usize) -> PackRef {
        let id = PackIdVersion::new(
            Uuid::from_u128(index as u128 + 1),
            SemVersion::new(1, 0, 0),
            PackType::Behavior,
        );
        PackRef::new(index, id, format!("pack {index}"))
    }

    fn finding(
        pack: Option<PackRef>,
        path: Option<&'static str>,
        offset: Option<u32>,
        severity: Severity,
    ) -> Diagnostic {
        Diagnostic::new(
            Code::from(&core_codes::FILE_NOT_LOADED),
            severity,
            Phase::Read,
            Location::new(
                pack,
                path.map(PackPath::from_static),
                JsonPath::new(),
                offset.map(|offset| Span { offset, len: 1 }),
            ),
        )
    }

    #[test]
    fn display_sorts_by_pack_path_and_offset_and_iter_keeps_load_order() {
        let findings = [
            finding(Some(pack(1)), Some("b.json"), Some(9), Severity::Warning),
            finding(
                Some(PackRef::app_data()),
                Some("a.json"),
                None,
                Severity::Info,
            ),
            finding(Some(pack(0)), Some("z.json"), None, Severity::Error),
            finding(Some(pack(1)), Some("b.json"), Some(2), Severity::Warning),
            finding(None, None, None, Severity::Info),
            finding(Some(pack(1)), Some("a.json"), None, Severity::Warning),
        ];
        let report = StackReport::new(findings.iter().collect());
        let order: Vec<usize> = report
            .iter()
            .map(|d| findings.iter().position(|f| std::ptr::eq(f, d)).unwrap())
            .collect();
        assert_eq!(order, [0, 1, 2, 3, 4, 5]);
        let lines: Vec<String> = report.to_string().lines().map(str::to_owned).collect();
        let expected: Vec<String> = [4, 2, 5, 3, 0, 1]
            .iter()
            .map(|index| findings[*index].to_string())
            .collect();
        assert_eq!(lines, expected);
        assert!(report.has_errors());
        assert_eq!(report.at_least(Severity::Warning).count(), 4);

        let packs: Vec<(Option<usize>, usize)> = report
            .by_pack()
            .map(|(pack, group)| (pack.index(), group.len()))
            .collect();
        assert_eq!(packs, [(Some(0), 1), (Some(1), 3), (None, 1)]);
        let files: Vec<(&str, usize)> = report
            .by_file()
            .map(|(path, group)| (path.as_str(), group.len()))
            .collect();
        assert_eq!(
            files,
            [("z.json", 1), ("a.json", 1), ("b.json", 2), ("a.json", 1)]
        );
    }

    /// Only a kept error: an error in the game's content log, after which the game keeps the
    /// document. It rejects nothing, so the stack has no error that stops a load.
    #[test]
    fn a_report_of_only_kept_errors_has_no_errors() {
        let findings = [
            finding(Some(pack(0)), Some("a.json"), None, Severity::Error).into_kept(),
            finding(None, None, None, Severity::Warning),
        ];
        let report = StackReport::new(findings.iter().collect());
        assert!(!report.has_errors());
        assert_eq!(report.at_least(Severity::Error).count(), 1);
    }

    #[test]
    fn a_report_with_a_rejecting_error_has_errors() {
        let findings = [
            finding(Some(pack(0)), Some("a.json"), None, Severity::Error).into_kept(),
            finding(Some(pack(0)), Some("b.json"), None, Severity::Error),
        ];
        assert!(StackReport::new(findings.iter().collect()).has_errors());
    }

    #[test]
    fn a_report_iterates_by_reference_and_clones() {
        let findings = [
            finding(Some(pack(0)), Some("a.json"), None, Severity::Error),
            finding(None, None, None, Severity::Info),
        ];
        let report = StackReport::new(findings.iter().collect());
        let copy = report.clone();
        let mut seen = 0;
        for diagnostic in &copy {
            assert!(std::ptr::eq(diagnostic, &raw const findings[seen]));
            seen += 1;
        }
        assert_eq!(seen, report.iter().count());
    }
}
