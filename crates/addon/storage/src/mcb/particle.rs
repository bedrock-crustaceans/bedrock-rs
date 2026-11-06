//! `particle_effect` bodies.
//!
//! Layout: the identifier, the material and the texture (strings), a `u32` component count,
//! then per component the FNV-1a 32 hash of its name (little-endian; the components are in
//! name order) and its fields, then the optional curves and the optional events. No tag or
//! length separates one component from the next: a component this reader does not know ends
//! the decoding.
//!
//! The building blocks:
//!
//! - **flag**: one byte, `00` or `01`: a boolean, the presence of an optional, or which of
//!   two forms a member has.
//! - **expression**: a flag, then an `f32` (`01`) or Molang text (`00`, a string).
//! - **vecN**: N expressions.
//!
//! Two versions of the game write two forms of a Molang expression under the same header.
//! Newer ones write the string alone, and the JSON twin has the plain text. Older ones write a
//! `u16` after the string, the Molang version the expression compiles at, and their twins have
//! `{"expression": …, "version": …}`. Nothing in the header tells the two apart, so a body is
//! read both ways: every body the game ships reads in exactly one, or to the same value in both
//! (it has no Molang text). A body that reads to two different values is unsupported.
//!
//! The components (`minecraft:` left out; `?` is an optional with its presence flag; `= x`
//! is the value a member has when the twin leaves it out, and such a member is in the text
//! only when it is not at that value):
//!
//! | component | fields |
//! | --- | --- |
//! | `emitter_initialization` | `creation_expression` = 0.0, `per_update_expression` = 0.0 |
//! | `emitter_lifetime_expression` | `activation_expression` = 1.0, `expiration_expression` = 0.0 |
//! | `emitter_lifetime_looping` | `active_time`, `sleep_time` = 0.0 |
//! | `emitter_lifetime_once` | `active_time` = 0.0 |
//! | `emitter_local_space` | `position`, `rotation` (flags), `00` |
//! | `emitter_rate_instant` | `num_particles` |
//! | `emitter_rate_manual` | `max_particles` |
//! | `emitter_rate_steady` | `spawn_rate`, `max_particles` |
//! | `emitter_shape_box` | `offset` vec3 = 0, `half_dimensions` vec3, `surface_only` = false, `direction`? (a flag: `00` a string, `01` a vec3) |
//! | `emitter_shape_custom` | `offset` vec3 = 0, `direction` vec3 = 0 |
//! | `emitter_shape_disc` | `offset` vec3 = 0, `radius`, `surface_only` = false, `direction`? as the box, `plane_normal` (a flag: `00` a string, `01` a vec3 = 0, 1, 0) |
//! | `emitter_shape_entity_aabb` | `surface_only` = false, `direction`? as the box |
//! | `emitter_shape_point` | `offset` vec3 = 0, `direction`? vec3 |
//! | `emitter_shape_sphere` | `offset` vec3 = 0, `radius`, `surface_only` = false, `direction`? as the box |
//! | `particle_appearance_billboard` | `size` vec2, `facing_camera_mode`? (string), `direction`? { `mode` (string), `custom_direction`? vec3, `min_speed_threshold` (f32) = 0.0 }, `uv`? { `texture_width`, `texture_height` (i32) = 1, `uv`? vec2, `uv_size`? vec2, `flipbook`? { `base_UV` vec2, `size_UV` 2 × f32 = 1, 1, `step_UV` 2 × f32, `frames_per_second` (f32) = 0.0, `max_frame`, `stretch_to_lifetime` = false, `loop` = false } } |
//! | `particle_appearance_lighting` | nothing |
//! | `particle_appearance_tinting` | a flag: `00` `color` vec4; `01` then `01`, `color.gradient`: count × (key string, vec4), `color.interpolant` |
//! | `particle_expire_if_not_in_blocks` | count × string |
//! | `particle_initial_speed` | a flag: `01` one expression, `00` a vec3 |
//! | `particle_initial_spin` | `rotation` = 0.0, `rotation_rate` = 0.0 |
//! | `particle_initialization` | `per_update_expression` = 0.0, `per_render_expression` = 0.0 |
//! | `particle_kill_plane` | 4 × f32 |
//! | `particle_lifetime_expression` | `max_lifetime`?, `expiration_expression` = 0.0 |
//! | `particle_motion_collision` | `enabled` = 1.0, `collision_drag`, `coefficient_of_restitution` (f32) = 0.0, `collision_radius` (f32), `expire_on_contact` = false, `events`?: count × { `min_speed` (f32) = 2.0, `event` (string) } |
//! | `particle_motion_dynamic` | `linear_acceleration` vec3 = 0, `linear_drag_coefficient`, `rotation_acceleration`, `rotation_drag_coefficient` = 0.0 |
//! | `particle_motion_parametric` | `relative_position`? vec3, `direction`? vec3, `rotation`? |
//!
//! Curves?: count × (name string, `00`, `type` string, `nodes`: count × expression, `input`,
//! `horizontal_range`). Events?: count × (name string, `01 00 00 00 00`, `effect` string,
//! `type` string, `01 00 00 00 00 00 00 00 00`), the one vanilla event (`particle_effect`).
//!
//! A gradient key is the time as text with six decimals (`0.000000`), as the twins have it.
// Checked against the 395 distinct particle effects of the game's packs against their JSON twins.

use super::McbValue;
use super::reader::{Decoded, Members, Reader, UNSUPPORTED, insert, named, object, string};

/// The components vanilla bakes.
const COMPONENTS: [&str; 26] = [
    "minecraft:emitter_initialization",
    "minecraft:emitter_lifetime_expression",
    "minecraft:emitter_lifetime_looping",
    "minecraft:emitter_lifetime_once",
    "minecraft:emitter_local_space",
    "minecraft:emitter_rate_instant",
    "minecraft:emitter_rate_manual",
    "minecraft:emitter_rate_steady",
    "minecraft:emitter_shape_box",
    "minecraft:emitter_shape_custom",
    "minecraft:emitter_shape_disc",
    "minecraft:emitter_shape_entity_aabb",
    "minecraft:emitter_shape_point",
    "minecraft:emitter_shape_sphere",
    "minecraft:particle_appearance_billboard",
    "minecraft:particle_appearance_lighting",
    "minecraft:particle_appearance_tinting",
    "minecraft:particle_expire_if_not_in_blocks",
    "minecraft:particle_initial_speed",
    "minecraft:particle_initial_spin",
    "minecraft:particle_initialization",
    "minecraft:particle_kill_plane",
    "minecraft:particle_lifetime_expression",
    "minecraft:particle_motion_collision",
    "minecraft:particle_motion_dynamic",
    "minecraft:particle_motion_parametric",
];

/// The payload of a particle effect document.
pub(super) fn decode(bytes: &[u8]) -> Decoded<McbValue> {
    let plain = Body::new(bytes, false).effect();
    let versioned = Body::new(bytes, true).effect();
    match (plain, versioned) {
        // Two readings with a NaN at the same place are one reading (`PartialEq` says they
        // differ): the text writer then names the float as the fault.
        (Ok(plain), Ok(versioned)) if same(&plain, &versioned) => Ok(plain),
        (Ok(_), Ok(_)) => Err(UNSUPPORTED),
        (Ok(effect), Err(_)) | (Err(_), Ok(effect)) => Ok(effect),
        (Err(error), Err(_)) => Err(error),
    }
}

/// Whether two decoded values are the same: as `==`, with floats compared by their bits, so
/// that a NaN is itself and `-0.0` is not `0.0`.
fn same(a: &McbValue, b: &McbValue) -> bool {
    match (a, b) {
        (McbValue::F32(a), McbValue::F32(b)) => a.to_bits() == b.to_bits(),
        (McbValue::F64(a), McbValue::F64(b)) => a.to_bits() == b.to_bits(),
        (McbValue::Array(a), McbValue::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same(a, b))
        }
        (McbValue::Object(a), McbValue::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|((key, a), (other, b))| key == other && same(a, b))
        }
        _ => a == b,
    }
}

/// A body read in one of the two forms.
struct Body<'a> {
    reader: Reader<'a>,
    /// Whether a Molang string is followed by its `u16` version.
    versioned: bool,
}

/// Adds `key` unless `value` is at the default the twins leave out.
fn unless(default: bool, members: &mut Members, key: &str, value: McbValue) {
    if !default {
        members.push((key.into(), value));
    }
}

impl<'a> Body<'a> {
    fn new(bytes: &'a [u8], versioned: bool) -> Body<'a> {
        Body {
            reader: Reader::new(bytes),
            versioned,
        }
    }

    fn effect(mut self) -> Decoded<McbValue> {
        let identifier = self.reader.string()?;
        let material = self.reader.string()?;
        let texture = self.reader.string()?;
        let description = object(vec![
            ("identifier".into(), string(identifier)),
            (
                "basic_render_parameters".into(),
                object(vec![
                    ("material".into(), string(material)),
                    ("texture".into(), string(texture)),
                ]),
            ),
        ]);
        let count = self.reader.u32()?;
        let mut components = Members::new();
        for _ in 0..count {
            let name = named(self.reader.u32()?, &COMPONENTS).ok_or(UNSUPPORTED)?;
            let value = self.component(name)?;
            insert(&mut components, name.into(), value)?;
        }
        let mut effect: Members = vec![("description".into(), description)];
        if self.reader.flag()? {
            effect.push(("curves".into(), self.curves()?));
        }
        if self.reader.flag()? {
            effect.push(("events".into(), self.events()?));
        }
        effect.push(("components".into(), object(components)));
        self.reader.end()?;
        Ok(object(effect))
    }

    /// A float or Molang text.
    fn expression(&mut self) -> Decoded<McbValue> {
        if self.reader.flag()? {
            return Ok(McbValue::F32(self.reader.f32()?));
        }
        let text = self.reader.string()?;
        if !self.versioned {
            return Ok(string(text));
        }
        let version = self.reader.u16()?;
        Ok(object(vec![
            ("expression".into(), string(text)),
            ("version".into(), McbValue::UInt(u64::from(version))),
        ]))
    }

    /// `N` expressions.
    fn vec<const N: usize>(&mut self) -> Decoded<McbValue> {
        let mut items = Vec::new();
        for _ in 0..N {
            items.push(self.expression()?);
        }
        Ok(McbValue::Array(items))
    }

    /// `N` floats.
    fn floats<const N: usize>(&mut self) -> Decoded<McbValue> {
        let mut items = Vec::new();
        for _ in 0..N {
            items.push(McbValue::F32(self.reader.f32()?));
        }
        Ok(McbValue::Array(items))
    }

    /// An expression member the twins leave out at `default`.
    fn expression_unless(&mut self, default: f32, members: &mut Members, key: &str) -> Decoded<()> {
        let value = self.expression()?;
        unless(value.is_f32(default), members, key, value);
        Ok(())
    }

    /// A vec3 member the twins leave out at `[0, 0, 0]`.
    fn vec3_unless_zero(&mut self, members: &mut Members, key: &str) -> Decoded<()> {
        let value = self.vec::<3>()?;
        unless(value.is_f32s(&[0.0; 3]), members, key, value);
        Ok(())
    }

    /// A flag member the twins leave out when false.
    fn flag_unless_false(&mut self, members: &mut Members, key: &str) -> Decoded<()> {
        let value = self.reader.flag()?;
        unless(!value, members, key, McbValue::Bool(value));
        Ok(())
    }

    /// An emitter shape's optional `direction`: a string (`inwards`, `outwards`) or a vec3.
    fn shape_direction(&mut self, members: &mut Members) -> Decoded<()> {
        if self.reader.flag()? {
            let direction = if self.reader.flag()? {
                self.vec::<3>()?
            } else {
                string(self.reader.string()?)
            };
            members.push(("direction".into(), direction));
        }
        Ok(())
    }

    fn component(&mut self, name: &str) -> Decoded<McbValue> {
        if name.starts_with("minecraft:emitter_") {
            self.emitter(name)
        } else {
            self.particle(name)
        }
    }

    /// An `emitter_*` component.
    fn emitter(&mut self, name: &str) -> Decoded<McbValue> {
        let mut members = Members::new();
        let m = &mut members;
        match name {
            "minecraft:emitter_initialization" => {
                self.expression_unless(0.0, m, "creation_expression")?;
                self.expression_unless(0.0, m, "per_update_expression")?;
            }
            "minecraft:emitter_lifetime_expression" => {
                self.expression_unless(1.0, m, "activation_expression")?;
                self.expression_unless(0.0, m, "expiration_expression")?;
            }
            "minecraft:emitter_lifetime_looping" => {
                m.push(("active_time".into(), self.expression()?));
                self.expression_unless(0.0, m, "sleep_time")?;
            }
            "minecraft:emitter_lifetime_once" => {
                self.expression_unless(0.0, m, "active_time")?;
            }
            "minecraft:emitter_local_space" => {
                m.push(("position".into(), McbValue::Bool(self.reader.flag()?)));
                m.push(("rotation".into(), McbValue::Bool(self.reader.flag()?)));
                // Not `velocity`: the byte is 00 where the twin has `velocity: true`.
                self.reader.pinned(&[0])?;
            }
            "minecraft:emitter_rate_instant" => {
                m.push(("num_particles".into(), self.expression()?));
            }
            "minecraft:emitter_rate_manual" => {
                m.push(("max_particles".into(), self.expression()?));
            }
            "minecraft:emitter_rate_steady" => {
                m.push(("spawn_rate".into(), self.expression()?));
                m.push(("max_particles".into(), self.expression()?));
            }
            "minecraft:emitter_shape_box" => {
                self.vec3_unless_zero(m, "offset")?;
                m.push(("half_dimensions".into(), self.vec::<3>()?));
                self.flag_unless_false(m, "surface_only")?;
                self.shape_direction(m)?;
            }
            "minecraft:emitter_shape_custom" => {
                self.vec3_unless_zero(m, "offset")?;
                self.vec3_unless_zero(m, "direction")?;
            }
            "minecraft:emitter_shape_disc" => {
                self.vec3_unless_zero(m, "offset")?;
                m.push(("radius".into(), self.expression()?));
                self.flag_unless_false(m, "surface_only")?;
                self.shape_direction(m)?;
                if self.reader.flag()? {
                    let normal = self.vec::<3>()?;
                    unless(normal.is_f32s(&[0.0, 1.0, 0.0]), m, "plane_normal", normal);
                } else {
                    // The string form (`x`, `y`, `z`) is in no vanilla effect.
                    return Err(UNSUPPORTED);
                }
            }
            "minecraft:emitter_shape_entity_aabb" => {
                self.flag_unless_false(m, "surface_only")?;
                self.shape_direction(m)?;
            }
            "minecraft:emitter_shape_point" => {
                self.vec3_unless_zero(m, "offset")?;
                if self.reader.flag()? {
                    m.push(("direction".into(), self.vec::<3>()?));
                }
            }
            "minecraft:emitter_shape_sphere" => {
                self.vec3_unless_zero(m, "offset")?;
                m.push(("radius".into(), self.expression()?));
                self.flag_unless_false(m, "surface_only")?;
                self.shape_direction(m)?;
            }
            _ => return Err(UNSUPPORTED),
        }
        Ok(object(members))
    }

    /// A `particle_*` component.
    fn particle(&mut self, name: &str) -> Decoded<McbValue> {
        let mut members = Members::new();
        let m = &mut members;
        match name {
            "minecraft:particle_appearance_billboard" => self.billboard(m)?,
            "minecraft:particle_appearance_lighting" => {}
            "minecraft:particle_appearance_tinting" => {
                let color = if self.reader.flag()? {
                    // The second flag is 01 in every vanilla gradient (the twins' map of
                    // times); what 00 holds is not shown.
                    self.reader.pinned(&[1])?;
                    let mut gradient = Members::new();
                    for _ in 0..self.reader.count()? {
                        let key = self.reader.string()?;
                        let color = self.vec::<4>()?;
                        insert(&mut gradient, key, color)?;
                    }
                    object(vec![
                        ("gradient".into(), object(gradient)),
                        ("interpolant".into(), self.expression()?),
                    ])
                } else {
                    self.vec::<4>()?
                };
                m.push(("color".into(), color));
            }
            "minecraft:particle_expire_if_not_in_blocks" => {
                return self.reader.list(|reader| Ok(string(reader.string()?)));
            }
            "minecraft:particle_initial_speed" => {
                return if self.reader.flag()? {
                    self.expression()
                } else {
                    self.vec::<3>()
                };
            }
            "minecraft:particle_initial_spin" => {
                self.expression_unless(0.0, m, "rotation")?;
                self.expression_unless(0.0, m, "rotation_rate")?;
            }
            "minecraft:particle_initialization" => {
                self.expression_unless(0.0, m, "per_update_expression")?;
                self.expression_unless(0.0, m, "per_render_expression")?;
            }
            "minecraft:particle_kill_plane" => return self.floats::<4>(),
            "minecraft:particle_lifetime_expression" => {
                if self.reader.flag()? {
                    m.push(("max_lifetime".into(), self.expression()?));
                }
                self.expression_unless(0.0, m, "expiration_expression")?;
            }
            "minecraft:particle_motion_collision" => self.collision(m)?,
            "minecraft:particle_motion_dynamic" => {
                self.vec3_unless_zero(m, "linear_acceleration")?;
                self.expression_unless(0.0, m, "linear_drag_coefficient")?;
                self.expression_unless(0.0, m, "rotation_acceleration")?;
                self.expression_unless(0.0, m, "rotation_drag_coefficient")?;
            }
            "minecraft:particle_motion_parametric" => {
                if self.reader.flag()? {
                    m.push(("relative_position".into(), self.vec::<3>()?));
                }
                if self.reader.flag()? {
                    m.push(("direction".into(), self.vec::<3>()?));
                }
                if self.reader.flag()? {
                    m.push(("rotation".into(), self.expression()?));
                }
            }
            _ => return Err(UNSUPPORTED),
        }
        Ok(object(members))
    }

    fn billboard(&mut self, m: &mut Members) -> Decoded<()> {
        m.push(("size".into(), self.vec::<2>()?));
        if self.reader.flag()? {
            m.push(("facing_camera_mode".into(), string(self.reader.string()?)));
        }
        if self.reader.flag()? {
            let mut direction: Members = vec![("mode".into(), string(self.reader.string()?))];
            if self.reader.flag()? {
                direction.push(("custom_direction".into(), self.vec::<3>()?));
            }
            let threshold = self.reader.f32()?;
            unless(
                threshold.to_bits() == 0,
                &mut direction,
                "min_speed_threshold",
                McbValue::F32(threshold),
            );
            m.push(("direction".into(), object(direction)));
        }
        if self.reader.flag()? {
            let mut uv = Members::new();
            for key in ["texture_width", "texture_height"] {
                let size = self.reader.i32()?;
                unless(size == 1, &mut uv, key, McbValue::Int(i64::from(size)));
            }
            if self.reader.flag()? {
                uv.push(("uv".into(), self.vec::<2>()?));
            }
            if self.reader.flag()? {
                uv.push(("uv_size".into(), self.vec::<2>()?));
            }
            if self.reader.flag()? {
                uv.push(("flipbook".into(), self.flipbook()?));
            }
            m.push(("uv".into(), object(uv)));
        }
        Ok(())
    }

    fn flipbook(&mut self) -> Decoded<McbValue> {
        let mut flipbook: Members = vec![("base_UV".into(), self.vec::<2>()?)];
        let size = self.floats::<2>()?;
        unless(size.is_f32s(&[1.0, 1.0]), &mut flipbook, "size_UV", size);
        flipbook.push(("step_UV".into(), self.floats::<2>()?));
        let rate = self.reader.f32()?;
        unless(
            rate.to_bits() == 0,
            &mut flipbook,
            "frames_per_second",
            McbValue::F32(rate),
        );
        flipbook.push(("max_frame".into(), self.expression()?));
        self.flag_unless_false(&mut flipbook, "stretch_to_lifetime")?;
        self.flag_unless_false(&mut flipbook, "loop")?;
        Ok(object(flipbook))
    }

    fn collision(&mut self, m: &mut Members) -> Decoded<()> {
        self.expression_unless(1.0, m, "enabled")?;
        for key in ["collision_drag", "coefficient_of_restitution"] {
            let value = self.reader.f32()?;
            unless(value.to_bits() == 0, m, key, McbValue::F32(value));
        }
        m.push(("collision_radius".into(), McbValue::F32(self.reader.f32()?)));
        self.flag_unless_false(m, "expire_on_contact")?;
        if self.reader.flag()? {
            let events = self.reader.list(|reader| {
                let mut event = Members::new();
                let min_speed = reader.f32()?;
                unless(
                    min_speed.to_bits() == 2.0_f32.to_bits(),
                    &mut event,
                    "min_speed",
                    McbValue::F32(min_speed),
                );
                event.push(("event".into(), string(reader.string()?)));
                Ok(object(event))
            })?;
            m.push(("events".into(), events));
        }
        Ok(())
    }

    fn curves(&mut self) -> Decoded<McbValue> {
        let mut curves = Members::new();
        for _ in 0..self.reader.count()? {
            let name = self.reader.string()?;
            // A field between the name and the type no vanilla curve sets.
            self.reader.pinned(&[0])?;
            let mut curve: Members = vec![("type".into(), string(self.reader.string()?))];
            let mut nodes = Vec::new();
            for _ in 0..self.reader.count()? {
                nodes.push(self.expression()?);
            }
            curve.push(("nodes".into(), McbValue::Array(nodes)));
            curve.push(("input".into(), self.expression()?));
            curve.push(("horizontal_range".into(), self.expression()?));
            insert(&mut curves, name, object(curve))?;
        }
        Ok(object(curves))
    }

    fn events(&mut self) -> Decoded<McbValue> {
        let mut events = Members::new();
        for _ in 0..self.reader.count()? {
            let name = self.reader.string()?;
            // The one vanilla event, `splash` of `water_drip`: a `particle_effect` with its
            // effect and type. The bytes around the two strings are the event's other
            // members, all unset there.
            self.reader.pinned(&[1, 0, 0, 0, 0])?;
            let effect = string(self.reader.string()?);
            let kind = string(self.reader.string()?);
            self.reader.pinned(&[1, 0, 0, 0, 0, 0, 0, 0, 0])?;
            let particle = object(vec![("effect".into(), effect), ("type".into(), kind)]);
            insert(
                &mut events,
                name,
                object(vec![("particle_effect".into(), particle)]),
            )?;
        }
        Ok(object(events))
    }
}
