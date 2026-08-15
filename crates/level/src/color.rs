use facet::Facet;

/// A Bedrock dye colour, stored in NBT as a `Byte` tag holding the numeric id
/// in most records — a `Bed`'s `color` key, for one. A `Banner`'s `Base` key
/// holds the same id under an `Int` tag instead: both forms are real, in the
/// same world, for the same enum, so the wider tag is accepted rather than
/// rejected.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(nbtx::variant_as(i8), nbtx::lenient_width(i32))]
#[repr(i8)]
pub enum Color {
    White = 0,
    Orange = 1,
    Magenta = 2,
    LightBlue = 3,
    Yellow = 4,
    Lime = 5,
    Pink = 6,
    Gray = 7,
    LightGray = 8,
    Cyan = 9,
    Purple = 10,
    Blue = 11,
    Brown = 12,
    Green = 13,
    Red = 14,
    Black = 15,
}
