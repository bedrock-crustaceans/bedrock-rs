//! What a render controller draws, against the client entities and attachables that use it.
//!
//! A controller names the slots of its user: `geometry.<name>`, `texture.<name>` and
//! `material.<name>` are keys of the user's `geometry`, `textures` and `materials` maps
//! A member, or an element of one of the controller's arrays, that is such a name
//! and nothing else is checked here against every user of the controller; a name inside a
//! longer expression is the game's to resolve when the expression runs, and is not looked at
//! A controller nothing uses is not checked, and a member is told once, at
//! the controller, however many of its users lack its slot.
//!
//! The game lower-cases a controller file before it reads it, and so does the model; a
//! finding names the controller, the array and the bone pattern as the file spells them,
//! read from the file's own text when the pack holds it ([`Written`]).
//!
//! The cost is, per controller in use, the slot names of its users and its own members.
//!
//! A controller that gives two textures to a material that needs three (one with both
//! the `GLINT` and the `MULTIPLICATIVE_TINT` define) is told with the game's text.

use bedrock_addon_core::diagnostic::builders;
use bedrock_addon_core::diagnostic::codes as core_codes;
use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};

use bedrock_addon_core::diagnostic::{JsonPath, Severity, sentence};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::MolangSource;
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use indexmap::IndexMap;

use super::super::{At, Findings, LinkCx, canonical, unresolved};
use super::user::{User, lowered_keys};
use super::{plain_reference, user_render};
use crate::families::render_controller::RenderController;

/// The kind of a finding about a key of `geometry` of a controller's user.
const GEOMETRY_SLOT: &str = "geometry slot";
/// The kind of a finding about a key of `textures` of a controller's user.
const TEXTURE_SLOT: &str = "texture slot";
/// The kind of a finding about a key of `materials` of a controller's user.
const MATERIAL_SLOT: &str = "material slot";

/// The game's text for a controller that gives a material too few textures: the controller's
/// name.
const NEEDS_THREE_TEXTURES: &str = "Render Controller: {} provides 2 textures, but using a Glint and Multiplicative-Tint material which requires 3 textures.";

/// The slot names of one user, in lower case as the game keys them.
#[derive(Debug)]
struct Slots {
    geometry: HashSet<String>,
    textures: HashSet<String>,
    /// Each material slot with the material it names, as the user's family reads the name.
    materials: HashMap<String, String>,
}

/// The users of each render controller, by the controller's name in lower case.
#[derive(Debug, Default)]
pub(super) struct Uses {
    slots: Vec<Slots>,
    of: HashMap<String, Vec<usize>>,
}

impl Uses {
    /// Counts `user` as a user of each render controller it names.
    pub(super) fn add(&mut self, user: &User<'_>) {
        if user.render_controllers.is_empty() {
            return;
        }
        let index = self.slots.len();
        self.slots.push(Slots {
            geometry: lowered_keys(user.geometry),
            textures: lowered_keys(user.textures),
            materials: user
                .materials
                .into_iter()
                .flatten()
                .map(|(key, value)| {
                    (
                        key.to_ascii_lowercase(),
                        user.folded(value.as_str()).into_owned(),
                    )
                })
                .collect(),
        });
        for entry in user.render_controllers {
            let Some(name) = entry.controller() else {
                continue;
            };
            let by = self.of.entry(name.as_str().to_ascii_lowercase());
            let by = by.or_default();
            if by.last() != Some(&index) {
                by.push(index);
            }
        }
    }
}

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, all: &Uses) {
    if all.of.is_empty() {
        return;
    }
    // The text of each controller file a finding is told in, parsed and indexed once.
    let mut files: HashMap<(Option<usize>, PackPath), Option<FileText>> = HashMap::new();
    for (id, document) in cx.resource().render_controllers() {
        let Some(indices) = all.of.get(canonical(id).as_ref()) else {
            continue;
        };
        let users: Vec<&Slots> = indices
            .iter()
            .filter_map(|index| all.slots.get(*index))
            .collect();
        let origin = document.origin();
        let key = origin.key.as_deref().unwrap_or(id.as_str());
        let controller = document.value();
        let mut found = slots(controller, &users);
        if needs_three_textures(cx, controller, &users) {
            found.push(Found::TwoTextures);
        }
        if found.is_empty() {
            continue;
        }
        let file = files
            .entry((origin.pack.index(), origin.path.clone()))
            .or_insert_with(|| {
                let bytes = cx.bytes(PackSide::Resource, &origin.pack, &origin.path, out)?;
                let value = bedrock_addon_core::json::from_slice_jsonc::<Value>(&bytes).ok()?;
                Some(FileText::new(value))
            });
        let written = Written::new(file.as_ref(), key);
        for finding in found {
            tell(out, origin, &written, finding);
        }
    }
}

/// What a rule found on one controller, before it is given its place in the file.
#[derive(Debug)]
enum Found {
    /// `geometry` names a slot a user lacks.
    Geometry(String),
    /// `textures[index]` does.
    Texture(usize, String),
    /// A `materials` entry does: the slot.
    Material(String),
    /// The element `index` of the array `name` of `arrays.<kind>` does.
    Element(&'static str, &'static str, String, usize, String),
    /// A material is given fewer textures than it needs.
    TwoTextures,
}

/// How many of the users of one controller have each slot name.
#[derive(Debug)]
struct Counts<'u> {
    users: usize,
    geometry: HashMap<&'u str, usize>,
    textures: HashMap<&'u str, usize>,
    materials: HashMap<&'u str, usize>,
}

impl<'u> Counts<'u> {
    fn new(users: &[&'u Slots]) -> Self {
        let mut counts = Self {
            users: users.len(),
            geometry: HashMap::new(),
            textures: HashMap::new(),
            materials: HashMap::new(),
        };
        for user in users {
            for slot in &user.geometry {
                *counts.geometry.entry(slot).or_default() += 1;
            }
            for slot in &user.textures {
                *counts.textures.entry(slot).or_default() += 1;
            }
            for slot in user.materials.keys() {
                *counts.materials.entry(slot).or_default() += 1;
            }
        }
        counts
    }

    /// The slot `source` names plainly with `prefix`, when one of the users lacks it.
    fn lacking(&self, source: &str, prefix: &str) -> Option<String> {
        let slot = plain_reference(source, prefix)?;
        let have = match prefix {
            GEOMETRY => &self.geometry,
            TEXTURE => &self.textures,
            _ => &self.materials,
        };
        (have.get(slot.as_str()).copied().unwrap_or(0) < self.users).then_some(slot)
    }
}

/// The prefix of a geometry slot's name in a controller.
const GEOMETRY: &str = "geometry.";
/// The prefix of a texture slot's name in a controller.
const TEXTURE: &str = "texture.";
/// The prefix of a material slot's name in a controller.
const MATERIAL: &str = "material.";

/// The render-controller rule on one controller: its members and array elements that name a
/// slot one of `users` lacks, in the order they are told.
fn slots(controller: &RenderController, users: &[&Slots]) -> Vec<Found> {
    let counts = Counts::new(users);
    let mut found = Vec::new();
    if let Some(slot) = counts.lacking(controller.geometry.as_str(), GEOMETRY) {
        found.push(Found::Geometry(slot));
    }
    for (index, texture) in controller.textures.iter().enumerate() {
        if let Some(slot) = counts.lacking(texture.as_str(), TEXTURE) {
            found.push(Found::Texture(index, slot));
        }
    }
    // Each slot once: its entries are found in the file's text.
    let mut seen: HashSet<&str> = HashSet::new();
    for (_, material) in &controller.materials {
        if !seen.insert(material.as_str()) {
            continue;
        }
        if let Some(slot) = counts.lacking(material.as_str(), MATERIAL) {
            found.push(Found::Material(slot));
        }
    }
    let Some(arrays) = &controller.arrays else {
        return found;
    };
    let kinds: [(_, _, &IndexMap<String, Vec<MolangSource>>); 3] = [
        ("geometries", GEOMETRY, &arrays.geometries),
        ("textures", TEXTURE, &arrays.textures),
        ("materials", MATERIAL, &arrays.materials),
    ];
    for (member, prefix, lists) in kinds {
        for (name, list) in lists {
            // An array often repeats a name: each name is asked once.
            let mut asked: HashMap<&str, Option<String>> = HashMap::new();
            for (index, element) in list.iter().enumerate() {
                let slot = asked
                    .entry(element.as_str())
                    .or_insert_with(|| counts.lacking(element.as_str(), prefix));
                if let Some(slot) = slot {
                    found.push(Found::Element(
                        member,
                        prefix,
                        name.clone(),
                        index,
                        slot.clone(),
                    ));
                }
            }
        }
    }
    found
}

/// The text of one controller file, with the spelling of each controller's key by its
/// lower-cased form (the last of two that fold alike, as the reader keeps it).
#[derive(Debug)]
struct FileText {
    value: Value,
    keys: HashMap<String, String>,
}

impl FileText {
    fn new(value: Value) -> Self {
        let controllers = member_of(Some(&value), "render_controllers").map(|(_, value)| value);
        let keys = controllers
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|object| object.iter())
            .map(|(key, _)| (key.as_str().to_ascii_lowercase(), key.as_str().to_owned()))
            .collect();
        Self { value, keys }
    }
}

/// One controller as its file writes it: the key it has there and its members, when the
/// pack holds the file's text. The model has both lower-cased. The names of its arrays and
/// the entries of its `materials` are indexed on their first use, once.
#[derive(Debug)]
struct Written<'v> {
    key: &'v str,
    members: Option<&'v Value>,
    /// Each array's name as written, by its kind (as written) and its lower-cased name.
    arrays: OnceCell<HashMap<(&'v str, String), &'v str>>,
    /// The paths of the `materials` entries that name each slot, in the file's order.
    materials: OnceCell<HashMap<String, Vec<JsonPath>>>,
}

/// The member of the object `value` whose name is `folded` without ASCII case (the last such
/// member, as the reader keeps it): its name as written and its value. A scan of the object:
/// for the few members of a controller, never for a map the author sizes.
fn member_of<'v>(value: Option<&'v Value>, folded: &str) -> Option<(&'v str, &'v Value)> {
    value?
        .as_object()?
        .iter()
        .filter(|(name, _)| name.as_str().eq_ignore_ascii_case(folded))
        .last()
        .map(|(name, value)| (name.as_str(), value))
}

impl<'v> Written<'v> {
    /// The controller `key` (as the model has it) in `file`.
    fn new(file: Option<&'v FileText>, key: &'v str) -> Self {
        let found = file.and_then(|file| {
            let written = file.keys.get(&key.to_ascii_lowercase())?;
            let (_, controllers) = member_of(Some(&file.value), "render_controllers")?;
            let members = controllers.as_object()?.get(written)?;
            Some((written.as_str(), members))
        });
        Self {
            key: found.map_or(key, |(written, _)| written),
            members: found.map(|(_, members)| members),
            arrays: OnceCell::new(),
            materials: OnceCell::new(),
        }
    }

    /// The path of the controller's member `steps` leads to, each step spelled as the file
    /// has it where the file has it.
    fn path(&self, steps: &[&str]) -> JsonPath {
        let mut path = JsonPath::new();
        path.push("render_controllers");
        path.push(self.key);
        let mut at = self.members;
        for step in steps {
            if let Some((name, value)) = member_of(at, step) {
                path.push(name);
                at = Some(value);
            } else {
                path.push(step);
                at = None;
            }
        }
        path
    }

    /// The path of the element `index` of the array `name` (as the model has it) of
    /// `arrays.<kind>`.
    fn element(&self, kind: &str, name: &str, index: usize) -> JsonPath {
        let arrays = self.arrays.get_or_init(|| {
            let mut names = HashMap::new();
            let all = member_of(self.members, "arrays").map(|(_, value)| value);
            let kinds = all
                .and_then(Value::as_object)
                .into_iter()
                .flat_map(|object| object.iter());
            for (kind, lists) in kinds {
                for (name, _) in lists
                    .as_object()
                    .into_iter()
                    .flat_map(|object| object.iter())
                {
                    // The last of two names that fold alike, as the reader keeps it.
                    names.insert(
                        (kind.as_str(), name.as_str().to_ascii_lowercase()),
                        name.as_str(),
                    );
                }
            }
            names
        });
        let mut path = self.path(&["arrays", kind]);
        // The kind as the file spells it is the path's last step.
        let spelled = member_of(
            member_of(self.members, "arrays").map(|(_, value)| value),
            kind,
        )
        .map(|(kind, _)| kind);
        let written = spelled.and_then(|kind| arrays.get(&(kind, name.to_ascii_lowercase())));
        path.push(written.copied().unwrap_or(name));
        path.push_index(index);
        path
    }

    /// The entries of `materials` that name `slot`: the path of each, element by element and
    /// inside an element in the file's order. Empty when the file's text is not at hand.
    fn material_entries(&self, slot: &str) -> Vec<JsonPath> {
        let entries = self.materials.get_or_init(|| {
            let mut entries: HashMap<String, Vec<JsonPath>> = HashMap::new();
            let Some((name, list)) = member_of(self.members, "materials") else {
                return entries;
            };
            let mut at = self.path(&[]);
            at.push(name);
            for (index, element) in list.as_array().into_iter().flatten().enumerate() {
                let members = element
                    .as_object()
                    .into_iter()
                    .flat_map(|object| object.iter());
                for (bone, value) in members {
                    let named = value
                        .as_string()
                        .and_then(|text| plain_reference(text.as_str(), MATERIAL));
                    if let Some(named) = named {
                        let mut path = at.clone();
                        path.push_index(index);
                        path.push(bone.as_str());
                        entries.entry(named).or_default().push(path);
                    }
                }
            }
            entries
        });
        entries.get(slot).cloned().unwrap_or_default()
    }
}

/// Tells `finding` of `controller` at its place in the file.
fn tell(out: &mut Findings, origin: &Origin, written: &Written<'_>, finding: Found) {
    let slot_at = |path: JsonPath, kind: &'static str, slot: &str| {
        // A slot the user lacks is a Molang variable that is not defined: the controller's
        // expression evaluates to a value of another type, the game reports the failure to the
        // content log at the error level, and draws with the type's default value. The
        // document is kept, so this is a kept error.
        let at = At::new(origin, path, Severity::Error);
        unresolved(at, kind, slot, None).into_kept()
    };
    match finding {
        Found::Geometry(slot) => {
            out.push(slot_at(written.path(&["geometry"]), GEOMETRY_SLOT, &slot));
        }
        Found::Texture(index, slot) => {
            let mut path = written.path(&["textures"]);
            path.push_index(index);
            out.push(slot_at(path, TEXTURE_SLOT, &slot));
        }
        Found::Material(slot) => {
            let mut entries = written.material_entries(&slot);
            if entries.is_empty() {
                // Without the file's text the entry is not known: the member.
                entries.push(written.path(&["materials"]));
            }
            for path in entries {
                out.push(slot_at(path, MATERIAL_SLOT, &slot));
            }
        }
        Found::Element(member, prefix, name, index, slot) => {
            let path = written.element(member, &name, index);
            let kind = match prefix {
                GEOMETRY => GEOMETRY_SLOT,
                TEXTURE => TEXTURE_SLOT,
                _ => MATERIAL_SLOT,
            };
            out.push(slot_at(path, kind, &slot));
        }
        Found::TwoTextures => {
            // The game logs the line at the error level when the entity is drawn and drops no
            // document: a kept error.
            let at = At::new(origin, written.path(&["textures"]), Severity::Error);
            out.push(
                at.diagnostic(builders::worded(
                    &core_codes::INVALID_VALUE,
                    &core_codes::INVALID_VALUE_WORDED,
                    Some(sentence(
                        NEEDS_THREE_TEXTURES,
                        &[origin.key.as_deref().unwrap_or(written.key).to_owned()],
                    ))
                    .as_deref(),
                ))
                .into_kept(),
            );
        }
    }
}

/// The texture-count rule on one controller: two textures, and a user whose material for one of the
/// controller's material slots has both defines.
fn needs_three_textures(cx: &LinkCx<'_>, controller: &RenderController, users: &[&Slots]) -> bool {
    if controller.textures.len() != 2 {
        return false;
    }
    let drawn: Vec<String> = controller
        .materials
        .iter()
        .filter_map(|(_, material)| plain_reference(material.as_str(), MATERIAL))
        .collect();
    let needs_three = |user: &&Slots| {
        drawn.iter().any(|slot| {
            user.materials
                .get(slot)
                .and_then(|name| user_render::resolved(cx, name))
                .is_some_and(|material| {
                    let defined = |define: &str| {
                        material
                            .defines
                            .iter()
                            .any(|written| written.split(' ').next() == Some(define))
                    };
                    defined("GLINT") && defined("MULTIPLICATIVE_TINT")
                })
        })
    };
    users.iter().any(needs_three)
}
