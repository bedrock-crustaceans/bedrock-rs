//! Struct selection: a chain's versions become version ranges by the family's `RangeModel`
//! (each model says where a struct's range starts and ends), and a document's version picks the
//! range that contains it.

use super::{Family, RangeModel, VersionChain, Versioned};
use crate::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput, VersionRange};

/// What a version selects in a chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    /// The struct `VERSIONS[i]`.
    Struct(usize),
    /// Below the family's floor (a pre-release of the floor included): "To use … use json
    /// format version … or higher".
    BelowFloor,
    /// No range covers it: above the model's top, between two exact versions, or a
    /// pre-release no half-open range reaches (vanilla's "no parser available").
    Unsupported,
}

/// The version range of each struct of a chain, in order: struct `i` covers `ranges[i]`.
///
/// The first range starts at `floor`; inner ranges are `[V_i, V_{i+1})`; the last one ends at
/// `ceiling` when given, else at the model's top (the engine, or `V_last` under
/// `ClosedToLatest`). Under `ExactOnly` every range is `[V_i, V_i]`.
pub fn ranges(
    versions: &[FormatVersion],
    floor: FormatVersion,
    model: RangeModel,
    ceiling: Option<FormatVersion>,
) -> Vec<VersionRange> {
    let last = versions.len().saturating_sub(1);
    versions
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let min = if i == 0 { floor.min(v) } else { v };
            if model == RangeModel::ExactOnly {
                return VersionRange::exact(v);
            }
            if i < last {
                return VersionRange::right_open(min, versions[i + 1]);
            }
            match model {
                RangeModel::ClosedToEngine => {
                    VersionRange::closed(min, ceiling.unwrap_or(ENGINE_VERSION))
                }
                // Only `V_last` itself: vanilla's sliced loaders close the last slice at
                // the newest struct's own version.
                RangeModel::ClosedToLatest => VersionRange::closed(v, ceiling.unwrap_or(v)),
                RangeModel::HalfOpenToEngine => {
                    VersionRange::right_open(min, ceiling.unwrap_or(ENGINE_VERSION))
                }
                RangeModel::ExactOnly => VersionRange::exact(v),
            }
        })
        .collect()
}

/// The struct of family `F` that a stable document version selects (the literal `"beta"`
/// selects no stable struct and is `Unsupported` here).
pub fn select<F: Family>(input: &FormatVersionInput) -> Selection {
    let versions = <F::Any as VersionChain<F>>::VERSIONS;
    select_versions(versions, F::FLOOR, F::RANGE_MODEL, F::CEILING, input)
}

/// [`select`] over explicit parts: a chain's `versions`, its floor, range model and ceiling.
pub fn select_versions(
    versions: &[FormatVersion],
    floor: FormatVersion,
    model: RangeModel,
    ceiling: Option<FormatVersion>,
    input: &FormatVersionInput,
) -> Selection {
    let Some(v) = input.version else {
        return Selection::Unsupported;
    };
    // `FLOOR-beta.n` sorts below the floor in every model.
    if v < floor || (input.prerelease && v == floor) {
        return Selection::BelowFloor;
    }
    // Under `ClosedToLatest` nothing covers `V_last-beta.n`: the last range is
    // `V_last` exactly, and the inner range below it stops short of `V_last` in vanilla's
    // pre-release comparison. `VersionRange::contains_input` would put it in that inner range.
    if model == RangeModel::ClosedToLatest && input.prerelease && versions.last() == Some(&v) {
        return Selection::Unsupported;
    }
    ranges(versions, floor, model, ceiling)
        .iter()
        .position(|range| range.contains_input(input))
        .map_or(Selection::Unsupported, Selection::Struct)
}

/// Why a family's chain is malformed (checked by a unit test per family).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ChainError {
    /// `VERSIONS` is not strictly ascending.
    #[error("VERSIONS is not strictly ascending at {at}")]
    NotAscending {
        /// The first version out of order.
        at: FormatVersion,
    },
    /// The last struct is not the family's `Latest`.
    #[error("the last struct is {last}, but Latest is {latest}")]
    LatestMismatch {
        /// `VERSIONS`' last entry.
        last: FormatVersion,
        /// `Latest::VERSION`.
        latest: FormatVersion,
    },
    /// The floor is above the first struct's version.
    #[error("the floor {floor} is above the first struct {first}")]
    FloorAboveFirst {
        /// `FLOOR`.
        floor: FormatVersion,
        /// `VERSIONS[0]`.
        first: FormatVersion,
    },
    /// Two ranges overlap, or leave a gap (not under `ExactOnly`).
    #[error("ranges {index} and {} are not contiguous", .index + 1)]
    NotContiguous {
        /// The first of the two ranges.
        index: usize,
    },
    /// The component table is not sorted by name, or repeats a name.
    #[error("the component table is not sorted by unique name")]
    TableUnsorted,
    /// A field of the table arrived after the engine this crate tracks.
    #[error("field `{field}` of `{component}` is newer than the engine")]
    FieldAfterEngine {
        /// The component.
        component: &'static str,
        /// The field.
        field: &'static str,
    },
}

/// Checks family `F`'s chain: ascending versions ending at `Latest`, a floor at or below the
/// first struct, contiguous non-overlapping ranges (except under `ExactOnly`), a sorted
/// component table, and no `FieldMeta.since_engine` above `ENGINE_VERSION`.
///
/// # Errors
///
/// Returns the first `ChainError` found in the chain.
pub fn check_chain<F: Family>() -> Result<(), ChainError> {
    let versions = <F::Any as VersionChain<F>>::VERSIONS;
    if let Some(pair) = versions.windows(2).find(|pair| pair[0] >= pair[1]) {
        return Err(ChainError::NotAscending { at: pair[1] });
    }
    if let (Some(&first), Some(&last)) = (versions.first(), versions.last()) {
        let latest = <F::Latest as Versioned>::VERSION;
        if last != latest {
            return Err(ChainError::LatestMismatch { last, latest });
        }
        if F::FLOOR > first {
            return Err(ChainError::FloorAboveFirst {
                floor: F::FLOOR,
                first,
            });
        }
    }
    if F::RANGE_MODEL != RangeModel::ExactOnly {
        let ranges = ranges(versions, F::FLOOR, F::RANGE_MODEL, F::CEILING);
        if let Some(index) = ranges
            .windows(2)
            .position(|pair| pair[0].max_inclusive || pair[0].max != pair[1].min)
        {
            return Err(ChainError::NotContiguous { index });
        }
    }
    let table = F::components();
    if !table.is_sorted() {
        return Err(ChainError::TableUnsorted);
    }
    for meta in table.0 {
        if let Some(field) = meta.fields.iter().find(|f| f.since_engine > ENGINE_VERSION) {
            return Err(ChainError::FieldAfterEngine {
                component: meta.name,
                field: field.name,
            });
        }
    }
    Ok(())
}
