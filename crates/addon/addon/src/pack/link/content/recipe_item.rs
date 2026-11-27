//! The items, item tags and smithing ingredients of a recipe.
//!
//! Vanilla resolves all of these when it reads the recipe and drops a recipe whose
//! ingredient or result fails; the recipes of a stack cannot read the items of the stack when
//! they load, so the pass tells each miss as an error and leaves the recipe where the load
//! put it.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::builders;
use std::collections::HashMap;

use bedrock_addon_core::diagnostic::{
    Diagnostics, JsonPath, Severity, sentence,
};
use bedrock_addon_core::document::{Document, Origin};
use bedrock_addon_core::id::ItemId;

use crate::pack::link::prelude::*;
use crate::families::item::component::{MaxStackSize, Tags};
use crate::families::item::trim::{ItemTags, TrimData};
use crate::families::recipe::{Ingredient, Recipe, Unlock};

/// The tag the addition of a smithing transform must carry.
const TRANSFORM_MATERIALS: &str = "minecraft:transform_materials";

/// The items of the game that carry [`TRANSFORM_MATERIALS`]: the one item the game's own
/// transform recipes add.
const VANILLA_TRANSFORM_MATERIALS: [&str; 1] = ["minecraft:netherite_ingot"];

/// The tag a smithing template item carries, and the one a trim material carries.
const TRIM_TEMPLATES: &str = "minecraft:trim_templates";
const TRIM_MATERIALS: &str = "minecraft:trim_materials";

/// The stack size of an item without `minecraft:max_stack_size`.
const DEFAULT_STACK: i32 = 64;

/// What an ingredient is in its recipe: which texts a miss gets.
#[derive(Debug, Clone, Copy)]
enum Role {
    /// An input, or the single output of a recipe type that words it itself: the item's own
    /// message, then the message of the place (`{}` is the recipe's identifier), if it has one.
    Part(Option<&'static str>),
    /// An entry of `result` of a shaped or shapeless recipe.
    Result,
    /// An entry of `unlock`: the game logs it at the content log's error level and keeps the
    /// recipe (a kept error).
    Unlock,
}

const MALFORMED: Role = Role::Part(Some("Recipe result malformed"));

/// What the stack's items say of one tag.
#[derive(Debug, Default)]
struct Carriers<'a> {
    /// The items that carry the tag and stack lower than every item before them, in load
    /// order with their stack sizes: the first item that stacks below a count is the first of
    /// these that does. At most one entry per stack size.
    lows: Vec<(&'a ItemId, i32)>,
}

impl<'a> Carriers<'a> {
    fn add(&mut self, item: &'a ItemId, stack: i32) {
        if self.lows.last().is_none_or(|(_, lowest)| stack < *lowest) {
            self.lows.push((item, stack));
        }
    }

    /// The first item, in load order, that stacks below `count`.
    fn first_below(&self, count: i32) -> Option<&'a ItemId> {
        self.lows
            .iter()
            .find(|(_, stack)| *stack < count)
            .map(|(item, _)| *item)
    }
}

#[derive(Debug)]
struct Rule<'c, 'a, 'o> {
    cx: &'c LinkCx<'a>,
    out: &'o mut Findings,
    /// The items of the stack by the tags they carry.
    tagged: HashMap<&'a str, Carriers<'a>>,
    /// The trims of the world; `None` in a world below 1.20.
    trims: Option<TrimData>,
    /// The first item of the stack tagged as a trim template that is the template of no
    /// pattern, and the first tagged as a trim material that is the item of no material.
    strays: [Option<&'a ItemId>; 2],
}

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let recipes = cx.behavior().recipes();
    if recipes.is_empty() {
        return;
    }
    let items = cx.behavior().items();
    // The trims of the world: the game's table behind the world gate and the template
    // filter of world start.
    let world = TrimData::for_world(&cx.behavior().options().base_game_version);
    let trims = (world.patterns().len() > 0).then(|| {
        world.filtered(items, |item| {
            matches!(cx.item_definition(item), Target::BuiltIn).then(|| ItemTags {
                tags: vec![TRIM_TEMPLATES.to_owned()],
            })
        })
    });
    let mut tagged: HashMap<&str, Carriers<'_>> = HashMap::new();
    let mut strays = [None, None];
    for (id, document) in items {
        let components = &document.value().components;
        let stack = components
            .get::<MaxStackSize>()
            .map_or(DEFAULT_STACK, |size| size.value);
        for tag in components.get::<Tags>().iter().flat_map(|tags| &tags.tags) {
            tagged.entry(tag.as_str()).or_default().add(id, stack);
            let Some(trims) = &trims else {
                continue;
            };
            let [template, material] = &mut strays;
            if tag == TRIM_TEMPLATES && trims.pattern_for_item(id).is_none() {
                template.get_or_insert(id);
            }
            if tag == TRIM_MATERIALS && trims.material_for_item(id).is_none() {
                material.get_or_insert(id);
            }
        }
    }
    let mut rule = Rule {
        cx,
        out,
        tagged,
        trims,
        strays,
    };
    for (id, document) in recipes {
        rule.recipe(id.as_str(), document);
    }
}

impl Rule<'_, '_, '_> {
    #[allow(clippy::too_many_lines, reason = "one arm per recipe type, each a list of members")]
    fn recipe(&mut self, id: &str, document: &Document<Recipe>) {
        let recipe = document.value();
        let place = Place {
            id,
            origin: document.origin(),
            found: document.diagnostics(),
        };
        match recipe {
            Recipe::Shaped(shaped) => {
                let base = at!["minecraft:recipe_shaped"];
                for (symbol, ingredient) in &shaped.key {
                    let role = Role::Part(Some("Recipe for: {} is missing (unknown) ingredient"));
                    self.ingredient(&place, at![&base; "key", symbol], ingredient, role);
                }
                self.list(&place, at![&base; "result"], &shaped.result, Role::Result);
                self.unlock(&place, &base, shaped.unlock.as_ref());
            }
            Recipe::Shapeless(shapeless) => {
                let base = at!["minecraft:recipe_shapeless"];
                for (index, ingredient) in shapeless.ingredients.iter().enumerate() {
                    self.ingredient(&place, at![&base; "ingredients", index], ingredient, MALFORMED);
                }
                self.list(&place, at![&base; "result"], &shapeless.result, Role::Result);
                self.unlock(&place, &base, shapeless.unlock.as_ref());
            }
            Recipe::Furnace(furnace) => {
                let base = at!["minecraft:recipe_furnace"];
                self.ingredient(&place, at![&base; "input"], &furnace.input, MALFORMED);
                self.ingredient(&place, at![&base; "output"], &furnace.output, MALFORMED);
                self.unlock(&place, &base, furnace.unlock.as_ref());
            }
            Recipe::BrewingMix(mix) => {
                let base = at!["minecraft:recipe_brewing_mix"];
                for (key, part) in [
                    ("input", &mix.input),
                    ("reagent", &mix.reagent),
                    ("output", &mix.output),
                ] {
                    self.ingredient(&place, at![&base; key], part, Role::Part(None));
                }
            }
            Recipe::BrewingContainer(container) => {
                let base = at!["minecraft:recipe_brewing_container"];
                for (key, part, text) in [
                    (
                        "input",
                        &container.input,
                        "Brewing Container Recipe: {} has an invalid input item",
                    ),
                    (
                        "reagent",
                        &container.reagent,
                        "Brewing Container Recipe: {} has an invalid reagent item",
                    ),
                    (
                        "output",
                        &container.output,
                        "Brewing Container Recipe: {} has an invalid output item",
                    ),
                ] {
                    self.ingredient(&place, at![&base; key], part, Role::Part(Some(text)));
                }
            }
            Recipe::MaterialReduction(reduction) => {
                let base = at!["minecraft:recipe_material_reduction"];
                let input = "Material Reduction Recipe: {} has an invalid input item";
                self.ingredient(&place, at![&base; "input"], &reduction.input, Role::Part(Some(input)));
                if let Some(output) = &reduction.output {
                    let text = "Material Reduction Recipe: {} has an invalid output item";
                    self.list(&place, at![&base; "output"], output, Role::Part(Some(text)));
                }
            }
            Recipe::SmithingTransform(transform) => {
                let base = at!["minecraft:recipe_smithing_transform"];
                // A template that names no item is no fault: the game skips its check.
                for (key, part, text) in [
                    (
                        "base",
                        &transform.base,
                        "Smithing Transform Recipe: {} has an invalid base item.",
                    ),
                    (
                        "addition",
                        &transform.addition,
                        "Smithing Transform Recipe: {} has an invalid addition item.",
                    ),
                    (
                        "result",
                        &transform.result,
                        "Smithing Transform Recipe: {} has an invalid result item.",
                    ),
                ] {
                    self.ingredient(&place, at![&base; key], part, Role::Part(Some(text)));
                }
                self.transform_addition(&place, at![&base; "addition"], &transform.addition);
            }
            Recipe::SmithingTrim(trim) => {
                let base = at!["minecraft:recipe_smithing_trim"];
                self.trim_part(&place, at![&base; "template"], &trim.template, Trim::Pattern);
                self.trim_part(&place, at![&base; "addition"], &trim.addition, Trim::Material);
            }
                    // `Recipe` is `#[non_exhaustive]`: a kind this pass does not know has no ingredients to check.
            _ => {}
        }
    }

    /// `result` or `output`: one ingredient at `path`, or a list of them below it.
    fn list(&mut self, place: &Place<'_>, path: JsonPath, output: &crate::families::recipe::Output, role: Role) {
        match output {
            crate::families::recipe::Output::One(item) => self.ingredient(place, path, item, role),
            crate::families::recipe::Output::Many(items) => {
                for (index, item) in items.iter().enumerate() {
                    // At the index the file has the entry at: the reader leaves out a
                    // `null` entry.
                    let entry = place.found.place_of(&at![&path; index]);
                    // The message of the place is told at the member, not at the entry.
                    let role = match role {
                        Role::Part(Some(text)) => {
                            if self.item_of(place, entry, item, Role::Part(None)) {
                                self.site(place, path.clone(), text);
                            }
                            continue;
                        }
                        role => role,
                    };
                    self.ingredient(place, entry, item, role);
                }
            }
        }
    }

    fn unlock(&mut self, place: &Place<'_>, base: &JsonPath, unlock: Option<&Unlock>) {
        if let Some(Unlock::Ingredients(items)) = unlock {
            for (index, item) in items.iter().enumerate() {
                self.ingredient(place, at![base; "unlock", index], item, Role::Unlock);
            }
        }
    }

    /// One ingredient at `path` in the role `role`: its item or its tag, and the
    /// message of its place after a miss.
    fn ingredient(&mut self, place: &Place<'_>, path: JsonPath, ingredient: &Ingredient, role: Role) {
        let missed = match ingredient.tag() {
            Some(tag) => self.tag_of(place, &path, ingredient, tag),
            None => self.item_of(place, path.clone(), ingredient, role),
        };
        if missed && let Role::Part(Some(text)) = role {
            self.site(place, path, text);
        }
    }

    /// The message of the place of a missed ingredient.
    fn site(&mut self, place: &Place<'_>, path: JsonPath, template: &'static str) {
        let text = if template.contains("{}") {
            formatted(template, &[place.id])
        } else {
            String::from(template)
        };
        let finding = At::new(place.origin, path, Severity::Error)
            .diagnostic(builders::worded(&core_codes::DOCUMENT_REJECTED, &core_codes::DOCUMENT_REJECTED_WORDED, Some(&text)));
        self.out.push(finding);
    }

    /// The item an ingredient names, read as the game reads an item name (one token
    /// is a `minecraft:` name; lower-cased; an `item.` prefix dropped; an aux value apart), is
    /// an item of the stack or of the game. Returns whether it is missing.
    fn item_of(&mut self, place: &Place<'_>, path: JsonPath, ingredient: &Ingredient, role: Role) -> bool {
        let (written, member) = match ingredient {
            Ingredient::Name(text) => (text.as_str(), path),
            Ingredient::Object(object) => match &object.item {
                Some(text) => (text.as_str(), at![&path; "item"]),
                None => return false,
            },
        };
        // An empty name was told when the recipe loaded; a potion is no item name.
        if written.is_empty() || ingredient.potion().is_some() {
            return false;
        }
        let Some(name) = ingredient.item_name() else {
            return false;
        };
        let id = name.id();
        let Target::Missing(hint) = self.cx.item(&ItemId::parse(id.as_str())) else {
            return false;
        };
        let (severity, text) = match role {
            Role::Part(_) => (
                Severity::Error,
                formatted("The Item: {} is missing or invalid, can't make the recipe", &[&id]),
            ),
            Role::Result => (Severity::Error, String::from("Recipe has an invalid result")),
            // The game logs the line at the error level, clears the requirement and the recipe
            // loads: a kept error.
            Role::Unlock => (
                Severity::Error,
                formatted("Recipe: {} has an invalid unlock ingredient", &[place.id]),
            ),
        };
        let at = At::new(place.origin, member, severity).with_text(text);
        let finding = unresolved(at, kinds::ITEM, written, hint);
        self.out.push(match role {
            Role::Unlock => finding.into_kept(),
            _ => finding,
        });
        true
    }

    /// The tag of a tag ingredient is one an item of the stack carries, one of the game's own
    /// recipes or one the host answers for (a `minecraft:` tag beyond those is a lint: the tags
    /// of the game's items are not known here), and no item of the stack that carries it stacks
    /// below the ingredient's `count`. Returns whether the ingredient is invalid.
    fn tag_of(&mut self, place: &Place<'_>, path: &JsonPath, ingredient: &Ingredient, tag: &str) -> bool {
        let Some(carriers) = self.tagged.get(tag) else {
            // The tags of the game's own recipes stand with any host, which can add to them.
            if !self.cx.item_tag(tag).is_missing() {
                return false;
            }
            // Which tags the game's items carry the crate does not know: a `minecraft:` tag
            // may be one of them, so it is an information and the ingredient stands. Every
            // pack item is in view: another tag that none carries is missing.
            if tag.starts_with("minecraft:") {
                if self.cx.lints() {
                    let at = At::new(place.origin, at![path; "tag"], Severity::Info);
                    self.out.push(unresolved(at, kinds::ITEM_TAG, tag, None));
                }
                return false;
            }
            let text = formatted(
                "The Item Tag: {} is missing or invalid, can't make the recipe",
                &[tag],
            );
            let at = At::new(place.origin, at![path; "tag"], Severity::Error).with_text(text);
            self.out.push(unresolved(at, kinds::ITEM_TAG, tag, None));
            return true;
        };
        let count = ingredient.count();
        let Some(item) = carriers.first_below(count) else {
            return false;
        };
        let text = formatted(
            "The Item from tag '{}': {} has invalid stack size: {}",
            &[tag, item.as_str(), &count.to_string()],
        );
        let at = At::new(place.origin, at![path; "count"], Severity::Error).with_text(text);
        self.out.push(invalid(at, kinds::ITEM, item.as_str()));
        true
    }

    /// Smithing transform: the addition is an item that carries
    /// `minecraft:transform_materials`. An item that is missing was told by the item rule; an
    /// addition taken by tag is every item of that tag, which the pass cannot list for the game.
    fn transform_addition(&mut self, place: &Place<'_>, path: JsonPath, addition: &Ingredient) {
        let (written, member) = match addition {
            Ingredient::Name(text) => (text.as_str(), path),
            Ingredient::Object(object) => match &object.item {
                Some(text) => (text.as_str(), at![&path; "item"]),
                None => return,
            },
        };
        let Some(name) = addition.item_name() else {
            return;
        };
        let id = ItemId::parse(name.id());
        // An empty name was told when the recipe loaded, a missing item by the item rule.
        if written.is_empty() || self.cx.item(&id).is_missing() {
            return;
        }
        let carries = match self.cx.item_definition(&id) {
            Target::Defined(_) => self.carries(&id, TRANSFORM_MATERIALS),
            Target::BuiltIn => VANILLA_TRANSFORM_MATERIALS.contains(&id.as_str()),
            // A block's item carries no item tag.
            _ => false,
        };
        if carries {
            return;
        }
        let text = formatted(
            "Smithing Transform Recipe: {} addition item must have the '{}' tag.",
            &[place.id, TRANSFORM_MATERIALS],
        );
        let at = At::new(place.origin, member, Severity::Error).with_text(text);
        self.out.push(invalid(at, kinds::ITEM, written));
    }

    /// Whether the item `id` of the stack carries `tag`.
    fn carries(&self, id: &ItemId, tag: &str) -> bool {
        let item = self.cx.behavior().items().get(id);
        item.and_then(|item| item.value().components.get::<Tags>())
            .is_some_and(|tags| tags.tags.iter().any(|own| own == tag))
    }

    /// Smithing trim: the template is the
    /// template item of a pattern, the addition the item of a material. An ingredient taken
    /// by tag is every item that carries the tag, and the game holds each of them to the
    /// table: the first item of the stack with the tag that is none is told.
    fn trim_part(&mut self, place: &Place<'_>, path: JsonPath, ingredient: &Ingredient, part: Trim) {
        // In a world below 1.20 the recipe did not load.
        let Some(trims) = &self.trims else {
            return;
        };
        let registered = |item: &ItemId| match part {
            Trim::Pattern => trims.pattern_for_item(item).is_some(),
            Trim::Material => trims.material_for_item(item).is_some(),
        };
        let (member, item) = if let Some(tag) = ingredient.tag() {
            let (own, stray) = match part {
                Trim::Pattern => (TRIM_TEMPLATES, self.strays[0]),
                Trim::Material => (TRIM_MATERIALS, self.strays[1]),
            };
            // Of another tag, the items the game gives it are not known here.
            match stray.filter(|_| tag == own) {
                Some(item) => (at![&path; "tag"], item.clone()),
                None => return,
            }
        } else {
            let (written, member) = match ingredient {
                Ingredient::Name(text) => (text.as_str(), path),
                Ingredient::Object(object) => match &object.item {
                    Some(text) => (text.as_str(), at![&path; "item"]),
                    None => return,
                },
            };
            let Some(name) = ingredient.item_name().filter(|_| !written.is_empty()) else {
                return;
            };
            let id = ItemId::parse(name.id());
            if registered(&id) {
                return;
            }
            (member, id)
        };
        let template = match part {
            Trim::Pattern => {
                "Smithing Trim Recipe: {} has {} as a template ingredient, which should be a valid trim pattern, but isn't. Did you register it in the Trim Pattern Registry?"
            }
            Trim::Material => {
                "Smithing Trim Recipe: {} has {} as an addition ingredient, which should be a valid trim material, but isn't."
            }
        };
        let text = formatted(template, &[place.id, item.as_str()]);
        let at = At::new(place.origin, member, Severity::Error).with_text(text);
        self.out.push(invalid(at, kinds::ITEM, item.as_str()));
    }
}

/// Which trim table an ingredient of a smithing trim is held to.
#[derive(Debug, Clone, Copy)]
enum Trim {
    Pattern,
    Material,
}

/// The recipe a finding is about.
#[derive(Debug)]
struct Place<'r> {
    /// Its identifier, as vanilla's messages name it.
    id: &'r str,
    origin: &'r Origin,
    /// The findings of its load, which know where the file has the entries of a list the
    /// reader left an entry out of.
    found: &'r Diagnostics,
}

/// Vanilla's text `template` with `args` in place of each `{}`.
fn formatted(template: &'static str, args: &[&str]) -> String {
    sentence(template, args)
}
