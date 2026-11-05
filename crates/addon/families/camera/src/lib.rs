//! Cameras (`Cameras`, `CameraPreset`, `CameraList`, `CameraSpline`): the three document kinds
//! below `cameras/` of a behaviour pack, presets (`cameras/presets/`,
//! `minecraft:camera_preset`), lists (`cameras/lists/`, `minecraft:camera_list`) and spline
//! files (`cameras/splines/`, `minecraft:camera_custom_splines`), loaded together as the
//! `Cameras` group, and the camera entity documents of vanilla's resource pack
//! (`CameraEntity`, read on request only).

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
pub use entity::{
    AttachToPlayer, Avoidance, AvoidanceRay, Camera, CameraComponents, CameraEntity,
    CameraEntityDescription, DirectLook, FlyMove, LookMode, Offset, Orbit, PerspectiveOption,
    PlayerViewMode, Present, RotationSpace, Shake, Stationary, Target, ThirdPersonBoom,
    UpdatePlayerFromCamera,
};
pub use list::CameraList;
pub use preset::{
    ALLOWED_MC_INHERIT, AudioListener, CameraPreset, ControlScheme, PresetAimAssist, TargetMode,
    VANILLA_CAMERAS,
};
pub use spline::{
    CameraSpline, ControlPoint, Easing, ProgressKeyFrame, RotationKeyFrame, Spline, SplineType,
};

mod entity;
mod list;
mod load;
mod new;
pub(crate) mod parse;
mod preset;
mod read_entity;
mod read_list;
mod read_spline;
mod spline;
mod stack;
mod texts;
mod validate;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub use load::LOADER;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::{Document, Registry, Rejected};

/// The camera documents of the behaviour packs (`bp.cameras()`): three collections loaded and
/// used together.
///
/// Presets and lists are keyed by their `identifier`; a later definition replaces an earlier
/// one whole, the packs read from the lowest priority up, so the top pack's wins. After the
/// presets of every pack are read, one pass discards the presets whose inheritance or vanilla
/// override is invalid; the lists are checked against what is left.
#[derive(Debug)]
#[non_exhaustive]
pub struct Cameras {
    /// The camera presets (`cameras/presets/**/*.json`).
    pub presets: Registry<CameraPreset>,
    /// The camera lists (`cameras/lists/**/*.json`).
    pub lists: Registry<CameraList>,
    /// The camera spline files (`cameras/splines/**/*.json`).
    pub splines: Splines,
}

impl Cameras {
    /// Every finding of the three collections, presets first.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.presets
            .diagnostics()
            .chain(self.lists.diagnostics())
            .chain(self.splines.diagnostics())
    }
}

/// The spline files of the stack, every one kept, in load order (packs from the lowest
/// priority up, files by path). Vanilla appends each file's splines to one list without
/// looking at their names, so two splines of one name both stay.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Splines {
    files: Vec<Document<CameraSpline>>,
    rejected: Vec<Rejected>,
    diagnostics: Diagnostics,
}

/// Every spline of a [`Splines`], from [`Splines::iter`].
#[derive(Clone, Debug)]
pub struct SplinesIter<'a> {
    files: std::slice::Iter<'a, Document<CameraSpline>>,
    current: std::slice::Iter<'a, Spline>,
}

impl<'a> Iterator for SplinesIter<'a> {
    type Item = &'a Spline;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(spline) = self.current.next() {
                return Some(spline);
            }
            self.current = self.files.next()?.splines.iter();
        }
    }
}

impl<'a> IntoIterator for &'a Splines {
    type Item = &'a Spline;
    type IntoIter = SplinesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Splines {
    /// The spline files that loaded, in load order.
    #[must_use]
    pub fn files(&self) -> &[Document<CameraSpline>] {
        &self.files
    }

    /// Every spline of every file, in load order.
    pub fn iter(&self) -> SplinesIter<'_> {
        SplinesIter {
            files: self.files.iter(),
            current: [].iter(),
        }
    }

    /// The first spline named `name`, in load order.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Spline> {
        self.iter().find(|spline| spline.name.0 == name)
    }

    /// The number of splines.
    #[must_use]
    pub fn len(&self) -> usize {
        self.iter().count()
    }

    /// Whether no spline is loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Files that failed to load, with their diagnostics.
    #[must_use]
    pub fn rejected(&self) -> &[Rejected] {
        &self.rejected
    }

    /// Every finding of the spline files: per file, rejected files and the duplicate names.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.diagnostics.iter()
    }
}
