//! Loading recipes: `Content` / `Definition`, the walk of `recipes/`, the per-pack duplicate
//! rule and the combine into one registry per namespace.

use std::collections::{HashMap, HashSet};
use std::fmt;

use super::parse::{Parsed, parse_document};
use super::{Recipe, RecipeNamespace, Recipes};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Phase};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Origin, Placed,
    Registry, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::{RecipeId, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

impl bedrock_addon_core::__private::Sealed for Recipe {}

impl Content for Recipe {
    const NAME: &'static str = "recipe";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Parses one recipe file as a file of a pack loaded with `cx`; the same-pack and
    /// cross-pack rules of the stack do not apply to a single file.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Behavior, path, bytes, cx).result
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl JsonContent for Recipe {}

impl Definition for Recipe {
    type Kind = kind::Recipe;
    type File = Recipe;
    type Beta = NoBeta<kind::Recipe>;
}

impl Placed for Recipe {
    /// `recipes/<identifier>.json`.
    fn default_path(&self) -> PackPath {
        default_path("recipes", self.identifier())
    }
}

/// One file's outcome: the pack, what the loader read of it before deciding, and the document
/// or its rejection.
pub struct FileOutcome {
    pack: PackRef,
    path: PackPath,
    parsed: Parsed,
    pub(super) result: Result<Document<Recipe>, Rejected>,
}

/// The location only.
impl fmt::Debug for FileOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileOutcome")
            .field("pack", &self.pack.name())
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

/// Loads one recipe file of `pack`.
pub(super) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> FileOutcome {
    let origin = Origin::file(pack.clone(), side, path.clone());
    // A number no double holds is read as the largest finite one, with a warning, so the document can always be written back.
    let finite = bedrock_addon_core::document::FiniteText::of(bytes);
    let (parsed, value, version, diagnostics) = parse_document(finite.bytes(), path, cx);
    let mut diagnostics = diagnostics;
    // An ingredient's `data` reports a number beyond `f32` itself, at the number's own path, so
    // the clamp is not reported a second time.
    finite.settle_unless_said(&mut diagnostics, None);
    diagnostics.locate(&origin);
    let result = match value {
        Some(value) if !diagnostics.has_errors() => {
            Ok(Document::new(value, origin, version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    };
    FileOutcome {
        pack,
        path: path.clone(),
        parsed,
        result,
    }
}

// ---- the collection ----------------------------------------------------------------------

/// A recipe file: the extension is exactly `json`; other files are skipped without a word.
// The game compares the extension case-sensitively, so `Recipe.JSON` is not read.
fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// `recipes/**/*.json`, packs top-down: vanilla walks the full stack front to back and the first
/// load of an identifier wins.
const WALK: Walk = Walk {
    directories: &["recipes"],
    // Subfolders are read: add-ons organise recipes in subfolders, and the game's lookup does
    // not say it ignores them.
    depth: ListDepth::Recursive,
    accept: Accept::Fn(is_json),
    order: WalkOrder::TopDown,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> FileOutcome {
    parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx)
}

/// One registry per namespace. Inside a pack the first file of an identifier wins and a later
/// one (of any type, loaded or not) is rejected; across packs the higher pack's valid recipe
/// wins and a failed one does not reserve its identifier (`TopWinsFallback`, silent).
fn combine(parsed: Vec<FileOutcome>, stack: &StackCx<'_>) -> Recipes {
    let builder = || {
        RegistryBuilder::new(
            CombineRule::TopWinsFallback { fallback: None },
            CombineRule::top_wins(),
            stack.options,
        )
    };
    let mut builders: HashMap<RecipeNamespace, RegistryBuilder<Recipe>> = RecipeNamespace::ALL
        .iter()
        .map(|namespace| (*namespace, builder()))
        .collect();
    let mut seen: HashMap<PackRef, HashSet<RecipeId>> = HashMap::new();
    let mut untyped = Vec::new();
    for outcome in parsed {
        let FileOutcome {
            pack,
            path,
            parsed,
            result,
        } = outcome;
        let Some(kind) = parsed.kind else {
            if let Err(rejected) = result {
                untyped.push(rejected);
            }
            continue;
        };
        let builder = builders
            .get_mut(&kind.namespace())
            .unwrap_or_else(|| unreachable!("every namespace has a builder"));
        let Some(id) = parsed.identifier else {
            if let Err(rejected) = result {
                builder.push(Contribution::Rejected(None, rejected));
            }
            continue;
        };
        if !seen.entry(pack).or_default().insert(id.clone()) {
            // The game's text is `JSON: %s has duplicate identifier in same pack: %s`.
            let (origin, mut diagnostics) = match result {
                Ok(document) => {
                    let origin = document.origin().clone();
                    (origin, document.diagnostics().clone())
                }
                Err(rejected) => (rejected.origin, rejected.diagnostics),
            };
            let finding = Diagnostic::error(DiagnosticCode::DuplicateIdentifier {
                text: super::parse::formatted(
                    "JSON: {} has duplicate identifier in same pack: {}",
                    [path.as_str().to_owned(), id.as_str().to_owned()],
                ),
            })
            .phase(Phase::Validate)
            .in_file(&origin);
            diagnostics.push(finding);
            builder.push(Contribution::Rejected(
                Some(id),
                Rejected::new(origin, diagnostics),
            ));
            continue;
        }
        builder.push(match result {
            Ok(document) => Contribution::Stable(id, document),
            Err(rejected) => Contribution::Rejected(Some(id), rejected),
        });
    }
    let mut finish = |namespace: RecipeNamespace| -> Registry<Recipe> {
        builders
            .remove(&namespace)
            .unwrap_or_else(|| unreachable!("every namespace has a builder"))
            .finish()
    };
    Recipes {
        crafting: finish(RecipeNamespace::Crafting),
        furnace: finish(RecipeNamespace::Furnace),
        brewing_mix: finish(RecipeNamespace::BrewingMix),
        brewing_container: finish(RecipeNamespace::BrewingContainer),
        material_reduction: finish(RecipeNamespace::MaterialReduction),
        smithing_transform: finish(RecipeNamespace::SmithingTransform),
        smithing_trim: finish(RecipeNamespace::SmithingTrim),
        untyped,
    }
}

/// The loader of `bp.recipes()`: `recipes/**/*.json` of every behaviour pack, one registry per
/// namespace.
pub const fn load_recipes() -> CollectionLoader<Recipes, FileOutcome> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}

impl bedrock_addon_core::load::Reported for Recipes {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

/// Every registry, then the files rejected before their type was known.
impl fmt::Debug for Recipes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Recipes")
            .field("crafting", &self.crafting)
            .field("furnace", &self.furnace)
            .field("brewing_mix", &self.brewing_mix)
            .field("brewing_container", &self.brewing_container)
            .field("material_reduction", &self.material_reduction)
            .field("smithing_transform", &self.smithing_transform)
            .field("smithing_trim", &self.smithing_trim)
            .field("untyped", &self.untyped)
            .finish()
    }
}
