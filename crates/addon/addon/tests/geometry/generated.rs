//! Generated tests: files and pack forests from a fixed seed, checked against a
//! property instead of a hand-written expectation.

use bedrock_addon::document::Content;
use bedrock_addon::families::geometry::{Geometry, GeometryFile, Locator};
use bedrock_addon::json::EncodeError;

use crate::common::{load, loaded, stack};

/// A xorshift generator: the same files on every run.
struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// A number below `n`.
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    /// One time in `n`.
    fn one_in(&mut self, n: u64) -> bool {
        self.below(n) == 0
    }

    fn pick<'a>(&mut self, of: &[&'a str]) -> &'a str {
        of[usize::try_from(self.below(of.len() as u64)).unwrap()]
    }
}

/// `, "<name>": <value>` one time in `n`, the value picked from `values`.
fn member(r: &mut Random, n: u64, name: &str, values: &[&str]) -> String {
    if r.one_in(n) {
        format!(r#", "{name}": {}"#, r.pick(values))
    } else {
        String::new()
    }
}

/// A file older than 1.12.0 over the member grid of the review: locators as a vector or an
/// object with and without `rotation`, cubes with and without members, every side-car member
/// present or not, in every version range (and without a version).
fn legacy_file(r: &mut Random) -> String {
    let version = r.pick(&[
        "",
        r#""format_version": "1.0.0", "#,
        r#""format_version": "1.1.0", "#,
        r#""format_version": "1.7.0", "#,
        r#""format_version": "1.8.0", "#,
        r#""format_version": "1.10.0", "#,
        r#""format_version": "1.11.0", "#,
    ]);
    let mut geometries = Vec::new();
    for g in 0..=r.below(3) {
        let parent = if r.one_in(4) { ":geometry.parent" } else { "" };
        let mut bones = Vec::new();
        for b in 0..r.below(3) {
            let mut cubes = Vec::new();
            for _ in 0..r.below(3) {
                cubes.push(
                    format!(
                        r#"{{"n": 0{}{}{}{}}}"#,
                        member(r, 2, "origin", &["[0, 0, 0]", "[-4, 12, -2]"]),
                        member(r, 2, "size", &["[1, 1, 1]", "[8, 12, 5]"]),
                        member(r, 3, "uv", &["[0, 0]", "[16.5, 16]"]),
                        member(r, 4, "mirror", &["true", r#""true""#, r#""maybe""#]),
                    )
                    .replace(r#""n": 0, "#, "")
                    .replace(r#""n": 0"#, ""),
                );
            }
            let mut locators = Vec::new();
            for l in 0..r.below(3) {
                locators.push(format!(
                    r#""l{l}": {}"#,
                    r.pick(&[
                        "[1, 2, 3]",
                        r#"{"offset": [0, 1, 0]}"#,
                        r#"{"offset": [0, 1, 0], "rotation": [0, 90, 0]}"#,
                        r#"{"offset": [0, 1, 0], "ignore_inherited_scale": true}"#,
                        r#"{"offset": [0, 1, 0], "rotation": [1, 2, 3], "ignore_inherited_scale": false}"#,
                    ])
                ));
            }
            let cubes = if r.one_in(3) {
                String::new()
            } else {
                format!(r#", "cubes": [{}]"#, cubes.join(", "))
            };
            let locators = if r.one_in(2) {
                String::new()
            } else {
                format!(r#", "locators": {{{}}}"#, locators.join(", "))
            };
            bones.push(format!(
                r#"{{"name": "b{b}"{}{}{}{}{}{}{}{}{cubes}{locators}}}"#,
                member(r, 5, "reset", &["true", "false", r#""true""#]),
                member(r, 5, "neverRender", &["true", "false"]),
                member(r, 5, "mirror", &["true", r#""true""#, r#""no""#]),
                member(r, 6, "parent", &[r#""b0""#, r#""ghost""#]),
                member(r, 6, "bind_pose_rotation", &["[0, 90, 0]"]),
                member(r, 3, "pivot", &["[0, 24, 0]"]),
                member(r, 8, "poly_mesh", &[r#"{"polys": "tri_list"}"#]),
                member(r, 8, "texture_meshes", &[r#"[{"texture": "t"}]"#]),
            ));
        }
        let bones = if r.one_in(4) {
            String::new()
        } else {
            format!(r#", "bones": [{}]"#, bones.join(", "))
        };
        geometries.push(
            format!(
                r#""geometry.g{g}{parent}": {{"n": 0{}{}{}{}{}{bones}}}"#,
                member(r, 3, "texturewidth", &["64", "128"]),
                member(r, 4, "visible_bounds_width", &["2", "0", "1.5"]),
                member(r, 5, "preserve_model_pose", &["true", "false"]),
                member(r, 6, "debug", &["true"]),
                member(r, 10, "cape", &[r#""geometry.cape""#]),
            )
            .replace(r#""n": 0, "#, "")
            .replace(r#""n": 0"#, ""),
        );
    }
    format!("{{{version}{}}}", geometries.join(", "))
}

/// The file with each cube's missing `pivot` filled with the cube's centre: what the reader's
/// 1.12 → 1.14 step gives a cube written without one.
fn with_pivots_filled(mut file: GeometryFile) -> GeometryFile {
    for geometry in &mut file.geometries {
        for bone in geometry.bones.iter_mut().flatten() {
            for cube in bone.cubes.iter_mut().flatten() {
                cube.pivot = Some(cube.pivot());
            }
        }
    }
    file
}

/// For every generated file that loads, `to_json` is either a refusal
/// that names a member, or text that loads again to an equal value. Every fourth file is first
/// changed as a caller would (a cube's `pivot` cleared), and is then equal once the reader has
/// filled the centre back in.
#[test]
fn a_generated_legacy_file_is_written_to_text_that_reads_back_equal_or_is_refused_at_a_member() {
    let mut r = Random(0x9E37_79B9_7F4A_7C15);
    let (mut read_back, mut refused, mut not_loaded) = (0, 0, 0);
    let (mut legacy_shape, mut object_locators) = (0, 0);
    for n in 0..4000 {
        let text = legacy_file(&mut r);
        let Ok(document) = load(&text) else {
            not_loaded += 1;
            continue;
        };
        let mut file = document.into_value();
        if n % 4 == 0 {
            let cube = file
                .geometries
                .iter_mut()
                .flat_map(|g| g.bones.iter_mut().flatten())
                .flat_map(|b| b.cubes.iter_mut().flatten())
                .next();
            if let Some(cube) = cube {
                cube.pivot = None;
            }
        }
        object_locators += usize::from(file.geometries.iter().any(|g| {
            g.bones().iter().any(|b| {
                b.locators
                    .iter()
                    .flatten()
                    .any(|(_, l)| matches!(l, Locator::Placed { rotation: None, .. }))
            })
        }));
        match file.to_json() {
            Err(EncodeError::NotRepresentable { at, .. }) => {
                assert!(!at.is_empty(), "a refusal without a member: {text}");
                refused += 1;
            }
            Err(other) => panic!("{other:?}: {text}"),
            Ok(written) => {
                let again =
                    load(&written).unwrap_or_else(|e| panic!("{text}\nwritten as\n{written}\n{e}"));
                // The legacy shapes have no `pivot` member: the reader's 1.12 -> 1.14 step
                // gives the cleared one back as the cube's centre. The newest shape writes
                // none and reads none.
                let legacy = !written.contains("minecraft:geometry");
                let expected = if legacy {
                    with_pivots_filled(file)
                } else {
                    file
                };
                assert_eq!(
                    again.value().geometries,
                    expected.geometries,
                    "{text}\n{written}"
                );
                assert_eq!(again.value().debug, expected.debug, "{text}");
                assert_eq!(again.to_json().unwrap(), written, "{text}");
                legacy_shape += usize::from(!written.contains("minecraft:geometry"));
                read_back += 1;
            }
        }
    }
    eprintln!(
        "generated legacy files: {read_back} read back ({legacy_shape} in their legacy shape), {refused} refused, {not_loaded} not loaded, {object_locators} with a locator object without rotation"
    );
    // The grid is exercised on every side. The one class of refusal of a file
    // nobody changed (a cube the reader left without a member) is written, so no
    // generated file is refused.
    assert!(read_back > 2000 && legacy_shape > 300 && object_locators > 300);
    assert_eq!(refused, 0);
}

/// A legacy geometry `key` with one bone hanging on `bone_parent`.
fn legacy_member(key: &str, bone_parent: &str) -> String {
    format!(
        r#""{key}": {{"bones": [{{"name": "own"}}, {{"name": "limb", "parent": "{bone_parent}"}}]}}"#
    )
}

/// Over 200 generated two-pack forests of legacy geometries (parents
/// that exist, are missing, form cycles, or are redefined by the other pack), no registered
/// geometry names a parent without having been given what it inherits.
#[test]
fn no_generated_two_pack_forest_registers_a_child_without_what_it_inherits() {
    let mut r = Random(0xD1B5_4A32_D192_ED03);
    let names = ["a", "b", "c", "d", "e"];
    let (mut registered, mut children) = (0, 0);
    let (mut top_whole, mut top_whole_with_parent) = (0, 0);
    for _ in 0..200 {
        let mut packs = Vec::new();
        // Per pack, each name it defines with the parent it names ("" for none).
        let mut defined: Vec<Vec<(&str, &str)>> = Vec::new();
        for _ in 0..2 {
            let mut members = Vec::new();
            let mut own = Vec::new();
            for name in names {
                if r.one_in(3) {
                    continue;
                }
                let parent = r.pick(&["", "a", "b", "c", "d", "e", "nowhere"]);
                own.push((name, parent));
                let key = if parent.is_empty() {
                    format!("geometry.{name}")
                } else {
                    format!("geometry.{name}:geometry.{parent}")
                };
                members.push(legacy_member(&key, r.pick(&["own", "ghost"])));
            }
            packs.push(format!(
                r#"{{"format_version": "1.8.0", "debug": true{}{}}}"#,
                if members.is_empty() { "" } else { ", " },
                members.join(", ")
            ));
            defined.push(own);
        }
        let set = stack(&[
            &[("models/mobs.json", packs[0].as_str())],
            &[("models/mobs.json", packs[1].as_str())],
        ]);
        for (id, document) in set.resource.geometries() {
            let geometry: &Geometry = document.value();
            registered += 1;
            children += usize::from(geometry.parent().is_some());
            assert_eq!(
                geometry.parent().is_some(),
                geometry.inherited.is_some(),
                "{id:?} of\n{}\n{}",
                packs[0],
                packs[1]
            );
            // And it is the top pack's definition whenever the top pack has one.
            let top_has = packs[0].contains(&format!(r#""{}""#, geometry.description.identifier))
                || packs[0].contains(&format!(r#""{}:"#, geometry.description.identifier));
            if top_has {
                assert_eq!(document.origin().pack.index(), Some(0), "{id:?}");
            }
        }
        // The other direction, which the loop above cannot see: a
        // definition of the top pack whose chain is whole must be registered, nothing of
        // the pack below standing in its way. The oracle: the definition of a name is the
        // top pack's when it has one, else the lower pack's; a chain is whole when it ends
        // at a definition without a parent, passing no name twice and no missing one.
        let definition = |name: &str| {
            defined
                .iter()
                .find_map(|pack| pack.iter().find(|(own, _)| *own == name))
                .map(|(_, parent)| *parent)
        };
        let whole = |name: &str| {
            let mut seen = vec![name];
            let mut at = name;
            loop {
                match definition(at) {
                    None => return false,
                    Some("") => return true,
                    Some(parent) if seen.contains(&parent) => return false,
                    Some(parent) => {
                        seen.push(parent);
                        at = parent;
                    }
                }
            }
        };
        for (name, parent) in &defined[0] {
            if !whole(name) {
                continue;
            }
            top_whole += 1;
            top_whole_with_parent += usize::from(!parent.is_empty());
            let found = set
                .resource
                .geometries()
                .iter()
                .find(|(_, document)| {
                    document.value().description.identifier == format!("geometry.{name}")
                })
                .unwrap_or_else(|| {
                    panic!(
                        "geometry.{name} of the top pack is not registered:\n{}\n{}",
                        packs[0], packs[1]
                    )
                });
            assert_eq!(found.1.origin().pack.index(), Some(0), "geometry.{name}");
        }
    }
    eprintln!(
        "generated forests: {registered} registered, {children} with a parent; {top_whole} whole chains from the top pack, {top_whole_with_parent} with a parent"
    );
    assert!(registered > 150 && children > 50);
    assert!(top_whole > 150 && top_whole_with_parent > 30);
    let _ = loaded::<GeometryFile>;
}
