//! The aim-assist preset a camera preset turns on and the categories an aim-assist preset
//! names; the blocks, entities and items an aim-assist document names are answered by the
//! pair files `aim_assist_block`, `aim_assist_entity` and `aim_assist_item`.

use std::collections::{HashMap, HashSet};

use super::super::{Findings, LinkCx, Validator, kinds, validator_text};
use crate::diagnostic::{Diagnostic, DiagnosticDetail, JsonPath, JsonPathSegment};
use crate::document::Origin;
use crate::families::aim_assist::{self, Targets};
use crate::vanilla::GeneratedBuiltIns;
use crate::version::FormatVersion;
use bedrock_addon_core::load::BuiltIns;

/// The lookups of the pass, for the family's checkers. Every name arrives in the form the
/// game registers it in: lower-cased, an item without its auxiliary value, an entity without
/// its event.
#[derive(Debug)]
struct World<'c, 'a>(&'c LinkCx<'a>);

impl World<'_, '_> {
    /// Whether the game has `key` of `kind` outside the stack: the host's word, or without a
    /// host the game's tables (the categories and preset of its `vanilla` behaviour pack,
    /// `names::AIM_ASSIST_CATEGORIES` and `AIM_ASSIST_PRESETS`).
    fn game_has(&self, kind: &'static str, key: &str) -> bool {
        match self.0.host_built_ins() {
            Some(host) => host.contains(kind, key),
            None => GeneratedBuiltIns.contains(kind, key),
        }
    }
}

impl Targets for World<'_, '_> {
    fn block(&self, key: &str) -> bool {
        #[cfg(feature = "block")]
        {
            super::aim_assist_block::resolves(self.0, key)
        }
        #[cfg(not(feature = "block"))]
        {
            // Nobody knows the blocks of a pack: none is told.
            let _ = key;
            true
        }
    }

    fn entity(&self, key: &str) -> bool {
        #[cfg(feature = "entity")]
        {
            super::aim_assist_entity::resolves(self.0, key)
        }
        #[cfg(not(feature = "entity"))]
        {
            // Nobody knows the entitys of a pack: none is told.
            let _ = key;
            true
        }
    }

    fn item(&self, key: &str) -> bool {
        #[cfg(feature = "item")]
        {
            super::aim_assist_item::resolves(self.0, key)
        }
        #[cfg(not(feature = "item"))]
        {
            // Nobody knows the items of a pack: none is told.
            let _ = key;
            true
        }
    }

    fn category(&self, key: &str) -> bool {
        // The checker has looked among the categories of the stack.
        self.game_has(kinds::AIM_ASSIST_CATEGORY, key)
    }

    fn preset(&self, key: &str) -> bool {
        // The checker has looked among the aim-assist presets of the stack.
        self.game_has(kinds::AIM_ASSIST_PRESET, key)
    }
}

/// A camera preset's `aim_assist.preset` that names no aim-assist preset, and the categories,
/// blocks, entities and items an aim-assist document names that nothing defines. Each is an
/// `UnresolvedReference` finding at its member, the name as written; [`with_validator_line`]
/// grades it as an error or an information.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let world = World(cx);
    let behavior = cx.behavior();
    let group = behavior.aim_assist();
    let found = aim_assist::link(group, &world);
    out.extend(
        told_as_the_file_has_it(group, found)
            .into_iter()
            .map(with_validator_line),
    );
    #[cfg(feature = "camera")]
    out.extend(aim_assist::link_camera_presets(
        behavior
            .cameras()
            .presets
            .iter()
            .filter_map(|(_, document)| {
                let id = document.aim_assist.as_ref()?.preset.as_ref()?;
                Some((document, id))
            }),
        group,
        &world,
    ));
}

/// The game's linked-asset check holds an aim-assist preset's exclusion entries, liquid
/// targeting items and category names, and logs one it cannot resolve: the finding carries its
/// line, the breadcrumb from the document's root
/// (`-> minecraft:aim_assist_preset -> exclusion_list[0]`). The item a key of `item_settings`
/// names, and the names of a categories file, are not held: the game keeps them without a
/// word, so such a finding is the crate's own, an information.
fn with_validator_line(mut finding: Diagnostic) -> Diagnostic {
    let held = held_by_the_validator(&finding);
    let id = match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { id, .. }) => id.clone(),
        _ => return finding,
    };
    if !held {
        finding.severity = crate::diagnostic::Severity::Info;
    }
    if held
        && let crate::diagnostic::DiagnosticCode::UnresolvedReference { text } = &mut finding.code
        && text.is_none()
    {
        let bare = id.split('<').next().unwrap_or_default();
        // The linked-asset check writes this line to the content log at the error level and
        // drops nothing, so the finding is an error.
        finding.severity = crate::diagnostic::Severity::Error;
        *text = Some(validator_text(
            Validator::Missing,
            &finding.location.json_path,
            0,
            bare,
        ));
    }
    finding
}

/// Whether the game's linked-asset check holds the reference `finding` is about.
fn held_by_the_validator(finding: &Diagnostic) -> bool {
    let mut segments = finding.location.json_path.segments();
    match (segments.next(), segments.next(), segments.next()) {
        (
            Some(JsonPathSegment::Key("minecraft:aim_assist_preset")),
            Some(JsonPathSegment::Key(
                "exclusion_list" | "liquid_targeting_list" | "exclusion_settings",
            )),
            _,
        )
        | (
            Some(JsonPathSegment::Key("minecraft:aim_assist_preset")),
            Some(JsonPathSegment::Key("default_item_settings" | "hand_settings")),
            None,
        ) => true,
        (
            Some(JsonPathSegment::Key("minecraft:aim_assist_preset")),
            Some(JsonPathSegment::Key("item_settings")),
            Some(JsonPathSegment::Key(_)),
        ) => matches!(
            finding.detail.as_deref(),
            Some(DiagnosticDetail::Reference { kind, .. }) if *kind == kinds::AIM_ASSIST_CATEGORY
        ),
        _ => false,
    }
}

/// The format an aim-assist preset has two exclusion lists from (`exclusion_settings.blocks`
/// and `.entities`); a document below it has the one `exclusion_list`.
const SEPARATE_LISTS: FormatVersion = FormatVersion::new(1, 21, 120);

/// What an entry of an old `exclusion_list` names: the game registers it once, as either.
const BLOCK_OR_ENTITY: &str = "block or entity";

/// The file of a document: its pack's place on the stack and its path.
fn file_of(origin: &Origin) -> (Option<usize>, &str) {
    (origin.pack.index(), origin.path.as_str())
}

/// Which exclusion list of a preset a finding is about, and the entry's index.
fn exclusion(path: &JsonPath) -> Option<(&'static str, usize)> {
    let mut segments = path.segments();
    let (_, Some(JsonPathSegment::Key("exclusion_settings"))) = (segments.next(), segments.next())
    else {
        return None;
    };
    let list = match segments.next()? {
        JsonPathSegment::Key("blocks") => "blocks",
        JsonPathSegment::Key("entities") => "entities",
        _ => return None,
    };
    match (segments.next()?, segments.next()) {
        (JsonPathSegment::Index(index), None) => Some((list, index)),
        _ => None,
    }
}

/// The findings of the aim-assist checker with the entries of an old `exclusion_list` told as
/// the file has them. A preset below format 1.21.120 has one list of blocks and entities,
/// which the upgrade copies into both lists of `exclusion_settings`; the checker looks every
/// entry up in both, so an entity is a missing block to it and a block a missing entity. Here
/// an entry of such a preset is missing only when it is neither, and it is told once, at
/// `exclusion_list/<index>`, as a block or entity. Which presets have the one list is read
/// from the format version of the document, not from whether an upgrade ran.
fn told_as_the_file_has_it(
    group: &aim_assist::AimAssist,
    found: impl IntoIterator<Item = Diagnostic>,
) -> Vec<Diagnostic> {
    let upgraded: HashSet<(Option<usize>, &str)> = group
        .presets
        .iter()
        .filter(|(_, document)| document.version().effective < SEPARATE_LISTS)
        .map(|(_, document)| file_of(document.origin()))
        .collect();
    let found: Vec<Diagnostic> = found.into_iter().collect();
    if upgraded.is_empty() {
        return found;
    }
    let old_entry = |finding: &Diagnostic| {
        let pack = finding.location.pack.as_ref()?.index();
        let file = finding.location.path.as_ref()?.as_str();
        if !upgraded.contains(&(pack, file)) {
            return None;
        }
        let (list, index) = exclusion(&finding.location.json_path)?;
        Some(((pack, file.to_owned(), index), list))
    };
    // How many of the two lists miss each entry of an old list.
    let mut missed: HashMap<(Option<usize>, String, usize), usize> = HashMap::new();
    for finding in &found {
        if let Some((entry, _)) = old_entry(finding) {
            *missed.entry(entry).or_default() += 1;
        }
    }
    let mut kept = Vec::with_capacity(found.len());
    for mut finding in found {
        match old_entry(&finding) {
            None => kept.push(finding),
            Some((entry, "blocks")) if missed.get(&entry) == Some(&2) => {
                let mut path = finding.location.json_path.prefix(1);
                path.push("exclusion_list");
                path.push_index(entry.2);
                finding.location.json_path = path;
                if let Some(DiagnosticDetail::Reference { kind, .. }) =
                    finding.detail.as_deref_mut()
                {
                    *kind = BLOCK_OR_ENTITY;
                }
                kept.push(finding);
            }
            Some(_) => {}
        }
    }
    kept
}
