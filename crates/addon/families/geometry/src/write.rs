//! Writing geometry files back: members in declaration order, only those
//! the value holds, the legacy side-cars as `__legacy`.

use facet_value::{VArray, VObject};
use indexmap::IndexMap;

use super::history::{v0_0_0, v1_8_0};
use super::schema_walk::has_geometry_name;
use super::types::LATEST;
use super::{
    Bone, BoneLegacy, Cube, CubeUv, Description, FaceUv, Faces, Geometry, GeometryFile,
    GeometryLegacy, ItemDisplayTransform, ItemDisplayTransforms, Locator, PolyMesh, Polys,
    TextureMesh,
};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{EncodeError, Unrepresentable, Value};
use bedrock_addon_core::version::{FormatVersion, known};

type Written = Result<Value, EncodeError>;

/// A refusal of the value in hand; each writer on the way out puts its member or index in
/// front of the path ([`below`]), so the error names where the value is in the document.
fn refused(what: Unrepresentable) -> EncodeError {
    EncodeError::NotRepresentable {
        at: JsonPath::new(),
        what,
    }
}

/// A member name or an element index on the way to a value.
#[derive(Clone, Copy)]
enum Step<'a> {
    Key(&'a str),
    Index(usize),
}

/// `error` with `step` in front of its path.
fn below(error: EncodeError, step: Step<'_>) -> EncodeError {
    match error {
        EncodeError::NotRepresentable { at, what } => {
            let mut path = JsonPath::new();
            match step {
                Step::Key(key) => path.push(key),
                Step::Index(index) => path.push_index(index),
            }
            path.append(&at);
            EncodeError::NotRepresentable { at: path, what }
        }
        other => other,
    }
}

/// An `f32` as the shortest number that reads back as the same `f32`; a whole number without a
/// fraction, but the negative zero, which is written `-0.0`.
#[allow(
    clippy::cast_possible_truncation,
    reason = "a whole number below 2^53 is its own integer"
)]
pub(crate) fn float(x: f32) -> Written {
    if x.is_nan() {
        return Err(refused(Unrepresentable::NaN));
    }
    if x.is_infinite() {
        return Err(refused(if x > 0.0 {
            Unrepresentable::Infinity
        } else {
            Unrepresentable::NegativeInfinity
        }));
    }
    if x == 0.0 && x.is_sign_negative() {
        // The zero of the file keeps its sign: an integer has none.
        return Ok(Value::from(-0.0_f64));
    }
    if x.fract() == 0.0 && x.abs() < 9.0e15 {
        return Ok(Value::from(f64::from(x) as i64));
    }
    let shortest = x.to_string().parse::<f64>().unwrap_or(f64::from(x));
    Ok(Value::from(shortest))
}

fn floats<const N: usize>(values: &[f32; N]) -> Written {
    values
        .iter()
        .enumerate()
        .map(|(index, x)| float(*x).map_err(|e| below(e, Step::Index(index))))
        .collect::<Result<VArray, _>>()
        .map(Value::from)
}

#[allow(
    clippy::unnecessary_wraps,
    clippy::trivially_copy_pass_by_ref,
    reason = "the shape every member writer has"
)]
fn boolean(value: &bool) -> Written {
    Ok(Value::from(*value))
}

#[allow(
    clippy::unnecessary_wraps,
    clippy::ptr_arg,
    reason = "the shape every member writer has"
)]
fn string(value: &String) -> Written {
    Ok(Value::from(value.as_str()))
}

#[allow(
    clippy::unnecessary_wraps,
    clippy::trivially_copy_pass_by_ref,
    reason = "the shape every member writer has"
)]
fn int(value: &i32) -> Written {
    Ok(Value::from(i64::from(*value)))
}

#[allow(
    clippy::unnecessary_wraps,
    clippy::trivially_copy_pass_by_ref,
    reason = "the shape every member writer has"
)]
fn unsigned(value: &u32) -> Written {
    Ok(Value::from(i64::from(*value)))
}

fn version(version: FormatVersion) -> Value {
    Value::from(version.to_string().as_str())
}

/// An object from the members that are present.
struct Object(VObject);

impl Object {
    fn new() -> Self {
        Self(VObject::new())
    }

    fn put(&mut self, key: &str, value: Value) {
        self.0.insert(key, value);
    }

    fn opt<T>(
        &mut self,
        key: &str,
        value: Option<&T>,
        write: impl FnOnce(&T) -> Written,
    ) -> Result<(), EncodeError> {
        if let Some(value) = value {
            self.req(key, write(value))?;
        }
        Ok(())
    }

    /// Puts a member whose value was just written, or hands its refusal on below `key`.
    fn req(&mut self, key: &str, value: Written) -> Result<(), EncodeError> {
        let value = value.map_err(|e| below(e, Step::Key(key)))?;
        self.put(key, value);
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn done(self) -> Value {
        Value::from(self.0)
    }
}

fn map<T>(map: &IndexMap<String, T>, mut write: impl FnMut(&T) -> Written) -> Written {
    let mut object = Object::new();
    for (key, value) in map {
        object.req(key, write(value))?;
    }
    Ok(object.done())
}

fn list<T>(list: &[T], mut write: impl FnMut(&T) -> Written) -> Written {
    list.iter()
        .enumerate()
        .map(|(index, item)| write(item).map_err(|e| below(e, Step::Index(index))))
        .collect::<Result<VArray, _>>()
        .map(Value::from)
}

// ---- the newest shape ------------------------------------------------------------------------

/// A file of 1.12.0 or later: `format_version`, `debug`, `minecraft:geometry`.
pub(crate) fn modern(
    format_version: FormatVersion,
    debug: Option<bool>,
    geometries: &[Geometry],
) -> Written {
    // Not `write_envelope`: `debug` sits between version and payload; older shapes differ.
    let mut o = Object::new();
    o.put("format_version", version(format_version));
    o.opt("debug", debug.as_ref(), boolean)?;
    o.req("minecraft:geometry", list(geometries, geometry))?;
    Ok(o.done())
}

impl GeometryFile {
    /// The file as JSON.
    ///
    /// In the newest shape, with `format_version` 1.26.40 for a file read from an older version
    /// (its members were upgraded to that schema), with two exceptions. A file one of whose
    /// geometries or bones carries a legacy side-car is written in the shape and at the
    /// version it was read in ([`legacy`]): no schema from 1.12.0 declares those members. So
    /// is a file below 1.8.0 with a locator object that has no `rotation`: only the shape
    /// older than 1.8 declares such a locator. From 1.8.0 on a rotation-less locator object
    /// has no shape at all (the 1.8 shape has no locator object, the shapes from 1.12 require
    /// the member) and the write is refused with `NotRepresentable` at the locator's
    /// `rotation`. A legacy-shape write reads back equal, except that a cube whose `pivot`
    /// is `None` reads back with its centre, and it refuses a value its shape cannot hold at the
    /// member (a cube without any member).
    pub(crate) fn to_value(&self) -> Written {
        if self.has_legacy_members() {
            return legacy(self);
        }
        if let Some(at) = self.locator_without_rotation() {
            // Only the shape older than 1.8 declares a locator object without `rotation`.
            if self.format_version < known::V1_8_0 {
                return legacy(self);
            }
            return Err(at);
        }
        if self.geometries.is_empty() {
            return self.no_geometry();
        }
        modern(
            self.format_version.max(LATEST),
            self.debug,
            &self.geometries,
        )
    }

    /// A file without a geometry: one below 1.19.30 whose every element of
    /// `minecraft:geometry` was left out as no geometry. It is written at the
    /// version it was read in, with ONE element that is no geometry, `null`: the text loads
    /// to the same file (the element is left out again, with its warning) and is written
    /// the same. No schema has a file of no geometry, and the elements themselves are
    /// not held. From 1.19.30 no file loads without a geometry, and none is written.
    fn no_geometry(&self) -> Written {
        if self.format_version >= crate::read::V1_19_30 {
            return Err(below(
                refused(Unrepresentable::Unsupported),
                Step::Key("minecraft:geometry"),
            ));
        }
        let mut o = Object::new();
        o.put("format_version", version(self.format_version));
        o.opt("debug", self.debug.as_ref(), boolean)?;
        let mut elements = VArray::new();
        elements.push(Value::NULL);
        o.put("minecraft:geometry", elements.into());
        Ok(o.done())
    }

    /// The refusal of the first locator object without `rotation`, at that member: every schema
    /// from 1.12.0 requires it, and the 1.8 schema has no locator object.
    fn locator_without_rotation(&self) -> Option<EncodeError> {
        for (g, geometry) in self.geometries.iter().enumerate() {
            for (b, bone) in geometry.bones().iter().enumerate() {
                for (name, locator) in bone.locators.iter().flatten() {
                    if matches!(locator, Locator::Placed { rotation: None, .. }) {
                        let mut error = no_place("rotation");
                        for step in [
                            Step::Key(name),
                            Step::Key("locators"),
                            Step::Index(b),
                            Step::Key("bones"),
                            Step::Index(g),
                            Step::Key("minecraft:geometry"),
                        ] {
                            error = below(error, step);
                        }
                        return Some(error);
                    }
                }
            }
        }
        None
    }

    /// A geometry or a bone of the file carries a legacy side-car.
    fn has_legacy_members(&self) -> bool {
        self.geometries
            .iter()
            .any(|g| !g.legacy.is_empty() || g.bones().iter().any(|bone| !bone.legacy.is_empty()))
    }
}

// ---- a file with legacy members, in the shape it was read in --------------------------------------

/// The refusal of the member `member`: the shape being written has no place for its value.
fn no_place(member: &str) -> EncodeError {
    below(refused(Unrepresentable::Unsupported), Step::Key(member))
}

/// `Err` at `member` when `present`.
fn absent(present: bool, member: &str) -> Result<(), EncodeError> {
    if present {
        Err(no_place(member))
    } else {
        Ok(())
    }
}

/// A file with legacy members as the root object of the shape older than 1.12.0: the 1.8 shape
/// for a file read at 1.8.0 or later, the one before it otherwise. A member the shape cannot
/// hold, or would not give back as it is, is refused at its path in the file as held
/// (`minecraft:geometry/<i>/…`): the file never becomes text that reads back as something else.
fn legacy(file: &GeometryFile) -> Written {
    if file.format_version >= FormatVersion::new(1, 12, 0) {
        return Err(first_side_car(file));
    }
    let v1_8 = file.format_version >= known::V1_8_0;
    let mut o = Object::new();
    o.put("format_version", version(file.format_version));
    o.opt("debug", file.debug.as_ref(), boolean)?;
    let mut previous: Option<String> = None;
    for (index, g) in file.geometries.iter().enumerate() {
        let here = |e| {
            below(
                below(e, Step::Index(index)),
                Step::Key("minecraft:geometry"),
            )
        };
        let key = legacy_key(g, previous.as_deref()).map_err(here)?;
        let value = legacy_geometry(g, v1_8).map_err(here)?;
        o.put(&key, value);
        previous = Some(key);
    }
    Ok(o.done())
}

/// The refusal of a file of 1.12.0 or later that carries a side-car, which no shape from 1.12.0
/// holds: at the first geometry's or bone's `__legacy`.
fn first_side_car(file: &GeometryFile) -> EncodeError {
    let mut error = no_place("__legacy");
    for (index, g) in file.geometries.iter().enumerate() {
        let bone = g.bones().iter().position(|bone| !bone.legacy.is_empty());
        if g.legacy.is_empty() && bone.is_none() {
            continue;
        }
        if let (true, Some(bone)) = (g.legacy.is_empty(), bone) {
            error = below(below(error, Step::Index(bone)), Step::Key("bones"));
        }
        error = below(error, Step::Index(index));
        break;
    }
    below(error, Step::Key("minecraft:geometry"))
}

/// The root member name of a geometry, `identifier[:parent]`. The reader splits it at its last
/// `:`, takes only names holding `geometry.<name>` and puts the geometries in key order, so an
/// identifier or parent that would be split elsewhere, a key the pattern refuses and a key out
/// of order are refused.
fn legacy_key(g: &Geometry, previous: Option<&str>) -> Result<String, EncodeError> {
    let identifier = &g.description.identifier;
    let at_identifier = || below(no_place("identifier"), Step::Key("description"));
    let key = match &g.legacy.parent {
        Some(parent) if parent.contains(':') => {
            return Err(below(no_place("parent"), Step::Key("__legacy")));
        }
        Some(parent) => format!("{identifier}:{parent}"),
        None if identifier.contains(':') => return Err(at_identifier()),
        None => identifier.clone(),
    };
    if !has_geometry_name(&key) || previous.is_some_and(|previous| previous >= key.as_str()) {
        return Err(at_identifier());
    }
    Ok(key)
}

fn legacy_geometry(g: &Geometry, v1_8: bool) -> Written {
    // No shape before 1.12.0 holds the newer members.
    absent(g.cape.is_some(), "cape")?;
    absent(
        g.item_display_transforms.is_some(),
        "item_display_transforms",
    )?;
    let side = |e| below(e, Step::Key("__legacy"));
    let mut m = Object::new();
    if v1_8 {
        absent(
            g.legacy.preserve_model_pose.is_some(),
            "preserve_model_pose",
        )
        .map_err(side)?;
        m.opt("debug", g.legacy.debug.as_ref(), boolean)?;
    } else {
        absent(g.legacy.debug.is_some(), "debug").map_err(side)?;
    }
    let d = &g.description;
    bounds(
        &mut m,
        d.visible_bounds_width,
        d.visible_bounds_height,
        d.visible_bounds_offset,
    )
    .map_err(|e| below(e, Step::Key("description")))?;
    m.opt("texturewidth", d.texture_width.as_ref(), int)?;
    m.opt("textureheight", d.texture_height.as_ref(), int)?;
    m.opt(
        "preserve_model_pose",
        g.legacy.preserve_model_pose.as_ref(),
        boolean,
    )?;
    m.opt("bones", g.bones.as_ref(), |bones| {
        list(bones, |bone| legacy_bone(bone, v1_8))
    })?;
    Ok(m.done())
}

fn legacy_bone(b: &Bone, v1_8: bool) -> Written {
    absent(b.binding.is_some(), "binding")?;
    let side = |e| below(e, Step::Key("__legacy"));
    if !v1_8 {
        absent(b.parent.is_some(), "parent")?;
        absent(b.poly_mesh.is_some(), "poly_mesh")?;
        absent(b.texture_meshes.is_some(), "texture_meshes")?;
        // The step from this shape removes a `reset` that is not `true`.
        absent(b.legacy.reset == Some(false), "reset").map_err(side)?;
        absent(b.legacy.bind_pose_rotation.is_some(), "bind_pose_rotation").map_err(side)?;
    }
    let mut o = Object::new();
    o.put("name", Value::from(b.name.as_str()));
    o.opt("reset", b.legacy.reset.as_ref(), boolean)?;
    o.opt("neverRender", b.legacy.never_render.as_ref(), boolean)?;
    o.opt("parent", b.parent.as_ref(), string)?;
    o.opt("pivot", b.pivot.as_ref(), floats)?;
    o.opt("rotation", b.rotation.as_ref(), floats)?;
    o.opt(
        "bind_pose_rotation",
        b.legacy.bind_pose_rotation.as_ref(),
        floats,
    )?;
    o.opt("mirror", b.mirror.as_ref(), boolean)?;
    o.opt("inflate", b.inflate.as_ref(), |x| float(*x))?;
    o.opt("debug", b.debug.as_ref(), boolean)?;
    o.opt("render_group_id", b.render_group_id.as_ref(), unsigned)?;
    o.opt("cubes", b.cubes.as_ref(), |cubes| {
        list(cubes, |cube| legacy_cube(cube, v1_8))
    })?;
    o.opt("locators", b.locators.as_ref(), |locators| {
        map(locators, |l| match l {
            // The 1.8 schema declares the vector alone.
            Locator::Placed { .. } if v1_8 => Err(refused(Unrepresentable::Unsupported)),
            other => locator(other),
        })
    })?;
    o.opt("poly_mesh", b.poly_mesh.as_ref(), poly_mesh)?;
    o.opt("texture_meshes", b.texture_meshes.as_ref(), |m| {
        list(m, texture_mesh)
    })?;
    Ok(o.done())
}

/// A cube of the older shapes: `origin`, `size`, a box `uv`, `inflate`, `mirror`. The upgrade
/// writes the cube's centre as its `pivot`, so that pivot (or none) is the one the shape gives
/// back; the step from the oldest shape removes a cube without members.
fn legacy_cube(c: &Cube, v1_8: bool) -> Written {
    absent(c.rotation.is_some(), "rotation")?;
    let centre = c.centre().map(f32::to_bits);
    absent(
        c.pivot
            .is_some_and(|pivot| pivot.map(f32::to_bits) != centre),
        "pivot",
    )?;
    let mut o = Object::new();
    o.opt("origin", c.origin.as_ref(), floats)?;
    o.opt("size", c.size.as_ref(), floats)?;
    o.opt("uv", c.uv.as_ref(), |uv| match uv {
        CubeUv::Box(uv) => floats(uv),
        CubeUv::Faces(_) => Err(refused(Unrepresentable::Unsupported)),
    })?;
    o.opt("inflate", c.inflate.as_ref(), |x| float(*x))?;
    o.opt("mirror", c.mirror.as_ref(), boolean)?;
    if o.is_empty() && !v1_8 {
        // The oldest step removes a cube without members. The one cube the reader leaves
        // without one is a cube whose only member was a `mirror` that is a string it does
        // not read (any other value of another type takes the cube with it); it has the
        // pivot the upgrade gave it. It is written with the member it had, as a string
        // that is not read either, the empty one: the text loads to the same cube and adds
        // no member the author did not write. A cube built without
        // anything is no cube of this shape.
        if c.pivot.is_none() {
            return Err(refused(Unrepresentable::Unsupported));
        }
        o.put("mirror", Value::from(""));
    }
    Ok(o.done())
}

fn geometry(g: &Geometry) -> Written {
    let mut o = Object::new();
    o.req("description", description(&g.description))?;
    o.opt("cape", g.cape.as_ref(), string)?;
    o.opt(
        "item_display_transforms",
        g.item_display_transforms.as_ref(),
        transforms,
    )?;
    o.opt("bones", g.bones.as_ref(), |bones| list(bones, bone))?;
    geometry_legacy(&mut o, &g.legacy)?;
    Ok(o.done())
}

fn geometry_legacy(o: &mut Object, legacy: &GeometryLegacy) -> Result<(), EncodeError> {
    let mut side = Object::new();
    side.opt("parent", legacy.parent.as_ref(), string)?;
    side.opt(
        "preserve_model_pose",
        legacy.preserve_model_pose.as_ref(),
        boolean,
    )?;
    side.opt("debug", legacy.debug.as_ref(), boolean)?;
    if !side.is_empty() {
        o.put("__legacy", side.done());
    }
    Ok(())
}

fn description(d: &Description) -> Written {
    let mut o = Object::new();
    o.put("identifier", Value::from(d.identifier.as_str()));
    bounds(
        &mut o,
        d.visible_bounds_width,
        d.visible_bounds_height,
        d.visible_bounds_offset,
    )?;
    o.opt("texture_width", d.texture_width.as_ref(), int)?;
    o.opt("texture_height", d.texture_height.as_ref(), int)?;
    Ok(o.done())
}

fn bounds(
    o: &mut Object,
    width: Option<f32>,
    height: Option<f32>,
    offset: Option<[f32; 3]>,
) -> Result<(), EncodeError> {
    o.opt("visible_bounds_width", width.as_ref(), |x| float(*x))?;
    o.opt("visible_bounds_height", height.as_ref(), |x| float(*x))?;
    o.opt("visible_bounds_offset", offset.as_ref(), floats)
}

fn bone(b: &Bone) -> Written {
    let mut o = Object::new();
    o.put("name", Value::from(b.name.as_str()));
    o.opt("parent", b.parent.as_ref(), string)?;
    o.opt("pivot", b.pivot.as_ref(), floats)?;
    o.opt("rotation", b.rotation.as_ref(), floats)?;
    o.opt("mirror", b.mirror.as_ref(), boolean)?;
    o.opt("inflate", b.inflate.as_ref(), |x| float(*x))?;
    o.opt("debug", b.debug.as_ref(), boolean)?;
    o.opt("render_group_id", b.render_group_id.as_ref(), unsigned)?;
    o.opt("binding", b.binding.as_ref(), |m| {
        bedrock_addon_core::molang::write(m).map_err(refused)
    })?;
    o.opt("cubes", b.cubes.as_ref(), |cubes| list(cubes, cube))?;
    o.opt("locators", b.locators.as_ref(), |l| map(l, locator))?;
    o.opt("poly_mesh", b.poly_mesh.as_ref(), poly_mesh)?;
    o.opt("texture_meshes", b.texture_meshes.as_ref(), |m| {
        list(m, texture_mesh)
    })?;
    bone_legacy(&mut o, &b.legacy)?;
    Ok(o.done())
}

fn bone_legacy(o: &mut Object, legacy: &BoneLegacy) -> Result<(), EncodeError> {
    let mut side = Object::new();
    side.opt("reset", legacy.reset.as_ref(), boolean)?;
    side.opt("never_render", legacy.never_render.as_ref(), boolean)?;
    side.opt(
        "bind_pose_rotation",
        legacy.bind_pose_rotation.as_ref(),
        floats,
    )?;
    if !side.is_empty() {
        o.put("__legacy", side.done());
    }
    Ok(())
}

fn cube(c: &Cube) -> Written {
    let mut o = Object::new();
    o.opt("origin", c.origin.as_ref(), floats)?;
    o.opt("size", c.size.as_ref(), floats)?;
    o.opt("rotation", c.rotation.as_ref(), floats)?;
    o.opt("pivot", c.pivot.as_ref(), floats)?;
    o.opt("inflate", c.inflate.as_ref(), |x| float(*x))?;
    o.opt("mirror", c.mirror.as_ref(), boolean)?;
    o.opt("uv", c.uv.as_ref(), |uv| match uv {
        CubeUv::Box(uv) => floats(uv),
        CubeUv::Faces(faces) => self::faces(faces),
    })?;
    Ok(o.done())
}

fn faces(f: &Faces) -> Written {
    let mut o = Object::new();
    o.opt("north", f.north.as_ref(), face)?;
    o.opt("south", f.south.as_ref(), face)?;
    o.opt("east", f.east.as_ref(), face)?;
    o.opt("west", f.west.as_ref(), face)?;
    o.opt("up", f.up.as_ref(), face)?;
    o.opt("down", f.down.as_ref(), face)?;
    Ok(o.done())
}

fn face(f: &FaceUv) -> Written {
    let mut o = Object::new();
    o.req("uv", floats(&f.uv))?;
    o.opt("uv_size", f.uv_size.as_ref(), floats)?;
    o.opt("uv_rotation", f.uv_rotation.as_ref(), |turn| {
        Ok(Value::from(i64::from(*turn)))
    })?;
    o.opt("material_instance", f.material_instance.as_ref(), string)?;
    Ok(o.done())
}

fn locator(l: &Locator) -> Written {
    match l {
        Locator::Position(at) => floats(at),
        Locator::Placed {
            offset,
            rotation,
            ignore_inherited_scale,
        } => {
            let mut o = Object::new();
            o.req("offset", floats(offset))?;
            o.opt("rotation", rotation.as_ref(), floats)?;
            o.opt(
                "ignore_inherited_scale",
                ignore_inherited_scale.as_ref(),
                boolean,
            )?;
            Ok(o.done())
        }
    }
}

fn poly_mesh(m: &PolyMesh) -> Written {
    let mut o = Object::new();
    o.opt("normalized_uvs", m.normalized_uvs.as_ref(), boolean)?;
    o.opt("positions", m.positions.as_ref(), |p| list(p, floats))?;
    o.opt("normals", m.normals.as_ref(), |p| list(p, floats))?;
    o.opt("uvs", m.uvs.as_ref(), |p| list(p, floats))?;
    let polys = match &m.polys {
        Polys::List(name) => Value::from(name.as_str()),
        Polys::Indices(polys) => list(polys, |poly| {
            list(poly, |vertex| {
                Ok(Value::from(
                    vertex
                        .iter()
                        .map(|index| Value::from(i64::from(*index)))
                        .collect::<VArray>(),
                ))
            })
        })?,
    };
    o.put("polys", polys);
    Ok(o.done())
}

fn texture_mesh(m: &TextureMesh) -> Written {
    let mut o = Object::new();
    o.put("texture", Value::from(m.texture.as_str()));
    o.opt("position", m.position.as_ref(), floats)?;
    o.opt("local_pivot", m.local_pivot.as_ref(), floats)?;
    o.opt("rotation", m.rotation.as_ref(), floats)?;
    o.opt("scale", m.scale.as_ref(), floats)?;
    o.opt("use_pixel_depth", m.use_pixel_depth.as_ref(), boolean)?;
    Ok(o.done())
}

fn transforms(t: &ItemDisplayTransforms) -> Written {
    let mut o = Object::new();
    for (name, transform) in t {
        o.req(name, self::transform(transform))?;
    }
    Ok(o.done())
}

fn transform(t: &ItemDisplayTransform) -> Written {
    let mut o = Object::new();
    o.opt("translation", t.translation.as_ref(), floats)?;
    o.opt("rotation", t.rotation.as_ref(), floats)?;
    o.opt("scale", t.scale.as_ref(), floats)?;
    o.opt("rotation_pivot", t.rotation_pivot.as_ref(), floats)?;
    o.opt("scale_pivot", t.scale_pivot.as_ref(), floats)?;
    o.opt("fit_to_frame", t.fit_to_frame.as_ref(), boolean)?;
    Ok(o.done())
}

// ---- the two shapes older than 1.12.0 ----------------------------------------------------------

#[allow(
    clippy::unnecessary_wraps,
    reason = "the shape every member writer has"
)]
fn flag(f: &v0_0_0::Flag) -> Written {
    Ok(match f {
        v0_0_0::Flag::Bool(b) => Value::from(*b),
        v0_0_0::Flag::Text(text) => Value::from(text.as_str()),
    })
}

impl v0_0_0::GeometryFile {
    /// The file as JSON in the shape older than 1.8.0.
    pub(crate) fn to_value(&self) -> Written {
        let mut o = Object::new();
        o.put("format_version", version(self.format_version));
        o.opt("debug", self.debug.as_ref(), boolean)?;
        for (key, g) in &self.geometries {
            let mut m = Object::new();
            bounds(
                &mut m,
                g.visible_bounds_width,
                g.visible_bounds_height,
                g.visible_bounds_offset,
            )?;
            m.opt("texturewidth", g.texturewidth.as_ref(), int)?;
            m.opt("textureheight", g.textureheight.as_ref(), int)?;
            m.opt(
                "preserve_model_pose",
                g.preserve_model_pose.as_ref(),
                boolean,
            )?;
            m.opt("cape", g.cape.as_ref(), string)?;
            m.opt("bones", g.bones.as_ref(), |bones| list(bones, old_bone))?;
            o.put(key, m.done());
        }
        Ok(o.done())
    }
}

fn old_bone(b: &v0_0_0::Bone) -> Written {
    let mut o = Object::new();
    o.put("name", Value::from(b.name.as_str()));
    o.opt("reset", b.reset.as_ref(), flag)?;
    o.opt("neverRender", b.never_render.as_ref(), boolean)?;
    o.opt("pivot", b.pivot.as_ref(), floats)?;
    o.opt("rotation", b.rotation.as_ref(), floats)?;
    o.opt("mirror", b.mirror.as_ref(), flag)?;
    o.opt("inflate", b.inflate.as_ref(), |x| float(*x))?;
    o.opt("debug", b.debug.as_ref(), boolean)?;
    o.opt("render_group_id", b.render_group_id.as_ref(), unsigned)?;
    o.opt("cubes", b.cubes.as_ref(), |cubes| {
        list(cubes, |slot| match slot {
            v0_0_0::CubeSlot::NotACube => Ok(Value::NULL),
            v0_0_0::CubeSlot::Cube(c) => {
                let mut o = Object::new();
                o.opt("origin", c.origin.as_ref(), floats)?;
                o.opt("size", c.size.as_ref(), floats)?;
                o.opt("uv", c.uv.as_ref(), floats)?;
                o.opt("inflate", c.inflate.as_ref(), |x| float(*x))?;
                o.opt("mirror", c.mirror.as_ref(), flag)?;
                Ok(o.done())
            }
        })
    })?;
    o.opt("locators", b.locators.as_ref(), |l| map(l, locator))?;
    Ok(o.done())
}

impl v1_8_0::GeometryFile {
    /// The file as JSON in the 1.8 shape.
    pub(crate) fn to_value(&self) -> Written {
        let mut o = Object::new();
        o.put("format_version", version(self.format_version));
        o.opt("debug", self.debug.as_ref(), boolean)?;
        for (key, g) in &self.geometries {
            let mut m = Object::new();
            m.opt("debug", g.debug.as_ref(), boolean)?;
            bounds(
                &mut m,
                g.visible_bounds_width,
                g.visible_bounds_height,
                g.visible_bounds_offset,
            )?;
            m.opt("texturewidth", g.texturewidth.as_ref(), int)?;
            m.opt("textureheight", g.textureheight.as_ref(), int)?;
            m.opt("cape", g.cape.as_ref(), string)?;
            m.opt("bones", g.bones.as_ref(), |bones| list(bones, bone_1_8))?;
            geometry_legacy(&mut m, &g.legacy)?;
            o.put(key, m.done());
        }
        Ok(o.done())
    }
}

fn bone_1_8(b: &v1_8_0::Bone) -> Written {
    let mut o = Object::new();
    o.put("name", Value::from(b.name.as_str()));
    o.opt("reset", b.reset.as_ref(), boolean)?;
    o.opt("neverRender", b.never_render.as_ref(), boolean)?;
    o.opt("parent", b.parent.as_ref(), string)?;
    o.opt("pivot", b.pivot.as_ref(), floats)?;
    o.opt("rotation", b.rotation.as_ref(), floats)?;
    o.opt("bind_pose_rotation", b.bind_pose_rotation.as_ref(), floats)?;
    o.opt("mirror", b.mirror.as_ref(), boolean)?;
    o.opt("inflate", b.inflate.as_ref(), |x| float(*x))?;
    o.opt("debug", b.debug.as_ref(), boolean)?;
    o.opt("render_group_id", b.render_group_id.as_ref(), unsigned)?;
    o.opt("cubes", b.cubes.as_ref(), |cubes| {
        list(cubes, |c| {
            let mut o = Object::new();
            o.opt("origin", c.origin.as_ref(), floats)?;
            o.opt("size", c.size.as_ref(), floats)?;
            o.opt("uv", c.uv.as_ref(), floats)?;
            o.opt("inflate", c.inflate.as_ref(), |x| float(*x))?;
            o.opt("mirror", c.mirror.as_ref(), boolean)?;
            Ok(o.done())
        })
    })?;
    o.opt("locators", b.locators.as_ref(), |l| map(l, locator))?;
    o.opt("poly_mesh", b.poly_mesh.as_ref(), poly_mesh)?;
    o.opt("texture_meshes", b.texture_meshes.as_ref(), |m| {
        list(m, texture_mesh)
    })?;
    Ok(o.done())
}
