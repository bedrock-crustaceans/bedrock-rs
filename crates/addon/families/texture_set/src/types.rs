//! Texture sets (`<texture>.texture_set.json`): the newest document, its layer values and the
//! layer kinds.

/// The name of a layer's image: a path relative to the folder of the texture set, with or
/// without an image extension. It never starts with `#` (such a string is a hex colour).
///
/// The client loads `<folder of the set>/<name>` from the texture set's own pack: the name as
/// written first, then with `.tga`, `.png`, `.jpg`, `.jpeg`, `.hdr`, `.ktx`. The value is not
/// normalised, so `sub/stone` names a file one folder down.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextureName(pub String);

impl TextureName {
    /// The name as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Three channels of a uniform layer, each 0 to 255 as authored (`[r, g, b]` or `#RRGGBB`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rgb {
    /// Red; metalness in `metalness_emissive_roughness`.
    pub r: f32,
    /// Green; emissive in `metalness_emissive_roughness`.
    pub g: f32,
    /// Blue; roughness in `metalness_emissive_roughness`.
    pub b: f32,
}

/// Four channels of a uniform layer, each 0 to 255 as authored (`[r, g, b, a]` or
/// `#AARRGGBB`, alpha first in the hex form).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rgba {
    /// Red; metalness in `metalness_emissive_roughness_subsurface`.
    pub r: f32,
    /// Green; emissive.
    pub g: f32,
    /// Blue; roughness.
    pub b: f32,
    /// Alpha; subsurface in `metalness_emissive_roughness_subsurface`.
    pub a: f32,
}

impl From<Rgb> for Rgba {
    /// A three-channel layer has alpha 1.0 (255), as the client fills it.
    // RGB nodes overwrite alpha with 1.0.
    fn from(color: Rgb) -> Self {
        Rgba {
            r: color.r,
            g: color.g,
            b: color.b,
            a: 255.0,
        }
    }
}

/// The value of a layer that may be uniform: an image, or one colour for the whole texture.
#[derive(Clone, Debug, PartialEq)]
pub enum LayerValue<C> {
    /// A string that does not start with `#`: the layer's image.
    Texture(TextureName),
    /// A hex string (`#AARRGGBB`, or `#RRGGBB` on a three-channel layer; any case). Written
    /// back in upper case.
    Hex(C),
    /// An array of numbers, one per channel. A value outside 0 to 255 is clamped as the client
    /// clamps it, and an element that is no integer is 255.
    Array(C),
}

impl<C: Copy> LayerValue<C> {
    /// The image name, for an image layer.
    #[must_use]
    pub fn texture(&self) -> Option<&TextureName> {
        match self {
            Self::Texture(name) => Some(name),
            Self::Hex(_) | Self::Array(_) => None,
        }
    }

    /// The colour, for a uniform layer.
    #[must_use]
    pub fn uniform(&self) -> Option<C> {
        match self {
            Self::Texture(_) => None,
            Self::Hex(color) | Self::Array(color) => Some(*color),
        }
    }
}

/// The layers a texture set can hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum LayerKind {
    /// `color`.
    Color,
    /// `metalness_emissive_roughness`.
    Mer,
    /// `metalness_emissive_roughness_subsurface`.
    Mers,
    /// `normal`.
    Normal,
    /// `heightmap`.
    Heightmap,
}

impl LayerKind {
    /// Every layer, in the order the client's schema registers them.
    pub const ALL: [LayerKind; 5] = [
        LayerKind::Color,
        LayerKind::Normal,
        LayerKind::Heightmap,
        LayerKind::Mer,
        LayerKind::Mers,
    ];

    /// The JSON key, which is also the name the client's messages print.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::Mer => "metalness_emissive_roughness",
            Self::Mers => "metalness_emissive_roughness_subsurface",
            Self::Normal => "normal",
            Self::Heightmap => "heightmap",
        }
    }

    /// The channel counts an image of this layer may have.
    #[must_use]
    pub fn channel_counts(self) -> &'static [u8] {
        match self {
            Self::Color | Self::Mer | Self::Normal => &[3, 4],
            Self::Mers => &[4],
            Self::Heightmap => &[1, 2, 3, 4],
        }
    }
}

/// A texture set (`minecraft:texture_set`, format 1.21.30): the layers of one texture for the
/// ray-traced and the deferred renderers. The file sits beside its texture as
/// `<texture>.texture_set.json`.
///
/// A set the client accepts has a `color`, at most one of `metalness_emissive_roughness` and
/// `metalness_emissive_roughness_subsurface`, and at most one of `normal` and `heightmap`.
#[doc(alias = "texture_set")]
#[doc(alias = "minecraft:texture_set")]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureSet {
    /// JSON `color`: the base colour, as an image or uniform (alpha is opacity).
    pub color: Option<LayerValue<Rgba>>,
    /// JSON `normal`: the normal map image.
    pub normal: Option<TextureName>,
    /// JSON `heightmap`: the height map image.
    pub heightmap: Option<TextureName>,
    /// JSON `metalness_emissive_roughness`: the three material channels, as an image or
    /// uniform.
    pub metalness_emissive_roughness: Option<LayerValue<Rgb>>,
    /// JSON `metalness_emissive_roughness_subsurface`: the four material channels, as an
    /// image or uniform. Format 1.21.30 only.
    pub metalness_emissive_roughness_subsurface: Option<LayerValue<Rgba>>,
}

impl TextureSet {
    /// The image name of `layer`, when the set has the layer as an image.
    #[must_use]
    pub fn texture(&self, layer: LayerKind) -> Option<&TextureName> {
        match layer {
            LayerKind::Color => self.color.as_ref()?.texture(),
            LayerKind::Mer => self.metalness_emissive_roughness.as_ref()?.texture(),
            LayerKind::Mers => self
                .metalness_emissive_roughness_subsurface
                .as_ref()?
                .texture(),
            LayerKind::Normal => self.normal.as_ref(),
            LayerKind::Heightmap => self.heightmap.as_ref(),
        }
    }

    /// The colour of `layer`, when the set has the layer as one colour; a three-channel layer
    /// with alpha 255.
    #[must_use]
    pub fn uniform(&self, layer: LayerKind) -> Option<Rgba> {
        match layer {
            LayerKind::Color => self.color.as_ref()?.uniform(),
            LayerKind::Mer => self
                .metalness_emissive_roughness
                .as_ref()?
                .uniform()
                .map(Rgba::from),
            LayerKind::Mers => self
                .metalness_emissive_roughness_subsurface
                .as_ref()?
                .uniform(),
            LayerKind::Normal | LayerKind::Heightmap => None,
        }
    }

    /// The image layers of the set with their names, in [`LayerKind::ALL`] order.
    pub fn textures(&self) -> impl Iterator<Item = (LayerKind, &TextureName)> + '_ {
        LayerKind::ALL
            .into_iter()
            .filter_map(|layer| Some((layer, self.texture(layer)?)))
    }
}
