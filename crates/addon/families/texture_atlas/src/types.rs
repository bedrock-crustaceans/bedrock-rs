//! The texture atlas document: `textures/terrain_texture.json` and `textures/item_texture.json`,
//! one type for both, as vanilla has one class and one loader for them.

use indexmap::IndexMap;

use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::AtlasKey;
use bedrock_addon_core::json::Value;
use bedrock_addon_support::texture::jsoncpp::strtouint32_hex;

/// Which of the two atlas files a [`TextureAtlas`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AtlasKind {
    /// `textures/terrain_texture.json`: block faces.
    Terrain,
    /// `textures/item_texture.json`: item icons.
    Item,
}

impl AtlasKind {
    /// The file's path in a resource pack.
    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::Terrain => "textures/terrain_texture.json",
            Self::Item => "textures/item_texture.json",
        }
    }

    /// The `texture_name` the renderers bind: `atlas.terrain` or `atlas.items`. Another name
    /// loads, and nothing draws from it.
    #[must_use]
    pub fn expected_name(self) -> &'static str {
        match self {
            Self::Terrain => "atlas.terrain",
            Self::Item => "atlas.items",
        }
    }
}

/// A texture atlas: the merged `textures/terrain_texture.json` or `textures/item_texture.json`
/// of a resource stack.
///
/// Vanilla merges every pack's copy as JSON and reads the result once, with its JSON library's
/// coercions; the fields hold what vanilla ends up with. A member the document does not write
/// is `None`, and the accessor of the same name gives vanilla's default.
#[doc(alias = "terrain_texture.json")]
#[doc(alias = "item_texture.json")]
#[derive(Clone, Debug, PartialEq)]
pub struct TextureAtlas {
    /// Which file this is.
    pub kind: AtlasKind,
    /// `texture_name`: the atlas page's resource name. Required: a merged document without
    /// it builds no atlas.
    pub texture_name: String,
    /// `padding`: the requested pad size around each tile; 0 when absent.
    pub padding: Option<i32>,
    /// `num_mip_levels`; 1 when absent, and 0 counts as 1.
    pub num_mip_levels: Option<i32>,
    /// `default_leather_color`, as written; the text after its first character is read as hex.
    pub default_leather_color: Option<String>,
    /// `default_horse_leather_color`, as written.
    pub default_horse_leather_color: Option<String>,
    /// `texture_data`: the tiles by short name, in document order. `None` when the document
    /// has no such member.
    pub texture_data: Option<IndexMap<AtlasKey, AtlasItem>>,
    /// `resource_pack_name` and `format_version`: written by vanilla's own files and read by
    /// nothing.
    pub ignored: UnknownMembers,
    /// Members vanilla does not read.
    pub extra: UnknownMembers,
}

impl TextureAtlas {
    /// The tile named `name`, compared byte for byte.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&AtlasItem> {
        let data = self.texture_data.as_ref()?;
        data.get(&AtlasKey::parse(name))
    }

    /// The tiles in document order.
    pub fn iter(&self) -> impl Iterator<Item = (&AtlasKey, &AtlasItem)> + '_ {
        self.texture_data.iter().flatten()
    }

    /// The tiles in the order vanilla processes them: bytewise by name.
    #[must_use]
    pub fn vanilla_order(&self) -> Vec<(&AtlasKey, &AtlasItem)> {
        let mut tiles: Vec<_> = self.iter().collect();
        tiles.sort_by(|a, b| a.0.as_str().as_bytes().cmp(b.0.as_str().as_bytes()));
        tiles
    }

    /// The tiles by name with the number of frames of each: the length of its `textures`
    /// array, 1 for any other form. What a reference to a tile is checked against.
    #[must_use]
    pub fn frame_counts(&self) -> IndexMap<&str, usize> {
        self.iter()
            .map(|(key, item)| (key.as_str(), item.frame_count()))
            .collect()
    }

    /// The name of a tile that equals `name` when case is ignored: the hint for a reference
    /// that misses by case only (the client's lookup is exact).
    #[must_use]
    pub fn name_ignoring_case(&self, name: &str) -> Option<&str> {
        self.iter()
            .map(|(key, _)| key.as_str())
            .find(|key| key.eq_ignore_ascii_case(name))
    }

    /// The number of tiles.
    #[must_use]
    pub fn len(&self) -> usize {
        self.texture_data.as_ref().map_or(0, IndexMap::len)
    }

    /// Whether the atlas has no tile.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `padding`, 0 when absent.
    #[must_use]
    pub fn padding(&self) -> i32 {
        self.padding.unwrap_or(0)
    }

    /// `num_mip_levels`, 1 when absent.
    #[must_use]
    pub fn num_mip_levels(&self) -> i32 {
        self.num_mip_levels.unwrap_or(1)
    }

    /// The mip level count the atlas is built with: 0 and 1 are 1. Vanilla compares unsigned,
    /// so a negative count is passed on unchanged.
    #[must_use]
    pub fn effective_num_mip_levels(&self) -> i32 {
        match self.num_mip_levels() {
            0 => 1,
            levels => levels,
        }
    }

    /// The colour `default_leather_color` sets, as `0xRRGGBB`; `None` when the member is
    /// absent or empty.
    #[must_use]
    pub fn default_leather_color_rgb(&self) -> Option<u32> {
        leather(self.default_leather_color.as_deref())
    }

    /// The colour `default_horse_leather_color` sets.
    #[must_use]
    pub fn default_horse_leather_color_rgb(&self) -> Option<u32> {
        leather(self.default_horse_leather_color.as_deref())
    }
}

/// A leather colour: nothing for an empty text, else the hex after the first character.
fn leather(text: Option<&str>) -> Option<u32> {
    let text = text.filter(|text| !text.is_empty())?;
    Some(strtouint32_hex(after_first(text)))
}

/// `text` without its first character (vanilla's `substr(1)`; the character is not checked
/// to be `#`).
pub(super) fn after_first(text: &str) -> &str {
    let mut chars = text.chars();
    chars.next();
    chars.as_str()
}

/// One tile of an atlas: an entry of `texture_data`.
#[derive(Clone, Debug, PartialEq)]
pub struct AtlasItem {
    /// `textures`: one element, or one per index. A block picks the index by its texture
    /// variant; an item by its frame.
    pub textures: AtlasTextures,
    /// `quad`; false when absent. Read at this level only.
    pub quad: Option<bool>,
    /// `additive`; false when absent. Read at this level only, and given to every layer.
    pub additive: Option<bool>,
    /// `mip_fade_amount`: the default of every layer; 0 when absent.
    pub mip_fade_amount: Option<f32>,
    /// `mip_fade_color`: the default of every layer; no colour when absent.
    pub mip_fade_color: Option<Color>,
    /// Members vanilla does not read.
    pub extra: UnknownMembers,
}

impl AtlasItem {
    /// `quad`, false when absent.
    #[must_use]
    pub fn quad(&self) -> bool {
        self.quad.unwrap_or(false)
    }

    /// `additive`, false when absent.
    #[must_use]
    pub fn additive(&self) -> bool {
        self.additive.unwrap_or(false)
    }

    /// `mip_fade_amount`, 0 when absent.
    #[must_use]
    pub fn mip_fade_amount(&self) -> f32 {
        self.mip_fade_amount.unwrap_or(0.0)
    }

    /// The elements, one per index.
    #[must_use]
    pub fn elements(&self) -> &[TextureElement] {
        match &self.textures {
            AtlasTextures::One(element) => std::slice::from_ref(element),
            AtlasTextures::Many(elements) => elements,
        }
    }

    /// The number of indices (frames): the length of a `textures` array, 1 for any other form.
    #[must_use]
    pub fn frame_count(&self) -> usize {
        self.elements().len()
    }

    /// The layers vanilla builds for the element at `index`, with the tile's values applied.
    /// Empty for an index the tile does not have and for an element that is no string and no
    /// object.
    #[must_use]
    pub fn layers(&self, index: usize) -> Vec<ResolvedLayer<'_>> {
        let Some(element) = self.elements().get(index) else {
            return Vec::new();
        };
        match element {
            TextureElement::Path(path) => vec![self.path_layer(path)],
            TextureElement::Layer(layer) => vec![self.object_layer(layer)],
            TextureElement::Variations { variations, .. } => variations
                .iter()
                .map(|entry| match entry {
                    VariationEntry::Path(path) => self.path_layer(path),
                    VariationEntry::Layer(layer) => self.object_layer(layer),
                    // The pre-initialised layer: the tile's values are not applied.
                    VariationEntry::Invalid(_) => ResolvedLayer {
                        path: None,
                        weight: 1.0,
                        additive: false,
                        mip_fade_amount: 0.0,
                        mip_fade_color: None,
                        tint_color: Color::WHITE.rgba(),
                    },
                })
                .collect(),
            TextureElement::Invalid(_) => Vec::new(),
        }
    }

    /// The layer of a string element.
    fn path_layer<'a>(&'a self, path: &'a str) -> ResolvedLayer<'a> {
        ResolvedLayer {
            path: Some(path),
            weight: 1.0,
            additive: self.additive(),
            mip_fade_amount: self.mip_fade_amount(),
            mip_fade_color: self.mip_fade_color,
            tint_color: Color::WHITE.rgba(),
        }
    }

    /// The layer of an object element.
    fn object_layer<'a>(&'a self, layer: &'a TextureLayer) -> ResolvedLayer<'a> {
        ResolvedLayer {
            path: layer.path.as_deref(),
            weight: layer.weight(),
            additive: self.additive(),
            mip_fade_amount: layer.mip_fade_amount.unwrap_or(self.mip_fade_amount()),
            mip_fade_color: layer.mip_fade_color.or(self.mip_fade_color),
            tint_color: match &layer.tint_color {
                Some(Tint::Color(color)) => color.rgba(),
                _ => Color::WHITE.rgba(),
            },
        }
    }
}

/// One layer as vanilla builds it from an element and its tile.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ResolvedLayer<'a> {
    /// The image path, without extension; `None` for a layer that names no image.
    pub path: Option<&'a str>,
    /// The weight among the variations, after the clamp.
    pub weight: f32,
    /// The tile's `additive`.
    pub additive: bool,
    /// The layer's `mip_fade_amount`, or the tile's.
    pub mip_fade_amount: f32,
    /// The layer's `mip_fade_color`, or the tile's.
    pub mip_fade_color: Option<Color>,
    /// The plain tint as `[r, g, b, a]`, white when the layer has none.
    pub tint_color: [f32; 4],
}

/// The `textures` member of a tile.
#[derive(Clone, Debug, PartialEq)]
pub enum AtlasTextures {
    /// Any value but an array: index 0.
    One(TextureElement),
    /// An array: one element per index. An empty array gives a tile without indices.
    Many(Vec<TextureElement>),
}

/// One element of `textures`.
#[derive(Clone, Debug, PartialEq)]
pub enum TextureElement {
    /// An object whose `variations` is an array: one weighted layer per entry. Vanilla reads
    /// nothing else of the object.
    Variations {
        /// The entries of `variations`.
        variations: Vec<VariationEntry>,
        /// The object's other members, which vanilla ignores.
        ignored: UnknownMembers,
    },
    /// Any other object: one layer.
    Layer(Box<TextureLayer>),
    /// A string: one layer with this image path.
    Path(String),
    /// A number, a bool, `null` or an array: no layer.
    Invalid(Value),
}

/// One entry of a `variations` array.
#[derive(Clone, Debug, PartialEq)]
pub enum VariationEntry {
    /// An object.
    Layer(Box<TextureLayer>),
    /// A string: the image path.
    Path(String),
    /// Anything else: vanilla keeps an empty layer, without a path and without the tile's
    /// values.
    Invalid(Value),
}

/// The lowest weight of a layer.
pub const MIN_WEIGHT: f32 = 0.05;
/// The highest weight of a layer.
pub const MAX_WEIGHT: f32 = 1_000_000.0;

/// An object element: one image and how it is drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureLayer {
    /// `path`: the image, pack-relative and without extension.
    pub path: Option<String>,
    /// `tint_map_path`: the tint map image.
    pub tint_map_path: Option<String>,
    /// `overlay_color`, as written; the text after its first character is read as `RRGGBB`.
    pub overlay_color: Option<String>,
    /// `tint_color`: a colour, or one colour per channel of the tint map.
    pub tint_color: Option<Tint>,
    /// `tint_base_color`: one colour per channel of the tint map.
    pub tint_base_color: Option<TintMapColor>,
    /// `mip_fade_amount`; the tile's when absent.
    pub mip_fade_amount: Option<f32>,
    /// `mip_fade_color`; the tile's when absent.
    pub mip_fade_color: Option<Color>,
    /// `weight`, within [[`MIN_WEIGHT`], [`MAX_WEIGHT`]]; 1 when absent.
    pub weight: Option<f32>,
    /// `isotropic`: the faces drawn without a fixed orientation.
    pub isotropic: Option<Isotropic>,
    /// Members vanilla does not read.
    pub extra: UnknownMembers,
}

impl TextureLayer {
    /// `weight`, 1 when absent.
    #[must_use]
    pub fn weight(&self) -> f32 {
        self.weight.unwrap_or(1.0)
    }

    /// The colour `overlay_color` sets, as `[r, g, b]` bytes; the alpha is always 1. `None`
    /// when the member is absent or empty.
    #[must_use]
    pub fn overlay_rgb(&self) -> Option<[u8; 3]> {
        let text = self
            .overlay_color
            .as_deref()
            .filter(|text| !text.is_empty())?;
        let [_, r, g, b] = strtouint32_hex(after_first(text)).to_be_bytes();
        Some([r, g, b])
    }

    /// The face mask of `isotropic`; 0 when absent.
    #[must_use]
    pub fn isotropic_mask(&self) -> u8 {
        self.isotropic.as_ref().map_or(0, Isotropic::mask)
    }
}

/// A `tint_color`.
#[derive(Clone, Debug, PartialEq)]
pub enum Tint {
    /// One colour for the whole image.
    Color(Color),
    /// One colour per channel of the tint map (a non-empty object).
    Map(TintMapColor),
}

/// The colours of the four tint map channels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TintMapColor {
    /// `r_color`.
    pub r_color: Option<Color>,
    /// `g_color`.
    pub g_color: Option<Color>,
    /// `b_color`.
    pub b_color: Option<Color>,
    /// `a_color`.
    pub a_color: Option<Color>,
}

/// A colour of an atlas document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    /// `[r, g, b]`: the alpha is 255.
    Rgb([u8; 3]),
    /// `[r, g, b, a]`.
    Rgba([u8; 4]),
    /// A string, read as hex after its first character: `0xAARRGGBB`. `"#RRGGBB"` therefore
    /// has alpha 0.
    Hex(u32),
}

impl Color {
    /// Opaque white, the tint of a layer that sets none.
    pub const WHITE: Color = Color::Rgba([255; 4]);

    /// The colour as `[r, g, b, a]` bytes.
    #[must_use]
    pub fn bytes(self) -> [u8; 4] {
        match self {
            Self::Rgb([r, g, b]) => [r, g, b, 255],
            Self::Rgba(bytes) => bytes,
            Self::Hex(bits) => {
                let [a, r, g, b] = bits.to_be_bytes();
                [r, g, b, a]
            }
        }
    }

    /// The colour as `[r, g, b, a]`, each in 0..=1.
    #[must_use]
    pub fn rgba(self) -> [f32; 4] {
        self.bytes().map(|byte| f32::from(byte) / 255.0)
    }
}

/// A block face of `isotropic`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    /// `down`.
    Down,
    /// `up`.
    Up,
    /// `north`.
    North,
    /// `south`.
    South,
    /// `west`.
    West,
    /// `east`.
    East,
    /// `side`: the four horizontal faces.
    Side,
}

impl Face {
    /// Every face, in mask-bit order.
    pub const ALL: [Face; 7] = [
        Face::Down,
        Face::Up,
        Face::North,
        Face::South,
        Face::West,
        Face::East,
        Face::Side,
    ];

    /// The JSON key.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::Up => "up",
            Self::North => "north",
            Self::South => "south",
            Self::West => "west",
            Self::East => "east",
            Self::Side => "side",
        }
    }

    /// The face of a JSON key.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Face> {
        Self::ALL.into_iter().find(|face| face.name() == name)
    }

    /// The face's bits in the mask: `down` 1, `up` 2, `north` 4, `south` 8, `west` 16,
    /// `east` 32, `side` 60.
    #[must_use]
    pub fn bits(self) -> u8 {
        match self {
            Self::Down => 1,
            Self::Up => 2,
            Self::North => 4,
            Self::South => 8,
            Self::West => 16,
            Self::East => 32,
            Self::Side => 60,
        }
    }
}

/// An `isotropic` member.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Isotropic {
    /// A bool: every face, or none.
    All(bool),
    /// An object of faces. With `side` among them vanilla reads `up`, `down` and `side` only;
    /// without it, the six single faces.
    Faces(IndexMap<Face, bool>),
}

impl Isotropic {
    /// The face mask vanilla stores: `0xFF` for `true`, else the bits of every face set to
    /// true among the keys it reads.
    #[must_use]
    pub fn mask(&self) -> u8 {
        match self {
            Self::All(true) => 0xFF,
            Self::All(false) => 0,
            Self::Faces(faces) => {
                let with_side = faces.contains_key(&Face::Side);
                faces
                    .iter()
                    .filter(|(face, on)| **on && face_is_read(**face, with_side))
                    .fold(0, |mask, (face, _)| mask | face.bits())
            }
        }
    }
}

/// Whether vanilla reads `face` of an `isotropic` object that has (or lacks) `side`.
pub(super) fn face_is_read(face: Face, with_side: bool) -> bool {
    match face {
        // `side` is only ever asked about when it is there.
        Face::Up | Face::Down | Face::Side => true,
        Face::North | Face::South | Face::West | Face::East => !with_side,
    }
}
