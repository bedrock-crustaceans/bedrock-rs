//! Constructing and checking structures, and positions in the box.

use bedrock_addon_core::load::LoadContext;

use super::{Contents, Layout, Structure, parse};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Phase};

impl Structure {
    /// The most positions [`Structure::new`] fills: 64 × 384 × 64, the largest box
    /// `/structure save` takes in the overworld.
    pub const NEW_FILL_LIMIT: usize = 64 * 384 * 64;

    /// A structure of `size` blocks that keeps every world block where it is placed: both
    /// index layers all -1, one per position; no palettes, no entities, origin `[0, 0, 0]`.
    ///
    /// The layers are left empty, which vanilla accepts for any size, when the box has no
    /// position (a component below 1) or more than [`Structure::NEW_FILL_LIMIT`] of them
    /// (a file itself has no size limit: fill `block_indices` yourself for a larger box).
    /// Never panics, whatever the size.
    pub fn new(size: [i32; 3]) -> Structure {
        let volume = size
            .iter()
            .try_fold(1_usize, |volume, side| {
                volume.checked_mul(usize::try_from(*side).unwrap_or(0))
            })
            .filter(|volume| *volume <= Structure::NEW_FILL_LIMIT)
            .unwrap_or(0);
        Structure {
            format_version: Structure::FORMAT_VERSION,
            neighbor_aware_block_upgrade_version: None,
            size,
            structure_world_origin: [0, 0, 0],
            structure: Contents {
                block_indices: [vec![-1; volume], vec![-1; volume]],
                ..Contents::default()
            },
            extra: nbtx::Compound::new(),
            layout: Layout::default(),
        }
    }

    /// Runs the format's load rules as if the structure were read from a pack with `cx`: the
    /// findings loading its file would give (none for a valid structure, apart from the crate's own
    /// infos).
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        match self.to_bytes() {
            Ok(bytes) => parse::read(&bytes, cx).1,
            Err(error) => {
                let mut findings = Diagnostics::default();
                findings.push(
                    Diagnostic::error(DiagnosticCode::MalformedInput {
                        message: error.to_string(),
                        text: None,
                    })
                    .phase(Phase::Validate),
                );
                findings
            }
        }
    }

    /// The index in each [`block_indices`](super::Contents::block_indices) layer of the
    /// position `[x, y, z]` relative to the box's minimum corner: z runs fastest, then y, then
    /// x. `None` outside the box, and for a position
    /// whose index does not fit a `usize`.
    pub fn index(&self, position: [i32; 3]) -> Option<usize> {
        let [x, y, z] = position.map(usize::try_from);
        let [sx, sy, sz] = self.size.map(usize::try_from);
        let ([x, y, z], [sx, sy, sz]) =
            ([x.ok()?, y.ok()?, z.ok()?], [sx.ok()?, sy.ok()?, sz.ok()?]);
        if x >= sx || y >= sy || z >= sz {
            return None;
        }
        x.checked_mul(sy)?
            .checked_add(y)?
            .checked_mul(sz)?
            .checked_add(z)
    }
}
