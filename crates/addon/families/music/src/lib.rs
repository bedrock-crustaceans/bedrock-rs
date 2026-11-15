//! `sounds/music_definitions.json`: which sound event each music situation plays, and the
//! pause between tracks.
//!
//! The client merges the packs' copies as JSON and reads the result with a jsoncpp that coerces
//! every member and refuses none. The merge stays a raw JSON merge ([`LOADER`]); what is typed is
//! the merged document, read by the derived reader of [`MusicDefinition`] under
//! `Scalars::Fork` and a silent `null`, with the root, a map keyed by music name, read by a hand
//! `FromJson` (a name that is empty, or a value that is no object, is kept as written).

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;

use facet::Facet;
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::document::unversioned::{lone, version_at};
use bedrock_addon_core::document::{
    Content, Document, JsonContent, Merge, Merged, Origin, Placed, RawLayer, Rejected,
};
use bedrock_addon_core::id::MusicDefinitionId;
use bedrock_addon_core::json::jsoncpp::fork;
use bedrock_addon_core::json::kit::write::{self, Object};
use bedrock_addon_core::json::kit::{Node, Present, Read, Written};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle};
use bedrock_addon_core::load::sound::{
    SoundCopy, layers, located, not_json, parse_copy, path_member, read_json, read_merged,
    readable, write_document,
};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_core::schema::{self, FromJson, Schema};
use bedrock_addon_core::version::ENGINE_VERSION;

/// Game ticks per second, for the delays vanilla stores as ticks.
const TICKS_PER_SECOND: f32 = 20.0;

/// The music definitions of the resource packs, `sounds/music_definitions.json`
/// (`#[doc(alias)]` `music_definitions.json`): one definition per music name (`game`, `menu`,
/// a biome's music key).
///
/// The value writes its typed members: each definition's members in the order of
/// [`MusicDefinition`]'s fields, a number as the shortest decimal of its float, then the
/// members the client does not read. A member the document did not write is written only
/// once its value is not what an absent member reads as, since the client merges the copies
/// of the file member by member. [`json`](Self::json) is the document the value was read
/// from.
#[doc(alias = "music_definitions.json")]
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct MusicDefinitions {
    /// The definitions by music name, in file order.
    pub definitions: IndexMap<MusicDefinitionId, MusicDefinition>,
    /// The members of the root that are no definition: one under the empty name, and one
    /// whose value is no object. Written after the definitions.
    pub extra: UnknownMembers,
    /// The root as written when it is no object and not `null` (an array, a number, a string,
    /// a bool): the value reads nothing of it. It is written as it is while the value holds no
    /// definition and no member of [`extra`](Self::extra), since the client's merge treats it
    /// otherwise than an empty object.
    pub foreign_root: Option<Value>,
    /// The document as read; left out of equality.
    pub(crate) json: Written,
}

impl MusicDefinitions {
    /// The document as JSON, as it was read (for a stack's value, the copies as the client
    /// merges them): what the typed members were read from. An edit of the value does not
    /// change it; the value writes its typed members.
    pub fn json(&self) -> &Value {
        static NULL: Value = Value::NULL;
        self.json.as_ref().unwrap_or(&NULL)
    }

    /// The definition of the music name `name`.
    pub fn get(&self, name: &str) -> Option<&MusicDefinition> {
        self.definitions.get(&MusicDefinitionId::parse(name))
    }

    /// The value of a document, read as it is; the findings of the read are dropped.
    /// `Content::parse` reads a file with them.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "the signature of the other two documents' `from_json`"
    )]
    pub fn from_json(json: Value) -> Self {
        Self::read(&json, &mut Diagnostics::default())
    }

    /// Reads a document.
    pub(crate) fn read(json: &Value, diagnostics: &mut Diagnostics) -> Self {
        read_merged(json, diagnostics)
    }

    /// The document the value writes: every definition, then the members of
    /// [`extra`](Self::extra); the [`foreign_root`](Self::foreign_root) while there is none.
    ///
    /// # Errors
    ///
    /// `EncodeError::NotRepresentable` for a delay that is no finite float.
    pub fn to_value(&self) -> Result<Value, EncodeError> {
        if let Some(root) = &self.foreign_root
            && self.definitions.is_empty()
            && self.extra.is_empty()
        {
            return Ok(root.clone());
        }
        let mut members = Vec::with_capacity(self.definitions.len() + self.extra.len());
        for (name, definition) in &self.definitions {
            members.push((
                name.as_str(),
                write::in_member(name.as_str(), definition.to_value())?,
            ));
        }
        members.extend(self.extra.iter().map(|(key, value)| (key, value.clone())));
        Ok(write::object(members))
    }
}

/// One music definition.
///
/// Read as the game reads it: each member coerced as jsoncpp's `asString`, `asFloat` and `asBool`
/// coerce it, a `null` the default, and the members it does not read kept in
/// [`extra`](Self::extra) and told as an information.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(schema::after_read = after_read)]
#[non_exhaustive]
pub struct MusicDefinition {
    /// `override_interruptible`: the `name` of each entry, the sound categories this music
    /// may interrupt. An entry that is no object, or has no `name`, is `""`. An object of
    /// entries (read in sorted key order) is kept as written in [`extra`](Self::extra), since
    /// the client's merge recurses into it where it replaces an array whole.
    #[facet(default, schema::read_with = interruptible)]
    pub override_interruptible: Vec<String>,
    /// `replace`, as written: a truthy value makes this definition replace the lower packs'
    /// instead of merging into it. Only the merge reads it.
    #[facet(schema::reads_null, schema::read_with = replace)]
    pub replace: Option<Value>,
    /// `event_name`: the sound event that holds the tracks; `""` when absent.
    #[facet(default)]
    pub event_name: String,
    /// `min_delay`, in seconds; 0.0 when absent.
    #[facet(default)]
    pub min_delay: f32,
    /// `max_delay`, in seconds; 0.0 when absent.
    #[facet(default)]
    pub max_delay: f32,
    /// `play_in_order`: play the event's sounds as a playlist; `false` when absent.
    #[facet(default)]
    pub play_in_order: bool,
    /// The members the client does not read, and an `override_interruptible` written as an
    /// object, as written.
    pub extra: UnknownMembers,
    /// The members the definition was read with.
    #[facet(skip, rename = "\u{0}read")]
    pub(crate) read: Present,
}

impl MusicDefinition {
    /// The definition as its JSON object.
    fn to_value(&self) -> Result<Value, EncodeError> {
        let mut out = Object::new(&self.read, &self.extra);
        out.member("event_name", self.event_name.is_empty(), || {
            Ok(Value::from(self.event_name.as_str()))
        })?;
        out.member("min_delay", self.min_delay == 0.0, || {
            write::float(self.min_delay)
        })?;
        out.member("max_delay", self.max_delay == 0.0, || {
            write::float(self.max_delay)
        })?;
        out.member("play_in_order", !self.play_in_order, || {
            Ok(Value::from(self.play_in_order))
        })?;
        out.member(
            "override_interruptible",
            self.override_interruptible.is_empty(),
            || {
                Ok(write::array(self.override_interruptible.iter().map(
                    |name| write::object([("name", Value::from(name.as_str()))]),
                )))
            },
        )?;
        if let Some(replace) = &self.replace {
            out.member("replace", false, || Ok(replace.clone()))?;
        }
        Ok(out.finish())
    }

    /// The shortest pause in game ticks, as vanilla stores it: `min_delay` × 20, truncated.
    #[allow(clippy::cast_possible_truncation, reason = "vanilla truncates")]
    pub fn min_delay_ticks(&self) -> i32 {
        (self.min_delay * TICKS_PER_SECOND) as i32
    }

    /// The longest pause in game ticks, as vanilla stores it: `max_delay` × 20, truncated.
    #[allow(clippy::cast_possible_truncation, reason = "vanilla truncates")]
    pub fn max_delay_ticks(&self) -> i32 {
        (self.max_delay * TICKS_PER_SECOND) as i32
    }
}

/// The situation a music name stands for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u8)]
pub enum MusicCategory {
    /// `menu`.
    Menu = 0,
    /// `game`, `creative` and `water`.
    Overworld = 1,
    /// `nether`.
    Nether = 2,
    /// `end`.
    End = 3,
    /// `endboss`.
    Endboss = 4,
    /// `credits`.
    Credits = 5,
}

impl MusicCategory {
    /// The category vanilla gives a music definition of this name; `None` for every other
    /// name, which leaves the category as it was.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "menu" => Self::Menu,
            "game" | "creative" | "water" => Self::Overworld,
            "nether" => Self::Nether,
            "end" => Self::End,
            "endboss" => Self::Endboss,
            "credits" => Self::Credits,
            _ => return None,
        })
    }
}

/// The merge predicate of the music file: a copy's level with a truthy `replace` member
/// replaces the lower level whole.
///
/// The predicate tests that the member is there, then reads it as a boolean defaulting to
/// `false`.
pub(crate) fn replace_predicate(dst: &mut Value, src: &Value) -> bool {
    if fork::has_member(src, "replace") && fork::as_bool(fork::member(src, "replace"), false) {
        *dst = src.clone();
        return true;
    }
    false
}

/// The merged document: one definition per member of the root object, and the members that are
/// none. A root that is no object has none and is kept as written.
impl FromJson for MusicDefinitions {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let mut out = Self {
            foreign_root: (!node.value().is_object() && !node.value().is_null())
                .then(|| node.value().clone()),
            json: Written(Some(node.value().clone())),
            ..Self::default()
        };
        if !node.value().is_object() {
            return Some(out);
        }
        for (name, member) in read.object(node)?.iter() {
            let at = member.path().clone();
            if name.is_empty() {
                // The game writes this line to the content log at the warning level.
                read.push(Diagnostic::of(&codes::NO_NAME).at(at));
            } else if member.value().is_object() {
                let definition = MusicDefinition::from_json(&member, read)?;
                out.definitions
                    .insert(MusicDefinitionId::parse(name), definition);
                continue;
            } else {
                // The game registers an empty music definition.
                read.push(Diagnostic::info(&core_codes::INVALID_VALUE).at(at));
            }
            out.extra.insert(name.to_owned(), member.value().clone());
        }
        Some(out)
    }
}

/// `override_interruptible`: the `name` of each entry as jsoncpp reads a list, an array's elements
/// or an object's values in key order, and nothing for any other value.
fn interruptible(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    fork::elements(node.value())
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let at = node.path().index(index).child("name");
            String::from_json(&Node::new(fork::member(entry, "name"), at), read)
        })
        .collect()
}

/// `replace`, as written; read as a boolean for the information it tells where it is not one,
/// since the merge reads it that way.
fn replace(node: &Node<'_>, read: &mut Read<'_>) -> Option<Value> {
    bool::from_json(node, read)?;
    Some(node.value().clone())
}

/// What a definition's members cannot say alone: an `override_interruptible` written as an object
/// is kept as written (the merge recurses into an object), and a minimum above the maximum is
/// kept with an information, as the game stores the values as given.
fn after_read(definition: &mut MusicDefinition, node: &Node<'_>, read: &mut Read<'_>) {
    let entries = fork::member(node.value(), "override_interruptible");
    if entries.is_object() {
        definition
            .extra
            .insert("override_interruptible".to_owned(), entries.clone());
    }
    if definition.min_delay > definition.max_delay {
        let at = node.path().child("min_delay");
        read.push(Diagnostic::info(&core_codes::INVALID_VALUE).at(at));
    }
}

/// Where a pack keeps its music definitions.
pub(crate) const MUSIC_PATH: &str = "sounds/music_definitions.json";

impl bedrock_addon_core::__private::Sealed for MusicDefinitions {}

impl Merge for MusicDefinitions {
    /// A music name.
    type Key = MusicDefinitionId;
}

/// Merges the copies of `sounds/music_definitions.json` and reads the result.
fn merge_music(copies: Vec<SoundCopy>, _: &StackCx<'_>) -> Merged<MusicDefinitions, RawLayer> {
    let mut diagnostics = Diagnostics::default();
    let copies = readable(MUSIC_PATH, copies, &mut diagnostics);
    let mut origins: IndexMap<MusicDefinitionId, Origin> = IndexMap::new();
    let mut target: Option<Value> = None;
    for (origin, json, _) in &copies {
        for name in fork::member_names(json) {
            origins.insert(MusicDefinitionId::parse(name), origin.clone());
        }
        match &mut target {
            None => target = Some(json.clone()),
            Some(target) => {
                fork::recursive_merge(target, json, Some(replace_predicate));
            }
        }
    }
    let merged = target.unwrap_or(Value::NULL);
    let mut found = Diagnostics::default();
    let value = MusicDefinitions::read(&merged, &mut found);
    let top = copies.last().map(|(origin, _, _)| origin);
    for mut finding in found {
        let owner = path_member(&finding.location.json_path, 0)
            .and_then(|name| origins.get(&MusicDefinitionId::parse(name)))
            .or(top);
        if let Some(origin) = owner {
            finding.location.pack = Some(origin.pack.clone());
            finding.location.path = Some(origin.path.clone());
        }
        diagnostics.push(finding);
    }
    origins.retain(|name, _| value.definitions.contains_key(name));
    Merged::new(value, origins, layers(&copies), diagnostics)
}

/// The loader of `rp.music()`: every resource pack's copy of the file, merged as JSON and read
/// once.
///
/// A hand loader rather than `load::merged`: a copy is the file\'s JSON, merged into the lower
/// copies before the one typed read, not a document read on its own.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Merged<MusicDefinitions, RawLayer>, SoundCopy> =
    CollectionLoader {
        walk: Walk::exact_files(&[MUSIC_PATH]),
        parse: parse_copy,
        combine: merge_music,
        needs: &[],
        then: None,
    };

impl Content for MusicDefinitions {
    const NAME: &'static str = "music definitions";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one pack's `sounds/music_definitions.json` on its own. A file that is no JSON is
    /// rejected.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        // A number no double holds is read as the largest finite one, with a warning.
        bedrock_addon_core::document::FiniteText::load(bytes, |bytes| {
            let origin = lone(path);
            let json = match read_json(bytes) {
                Ok(json) => json,
                Err(message) => return Err(not_json(origin, &message)),
            };
            let mut found = Diagnostics::default();
            let value = Self::read(&json, &mut found);
            let mut diagnostics = Diagnostics::default();
            located(found, &origin, &mut diagnostics);
            Ok(Document::new(
                value,
                origin,
                version_at(ENGINE_VERSION),
                diagnostics,
            ))
        })
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_document(&self.to_value()?, out, style)
    }
}

impl JsonContent for MusicDefinitions {}

impl Placed for MusicDefinitions {
    /// `sounds/music_definitions.json`: the one path the client reads.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(MUSIC_PATH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The keys of a written object, in order.
    fn keys(value: &Value) -> Vec<&str> {
        value
            .as_object()
            .expect("an object")
            .iter()
            .map(|(key, _)| key.as_str())
            .collect()
    }

    /// The members the derived read takes are the members the writer writes: a definition whose
    /// every member is set writes exactly the keys of the type's fields.
    #[test]
    fn every_key_the_reader_knows_is_a_member_the_writer_writes() {
        use facet::{Type, UserType};
        let definition = MusicDefinition {
            event_name: "music.game".to_owned(),
            min_delay: 1.0,
            max_delay: 2.0,
            play_in_order: true,
            override_interruptible: vec!["hostile".to_owned()],
            replace: Some(Value::from(true)),
            ..MusicDefinition::default()
        };
        let Type::User(UserType::Struct(shape)) = MusicDefinition::SHAPE.ty else {
            unreachable!("a struct");
        };
        let mut read: Vec<&str> = shape
            .fields
            .iter()
            .map(|field| field.name)
            .filter(|name| !matches!(*name, "extra" | "read"))
            .collect();
        read.sort_unstable();
        let written = definition.to_value().expect("written");
        let mut written = keys(&written);
        written.sort_unstable();
        assert_eq!(written, read);
    }
}
