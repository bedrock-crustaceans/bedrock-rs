//! Format below 1.8.0 (and a file without a readable `format_version`): geometries as
//! `"geometry.<name>"` members of the root, `mirror` and `reset` as a bool or a string, object
//! locators; and the step to 1.8, which normalises `mirror` and `reset` and removes empty cubes.
//!
//! The game's step to 1.8 turns the text forms of `mirror` and `reset` into booleans and removes
//! an empty cube by moving the last cube into its place.

use indexmap::IndexMap;

use super::super::read::{LocatorForm, read_locators, read_render_group_id};
use super::super::schema::{Group, Hit, JsonKind, Node, Pattern, SchemaWalk};
use super::super::{GeometryLegacy, Locator};
use super::{settle, v1_8_0, walk_for};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Geometry {
    /// `visible_bounds_width`.
    pub visible_bounds_width: Option<f32>,
    /// `visible_bounds_height`.
    pub visible_bounds_height: Option<f32>,
    /// `visible_bounds_offset`.
    pub visible_bounds_offset: Option<[f32; 3]>,
    /// `texturewidth`.
    pub texturewidth: Option<i32>,
    /// `textureheight`.
    pub textureheight: Option<i32>,
    /// `preserve_model_pose`.
    pub preserve_model_pose: Option<bool>,
    /// `cape`.
    pub cape: Option<String>,
    /// `bones`.
    pub bones: Option<Vec<Bone>>,
}

/// A bool member that this era also takes as a string (`mirror`, `reset`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Flag {
    /// A JSON bool.
    Bool(bool),
    /// A JSON string: `"true"` and `"false"` are read, compared exactly.
    Text(String),
}

/// One bone of a file older than 1.8.0. It has no `parent`: vanilla's own old mobs get their
/// hierarchy from a table in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bone {
    /// `name`.
    pub name: String,
    /// `reset`.
    pub reset: Option<Flag>,
    /// `neverRender`.
    pub never_render: Option<bool>,
    /// `pivot`.
    pub pivot: Option<[f32; 3]>,
    /// `rotation`.
    pub rotation: Option<[f32; 3]>,
    /// `mirror`.
    pub mirror: Option<Flag>,
    /// `inflate`.
    pub inflate: Option<f32>,
    /// `debug`.
    pub debug: Option<bool>,
    /// `render_group_id`.
    pub render_group_id: Option<u32>,
    /// `cubes`.
    pub cubes: Option<Vec<CubeSlot>>,
    /// `locators`: a vector, or `{"offset", "rotation", "ignore_inherited_scale"}` with only
    /// `offset` required.
    pub locators: Option<IndexMap<String, Locator>>,
}

/// One element of `cubes`.
#[derive(Clone, Debug, PartialEq)]
pub enum CubeSlot {
    /// An object.
    Cube(Cube),
    /// Anything else (a number, `null`, an array): reported when read, removed by the step.
    NotACube,
}

/// One cube of a file older than 1.8.0.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cube {
    /// `origin`.
    pub origin: Option<[f32; 3]>,
    /// `size`.
    pub size: Option<[f32; 3]>,
    /// `uv`.
    pub uv: Option<[f32; 2]>,
    /// `inflate`.
    pub inflate: Option<f32>,
    /// `mirror`.
    pub mirror: Option<Flag>,
}

// ---- reading -------------------------------------------------------------------------------------

/// The root of a file older than 1.12.0: `format_version`, `debug` and at least one
/// `geometry.<name>` member, each read by `read`.
pub(crate) fn read_root<'a, G>(
    walk: &mut SchemaWalk,
    root: Node<'a>,
    mut read: impl FnMut(&mut SchemaWalk, Node<'a>) -> Option<G>,
) -> Option<(Option<bool>, IndexMap<String, G>)> {
    let mut debug = None;
    let mut geometries = IndexMap::new();
    let groups = [
        // Vanilla writes the version it validates with into the file first, so the member is
        // never missing.
        Group::name("format_version"),
        Group::name("debug"),
        Group::pattern(Pattern::Geometry, true),
    ];
    let ok = walk.object(root, &groups, |walk, group, key, node| match group {
        0 => Hit::Ok,
        1 => walk.bool(node).map(|b| debug = Some(b)).into(),
        _ => read(walk, node)
            .map(|g| geometries.insert(key.to_owned(), g))
            .into(),
    });
    ok.then_some((debug, geometries))
}

/// The geometry members both eras below 1.12.0 share, in the order their readers match them
/// (groups 0 to 5); each era appends its own.
pub(crate) const SHARED_GEOMETRY_MEMBERS: [Group; 6] = [
    Group::name("visible_bounds_width"),
    Group::name("visible_bounds_height"),
    Group::name("visible_bounds_offset"),
    Group::name("texturewidth"),
    Group::name("textureheight"),
    Group::name("cape"),
];

fn read_flag(walk: &mut SchemaWalk, node: Node<'_>) -> Option<Flag> {
    match node.kind() {
        JsonKind::Bool => walk.bool(node).map(Flag::Bool),
        JsonKind::String => walk.string(node).map(Flag::Text),
        _ => {
            walk.bad_type("bool, string", node);
            None
        }
    }
}

fn read_geometry(walk: &mut SchemaWalk, node: Node<'_>) -> Option<Geometry> {
    let mut g = Geometry::default();
    let mut groups = SHARED_GEOMETRY_MEMBERS.to_vec();
    groups.push(Group::name("preserve_model_pose"));
    groups.push(Group::name("bones"));
    let ok = walk.object(node, &groups, |walk, group, _, v| match group {
        0 => walk
            .float(v)
            .map(|x| g.visible_bounds_width = Some(x))
            .into(),
        1 => walk
            .float(v)
            .map(|x| g.visible_bounds_height = Some(x))
            .into(),
        2 => walk
            .floats(v)
            .map(|x| g.visible_bounds_offset = Some(x))
            .into(),
        3 => walk.int(v).map(|x| g.texturewidth = Some(x)).into(),
        4 => walk.int(v).map(|x| g.textureheight = Some(x)).into(),
        5 => walk.string(v).map(|x| g.cape = Some(x)).into(),
        6 => walk.bool(v).map(|x| g.preserve_model_pose = Some(x)).into(),
        _ => walk.list(v, read_bone).map(|b| g.bones = Some(b)).into(),
    });
    ok.then_some(g)
}

fn read_bone(walk: &mut SchemaWalk, node: Node<'_>) -> Option<Bone> {
    let mut b = Bone::default();
    let groups = [
        Group::required("name"),
        Group::name("reset"),
        Group::name("neverRender"),
        Group::name("pivot"),
        Group::name("rotation"),
        Group::name("mirror"),
        Group::name("inflate"),
        Group::name("debug"),
        Group::name("render_group_id"),
        Group::name("cubes"),
        Group::name("locators"),
    ];
    let ok = walk.object(node, &groups, |walk, group, _, v| match group {
        0 => walk.string(v).map(|s| b.name = s).into(),
        1 => read_flag(walk, v).map(|f| b.reset = Some(f)).into(),
        2 => walk.bool(v).map(|x| b.never_render = Some(x)).into(),
        3 => walk.floats(v).map(|x| b.pivot = Some(x)).into(),
        4 => walk.floats(v).map(|x| b.rotation = Some(x)).into(),
        5 => read_flag(walk, v).map(|f| b.mirror = Some(f)).into(),
        6 => walk.float(v).map(|x| b.inflate = Some(x)).into(),
        7 => walk.bool(v).map(|x| b.debug = Some(x)).into(),
        8 => read_render_group_id(walk, v)
            .map(|x| b.render_group_id = Some(x))
            .into(),
        9 => walk
            .list(v, |walk, cube| {
                // An element that is not an object stays in the array until the step removes
                // it (its `size()` is 0); it is reported here.
                Some(read_cube(walk, cube).map_or(CubeSlot::NotACube, CubeSlot::Cube))
            })
            .map(|cubes| b.cubes = Some(cubes))
            .into(),
        _ => read_locators(walk, v, LocatorForm::Pre18)
            .map(|l| b.locators = Some(l))
            .into(),
    });
    ok.then_some(b)
}

fn read_cube(walk: &mut SchemaWalk, node: Node<'_>) -> Option<Cube> {
    let mut c = Cube::default();
    let groups = [
        Group::name("origin"),
        Group::name("size"),
        Group::name("uv"),
        Group::name("inflate"),
        Group::name("mirror"),
    ];
    let ok = walk.object(node, &groups, |walk, group, _, v| match group {
        0 => walk.floats(v).map(|x| c.origin = Some(x)).into(),
        1 => walk.floats(v).map(|x| c.size = Some(x)).into(),
        2 => walk.texels(v).map(|x| c.uv = Some(x)).into(),
        3 => walk.float(v).map(|x| c.inflate = Some(x)).into(),
        _ => read_flag(walk, v).map(|f| c.mirror = Some(f)).into(),
    });
    ok.then_some(c)
}

impl Versioned for GeometryFile {
    const VERSION: FormatVersion = FormatVersion::new(0, 0, 0);
}

impl MolangFields for GeometryFile {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl VersionStruct for GeometryFile {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut walk = walk_for(cx);
        let file = Node::of(payload).and_then(|root| read_root(&mut walk, root, read_geometry));
        settle(walk, file, d).map(|(debug, geometries)| Self {
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
        Diagnostic::info(DiagnosticCode::DroppedField { text: None })
            .at(at)
            .phase(Phase::Upgrade),
    );
}

/// Notes a `mirror` or `reset` written as a text that the step holds, and the file is written
/// with, as the boolean it spells (the crate's own note; vanilla is silent).
fn rewritten(cx: &mut UpgradeCx<'_>, at: &JsonPath, member: &str) {
    let at = at.child(member);
    cx.diagnostics.push(
        Diagnostic::info(DiagnosticCode::ValueRewritten { text: None })
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
                Diagnostic::info(DiagnosticCode::DroppedField { text: None })
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
