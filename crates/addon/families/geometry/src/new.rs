//! Constructors and the accessors that apply vanilla's defaults.

use super::{
    Bone, Cube, Description, FitToFrame, Geometry, GeometryFile, ItemDisplayTransform,
    ItemDisplayTransforms, TextureMesh,
};
use bedrock_addon_core::version::FormatVersion;

/// The newest geometry schema, 1.26.40.
pub(crate) const LATEST: FormatVersion = FormatVersion::new(1, 26, 40);
/// From this file version on, `gui.fit_to_frame: false` does not disable the fit.
const FIT_TO_FRAME_CHANGE: FormatVersion = FormatVersion::new(1, 21, 130);

impl GeometryFile {
    /// A file of the newest format holding `geometries`.
    #[must_use]
    pub fn new(geometries: Vec<Geometry>) -> Self {
        Self {
            format_version: LATEST,
            debug: None,
            geometries,
        }
    }

    /// The geometry with `identifier`, compared without case; the last one when the file
    /// defines it twice, as the later definition replaces the earlier one.
    #[must_use]
    pub fn geometry(&self, identifier: &str) -> Option<&Geometry> {
        self.geometries
            .iter()
            .rev()
            .find(|g| g.description.identifier.eq_ignore_ascii_case(identifier))
    }
}

impl Geometry {
    /// A geometry named `identifier` (`geometry.<name>`) without bones.
    pub fn new(identifier: impl Into<String>) -> Self {
        Self {
            description: Description {
                identifier: identifier.into(),
                ..Description::default()
            },
            ..Self::default()
        }
    }

    /// The geometry's own bones.
    #[must_use]
    pub fn bones(&self) -> &[Bone] {
        self.bones.as_deref().unwrap_or_default()
    }

    /// The bone called `name`, compared without case: one of the geometry's own bones, else one
    /// it inherits, nearest parent first.
    pub fn bone(&self, name: &str) -> Option<&Bone> {
        let find = |bones: &'_ [Bone]| -> Option<usize> {
            bones.iter().position(|b| b.name.eq_ignore_ascii_case(name))
        };
        if let Some(at) = find(self.bones()) {
            return self.bones().get(at);
        }
        self.inherited
            .as_deref()
            .into_iter()
            .flat_map(super::Inherited::chain)
            .find_map(|node| find(node.bones()).and_then(|at| node.bones().get(at)))
    }

    /// The identifier of the geometry this one inherits from (a file older than 1.12.0 only).
    #[must_use]
    pub fn parent(&self) -> Option<&str> {
        self.legacy.parent.as_deref()
    }
}

impl Description {
    /// `visible_bounds_width`, 1 when absent.
    #[must_use]
    pub fn visible_bounds_width(&self) -> f32 {
        self.visible_bounds_width.unwrap_or(1.0)
    }

    /// `visible_bounds_height`, 1 when absent.
    #[must_use]
    pub fn visible_bounds_height(&self) -> f32 {
        self.visible_bounds_height.unwrap_or(1.0)
    }

    /// `visible_bounds_offset`, (0, 0, 0) when absent.
    #[must_use]
    pub fn visible_bounds_offset(&self) -> [f32; 3] {
        self.visible_bounds_offset.unwrap_or_default()
    }

    /// `texture_width`, 64 when absent.
    #[must_use]
    pub fn texture_width(&self) -> i32 {
        self.texture_width.unwrap_or(64)
    }

    /// `texture_height`, 64 when absent.
    #[must_use]
    pub fn texture_height(&self) -> i32 {
        self.texture_height.unwrap_or(64)
    }
}

impl Bone {
    /// A bone called `name` without cubes. The name is stored lower-cased, as vanilla compares
    /// bone names without regard to case.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into().to_lowercase(),
            ..Self::default()
        }
    }

    /// `mirror`, `false` when absent.
    #[must_use]
    pub fn mirror(&self) -> bool {
        self.mirror.unwrap_or(false)
    }

    /// `inflate`, 0 when absent.
    #[must_use]
    pub fn inflate(&self) -> f32 {
        self.inflate.unwrap_or(0.0)
    }

    /// The bone's cubes.
    #[must_use]
    pub fn cubes(&self) -> &[Cube] {
        self.cubes.as_deref().unwrap_or_default()
    }
}

impl Cube {
    /// A cube of `size` whose lowest corner is `origin`.
    #[must_use]
    pub fn new(origin: [f32; 3], size: [f32; 3]) -> Self {
        Self {
            origin: Some(origin),
            size: Some(size),
            ..Self::default()
        }
    }

    /// The point the cube rotates around: `pivot`, or the cube's centre `origin + size / 2`
    /// when absent (a missing `origin` or `size` counts as 0), in every format version.
    #[must_use]
    pub fn pivot(&self) -> [f32; 3] {
        self.pivot.unwrap_or_else(|| self.centre())
    }

    /// `origin + size / 2`.
    pub(crate) fn centre(&self) -> [f32; 3] {
        let origin = self.origin.unwrap_or_default();
        let size = self.size.unwrap_or_default();
        [0, 1, 2].map(|axis| size[axis].mul_add(0.5, origin[axis]))
    }

    /// `inflate`, or the bone's.
    #[must_use]
    pub fn inflate(&self, bone: &Bone) -> f32 {
        self.inflate.unwrap_or_else(|| bone.inflate())
    }

    /// `mirror`, or the bone's.
    #[must_use]
    pub fn mirror(&self, bone: &Bone) -> bool {
        self.mirror.unwrap_or_else(|| bone.mirror())
    }
}

impl TextureMesh {
    /// A texture mesh of `texture`.
    pub fn new(texture: impl Into<String>) -> Self {
        Self {
            texture: texture.into(),
            ..Self::default()
        }
    }

    /// `position`, (0, 0, 0) when absent.
    #[must_use]
    pub fn position(&self) -> [f32; 3] {
        self.position.unwrap_or_default()
    }

    /// `local_pivot`, (0, 0, 0) when absent.
    #[must_use]
    pub fn local_pivot(&self) -> [f32; 3] {
        self.local_pivot.unwrap_or_default()
    }

    /// `rotation`, (0, 0, 0) when absent.
    #[must_use]
    pub fn rotation(&self) -> [f32; 3] {
        self.rotation.unwrap_or_default()
    }

    /// `scale`, (1, 1, 1) when absent.
    #[must_use]
    pub fn scale(&self) -> [f32; 3] {
        self.scale.unwrap_or([1.0; 3])
    }

    /// `use_pixel_depth`, `true` when absent.
    #[must_use]
    pub fn use_pixel_depth(&self) -> bool {
        self.use_pixel_depth.unwrap_or(true)
    }
}

impl ItemDisplayTransform {
    /// How a `gui` transform fits the item, in a file written at `format_version`
    /// ([`GeometryFile::format_version`]).
    #[must_use]
    pub fn fit_to_frame(&self, format_version: FormatVersion) -> FitToFrame {
        match self.fit_to_frame {
            None | Some(true) => FitToFrame::Fit,
            Some(false) if format_version < FIT_TO_FRAME_CHANGE => FitToFrame::Legacy,
            Some(false) => FitToFrame::Unscaled,
        }
    }
}

/// The transforms of an [`ItemDisplayTransforms`] that are present, from
/// [`ItemDisplayTransforms::iter`].
#[derive(Clone, Debug)]
pub struct ItemDisplayTransformsIter<'a> {
    slots: std::iter::Zip<
        std::array::IntoIter<&'static str, 10>,
        std::array::IntoIter<&'a Option<ItemDisplayTransform>, 10>,
    >,
}

impl<'a> Iterator for ItemDisplayTransformsIter<'a> {
    type Item = (&'static str, &'a ItemDisplayTransform);

    fn next(&mut self) -> Option<Self::Item> {
        self.slots
            .find_map(|(name, slot)| slot.as_ref().map(|transform| (name, transform)))
    }
}

impl<'a> IntoIterator for &'a ItemDisplayTransforms {
    type Item = (&'static str, &'a ItemDisplayTransform);
    type IntoIter = ItemDisplayTransformsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl ItemDisplayTransforms {
    /// The member names in vanilla's order (`ItemTransforms::mTypeNames`).
    pub const NAMES: [&'static str; 10] = [
        "gui",
        "firstperson_righthand",
        "firstperson_lefthand",
        "thirdperson_righthand",
        "thirdperson_lefthand",
        "ground",
        "fixed",
        "head",
        "embedded",
        "shelf",
    ];

    /// The transforms present, with their member names, in vanilla's order.
    #[must_use]
    pub fn iter(&self) -> ItemDisplayTransformsIter<'_> {
        ItemDisplayTransformsIter {
            slots: Self::NAMES.into_iter().zip(self.slots()),
        }
    }

    /// The members in the order of [`NAMES`](Self::NAMES).
    pub(crate) fn slots(&self) -> [&Option<ItemDisplayTransform>; 10] {
        [
            &self.gui,
            &self.firstperson_righthand,
            &self.firstperson_lefthand,
            &self.thirdperson_righthand,
            &self.thirdperson_lefthand,
            &self.ground,
            &self.fixed,
            &self.head,
            &self.embedded,
            &self.shelf,
        ]
    }

    /// The members in the order of [`NAMES`](Self::NAMES), to change.
    pub(crate) fn slots_mut(&mut self) -> [&mut Option<ItemDisplayTransform>; 10] {
        [
            &mut self.gui,
            &mut self.firstperson_righthand,
            &mut self.firstperson_lefthand,
            &mut self.thirdperson_righthand,
            &mut self.thirdperson_lefthand,
            &mut self.ground,
            &mut self.fixed,
            &mut self.head,
            &mut self.embedded,
            &mut self.shelf,
        ]
    }
}
