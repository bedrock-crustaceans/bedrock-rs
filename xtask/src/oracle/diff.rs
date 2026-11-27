//! Structural diff between two normalized dumps (see [`super::dump`]),
//! filtered through an explicit expected-difference set rather than
//! asserting exact equality -- per the roadmap task this harness
//! implements, a real oracle and this crate can legitimately disagree on
//! things that are not bugs (a known gap in one side, or a deliberate
//! choice on this crate's side), and the point of the harness is to
//! surface everything *else*.

/// One line present on only one side of a diff, tagged with which side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Side {
    /// Present in this crate's dump, absent from the oracle's.
    Ours,
    /// Present in the oracle's dump, absent from this crate's.
    Oracle,
}

#[derive(Debug, Clone)]
pub struct Divergence {
    pub side: Side,
    pub line: String,
}

/// A rule matching an *expected* divergence line, so it is filtered out of
/// [`DiffReport::unexplained`] rather than reported as a real disagreement.
/// Matching is substring-based (not regex) deliberately: every rule this
/// harness documents names a concrete record kind or field, which a
/// substring check states as plainly as a pattern would, with nothing to
/// get a character class wrong in.
pub struct ExpectedDifference {
    pub side: Side,
    /// Substring the line must contain to match this rule.
    pub contains: &'static str,
    /// Why this divergence is expected, not a bug -- surfaced in the
    /// report so a reader does not have to cross-reference documentation
    /// to see the reasoning.
    pub reason: &'static str,
}

pub struct DiffReport {
    /// Divergences a rule in the expected-difference set accounts for,
    /// paired with the rule's reason.
    pub explained: Vec<(Divergence, &'static str)>,
    /// Divergences no rule accounts for -- the actual finding this harness
    /// exists to produce.
    pub unexplained: Vec<Divergence>,
    pub ours_total: usize,
    pub oracle_total: usize,
    pub agree_total: usize,
}

/// Diffs two sorted, deduplicated line sets and sorts every divergence
/// through `expected`.
pub fn diff(ours: &[String], oracle: &[String], expected: &[ExpectedDifference]) -> DiffReport {
    let mut ours_only = Vec::new();
    let mut oracle_only = Vec::new();
    let mut agree = 0usize;

    let (mut i, mut j) = (0usize, 0usize);
    while i < ours.len() && j < oracle.len() {
        match ours[i].cmp(&oracle[j]) {
            std::cmp::Ordering::Equal => {
                agree += 1;
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => {
                ours_only.push(ours[i].clone());
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                oracle_only.push(oracle[j].clone());
                j += 1;
            }
        }
    }
    ours_only.extend(ours[i..].iter().cloned());
    oracle_only.extend(oracle[j..].iter().cloned());

    let mut explained = Vec::new();
    let mut unexplained = Vec::new();

    for line in ours_only {
        classify(Side::Ours, line, expected, &mut explained, &mut unexplained);
    }
    for line in oracle_only {
        classify(
            Side::Oracle,
            line,
            expected,
            &mut explained,
            &mut unexplained,
        );
    }

    DiffReport {
        explained,
        unexplained,
        ours_total: ours.len(),
        oracle_total: oracle.len(),
        agree_total: agree,
    }
}

fn classify(
    side: Side,
    line: String,
    expected: &[ExpectedDifference],
    explained: &mut Vec<(Divergence, &'static str)>,
    unexplained: &mut Vec<Divergence>,
) {
    for rule in expected {
        if rule.side == side && line.contains(rule.contains) {
            explained.push((Divergence { side, line }, rule.reason));
            return;
        }
    }
    unexplained.push(Divergence { side, line });
}
