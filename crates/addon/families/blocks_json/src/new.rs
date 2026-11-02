//! Constructors, defaults and the derived views of the client-side block types.

use super::{ClientBlock, FaceBools3, FaceBools6, FaceTextures, Isotropic};

/// The sound type of a block no pack gives one.
pub(super) const NORMAL_SOUND: &str = "normal";

/// The texture of a face no pack gives one: the client registers every block with it.
pub(super) const MISSING_TILE: &str = "missing_tile";

impl ClientBlock {
    /// The exponent the client uses: the pack's, or 1.0, which the client starts every block
    /// with.
    #[must_use]
    pub fn ambient_occlusion_exponent(&self) -> f32 {
        self.ambient_occlusion_exponent.unwrap_or(1.0)
    }

    /// The sound type the client uses: the pack's, or `normal`.
    #[must_use]
    pub fn sound(&self) -> &str {
        self.sound.as_deref().unwrap_or(NORMAL_SOUND)
    }

    /// The textures the client draws the six faces with, in the order up, down, north, south,
    /// west, east: the pack's, or `missing_tile` on every face.
    #[must_use]
    pub fn face_textures(&self) -> [&str; 6] {
        self.textures
            .as_ref()
            .map_or([MISSING_TILE; 6], FaceTextures::faces)
    }

    /// The textures the block is carried with, in the order up, down, north, south, west,
    /// east: `carried_textures`, or the block's own face textures.
    #[must_use]
    pub fn carried_face_textures(&self) -> [&str; 6] {
        self.carried_textures
            .as_ref()
            .map_or_else(|| self.face_textures(), FaceTextures::faces)
    }

    /// The client's isotropic face mask: `down` 0x01, `up` 0x02, `north` 0x04, `south` 0x08,
    /// `west` 0x10, `east` 0x20 (0xff for `true`); 0 when no pack sets `isotropic`.
    #[must_use]
    pub fn isotropic_mask(&self) -> u8 {
        self.isotropic.map_or(0, Isotropic::mask)
    }

    /// The number of `blockshape` (vanilla's shape number), when the pack names a shape.
    #[must_use]
    pub fn blockshape_value(&self) -> Option<i16> {
        self.blockshape
            .as_deref()
            .and_then(super::block_shape_value)
    }
}

impl FaceTextures {
    /// The texture of each face, in the order up, down, north, south, west, east.
    #[must_use]
    pub fn faces(&self) -> [&str; 6] {
        match self {
            Self::All(all) => [all; 6],
            Self::UpDownSide { up, down, side } => [up, down, side, side, side, side],
            Self::SixFaces {
                up,
                down,
                north,
                south,
                west,
                east,
            } => [up, down, north, south, west, east],
        }
    }

    /// The same textures in the six-face form.
    #[must_use]
    pub fn to_six_faces(&self) -> Self {
        let [up, down, north, south, west, east] = self.faces().map(str::to_owned);
        Self::SixFaces {
            up,
            down,
            north,
            south,
            west,
            east,
        }
    }
}

impl Isotropic {
    /// The client's face mask: `down` 0x01, `up` 0x02, `north` 0x04, `south` 0x08, `west`
    /// 0x10, `east` 0x20; `true` is 0xff.
    #[must_use]
    pub fn mask(self) -> u8 {
        let bit = |on: bool, bits: u8| if on { bits } else { 0 };
        match self {
            Self::All(true) => 0xff,
            Self::All(false) => 0,
            Self::UpDownSide(FaceBools3 { up, down, side }) => {
                bit(up, 0x02) | bit(down, 0x01) | bit(side, 0x3c)
            }
            Self::SixFaces(FaceBools6 {
                up,
                down,
                north,
                south,
                west,
                east,
            }) => {
                bit(down, 0x01)
                    | bit(up, 0x02)
                    | bit(north, 0x04)
                    | bit(south, 0x08)
                    | bit(west, 0x10)
                    | bit(east, 0x20)
            }
        }
    }

    /// The same faces in the six-face form (`true` keeps the client's whole mask, 0xff, as
    /// [`Isotropic::All`]).
    #[must_use]
    pub fn to_six_faces(self) -> Self {
        match self {
            Self::All(_) | Self::SixFaces(_) => self,
            Self::UpDownSide(FaceBools3 { up, down, side }) => Self::SixFaces(FaceBools6 {
                up,
                down,
                north: side,
                south: side,
                west: side,
                east: side,
            }),
        }
    }
}
