//! Writing [`Block`]: the payload, and the document with its envelope.

use super::component::write_components;
use super::history::BlockFamily;
use super::read::{CEREAL, NEWEST, parse_block};
use super::{Block, Permutation};
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath};
use bedrock_addon_core::document::{Unknown, UnknownMembers};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, Unrepresentable, Value, WriteStyle, merge_unknown, write_json,
};

/// A written document and the findings of the unknown members it left out.
type Written = (Value, Vec<Diagnostic>);
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang;
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{
    Loaded, ParseCx, VersionStruct, Versioned, write_envelope, write_payload,
};

impl Versioned for Block {
    const VERSION: FormatVersion = NEWEST;
}

impl VersionStruct for Block {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_block(payload, cx, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let mut object = facet_value::VObject::new();
        object.insert("description", write_payload(&self.description)?);
        object.insert("components", write_components(&self.components)?);
        if !self.permutations.is_empty() {
            let mut permutations = facet_value::VArray::new();
            for permutation in &self.permutations {
                permutations.push(permutation.write()?);
            }
            object.insert("permutations", permutations);
        }
        if !self.deprecated.is_empty() {
            object.insert("deprecated", write_payload(&self.deprecated)?);
        }
        if !self.deprecated_event_handlers.is_empty() {
            object.insert(
                "deprecated_event_handlers",
                write_payload(&self.deprecated_event_handlers)?,
            );
        }
        if !self.ordered_v1_custom_components.is_empty() {
            object.insert(
                "ordered_v1_custom_components",
                write_payload(&self.ordered_v1_custom_components)?,
            );
        }
        // The unknown members are the document's, written from its table.
        Ok(object.into())
    }
}

impl Block {
    /// The payload as a document of `version` writes it: below 1.26.60 each collision box
    /// written by `min`/`max` is written by `origin`/`size` (the 1.26.20 reading, which the
    /// 1.26.20 step turns back into the same corners), and a list of one box as the
    /// one object (both forms are read alike below 1.26.60, the object by every version).
    pub(crate) fn write_at(&self, version: FormatVersion) -> Result<Value, EncodeError> {
        let mut payload = self.write()?;
        if version >= NEWEST {
            return Ok(payload);
        }
        let Some(object) = payload.as_object_mut() else {
            return Ok(payload);
        };
        legacy_boxes(
            object.get_mut("components"),
            &JsonPath::of_keys(["minecraft:block", "components"]),
        )?;
        for (index, permutation) in object
            .get_mut("permutations")
            .and_then(Value::as_array_mut)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let components = permutation
                .as_object_mut()
                .and_then(|permutation| permutation.get_mut("components"));
            let mut at = JsonPath::of_keys(["minecraft:block", "permutations"]);
            at.push_index(index);
            at.push("components");
            legacy_boxes(components, &at)?;
        }
        Ok(payload)
    }
}

/// A section's written `minecraft:collision_box` as a document below 1.26.60 holds it.
fn legacy_boxes(components: Option<&mut Value>, at: &JsonPath) -> Result<(), EncodeError> {
    let Some(collision) = components
        .and_then(Value::as_object_mut)
        .and_then(|components| components.get_mut("minecraft:collision_box"))
    else {
        return Ok(());
    };
    let Some(boxes) = collision.as_array() else {
        return Ok(());
    };
    let mut written = facet_value::VArray::new();
    for (index, b) in boxes.iter().enumerate() {
        let read: super::component::CollisionBoundingBox = bedrock_addon_core::json::from_str(
            &bedrock_addon_core::json::to_json(b)?,
        )
        .map_err(|_| {
            // The box the writer holds has no reading of its own: named where it is.
            let at = at.child("minecraft:collision_box").index(index);
            EncodeError::NotRepresentable {
                at,
                what: Unrepresentable::Unsupported,
            }
        })?;
        if !read.has_min_max() {
            written.push(b.clone());
            continue;
        }
        let (origin, size) = read.legacy_origin_size();
        let mut object = facet_value::VObject::new();
        object.insert("origin", write_payload(&origin)?);
        object.insert("size", write_payload(&size)?);
        // The box's unknown members, as written.
        for (key, value) in b.as_object().into_iter().flatten() {
            if !matches!(key.as_str(), "origin" | "size" | "min" | "max") {
                object.insert(key.as_str(), value.clone());
            }
        }
        written.push(object);
    }
    *collision = if written.len() == 1 {
        written.into_iter().next().unwrap_or(Value::NULL)
    } else {
        written.into()
    };
    Ok(())
}

impl Permutation {
    /// The permutation as JSON.
    fn write(&self) -> Result<Value, EncodeError> {
        let mut object = facet_value::VObject::new();
        let condition =
            molang::write(&self.condition).map_err(|what| EncodeError::NotRepresentable {
                at: bedrock_addon_core::diagnostic::JsonPath::new(),
                what,
            })?;
        object.insert("condition", condition);
        object.insert("components", write_components(&self.components)?);
        if !self.deprecated.is_empty() {
            object.insert("deprecated", write_payload(&self.deprecated)?);
        }
        if !self.ordered_v1_custom_components.is_empty() {
            object.insert(
                "ordered_v1_custom_components",
                write_payload(&self.ordered_v1_custom_components)?,
            );
        }
        // The unknown members are the document's, written from its table.
        Ok(object.into())
    }
}

impl Block {
    /// The `format_version` the block is written with: its own when it is 1.26.20 or newer (a
    /// 1.26.30 stamp stays 1.26.30), else 1.26.20.
    pub(crate) fn written_version(&self) -> FormatVersion {
        self.source_format_version.max(CEREAL)
    }

    /// The whole document: `format_version` and the payload.
    ///
    /// A block read from a file below 1.26.20 that breaks a rule of 1.26.20 has no text in
    /// that shape (the written text would be read as another block, or refused). It is
    /// written at its own format version, where the game holds it to no such rule, when
    /// that text reads back as it was written ([`Block::at_own_version`]); otherwise it is
    /// `NotRepresentable` at the member the rule names, where the load gave its warning.
    pub(crate) fn document(&self) -> Result<Value, EncodeError> {
        self.document_with(&Unknown::new())
            .map(|(document, _)| document)
    }

    /// [`document`](Self::document) with the document's unknown members in it, and the
    /// findings of those left out: the format is chosen by the members the document has.
    pub(crate) fn document_with(
        &self,
        unknown: &Unknown,
    ) -> Result<(Value, Vec<Diagnostic>), EncodeError> {
        let at = JsonPath::new().child("minecraft:block");
        // The crate's own members that no format of the game has (`deprecated`,
        // `deprecated_event_handlers`) and the 1.26.20 document deprecates by name
        // (`ordered_v1_custom_components`) are never written into a cereal document, which the
        // game refuses for them (as an unknown member and as a deprecated one). They are written
        // in the shapes the legacy reader takes, at the block's own version.
        if let Some(member) = self.first_crate_member() {
            if let Some(document) = self.legacy_document(unknown)? {
                return Ok(document);
            }
            return Err(EncodeError::NotRepresentable {
                at: member,
                what: Unrepresentable::Unsupported,
            });
        }
        if let Some(broken) = super::validate::refused_by_newest(self, &at).first() {
            if let Some(document) = self.at_own_version(unknown)? {
                return Ok(document);
            }
            // At the member the file has, where a step moved it.
            let broken = &broken.location.json_path;
            let in_file = self
                .refused_at
                .iter()
                .find(|(of_block, _)| of_block == broken)
                .map_or(broken, |(_, of_file)| of_file);
            return Err(EncodeError::NotRepresentable {
                at: in_file.clone(),
                what: Unrepresentable::Unsupported,
            });
        }
        let mut document = write_envelope::<BlockFamily>(
            self.written_version(),
            self.write_at(self.written_version())?,
            &UnknownMembers::default(),
        );
        let told = merge_unknown(&mut document, unknown);
        // The cereal slices are strict: a member or component the document does not
        // bind, or one it deprecates, rejects the block. A block that holds one (an unknown
        // member kept from an older file, or set by hand) is not written into a cereal
        // document the game refuses: at its own version below 1.26.20, where the legacy
        // reader takes it, when that text reads back as the block; otherwise not at all, at
        // the member the load told of.
        if let Some(unread) = unread_by_cereal(&document)? {
            if self.source_format_version < CEREAL
                && let Some(legacy) = self.at_own_version(unknown)?
            {
                return Ok(legacy);
            }
            return Err(EncodeError::NotRepresentable {
                at: unread,
                what: Unrepresentable::Unsupported,
            });
        }
        Ok((document, told))
    }

    /// The block of a file below 1.26.20 as a legacy document that reads back as it: at its
    /// own version, else, as the trade family writes its legacy blob, at 1.26.0, the newest legacy
    /// version, whose reader takes the shapes of 1.26.20 as they are. `None` for a block of
    /// the newest format, which no legacy file gave.
    fn legacy_document(&self, unknown: &Unknown) -> Result<Option<Written>, EncodeError> {
        if self.source_format_version >= CEREAL {
            return Ok(None);
        }
        if let Some(document) = self.at_own_version(unknown)? {
            return Ok(Some(document));
        }
        if self.source_format_version == NEWEST_LEGACY {
            return Ok(None);
        }
        self.at_version(NEWEST_LEGACY, unknown)
    }

    /// The first of the crate's own members the block holds that a cereal document does not take: the
    /// side-car `deprecated` (of the block or of a permutation), `deprecated_event_handlers`,
    /// or a non-empty `ordered_v1_custom_components`.
    fn first_crate_member(&self) -> Option<JsonPath> {
        let at = |keys: &[&str], index: Option<usize>| {
            let mut path = JsonPath::new().child("minecraft:block");
            if let Some(index) = index {
                path.push("permutations");
                path.push_index(index);
            }
            for key in keys {
                path.push(key);
            }
            path
        };
        if !self.deprecated.is_empty() {
            return Some(at(&["deprecated"], None));
        }
        if !self.deprecated_event_handlers.is_empty() {
            return Some(at(&["deprecated_event_handlers"], None));
        }
        if !self.ordered_v1_custom_components.is_empty() {
            return Some(at(&["ordered_v1_custom_components"], None));
        }
        self.permutations
            .iter()
            .enumerate()
            .find_map(|(index, permutation)| {
                if !permutation.deprecated.is_empty() {
                    Some(at(&["deprecated"], Some(index)))
                } else if !permutation.ordered_v1_custom_components.is_empty() {
                    Some(at(&["ordered_v1_custom_components"], Some(index)))
                } else {
                    None
                }
            })
    }

    /// The document at the block's own format version (one below 1.26.20): the payload as it
    /// is written at 1.26.20, without the components a step of the upgrade put there (the
    /// reading puts them there again), and with the crate's own members in the shapes the legacy
    /// reader takes them in: the side-car's components back in `components`,
    /// `deprecated_event_handlers` as the legacy `events`, and, below 1.21.90, an
    /// `ordered_v1_custom_components` list as the v1 `minecraft:custom_components` list.
    /// `None` unless that text reads back as the block (in an allow-listed pack's context
    /// when the block holds legacy components or events, which only such a pack has).
    ///
    /// The game reads a file below 1.26.20 as raw JSON that the fixers of the versions from
    /// its own on rewrite, before the one parser reads the result. The payload of a block
    /// that came through them is in the shapes they give, so they have nothing left to
    /// rewrite in it: the text is checked to be such a fixed point (read in a detached
    /// context and written at 1.26.20, it is the payload this block has there) and to keep
    /// every member where the block has it (a member the older reader does not know would
    /// come back as an unknown one: `deprecated` and `deprecated_event_handlers`, which no
    /// file has, and `ordered_v1_custom_components` below 1.21.90).
    fn at_own_version(&self, unknown: &Unknown) -> Result<Option<Written>, EncodeError> {
        self.at_version(self.source_format_version, unknown)
    }

    /// [`at_own_version`](Self::at_own_version) at `version`, a legacy one: the text when
    /// it reads back, at `version`, as the block, its unknown members (in `unknown`) where
    /// the document has them.
    fn at_version(
        &self,
        version: FormatVersion,
        unknown: &Unknown,
    ) -> Result<Option<Written>, EncodeError> {
        let whole = self.write()?;
        let mut payload = self.write_at(version)?;
        // At its own version the steps that put a component there put it there again; at
        // another one only those whose step runs from it: the 1.26.0 step's chest
        // obstruction, not the 1.20.60 step's `redstone_conductivity`, which is written.
        let kept: &[&str] = if version == self.source_format_version {
            &[]
        } else {
            &["minecraft:redstone_conductivity"]
        };
        if let Some(payload) = payload.as_object_mut() {
            without_synthesized(payload.get_mut("components"), &self.components, kept);
            let permutations = payload
                .get_mut("permutations")
                .and_then(Value::as_array_mut)
                .map(|permutations| permutations.iter_mut());
            for (written, permutation) in permutations.into_iter().flatten().zip(&self.permutations)
            {
                let components = written
                    .as_object_mut()
                    .and_then(|written| written.get_mut("components"));
                without_synthesized(components, &permutation.components, kept);
            }
        }
        let legacy_lcf = !self.deprecated.is_empty()
            || !self.deprecated_event_handlers.is_empty()
            || self.permutations.iter().any(|p| !p.deprecated.is_empty());
        if let Some(payload) = payload.as_object_mut() {
            legacy_shapes(payload, version);
        }
        // Not `write_envelope`: below 1.26.20 the family reads the raw text
        // (`EnvelopeCheck::From`), where it writes the payload alone.
        let mut object = facet_value::VObject::new();
        object.insert("format_version", version.to_string());
        object.insert("minecraft:block", payload);
        let mut document = Value::from(object);
        let told = merge_unknown(&mut document, unknown);
        let mut text = Vec::new();
        write_json(&document, &mut text, WriteStyle::default())?;
        let Ok(text) = std::str::from_utf8(&text) else {
            return Ok(None);
        };
        let mut cx = LoadContext::detached();
        if legacy_lcf {
            cx.pack_uuid = ALLOW_LISTED;
        }
        let outcome =
            bedrock_addon_core::versioning::load::<BlockFamily>(JsonSlice::new(text), &cx);
        let Some(Loaded::Stable(read)) = &outcome.value else {
            return Ok(None);
        };
        let kept = read.source_format_version == version
            && read.permutations.len() == self.permutations.len()
            && outcome.diagnostics.unknown() == unknown
            && read.write().ok().as_ref() == Some(&whole);
        Ok(kept.then_some((document, told)))
    }
}

/// One of the ten packs allowed the legacy creator features
/// (`LoadRequirement::LegacyCreatorFeatures`): the context a block with legacy components or
/// events is read back in, as only such a pack has them.
const ALLOW_LISTED: uuid::Uuid = uuid::Uuid::from_u128(0xda09_e399_6f5e_4287_a013_4cf1_3de3_2166);

/// The newest version the legacy reader reads (`v1_26_0`).
const NEWEST_LEGACY: FormatVersion = known::V1_26_0;

/// The version from which the v1 `minecraft:custom_components` list is deprecated.
const V1_LIST_DEPRECATED: FormatVersion = known::V1_21_90;

/// The crate's own members of a payload written in the 1.26.20 shape, in the shapes the legacy
/// reader of `version` takes: the inverse of the 1.21.60 step's list conversion
/// (below 1.21.90) and of the 1.26.0 step's side-car and events.
fn legacy_shapes(payload: &mut facet_value::VObject, version: FormatVersion) {
    let list = payload.remove("ordered_v1_custom_components");
    let sidecar = payload.remove("deprecated");
    if let Some(components) = payload.get_mut("components").and_then(Value::as_object_mut) {
        legacy_components(components, sidecar.as_ref(), list.as_ref(), version);
    }
    for permutation in payload
        .get_mut("permutations")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
    {
        let Some(permutation) = permutation.as_object_mut() else {
            continue;
        };
        let list = permutation.remove("ordered_v1_custom_components");
        let sidecar = permutation.remove("deprecated");
        if let Some(components) = permutation
            .get_mut("components")
            .and_then(Value::as_object_mut)
        {
            legacy_components(components, sidecar.as_ref(), list.as_ref(), version);
        }
    }
    if let Some(handlers) = payload.remove("deprecated_event_handlers") {
        let mut events = facet_value::VObject::new();
        for (name, node) in handlers.as_object().into_iter().flatten() {
            events.insert(name.as_str(), legacy_event(node));
        }
        payload.insert("events", events);
    }
}

/// One section's components with its side-car's legacy components put back and its v1 list
/// (when `version` still reads one) in place of the keyed entries it became. A list that has
/// no such form stays as the crate's own member it is: the reading back then differs, and the
/// block is not written at that version.
fn legacy_components(
    components: &mut facet_value::VObject,
    sidecar: Option<&Value>,
    list: Option<&Value>,
    version: FormatVersion,
) {
    let names: Vec<String> = list
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|name| name.as_string().map(|name| name.as_str().to_owned()))
        .collect();
    let mut rebuilt = facet_value::VObject::new();
    let mut listed = false;
    for (key, value) in components.iter() {
        let empty = value
            .as_object()
            .is_some_and(facet_value::VObject::is_empty);
        if version < V1_LIST_DEPRECATED && empty && names.iter().any(|n| n == key.as_str()) {
            if !listed {
                listed = true;
                let mut array = facet_value::VArray::new();
                for name in &names {
                    array.push(name.as_str());
                }
                rebuilt.insert("minecraft:custom_components", array);
            }
            continue;
        }
        rebuilt.insert(key.as_str(), value.clone());
    }
    if let Some(sidecar) = sidecar.and_then(Value::as_object) {
        for (name, value) in sidecar {
            let value = match name.as_str() {
                // The side-car says the section was a unit cube; the component is empty.
                "unit_cube" => facet_value::VObject::new().into(),
                "queued_ticking" => legacy_queued_ticking(value),
                _ => value.clone(),
            };
            rebuilt.insert(format!("minecraft:{}", name.as_str()), value);
        }
    }
    *components = rebuilt;
}

/// The side-car's `{min_ticks, max_ticks, …}` as the legacy `{interval_range, …}`.
fn legacy_queued_ticking(value: &Value) -> Value {
    let Some(object) = value.as_object() else {
        return value.clone();
    };
    let mut out = facet_value::VObject::new();
    if let (Some(min), Some(max)) = (object.get("min_ticks"), object.get("max_ticks")) {
        let mut range = facet_value::VArray::new();
        range.push(min.clone());
        range.push(max.clone());
        out.insert("interval_range", range);
    }
    for (key, value) in object {
        if !matches!(key.as_str(), "min_ticks" | "max_ticks") {
            out.insert(key.as_str(), value.clone());
        }
    }
    out.into()
}

/// A handler of `deprecated_event_handlers` as the legacy event node: `probability` is
/// `weight`, each response is a member of the node, the children are nodes alike.
fn legacy_event(node: &Value) -> Value {
    let Some(object) = node.as_object() else {
        return node.clone();
    };
    let mut out = facet_value::VObject::new();
    for (key, value) in object {
        match key.as_str() {
            "probability" => {
                out.insert("weight", value.clone());
            }
            "sequence" | "randomize" => {
                let mut children = facet_value::VArray::new();
                for child in value.as_array().into_iter().flatten() {
                    children.push(legacy_event(child));
                }
                out.insert(key.as_str(), children);
            }
            "responses" => {
                for response in value.as_array().into_iter().flatten() {
                    for (name, value) in response.as_object().into_iter().flatten() {
                        out.insert(name.as_str(), value.clone());
                    }
                }
            }
            _ => {
                out.insert(key.as_str(), value.clone());
            }
        }
    }
    out.into()
}

/// Where the cereal reader would refuse `document` (a whole block document of 1.26.20 or
/// later) for a member it does not read: the first parse finding of an unknown member or
/// component, or of a deprecated member, which the strict slice makes an error. `None` when
/// the reader reads every member.
fn unread_by_cereal(document: &Value) -> Result<Option<JsonPath>, EncodeError> {
    let mut text = Vec::new();
    write_json(document, &mut text, WriteStyle::default())?;
    let Ok(text) = std::str::from_utf8(&text) else {
        return Ok(None);
    };
    let read = bedrock_addon_core::versioning::load::<BlockFamily>(
        JsonSlice::new(text),
        &LoadContext::detached(),
    );
    Ok(read
        .diagnostics
        .iter()
        .find(|finding| {
            finding.phase == bedrock_addon_core::diagnostic::Phase::Parse
                && finding.severity >= bedrock_addon_core::diagnostic::Severity::Error
                // By legacy name: the variant and the definitions that were it.
                && matches!(finding.code.name(), "UnrecognizedField" | "UnrecognizedComponent" | "DeprecatedMember")
        })
        .map(|finding| finding.location.json_path.clone()))
}

/// Takes out of the written `components` those a step of the upgrade put into `map`.
fn without_synthesized(
    components: Option<&mut Value>,
    map: &bedrock_addon_core::document::ComponentMap<super::component::BlockComponent>,
    kept: &[&str],
) {
    let Some(components) = components.and_then(Value::as_object_mut) else {
        return;
    };
    for entry in map.iter() {
        if let bedrock_addon_core::document::ComponentRef::Typed {
            name,
            synthesized: true,
            ..
        } = entry
            && !kept.contains(&name)
        {
            components.remove(name);
        }
    }
}
