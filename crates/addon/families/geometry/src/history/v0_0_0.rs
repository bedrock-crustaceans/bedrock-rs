//! Format below 1.8.0 (and a file without a readable `format_version`): geometries as
//! `"geometry.<name>"` members of the root, `mirror` and `reset` as a bool or a string, object
//! locators; and the step to 1.8, which normalises `mirror` and `reset` and removes empty cubes.
//!
//! The game's step to 1.8 turns the text forms of `mirror` and `reset` into booleans and removes
//! an empty cube by moving the last cube into its place.

use facet::Facet;
use indexmap::IndexMap;

use super::super::walk::{LOCATORS, floats, int, read_payload, read_root, render_group_id, texels};
use super::super::{GeometryLegacy, Locator};
use super::v1_8_0;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::MolangVisitor;
use bedrock_addon_core::schema::{self, FromJson, Schema};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

/// A geometry file older than 1.8.0.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryFile {
    /// `format_version`: 1.0.0 when the file has none it can read (vanilla validates every such
    /// file as "1.1").
    pub format_version: FormatVersion,
    /// `debug`.
    pub debug: Option<bool>,
    /// The `geometry.<name>` members, by key as written (`geometry.child:geometry.parent`
    /// declares inheritance).
    pub geometries: IndexMap<String, Geometry>,
}

/// One geometry of a file older than 1.8.0.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = none)]
pub struct Geometry {
    /// `visible_bounds_width`.
    pub visible_bounds_width: Option<f32>,
    /// `visible_bounds_height`.
    pub visible_bounds_height: Option<f32>,
    /// `visible_bounds_offset`.
    #[facet(schema::read_with = floats)]
    pub visible_bounds_offset: Option<[f32; 3]>,
    /// `texturewidth`.
    #[facet(schema::read_with = int)]
    pub texturewidth: Option<i32>,
    /// `textureheight`.
    #[facet(schema::read_with = int)]
    pub textureheight: Option<i32>,
    /// `preserve_model_pose`.
    pub preserve_model_pose: Option<bool>,
    /// `cape`.
    pub cape: Option<String>,
    /// `bones`.
    pub bones: Option<Vec<Bone>>,
}

/// A bool member that this era also takes as a string (`mirror`, `reset`).
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[repr(u8)]
pub enum Flag {
    /// A JSON bool.
    Bool(bool),
    /// A JSON string: `"true"` and `"false"` are read, compared exactly.
    Text(String),
}

/// One bone of a file older than 1.8.0. It has no `parent`: vanilla's own old mobs get their
/// hierarchy from a table in the game.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = none)]
pub struct Bone {
    /// `name`.
    pub name: String,
    /// `reset`.
    pub reset: Option<Flag>,
    /// `neverRender`.
    #[facet(rename = "neverRender")]
    pub never_render: Option<bool>,
    /// `pivot`.
    #[facet(schema::read_with = floats)]
    pub pivot: Option<[f32; 3]>,
    /// `rotation`.
    #[facet(schema::read_with = floats)]
    pub rotation: Option<[f32; 3]>,
    /// `mirror`.
    pub mirror: Option<Flag>,
    /// `inflate`.
    pub inflate: Option<f32>,
    /// `debug`.
    pub debug: Option<bool>,
    /// `render_group_id`.
    #[facet(schema::read_with = render_group_id)]
    pub render_group_id: Option<u32>,
    /// `cubes`.
    pub cubes: Option<Vec<CubeSlot>>,
    /// `locators`: a vector, or `{"offset", "rotation", "ignore_inherited_scale"}` with only
    /// `offset` required.
    #[facet(schema::keys = LOCATORS)]
    pub locators: Option<IndexMap<String, Locator>>,
}

/// One element of `cubes`.
#[derive(Clone, Debug, PartialEq, Facet)]
#[repr(u8)]
pub enum CubeSlot {
    /// An object.
    Cube(Cube),
    /// Anything else (a number, `null`, an array): reported when read, removed by the step.
    NotACube,
}

/// One cube of a file older than 1.8.0.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = none)]
pub struct Cube {
    /// `origin`.
    #[facet(schema::read_with = floats)]
    pub origin: Option<[f32; 3]>,
    /// `size`.
    #[facet(schema::read_with = floats)]
    pub size: Option<[f32; 3]>,
    /// `uv`.
    #[facet(schema::read_with = texels)]
    pub uv: Option<[f32; 2]>,
    /// `inflate`.
    pub inflate: Option<f32>,
    /// `mirror`.
    pub mirror: Option<Flag>,
}

// ---- reading -------------------------------------------------------------------------------------

/// The Molang walk of a type of this era: none, it holds no Molang.
fn none<T>(_: &mut T, _: &mut MolangVisitor<'_>) {}

/// A bool, or a string.
impl FromJson for Flag {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if let Some(flag) = value.as_bool() {
            return Some(Self::Bool(flag));
        }
        if let Some(text) = value.as_string() {
            return Some(Self::Text(text.as_str().to_owned()));
        }
        read.wrong_accepting(node, "bool, string");
        None
    }
}

/// An element of `cubes`: a cube, or anything else, which stays in the array until the step
/// removes it (its `size()` is 0); it is reported here.
impl FromJson for CubeSlot {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        Some(Cube::from_json(node, read).map_or(Self::NotACube, Self::Cube))
    }
}

bedrock_addon_core::impl_schema_leaf!(Flag, CubeSlot);
bedrock_addon_core::impl_no_molang!(Flag, CubeSlot);

impl Versioned for GeometryFile {
    const VERSION: FormatVersion = known::V0_0_0;
}

bedrock_addon_core::impl_no_molang!(GeometryFile);

impl VersionStruct for GeometryFile {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let (debug, geometries) = read_payload(payload, cx, d, read_root::<Geometry>)?;
        Some(Self {
            format_version: cx.original_version,
            debug,
            geometries,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        self.to_value()
    }
}

// ---- the step ------------------------------------------------------------------------------------

/// Reports a `mirror` or `reset` value the step removes (the crate's own note; vanilla is silent).
fn removed(cx: &mut UpgradeCx<'_>, at: &JsonPath, member: &str) {
    let at = at.child(member);
    cx.diagnostics.push(
        Diagnostic::of(&core_codes::DROPPED_FIELD)
            .at(at)
            .phase(Phase::Upgrade),
    );
}

/// Notes a `mirror` or `reset` written as a text that the step holds, and the file is written
/// with, as the boolean it spells (the crate's own note; vanilla is silent).
fn rewritten(cx: &mut UpgradeCx<'_>, at: &JsonPath, member: &str) {
    let at = at.child(member);
    cx.diagnostics.push(
        Diagnostic::of(&core_codes::VALUE_REWRITTEN)
            .at(at)
            .phase(Phase::Upgrade),
    );
}

/// The step on `mirror`: a bool stays; `"true"` and `"false"` become bools; any other
/// string is removed.
fn mirror(flag: Option<Flag>, cx: &mut UpgradeCx<'_>, at: &JsonPath) -> Option<bool> {
    match flag? {
        Flag::Bool(b) => Some(b),
        Flag::Text(text) if text == "true" || text == "false" => {
            rewritten(cx, at, "mirror");
            Some(text == "true")
        }
        Flag::Text(_) => {
            removed(cx, at, "mirror");
            None
        }
    }
}

/// The step on `reset`: `true` and `"true"` become `true`; anything else is removed.
fn reset(flag: Option<Flag>, cx: &mut UpgradeCx<'_>, at: &JsonPath) -> Option<bool> {
    match flag? {
        Flag::Bool(true) => Some(true),
        Flag::Text(text) if text == "true" => {
            rewritten(cx, at, "reset");
            Some(true)
        }
        _ => {
            removed(cx, at, "reset");
            None
        }
    }
}

/// The cube pass: an element without members (`{}`, or anything that is not an object) is
/// removed by putting the last element in its place, which is examined next, so the order of
/// the cubes changes as it does in vanilla.
fn cubes(slots: Vec<CubeSlot>, cx: &mut UpgradeCx<'_>, at: &JsonPath) -> Vec<v1_8_0::Cube> {
    // Each cube with the index it has in the file: what is told of it is told there, not
    // at the place the pass moves it to.
    let mut slots: Vec<(usize, CubeSlot)> = slots.into_iter().enumerate().collect();
    let mut index = 0;
    while index < slots.len() {
        let empty = match &slots[index].1 {
            CubeSlot::NotACube => true,
            CubeSlot::Cube(cube) => *cube == Cube::default(),
        };
        if empty {
            // Noted at the index the file has the element at (the crate's own note; vanilla is silent).
            let gone = at.child("cubes").index(slots[index].0);
            cx.diagnostics.push(
                Diagnostic::of(&core_codes::DROPPED_FIELD)
                    .at(gone)
                    .phase(Phase::Upgrade),
            );
            slots.swap_remove(index);
        } else {
            index += 1;
        }
    }
    let mut at = at.clone();
    at.push("cubes");
    slots
        .into_iter()
        .filter_map(|(index, slot)| {
            let CubeSlot::Cube(cube) = slot else {
                return None;
            };
            at.push_index(index);
            let mirror = mirror(cube.mirror, cx, &at);
            at.pop();
            Some(v1_8_0::Cube {
                origin: cube.origin,
                size: cube.size,
                uv: cube.uv,
                inflate: cube.inflate,
                mirror,
            })
        })
        .collect()
}

impl Upgrade for GeometryFile {
    type Next = v1_8_0::GeometryFile;

    /// The step to 1.8: `preserve_model_pose` moves to the side-car, `mirror` and
    /// `reset` are normalised, empty cubes are removed.
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> v1_8_0::GeometryFile {
        let geometries = self
            .geometries
            .into_iter()
            .map(|(key, g)| {
                let mut at = JsonPath::new();
                at.push(&key);
                at.push("bones");
                let bones = g.bones.map(|bones| {
                    bones
                        .into_iter()
                        .enumerate()
                        .map(|(index, b)| {
                            at.push_index(index);
                            let bone = v1_8_0::Bone {
                                name: b.name,
                                reset: reset(b.reset, cx, &at),
                                never_render: b.never_render,
                                pivot: b.pivot,
                                rotation: b.rotation,
                                mirror: mirror(b.mirror, cx, &at),
                                inflate: b.inflate,
                                debug: b.debug,
                                render_group_id: b.render_group_id,
                                cubes: b.cubes.map(|slots| cubes(slots, cx, &at)),
                                locators: b.locators,
                                ..v1_8_0::Bone::default()
                            };
                            at.pop();
                            bone
                        })
                        .collect()
                });
                let geometry = v1_8_0::Geometry {
                    debug: None,
                    visible_bounds_width: g.visible_bounds_width,
                    visible_bounds_height: g.visible_bounds_height,
                    visible_bounds_offset: g.visible_bounds_offset,
                    texturewidth: g.texturewidth,
                    textureheight: g.textureheight,
                    cape: g.cape,
                    bones,
                    legacy: GeometryLegacy {
                        preserve_model_pose: g.preserve_model_pose,
                        ..GeometryLegacy::default()
                    },
                };
                (key, geometry)
            })
            .collect();
        v1_8_0::GeometryFile {
            format_version: self.format_version,
            debug: self.debug,
            geometries,
        }
    }
}
