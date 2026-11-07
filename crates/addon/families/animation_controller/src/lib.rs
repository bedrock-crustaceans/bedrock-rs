//! Animation controllers: `AnimationController`, `AnimationControllerFile`, `State`.
//!
//! Animation controllers (`animation_controllers/*.json`) of both pack kinds. Vanilla reads the
//! behaviour-pack and the resource-pack files with the same loader and the same schema, so one
//! type serves both stacks; what differs is runtime: the registry each side keeps, the Molang
//! query catalogue the compile check uses, and a few diagnostics of the crate's own.
//!
//! The reader stays a schema walk on core's `json::node_tree` walker under
//! `bedrock_addon_shared_animation`'s rules, because vanilla
//! checks the files against its controller schema with a loader of its own rather than reading a
//! typed struct.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;

pub mod codes;
mod molang;
mod read;
mod types;
mod validate;
mod write;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostics, Severity};
use bedrock_addon_core::document::{
    Content, Definition, Document, JsonContent, NoBeta, Notice, Origin, Placed, Registry, Rejected,
    VersionInfo,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{self, CollectionLoader, FileCopy, LoadContext, StackCx, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::known;
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use bedrock_addon_shared_animation::read::{side_of, version_info, whole};
use bedrock_addon_shared_animation::registry::combine;
use bedrock_addon_shared_animation::write::file_path;
use history::AnimationControllerHistory;
use read::{V1_8_0, V1_10_0};

pub use bedrock_addon_shared_animation::{EventString, ParticleEffect, SoundEffect};
pub use types::{
    AnimationController, AnimationControllerFile, BlendTransition, State, StateAnimation,
    StateVariable, Transition,
};

impl bedrock_addon_core::__private::Sealed for AnimationControllerFile {}
impl bedrock_addon_core::__private::Sealed for AnimationController {}

fn controller_parsed_as(written: &SemVersion) -> (FormatVersion, bool) {
    if *written < V1_10_0 {
        (known::V1_8_0, true)
    } else {
        (known::V1_10_0, false)
    }
}

impl Content for AnimationControllerFile {
    const NAME: &'static str = "animation controller file";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior, PackSide::Resource];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), side_of(cx), path.clone());
        whole(read::parse_file(bytes, cx), origin, controller_parsed_as)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        bedrock_addon_core::json::write_json(&self.to_value()?, out, style)
    }
}

impl JsonContent for AnimationControllerFile {}

impl Definition for AnimationController {
    type Kind = kind::Animation;
    type File = AnimationControllerFile;
    type Beta = NoBeta<kind::Animation>;
}

impl Placed for AnimationControllerFile {
    /// `animation_controllers/<name>.json`, after the first controller
    /// (`controller.animation.<name>`).
    fn default_path(&self) -> PackPath {
        file_path(
            "animation_controllers",
            "controller.animation.",
            self.animation_controllers.keys().next(),
        )
    }
}

impl AnimationControllerFile {
    /// Parses one controller file without upgrading it: the struct its `format_version` selects;
    /// [`history::upgrade`](history::upgrade) finishes it.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`](bedrock_addon_core::document::Rejected) with the findings the game's content
    /// log would print when the bytes cannot be read as this file at all: not JSON, a
    /// `format_version` with no struct here, or a document that does not fit the struct it selects.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<AnimationControllerHistory>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), side_of(cx), path.clone());
        let (any, written, diagnostics) = read::parse_as_written(bytes, cx);
        let diagnostics = diagnostics.located(&origin);
        match (any, written) {
            (Some(any), Some(written)) => {
                let as_ = any.parsed_version();
                Ok(Document::new(
                    any,
                    origin,
                    version_info(&written, as_, false),
                    diagnostics,
                ))
            }
            _ => Err(Rejected::new(origin, diagnostics)),
        }
    }
}

/// `history::upgrade`: the step and the checks on a document `parse_as_written` gave.
pub(crate) fn upgrade_document(
    document: Document<AnimationControllerHistory>,
    cx: &LoadContext,
) -> Result<Document<AnimationControllerFile>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let mut diagnostics = Diagnostics::default();
    let written = version
        .written
        .map_or(V1_8_0, |v| SemVersion::new(v.major, v.minor, v.patch));
    let upgraded = matches!(document.value(), AnimationControllerHistory::V1_8_0(_));
    let file = read::upgrade(document.into_value(), &written, cx, &mut diagnostics)
        .and_then(|file| read::finish(file, &written, cx, &mut diagnostics));
    let diagnostics = diagnostics.located(&origin);
    let info = VersionInfo::new(
        version.written,
        false,
        version.effective,
        version.parsed_as,
        upgraded,
    );
    match file {
        Some(file) => Ok(Document::new(file, origin, info, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

/// `animation_controllers/*.json`, packs bottom-up, sub-folders not read.
const CONTROLLER_WALK: Walk = Walk::json_children_in(&["animation_controllers"]);

/// The file's controllers, by their keys as written.
fn controllers(file: AnimationControllerFile) -> Vec<(String, AnimationController)> {
    file.animation_controllers.into_iter().collect()
}

fn combine_controllers(
    parsed: Vec<FileCopy<AnimationControllerFile>>,
    stack: &StackCx<'_>,
) -> Registry<AnimationController> {
    combine(
        parsed,
        stack,
        "animation_controllers",
        controllers,
        Notice {
            code: Code::from(&core_codes::ANIMATION_CONTROLLER_SHADOWED),
            severity: Severity::Info,
        },
    )
}

/// The loader of `animation_controllers()`, on either stack: `animation_controllers/*.json`,
/// each file read whole, its controllers combined per identifier, the higher pack winning.
#[doc(hidden)]
pub const LOADER: CollectionLoader<
    Registry<AnimationController>,
    FileCopy<AnimationControllerFile>,
> = load::merged::<AnimationControllerFile, _>(CONTROLLER_WALK, combine_controllers);
