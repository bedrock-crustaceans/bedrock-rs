//! Reading and writing [`ClientEntity`] as a version struct: the 1.26 schema, and the document
//! with its `format_version`.

use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
use bedrock_addon_core::id::{
    AnimationRef, EntityId, GeometryId, MaterialId, ParticleId, SoundEventName, TexturePath,
};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};
use bedrock_addon_shared_visual::{ControllerUse, HeldItemScale, Scripts, SpawnEgg, VisualSlots};
use facet::Facet;
use indexmap::IndexMap;

use super::read;
use super::{ClientEntity, Description};

impl Versioned for ClientEntity {
    const VERSION: FormatVersion = known::V1_26_0;
}

impl VersionStruct for ClientEntity {
    fn parse(document: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        // The 1.26 schema.
        let described = read::describe(document, cx, d)?;
        // `held_item_scale` is the description's one float: a number beyond `f32` is read as
        // the largest finite float, and the typed read is told which path that applies to.
        let description: Description =
            read::typed::<DescriptionRead>(&described, false, &[&["held_item_scale"]], false, d)?
                .into();
        let json = &described.json;
        let at = &described.at;
        let before = d.len();
        let de = &description;
        read::check_identifier(de.identifier.as_str(), at, d);
        read::check_min_engine_version(de.min_engine_version.as_ref(), at, d);
        de.slots.check(json, at, cx.load.engine_version, d, |_| {});
        // A dropped member is a warning; only an error fails the schema.
        let failed = d
            .iter()
            .skip(before)
            .any(|found| found.severity == Severity::Error);
        (!failed).then_some(Self {
            format_version: cx.original_version,
            description,
        })
    }

    /// The whole document (the struct reads the whole text, `EnvelopeCheck::Never`), at
    /// `written_version`.
    fn write(&self) -> Result<Value, EncodeError> {
        self.document(self.written_version())
    }
}

/// The whole document of a client entity: `format_version` and the payload holding
/// `description`.
pub(crate) fn document<'f, D: Facet<'f>>(
    version: FormatVersion,
    description: &D,
) -> Result<Value, EncodeError> {
    let mut payload = facet_value::VObject::new();
    payload.insert("description", write_payload(description)?);
    let mut root = facet_value::VObject::new();
    root.insert("format_version", version.to_string());
    root.insert(read::PAYLOAD, Value::from(payload));
    Ok(root.into())
}

impl ClientEntity {
    /// The `format_version` the document is written with: its own, and never below 1.26.0,
    /// the oldest version this shape is.
    pub(crate) fn written_version(&self) -> FormatVersion {
        self.format_version.max(Self::VERSION)
    }

    /// The whole document, stamped `version`.
    fn document(&self, version: FormatVersion) -> Result<Value, EncodeError> {
        let mut description = facet_value::VObject::new();
        for (key, value) in write_payload(&self.description)?
            .as_object()
            .into_iter()
            .flatten()
        {
            // `held_item_scale` comes as the shortest number of its `f32` from core.
            description.insert(key.as_str(), value.clone());
        }
        let mut payload = facet_value::VObject::new();
        payload.insert("description", Value::from(description));
        let mut root = facet_value::VObject::new();
        root.insert("format_version", version.to_string());
        root.insert(read::PAYLOAD, Value::from(payload));
        Ok(root.into())
    }
}

/// [`Description`] as the typed read takes it: the same members in the same order, the shared
/// ones written out rather than flattened. A workaround of facet's flatten reader (facet
/// 0.50.0-rc.7): where a proxied value (an identifier, a slot's id, a render controller)
/// refuses its JSON, it fails at the end of the object, without the key or the index below
/// the member, so the findings would lose their place (pinned in core's `json_flatten`). It
/// goes when that reader places the failure; `Description` itself is what is written.
#[derive(Facet)]
struct DescriptionRead {
    identifier: EntityId,
    min_engine_version: Option<Value>,
    #[facet(default)]
    materials: IndexMap<String, MaterialId>,
    #[facet(default)]
    textures: IndexMap<String, TexturePath>,
    #[facet(default)]
    geometry: IndexMap<String, GeometryId>,
    queryable_geometry: Option<String>,
    spawn_egg: Option<SpawnEgg>,
    scripts: Option<Scripts>,
    particle_effects: Option<IndexMap<String, ParticleId>>,
    particle_emitters: Option<IndexMap<String, String>>,
    animations: Option<IndexMap<String, AnimationRef>>,
    render_controllers: Option<Vec<ControllerUse>>,
    sound_effects: Option<IndexMap<String, SoundEventName>>,
    enable_attachables: Option<bool>,
    held_item_ignores_lighting: Option<bool>,
    held_item_scale: Option<HeldItemScale>,
    hide_armor: Option<bool>,
}

impl From<DescriptionRead> for Description {
    fn from(read: DescriptionRead) -> Self {
        Self {
            identifier: read.identifier,
            min_engine_version: read.min_engine_version,
            slots: VisualSlots {
                materials: read.materials,
                textures: read.textures,
                geometry: read.geometry,
                queryable_geometry: read.queryable_geometry,
                spawn_egg: read.spawn_egg,
                scripts: read.scripts,
                particle_effects: read.particle_effects,
                particle_emitters: read.particle_emitters,
                animations: read.animations,
                render_controllers: read.render_controllers,
                sound_effects: read.sound_effects,
                enable_attachables: read.enable_attachables,
                held_item_ignores_lighting: read.held_item_ignores_lighting,
                held_item_scale: read.held_item_scale,
                hide_armor: read.hide_armor,
            },
        }
    }
}
