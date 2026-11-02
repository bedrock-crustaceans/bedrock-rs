//! The fixed name sets of the family: piece types, body and arm sizes and sides, each with
//! the text a file spells it by.

/// A value the game reads from a fixed set of names.
///
/// Implemented by the family's enums only; the trait is sealed. [`Written`] is generic over it.
pub trait Named: Copy + PartialEq + sealed::Sealed {
    /// The value `name` spells, compared byte for byte; `None` for any other text.
    fn from_name(name: &str) -> Option<Self>;
    /// The text a file spells the value by.
    fn name(self) -> &'static str;
}

mod sealed {
    pub trait Sealed {}
}

/// A member the game reads as one of a fixed set of names, as the file wrote it.
///
/// A text outside the set is kept ([`Other`](Self::Other)) so that it is written back as it
/// was read; what the game takes it for is on the owning struct's accessor
/// ([`PieceMetadata::piece_type`](super::PieceMetadata::piece_type) and its like).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Written<T> {
    /// A name of the set.
    Known(T),
    /// Any other text, the empty string included.
    Other(String),
}

impl<T: Named> Written<T> {
    /// Reads `text`: the value it names, or the text itself.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        T::from_name(text).map_or_else(|| Self::Other(text.to_owned()), Self::Known)
    }

    /// The text as the file wrote it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Known(value) => value.name(),
            Self::Other(text) => text,
        }
    }

    /// The value, when the text is a name of the set.
    #[must_use]
    pub fn known(&self) -> Option<T> {
        match self {
            Self::Known(value) => Some(*value),
            Self::Other(_) => None,
        }
    }
}

/// Declares one name set: the enum, its numbers and its spellings.
macro_rules! names {
    (
        $(#[$meta:meta])*
        $name:ident { $( $(#[$vmeta:meta])* $variant:ident = $value:literal => $text:literal, )+ }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[non_exhaustive]
        #[repr(u8)]
        pub enum $name {
            $( $(#[$vmeta])* $variant = $value, )+
        }

        impl $name {
            /// Every value, in the game's numbering.
            pub const ALL: &'static [$name] = &[ $( $name::$variant, )+ ];

            /// The game's number for the value.
            #[must_use]
            pub const fn value(self) -> u8 {
                self as u8
            }
        }

        impl sealed::Sealed for $name {}
    };
}

names! {
    /// What kind of Character Creator item a piece is (`piece_type`, `piece_sub_type`): the
    /// game's `persona::PieceType`, with its numbers.
    ///
    /// A file spells 25 of them ([`Named::from_name`]); [`Unknown`](Self::Unknown) and the four
    /// one-sided arm and leg types are values the game uses internally, with no spelling a
    /// file can use from 1.26.40.
    PieceType {
        /// Any text outside the set; `persona_unknown` when the game prints it.
        Unknown = 0 => "persona_unknown",
        /// `persona_skeleton`.
        Skeleton = 1 => "persona_skeleton",
        /// `persona_body`.
        Body = 2 => "persona_body",
        /// `persona_skin`.
        Skin = 3 => "persona_skin",
        /// `persona_bottom`.
        Bottom = 4 => "persona_bottom",
        /// `persona_feet`.
        Feet = 5 => "persona_feet",
        /// `persona_dress`.
        Dress = 6 => "persona_dress",
        /// `persona_top`.
        Top = 7 => "persona_top",
        /// `persona_high_pants`.
        HighPants = 8 => "persona_high_pants",
        /// `persona_hand`.
        Hand = 9 => "persona_hand",
        /// `persona_outerwear`.
        Outerwear = 10 => "persona_outerwear",
        /// `persona_facial_hair`.
        FacialHair = 11 => "persona_facial_hair",
        /// `persona_mouth`.
        Mouth = 12 => "persona_mouth",
        /// `persona_eyes`.
        Eyes = 13 => "persona_eyes",
        /// `persona_hair`.
        Hair = 14 => "persona_hair",
        /// `persona_hood`.
        Hood = 15 => "persona_hood",
        /// `persona_back`.
        Back = 16 => "persona_back",
        /// `persona_face_accessory`.
        FaceAccessory = 17 => "persona_face_accessory",
        /// `persona_head`.
        Head = 18 => "persona_head",
        /// `persona_legs`.
        Legs = 19 => "persona_legs",
        /// `persona_left_leg`: internal-only from 1.26.40 (1.26.20 read the name).
        LeftLeg = 20 => "persona_left_leg",
        /// `persona_right_leg`: internal-only from 1.26.40 (1.26.20 read the name).
        RightLeg = 21 => "persona_right_leg",
        /// `persona_arms`.
        Arms = 22 => "persona_arms",
        /// `persona_left_arm`: internal-only from 1.26.40 (1.26.20 read the name).
        LeftArm = 23 => "persona_left_arm",
        /// `persona_right_arm`: internal-only from 1.26.40 (1.26.20 read the name).
        RightArm = 24 => "persona_right_arm",
        /// `persona_capes`.
        Capes = 25 => "persona_capes",
        /// `persona_classic_skin`.
        ClassicSkin = 26 => "persona_classic_skin",
        /// `persona_emote`.
        Emote = 27 => "persona_emote",
        /// `persona_coco`: a name from 1.26.60.
        Coco = 28 => "persona_coco",
        /// `unsupported`: 28 before 1.26.60, 29 from it.
        Unsupported = 29 => "unsupported",
    }
}

impl Named for PieceType {
    /// The 25 names the 1.26.40+ enum binds: neither `persona_unknown` nor the four one-sided
    /// arm and leg types, which 1.26.20 read and which are internal-only from 1.26.40.
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "persona_skeleton" => Self::Skeleton,
            "persona_body" => Self::Body,
            "persona_skin" => Self::Skin,
            "persona_bottom" => Self::Bottom,
            "persona_feet" => Self::Feet,
            "persona_dress" => Self::Dress,
            "persona_top" => Self::Top,
            "persona_high_pants" => Self::HighPants,
            "persona_hand" => Self::Hand,
            "persona_outerwear" => Self::Outerwear,
            "persona_facial_hair" => Self::FacialHair,
            "persona_mouth" => Self::Mouth,
            "persona_eyes" => Self::Eyes,
            "persona_hair" => Self::Hair,
            "persona_hood" => Self::Hood,
            "persona_back" => Self::Back,
            "persona_face_accessory" => Self::FaceAccessory,
            "persona_head" => Self::Head,
            "persona_legs" => Self::Legs,
            "persona_arms" => Self::Arms,
            "persona_capes" => Self::Capes,
            "persona_classic_skin" => Self::ClassicSkin,
            "persona_emote" => Self::Emote,
            "persona_coco" => Self::Coco,
            "unsupported" => Self::Unsupported,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Unknown => "persona_unknown",
            Self::Skeleton => "persona_skeleton",
            Self::Body => "persona_body",
            Self::Skin => "persona_skin",
            Self::Bottom => "persona_bottom",
            Self::Feet => "persona_feet",
            Self::Dress => "persona_dress",
            Self::Top => "persona_top",
            Self::HighPants => "persona_high_pants",
            Self::Hand => "persona_hand",
            Self::Outerwear => "persona_outerwear",
            Self::FacialHair => "persona_facial_hair",
            Self::Mouth => "persona_mouth",
            Self::Eyes => "persona_eyes",
            Self::Hair => "persona_hair",
            Self::Hood => "persona_hood",
            Self::Back => "persona_back",
            Self::FaceAccessory => "persona_face_accessory",
            Self::Head => "persona_head",
            Self::Legs => "persona_legs",
            Self::LeftLeg => "persona_left_leg",
            Self::RightLeg => "persona_right_leg",
            Self::Arms => "persona_arms",
            Self::LeftArm => "persona_left_arm",
            Self::RightArm => "persona_right_arm",
            Self::Capes => "persona_capes",
            Self::ClassicSkin => "persona_classic_skin",
            Self::Emote => "persona_emote",
            Self::Coco => "persona_coco",
            Self::Unsupported => "unsupported",
        }
    }
}

impl PieceType {
    /// Whether clients from 1.26.40 have no name for the type: the four one-sided arm and leg
    /// types, which 1.26.20 reads.
    #[must_use]
    pub const fn is_one_sided(self) -> bool {
        matches!(
            self,
            Self::LeftLeg | Self::RightLeg | Self::LeftArm | Self::RightArm
        )
    }

    /// Whether a geometry source of a piece of this type may name a `side`: the six arm and
    /// leg types.
    #[must_use]
    pub const fn is_arm_or_leg(self) -> bool {
        matches!(
            self,
            Self::Legs
                | Self::LeftLeg
                | Self::RightLeg
                | Self::Arms
                | Self::LeftArm
                | Self::RightArm
        )
    }

    /// Whether a geometry source of a piece of this type is used with `side` (the text as
    /// written): a left type needs `left`, a right type `right`, `persona_arms` and
    /// `persona_legs` take either, and no other type takes any.
    #[must_use]
    pub fn accepts_side(self, side: &str) -> bool {
        match self {
            Self::LeftLeg | Self::LeftArm => side == "left",
            Self::RightLeg | Self::RightArm => side == "right",
            Self::Legs | Self::Arms => side == "left" || side == "right",
            _ => false,
        }
    }

    /// Whether the Character Creator lets the player recolour a piece of this type that has no
    /// colour swatches: skin, facial hair, mouth, eyes and hair.
    #[must_use]
    pub const fn is_recolorable(self) -> bool {
        matches!(
            self,
            Self::Skin | Self::FacialHair | Self::Mouth | Self::Eyes | Self::Hair
        )
    }

    /// Whether the Character Creator offers "none" for this type's section.
    #[must_use]
    pub const fn can_be_none(self) -> bool {
        matches!(
            self,
            Self::Feet
                | Self::Hand
                | Self::Outerwear
                | Self::FacialHair
                | Self::Mouth
                | Self::Eyes
                | Self::Hair
                | Self::Hood
                | Self::Back
                | Self::FaceAccessory
                | Self::Head
                | Self::Legs
                | Self::Arms
                | Self::Capes
                | Self::Emote
        )
    }
}

names! {
    /// A character's height (`body_variant`, `geometry_sources[].body_size`): the game's
    /// `persona::BodySize`.
    BodySize {
        /// `smaller`.
        Smaller = 0 => "smaller",
        /// `small`.
        Small = 1 => "small",
        /// `medium`: what the game takes a missing, empty or unknown text for.
        Medium = 2 => "medium",
        /// `tall`.
        Tall = 3 => "tall",
        /// `""`: a name of the set from 1.26.40; the game takes it for medium.
        Unspecified = 5 => "",
    }
}

impl Named for BodySize {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "smaller" => Self::Smaller,
            "small" => Self::Small,
            "medium" => Self::Medium,
            "tall" => Self::Tall,
            "" => Self::Unspecified,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Smaller => "smaller",
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Tall => "tall",
            Self::Unspecified => "",
        }
    }
}

names! {
    /// A character's arm width (`geometry_sources[].arm_size`): the game's `persona::ArmSize`.
    ArmSize {
        /// `slim`.
        Slim = 0 => "slim",
        /// `wide`: what the game takes an unknown text for.
        Wide = 1 => "wide",
        /// `""`, which vanilla's files write for "either": a name of the set from 1.26.40.
        Unspecified = 3 => "",
    }
}

impl Named for ArmSize {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "slim" => Self::Slim,
            "wide" => Self::Wide,
            "" => Self::Unspecified,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Slim => "slim",
            Self::Wide => "wide",
            Self::Unspecified => "",
        }
    }
}

names! {
    /// Which side a geometry source is for (`geometry_sources[].side`): the game's
    /// `PieceSide`. 1.26.20 reads `left` and `right` only; `both` is a name from 1.26.40.
    PieceSide {
        /// `left`.
        Left = 0 => "left",
        /// `both`.
        Both = 1 => "both",
        /// `right`.
        Right = 2 => "right",
    }
}

impl Named for PieceSide {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "left" => Self::Left,
            "both" => Self::Both,
            "right" => Self::Right,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Both => "both",
            Self::Right => "right",
        }
    }
}
