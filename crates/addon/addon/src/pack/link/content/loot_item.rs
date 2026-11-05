//! The items a loot table names and the armour trims its `set_armor_trim` function names.

use bedrock_addon_core::diagnostic::{
    DiagnosticCode, JsonPath, ReferenceHint, Severity,
};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::id::{Kind, kind};

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use super::loot::{Site, walk};
use crate::families::item::trim::{ItemTags, TrimData};
use crate::families::loot::texts::{INVALID_ITEM, formatted};
use crate::families::loot::{ConditionKind, Entry, EntryKind, FunctionKind};

/// The tag a smithing template item carries.
const TRIM_TEMPLATES: &str = "minecraft:trim_templates";

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let trims = Trims::of(cx);
    for (_, document) in cx.behavior().loot_tables() {
        let origin = document.origin();
        walk(document, &mut |path, site| match site {
            Site::Entry(entry) => given(cx, out, origin, path, entry),
            Site::Condition(condition) => {
                if let ConditionKind::MatchTool {
                    item: Some(item), ..
                } = &condition.kind
                    && cx.lints()
                    && !item.as_str().is_empty()
                    && let Target::Missing(hint) = cx.item(item)
                {
                    // The game accepts the tool as read; a lint.
                    let place = At::new(origin, at![path; "item"], Severity::Info);
                    out.push(unresolved(place, kinds::ITEM, item.as_str(), hint));
                }
            }
            Site::Function(function) => {
                if let FunctionKind::SetArmorTrim { material, pattern } = &function.kind {
                    trims.function(
                        cx,
                        out,
                        origin,
                        path,
                        pattern.as_deref(),
                        material.as_deref(),
                    );
                }
            }
        });
    }
}

/// The `name` of an `item` entry is an item of the stack (a block's item too), of the
/// game, or a name of an older version the game still answers to. A name that is none is
/// vanilla's message at the content log's error level; the game keeps the entry, which then
/// gives nothing: a kept error.
fn given(cx: &LinkCx<'_>, out: &mut Findings, origin: &Origin, path: &JsonPath, entry: &Entry) {
    let EntryKind::Item {
        name: Some(name), ..
    } = &entry.kind
    else {
        return;
    };
    if let Target::Missing(hint) = cx.item(name) {
        // A name the game's own loot tables write here and the game does not have: let
        // through in this member alone.
        if crate::vanilla::names::is_loot_entry_only_item(&super::super::canonical(name)) {
            return;
        }
        let text = formatted(INVALID_ITEM, [name.as_str()]);
        // The game logs the line at the error level and falls through to the entry's
        // `functions`: a kept error.
        let place = At::new(origin, at![path; "name"], Severity::Error).with_text(text);
        out.push(unresolved(place, kinds::ITEM, name.as_str(), hint).into_kept());
    }
}

/// The armour trims of the world: the game's table behind the world gate and the template
/// filter of world start.
#[derive(Debug)]
struct Trims {
    /// `None` in a world below base-game version 1.20: trims are disabled.
    data: Option<TrimData>,
}

impl Trims {
    fn of(cx: &LinkCx<'_>) -> Self {
        let stack = cx.behavior();
        let world = TrimData::for_world(&stack.options().base_game_version);
        if world.patterns().len() == 0 && world.materials().is_empty() {
            return Self { data: None };
        }
        // A template item the stack does not define is the game's when the game has an item
        // of that name, and the game's template items carry the tag.
        let data = world.filtered(stack.items(), |item| {
            matches!(cx.item_definition(item), Target::BuiltIn).then(|| ItemTags {
                tags: vec![TRIM_TEMPLATES.to_owned()],
            })
        });
        Self { data: Some(data) }
    }

    /// One `set_armor_trim` function at `path`.
    fn function(
        &self,
        cx: &LinkCx<'_>,
        out: &mut Findings,
        origin: &Origin,
        path: &JsonPath,
        pattern: Option<&str>,
        material: Option<&str>,
    ) {
        let Some(data) = &self.data else {
            // In a world below 1.20 both registries are empty and the function trims
            // nothing; its names are not looked up.
            if cx.lints() {
                let place = At::new(origin, path.clone(), Severity::Info);
                out.push(place.diagnostic(DiagnosticCode::NoEffect { text: None }));
            }
            return;
        };
        let members = [
            (
                "pattern",
                pattern,
                <kind::TrimPattern as Kind>::NAME,
                data.patterns()
                    .map(|pattern| pattern.pattern_id.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "material",
                material,
                <kind::TrimMaterial as Kind>::NAME,
                data.materials()
                    .iter()
                    .map(|material| material.material_id.as_str())
                    .collect(),
            ),
        ];
        for (key, name, kind, known) in members {
            // The game reads both names without a line and only asserts for a pattern it
            // lacks: informations.
            let place = At::new(origin, at![path; key], Severity::Info);
            match name {
                // An absent name is read as the empty one; the function trims nothing.
                None | Some("") => {
                    out.push(place.diagnostic(DiagnosticCode::MissingRequiredField { text: None }));
                }
                // A name of no pattern, of no material.
                Some(name) if !known.contains(&name) => {
                    let hint = known
                        .iter()
                        .find(|known| known.eq_ignore_ascii_case(name))
                        .map(|known| ReferenceHint::CaseMismatch((*known).to_owned()));
                    out.push(unresolved(place, kind, name, hint));
                }
                Some(_) => {}
            }
        }
    }
}
