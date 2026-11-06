//! Animations: `Animation`, `AnimationFile`, `Bone`, `Channel`, `Keyframe`.
//!
//! Animations (`animations/**/*.json`) of both pack kinds. Vanilla reads the behaviour-pack and
//! the resource-pack files with the same loader and the same schema, so
//! one type serves both stacks; what differs is runtime: the registry each side keeps, the Molang
//! query catalogue the compile check uses, and a few diagnostics of the crate's own.
//!
//! The reader stays a schema walk on `bedrock_addon_shared_animation`'s walker, because vanilla
//! checks the files against its animation schema with a loader of its own rather than reading a
//! typed struct.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod molang;
mod read;
mod types;
mod validate;
mod write;

use bedrock_addon_core::document::{
    Content, Definition, Document, JsonContent, NoBeta, Notice, Origin, Placed, Registry, Rejected,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{self, CollectionLoader, FileCopy, LoadContext, StackCx, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::known;
use bedrock_addon_shared_animation::read::{side_of, whole};
use bedrock_addon_shared_animation::registry::combine;
use bedrock_addon_shared_animation::write::file_path;

pub use bedrock_addon_shared_animation::{EventString, ParticleEffect, SoundEffect};
pub use types::{
    Animation, AnimationFile, AxisElement, Bone, Channel, Keyframe, KeyframeParts, LerpMode,
    LoopMode, OneOrMany, RelativeTo,
};

impl bedrock_addon_core::__private::Sealed for AnimationFile {}
impl bedrock_addon_core::__private::Sealed for Animation {}

impl Content for AnimationFile {
    const NAME: &'static str = "animation file";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior, PackSide::Resource];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), side_of(cx), path.clone());
        whole(read::parse_file(bytes, cx), origin, |_| {
            (known::V1_8_0, false)
        })
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        bedrock_addon_core::json::write_json(&self.to_value()?, out, style)
    }
}

impl JsonContent for AnimationFile {}

impl Definition for Animation {
    type Kind = kind::Animation;
    type File = AnimationFile;
    type Beta = NoBeta<kind::Animation>;
}

impl Placed for AnimationFile {
    /// `animations/<name>.json`, after the first animation (`animation.<name>`).
    fn default_path(&self) -> PackPath {
        file_path("animations", "animation.", self.animations.keys().next())
    }
}

/// `animations/**/*.json`, packs bottom-up. Vanilla's directory strategy recurses for every pack
/// but those shipped inside the app, which it lists flat.
const ANIMATION_WALK: Walk = Walk::json_tree_in(&["animations"]);

/// The file's animations, by their keys as written.
fn animations(file: AnimationFile) -> Vec<(String, Animation)> {
    file.animations.into_iter().collect()
}

fn combine_animations(
    parsed: Vec<FileCopy<AnimationFile>>,
    stack: &StackCx<'_>,
) -> Registry<Animation> {
    combine(parsed, stack, "animations", animations, Notice::shadowed())
}

/// The loader of `animations()`, on either stack: `animations/**/*.json`, each file read whole,
/// its animations combined per identifier, the higher pack winning.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<Animation>, FileCopy<AnimationFile>> =
    load::merged::<AnimationFile, _>(ANIMATION_WALK, combine_animations);
