//! Reading and writing [`ClientEntity`] as a version struct: the 1.26 schema, and the document
//! with its `format_version`.

use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, Unrepresentable, Value, WriteStyle, write_json,
};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};
use bedrock_addon_core::visual::HeldItemScale;
use facet::Facet;

use super::read;
use super::{ClientEntity, Description};

/// The engine version that first declares `scripts.hide_held_items` and reads
/// `held_item_scale` as Molang. Before it the member is a plain float.
const ENGINE_1_26_30: FormatVersion = FormatVersion::new(1, 26, 30);

impl Versioned for ClientEntity {
    const VERSION: FormatVersion = FormatVersion::new(1, 26, 0);
}

impl VersionStruct for ClientEntity {
    fn parse(document: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        // The 1.26 schema.
        let described = read::describe(document, cx, d)?;
        // `held_item_scale` is the description's one float: a number beyond `f32` is read as
        // the largest finite float, and the typed read is told which path that applies to.
        let description: Description =
            read::typed(&described, false, &[&["held_item_scale"]], false, d)?;
        let json = &described.json;
        let at = &described.at;
        let before = d.len();
        let de = &description;
        read::check_identifier(de.identifier.as_str(), at, d);
        read::check_min_engine_version(de.min_engine_version.as_ref(), at, d);
        // An absent map and an empty one are the same value here; the JSON tells them apart.
        let written = |name: &str| json.as_object().is_some_and(|o| o.get(name).is_some());
        let present = |name: &str, required: bool| written(name) && required;
        read::check_map(
            Some(&de.materials),
            "materials",
            present("materials", true),
            at,
            d,
        );
        read::check_map(
            Some(&de.textures),
            "textures",
            present("textures", true),
            at,
            d,
        );
        read::check_map(
            Some(&de.geometry),
            "geometry",
            present("geometry", true),
            at,
            d,
        );
        read::check_map(
            de.particle_effects.as_ref(),
            "particle_effects",
            false,
            at,
            d,
        );
        read::check_map(
            de.particle_emitters.as_ref(),
            "particle_emitters",
            false,
            at,
            d,
        );
        read::check_map(de.animations.as_ref(), "animations", false, at, d);
        read::check_map(de.sound_effects.as_ref(), "sound_effects", false, at, d);
        read::check_controllers(de.render_controllers.as_deref(), at, d);
        let engine = cx.load.engine_version;
        if let Some(scripts) = &de.scripts {
            let scripts_at = at.with(&["scripts"]);
            read::check_variables(scripts.variables.as_ref(), &scripts_at, d);
            read::check_animate(scripts.animate.as_deref(), &scripts_at, d);
            if scripts.hide_held_items.is_some() && engine < ENGINE_1_26_30 {
                // Undeclared before 1.26.30: "child 'hide_held_items' not valid here.".
                d.push(read::not_valid_here(
                    "hide_held_items",
                    scripts_at.with(&["hide_held_items"]),
                ));
            }
        }
        if matches!(de.held_item_scale, Some(HeldItemScale::Molang(_))) && engine < ENGINE_1_26_30 {
            // Before engine 1.26.30 the schema declares `held_item_scale` as a float only.
            d.push(read::wrong_type(
                "float",
                at.with(&["held_item_scale"]),
                None,
            ));
        }
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

    fn write(&self) -> Result<Value, EncodeError> {
        self.document(self.format_version)
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

    /// Writes the document into `out`, or refuses it with the path of what this loader would
    /// not read. Whether its Molang compiles is not asked: that depends on the pack.
    pub(crate) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        let mut text = Vec::new();
        write_json(&self.document(self.written_version())?, &mut text, style)?;
        // A value built in code may hold what no file can: an empty identifier, a slot name
        // outside its pattern, a version above the engine's. The text is given out only when
        // this loader reads it.
        if let Some(at) = super::load::refused_at(&text) {
            return Err(EncodeError::NotRepresentable {
                at,
                what: Unrepresentable::Unsupported,
            });
        }
        out.extend_from_slice(&text);
        Ok(())
    }
}
