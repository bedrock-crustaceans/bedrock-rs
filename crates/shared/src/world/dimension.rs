/// A dimension id, as it appears on the wire and in a chunk key's dimension field.
///
/// `Overworld`, `Nether`, and `End` are Bedrock's three built-in dimensions (ids 0-2).
/// `Undefined` is id 3, which this crate has long treated as a fourth recognized id (see
/// `Key`'s dimension-id acceptance) even though it names no real game dimension. `Other`
/// carries every other id -- most importantly the ids an add-on's custom dimension
/// registers, which are not a fixed set this crate can enumerate up front. `Other` is
/// never constructed for 0-3: those four ids always produce the named variants above, so
/// the two never overlap, and `Other`'s payload is always the exact on-disk id -- round
/// tripping through `i32` reproduces it unchanged, the same guarantee the named variants
/// already gave.
///
/// This type does not derive `ProtoCodec`. That macro represents an enum as a match over
/// fixed per-variant discriminant literals, which can encode "id 3 means `Undefined`" but
/// not "any id outside the known set means `Other` of that same id" -- there is no literal
/// to match an unbounded, unknown value against. Nothing in the workspace actually drives
/// a `Dimension` through `ProtoCodec` (packet types carry a bare `i32` dimension field and
/// convert through `From`/`Into` at the call site instead), so there is no wire format at
/// stake here; the derive is simply not attempted rather than emitted incorrectly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dimension {
    Overworld,
    Nether,
    End,
    Undefined,
    /// A dimension id this crate has no fixed name for. Carries the id exactly as read
    /// from disk or wire.
    Other(i32),
}

impl From<i32> for Dimension {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Overworld,
            1 => Self::Nether,
            2 => Self::End,
            3 => Self::Undefined,
            other => Self::Other(other),
        }
    }
}

impl From<Dimension> for i32 {
    fn from(value: Dimension) -> i32 {
        match value {
            Dimension::Overworld => 0,
            Dimension::Nether => 1,
            Dimension::End => 2,
            Dimension::Undefined => 3,
            Dimension::Other(id) => id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_variants_round_trip() {
        for (id, dim) in [
            (0, Dimension::Overworld),
            (1, Dimension::Nether),
            (2, Dimension::End),
            (3, Dimension::Undefined),
        ] {
            assert_eq!(Dimension::from(id), dim);
            assert_eq!(i32::from(dim), id);
        }
    }

    #[test]
    fn unrecognized_ids_round_trip_through_other() {
        for id in [4, 1000, 1001, 65535, 65536, -1, i32::MIN, i32::MAX] {
            let dim = Dimension::from(id);
            assert_eq!(dim, Dimension::Other(id));
            assert_eq!(i32::from(dim), id);
        }
    }
}
