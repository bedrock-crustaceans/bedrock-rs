//! The envelope of a written document ([`write_document`]): the one place a family's writer
//! spells `"format_version"`.

use facet_value::VObject;

use super::{BETA_VERSION, EnvelopeCheck, Family};
use crate::document::Extra;
use crate::json::Value;
use crate::version::FormatVersion;

/// The whole document of family `F` at `version`: `{ "format_version": version,
/// F::PAYLOAD_KEY: payload, ..extras }`, the members in that order, the version written
/// `"beta"` for [`BETA_VERSION`]. On the raw-text path (`F::ENVELOPE` reads the root struct from
/// the whole text at `version`) the payload is the document, and it is returned unchanged:
/// a raw-text struct writes its own root, root members included, so `extras` is not written.
///
/// This is the envelope the generated `Any::to_json` writes (`__macro::document_json`), as a
/// [`Value`].
///
/// The envelopes the families build by hand today that are this shape, version first, one
/// payload key, then the root extras: `voxel_shape`, `feature_rules`, dimension (`root_extra`
/// after), `block_culling`, fog (behind its pre-write refusal), trade (the cereal branch),
/// `crafting_catalog`, `texture_set`, particle, biome, `client_biome` (`root_extra` after), item,
/// entity (`"beta"` for the beta struct), block (both writers), attachable and `client_entity`
/// (two copies each), the generic copy of camera, jigsaw, poi and `deferred_settings`, support's
/// `chain_file::document`, ddui (two copies) and `atomic_client_entity`. Recipe (the key per
/// recipe kind, the version member left out when there is none), the spawn legacy writer (an
/// optional version), dialogue (the version as written) and feature (the key per feature type)
/// are the shape but not of one family's `PAYLOAD_KEY`, or not always with a version.
///
/// Not this shape, and kept as their own writers: animation, `animation_controller`,
/// `render_controller` and geometry (`format_version` beside a member that is not a family
/// payload; geometry writes `debug` between them and changes shape by version), `texture_atlas`
/// (`format_version` is a passthrough member), `blocks_json` (the version among the block keys),
/// persona (an optional envelope) and `sound_definitions` (its upgrade steps edit the document in
/// place).
pub fn write_document<F: Family>(version: FormatVersion, payload: Value, extras: &Extra) -> Value {
    if raw_text::<F>(version) {
        return payload;
    }
    let written: Value = if version == BETA_VERSION {
        "beta".into()
    } else {
        version.to_string().into()
    };
    let mut root = VObject::new();
    root.insert("format_version", written);
    root.insert(F::PAYLOAD_KEY, payload);
    for (key, value) in extras.iter() {
        root.insert(key, value.clone());
    }
    root.into()
}

/// Whether a document of `version` is read on the raw-text path, where the payload is the
/// whole document and no envelope is checked.
pub(crate) fn raw_text<F: Family>(version: FormatVersion) -> bool {
    match F::ENVELOPE {
        EnvelopeCheck::Always | EnvelopeCheck::Uncounted => false,
        EnvelopeCheck::Never => true,
        EnvelopeCheck::From(minimum) => version < minimum,
    }
}
