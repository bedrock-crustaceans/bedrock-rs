//! Reading a recipe file the way vanilla's hand-written walker does: one walk over the JSON
//! document, no parser table, no struct per version. The walk is over an untyped `Value` because
//! vanilla's coercions (a non-bool `assume_symmetry` ignored, a non-integer `count` ignored, a
//! string or a list for `tags` and `result`) are not typed shapes. Nothing is deserialised from
//! the `Value`, so members are read in vanilla's key order and unknown members go to the `extra`
//! of their level with an info finding.

#![allow(
    clippy::single_match_else,
    reason = "each member is read as found or not, as vanilla's branches read it"
)]

use std::collections::HashSet;

use super::ingredient::{constant_data, parse_item, potion};
use super::new::MISSING_VERSION;
use super::{
    BrewingContainer, BrewingMix, Description, Furnace, Ingredient, IngredientObject,
    MaterialReduction, Output, Recipe, RecipeKind, Shaped, Shapeless, SmithingTransform,
    SmithingTrim, Tags, Unlock, UnlockContext, input_count_allowed, unlock_required,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::{Extra, VersionInfo};
use bedrock_addon_core::id::RecipeId;
use bedrock_addon_core::json::Prepared;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{
    JsonType, MolangList, MolangSource, ReadField, ReaderKind, ReprError, beyond_f32_warning,
    json_scalar, read_finite,
};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::{FormatVersion, SemVersion};

/// What the loader learnt of a file before deciding on it: its type and identifier.
#[derive(Clone, Debug, Default)]
pub(super) struct Parsed {
    /// The payload's type, when the file has a payload key.
    pub(super) kind: Option<RecipeKind>,
    /// The identifier, when the payload has a non-empty one.
    pub(super) identifier: Option<RecipeId>,
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "every call site fills an `Option<VanillaText>` field"
)]
/// A vanilla text with arguments: `template` with each `{}` replaced by the next argument. The
/// recipe module builds every formatted text through this, so a test can check the arity.
pub(super) fn formatted<const N: usize>(
    template: &'static str,
    args: [String; N],
) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.into(),
    })
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "every call site fills an `Option<VanillaText>` field"
)]
/// A vanilla text without arguments.
fn fixed(text: &'static str) -> Option<VanillaText> {
    Some(VanillaText::Fixed(text))
}

/// The smithing station tag, the only tag value the loader compares.
const SMITHING_TABLE: &str = "smithing_table";

/// The most outputs a material-reduction recipe may have.
const MAX_REDUCER_OUTPUTS: usize = 9;

/// The world version smithing trims need, and from which a transform's `template` is read: the
/// 1.20 release that added trims.
const TRAILS_AND_TALES: SemVersion = SemVersion::new(1, 20, 0);

/// The JSON value type number of an object, as vanilla's Molang reader prints it in its message.
const OBJECT_VALUE_TYPE: &str = "7";

/// Parses one recipe file: what the loader learnt of it, the recipe (also when an error-level
/// finding rejects it), its version and the findings.
pub(super) fn parse_document(
    bytes: &[u8],
    path: &PackPath,
    cx: &LoadContext,
) -> (Parsed, Option<Recipe>, VersionInfo, Diagnostics) {
    let mut reader = Reader::new(path, cx);
    let (parsed, value, written) = reader.document(bytes);
    let effective = written.unwrap_or(MISSING_VERSION);
    let version = VersionInfo::new(written, false, effective, effective, false);
    // A recipe has no upgrade to finish its load: the places of the elements the reader
    // kept are settled here, before anything else is told of the file.
    reader.diagnostics.name_as_the_file_has_it();
    (parsed, value, version, reader.diagnostics)
}

/// How an ingredient is read at one call site: whether `count` and `tag` are allowed, and
/// whether its findings are kept errors (in `unlock`, where the game logs the same error lines
/// and the recipe loads without the requirement) or rejecting ones.
#[derive(Clone, Copy)]
struct Flags {
    count: bool,
    tag: bool,
    kept: bool,
}

impl Flags {
    const fn new(count: bool, tag: bool) -> Self {
        Self {
            count,
            tag,
            kept: false,
        }
    }
}

/// The members of one JSON object as the walker reads them: the ones read into a typed field,
/// the ones moved to `extra` with a finding already raised, and the rest, unknown.
struct Members<'v> {
    object: &'v facet_value::VObject,
    read: HashSet<String>,
    quiet: HashSet<String>,
}

impl<'v> Members<'v> {
    fn new(object: &'v facet_value::VObject) -> Self {
        Self {
            object,
            read: HashSet::new(),
            quiet: HashSet::new(),
        }
    }

    /// Member `key`, now read.
    fn take(&mut self, key: &str) -> Option<&'v Value> {
        let value = self.object.get(key)?;
        self.read.insert(key.to_owned());
        Some(value)
    }

    /// Member `key` is kept in `extra` after all, its finding already raised.
    fn keep(&mut self, key: &str) {
        self.read.remove(key);
        self.quiet.insert(key.to_owned());
    }
}

/// One file's walk: where findings go and the context they depend on.
struct Reader<'c> {
    path: &'c PackPath,
    cx: &'c LoadContext,
    diagnostics: Diagnostics,
    /// The identifier, for the texts that name it.
    id: String,
    /// The effective `format_version`.
    version: FormatVersion,
    /// Molang `data` expressions not evaluated at load.
    not_checked: u32,
}

impl<'c> Reader<'c> {
    fn new(path: &'c PackPath, cx: &'c LoadContext) -> Self {
        Self {
            path,
            cx,
            diagnostics: Diagnostics::default(),
            id: String::new(),
            version: MISSING_VERSION,
            not_checked: 0,
        }
    }

    fn report(&mut self, at: &JsonPath, code: DiagnosticCode, severity: Severity) {
        self.diagnostics
            .push(Diagnostic::error(code).severity(severity).at(at.clone()));
    }

    /// A kept error: a line the game logs at error level and then goes on, so the recipe still
    /// loads. It is an error finding that does not reject the document.
    fn report_kept(&mut self, at: &JsonPath, code: DiagnosticCode) {
        self.diagnostics
            .push_kept_error(Diagnostic::error(code).at(at.clone()));
    }

    /// An ingredient's own finding: kept or rejecting as `flags` says.
    fn report_ingredient(&mut self, at: &JsonPath, code: DiagnosticCode, flags: Flags) {
        if flags.kept {
            self.report_kept(at, code);
        } else {
            self.report(at, code, Severity::Error);
        }
    }

    /// The members of `members` nobody read, kept in an `Extra`; the unknown ones raise
    /// `UnrecognizedField` at info (vanilla reads no other key and says nothing).
    fn rest(&mut self, members: &Members<'_>, at: &JsonPath) -> Extra {
        let mut extra = Extra::default();
        for (key, value) in members.object {
            let key = key.as_str();
            if members.read.contains(key) {
                continue;
            }
            if !members.quiet.contains(key) {
                self.report(
                    &at.child(key),
                    DiagnosticCode::UnrecognizedField { text: None },
                    Severity::Info,
                );
            }
            extra.insert(key.to_owned(), value.clone());
        }
        extra
    }

    // ---- the document --------------------------------------------------------------------

    fn document(&mut self, bytes: &[u8]) -> (Parsed, Option<Recipe>, Option<FormatVersion>) {
        let mut parsed = Parsed::default();
        let root_path = JsonPath::new();
        let Ok(prepared) = Prepared::new(bytes) else {
            self.report(
                &root_path,
                DiagnosticCode::MalformedInput {
                    message: "not UTF-8 JSON within the nesting limit".to_owned(),
                    text: None,
                },
                Severity::Error,
            );
            return (parsed, None, None);
        };
        // Of a member written twice the last stands.
        let cleaned = bedrock_addon_core::json::last_wins(prepared.as_str());
        let text: &str = &cleaned;
        // The game's jsoncpp reader accepts comments.
        let root: Value = match bedrock_addon_core::json::from_str_jsonc(text) {
            Ok(root) => root,
            Err(error) => {
                let message = error.to_string();
                // The game's text is `JSON: %s has an error: \n %s`.
                self.report(
                    &root_path,
                    DiagnosticCode::MalformedInput {
                        message: message.clone(),
                        text: formatted(
                            "JSON: {} has an error: \n {}",
                            [self.path.as_str().to_owned(), message],
                        ),
                    },
                    Severity::Error,
                );
                return (parsed, None, None);
            }
        };
        let Some(object) = root.as_object() else {
            // The game's text is `JSON: %s root value is not a object`.
            self.report(
                &root_path,
                DiagnosticCode::MalformedInput {
                    message: "the root is not an object".to_owned(),
                    text: formatted(
                        "JSON: {} root value is not a object",
                        [self.path.as_str().to_owned()],
                    ),
                },
                Severity::Error,
            );
            return (parsed, None, None);
        };
        let mut root_members = Members::new(object);
        let written = self.format_version(&mut root_members);
        self.version = written.unwrap_or(MISSING_VERSION);
        // The first present payload key, `null` counting as absent.
        let payload = RecipeKind::ALL.iter().find_map(|kind| {
            let value = object.get(kind.payload_key())?;
            (!value.is_null()).then_some((*kind, value))
        });
        for kind in RecipeKind::ALL {
            let key = kind.payload_key();
            if object.contains_key(key) && payload.is_none_or(|(chosen, _)| chosen != *kind) {
                root_members.keep(key);
                self.report(
                    &root_path.child(key),
                    DiagnosticCode::IgnoredPayload,
                    Severity::Info,
                );
            }
        }
        let Some((kind, payload)) = payload else {
            // The game reads the identifier from the (null) payload first, so a file with no
            // payload is reported as one without an identifier.
            self.no_identifier(&root_path);
            return (parsed, None, written);
        };
        root_members.take(kind.payload_key());
        parsed.kind = Some(kind);
        let at = root_path.child(kind.payload_key());
        let Some(payload) = payload.as_object() else {
            self.no_identifier(&at);
            return (parsed, None, written);
        };
        let Some(mut recipe) = self.payload(kind, payload, &at, &mut parsed) else {
            return (parsed, None, written);
        };
        let root_extra = self.rest(&root_members, &root_path);
        super::new::each!(&mut recipe, r => {
            r.format_version = written;
            r.root_extra = root_extra;
        });
        if self.not_checked > 0 {
            self.report(
                &root_path,
                DiagnosticCode::MolangNotChecked {
                    fields: self.not_checked,
                },
                Severity::Info,
            );
        }
        (parsed, Some(recipe), written)
    }

    /// The payload of `kind`: its description, tags and the type's members.
    fn payload(
        &mut self,
        kind: RecipeKind,
        payload: &facet_value::VObject,
        at: &JsonPath,
        parsed: &mut Parsed,
    ) -> Option<Recipe> {
        let mut members = Members::new(payload);
        let Some(description) = self.description(&mut members, at) else {
            self.no_identifier(at);
            return None;
        };
        parsed.identifier = Some(description.identifier.clone());
        description.identifier.as_str().clone_into(&mut self.id);
        let tags = self.tags(&mut members, at, kind);
        let mut recipe = match kind {
            RecipeKind::Shaped => self.shaped(&mut members, at),
            RecipeKind::Shapeless => self.shapeless(&mut members, at),
            RecipeKind::Furnace => self.furnace(&mut members, at),
            RecipeKind::BrewingMix => self.brewing_mix(&mut members, at),
            RecipeKind::BrewingContainer => self.brewing_container(&mut members, at),
            RecipeKind::MaterialReduction => self.material_reduction(&mut members, at),
            RecipeKind::SmithingTransform => {
                self.smithing_transform(&mut members, at, tags.as_ref())
            }
            RecipeKind::SmithingTrim => self.smithing_trim(&mut members, at, tags.as_ref()),
        };
        let extra = self.rest(&members, at);
        super::new::each!(&mut recipe, r => {
            r.description = description;
            r.tags = tags;
            r.extra = extra;
        });
        Some(recipe)
    }

    /// `JSON: %s has no identifier`.
    fn no_identifier(&mut self, at: &JsonPath) {
        self.report(
            at,
            DiagnosticCode::MissingIdentifier {
                text: formatted(
                    "JSON: {} has no identifier",
                    [self.path.as_str().to_owned()],
                ),
            },
            Severity::Error,
        );
    }

    /// `format_version`: a string `a.b[.c]` or three unsigned integers; anything else, or none,
    /// is 1.0.0 with `Missing version tag` (a kept error: the game logs an error and the recipe
    /// still loads). Above the engine version: an info, there is no ceiling.
    fn format_version(&mut self, root: &mut Members<'_>) -> Option<FormatVersion> {
        let at = JsonPath::of_keys(["format_version"]);
        let value = root.take("format_version");
        let version = value
            .and_then(SemVersion::from_json)
            .filter(|version| !version.is_any())
            .map(|version| version.format_version());
        let Some(version) = version else {
            if value.is_some() {
                root.keep("format_version");
            }
            // The game logs the line as an error and goes on with 1.0.0, so it is a kept error.
            self.report_kept(
                &at,
                DiagnosticCode::MissingFormatVersion {
                    text: fixed("Missing version tag"),
                },
            );
            return None;
        };
        if version > self.cx.engine_version {
            self.report(
                &at,
                DiagnosticCode::FormatVersionAboveEngine {
                    version: version.to_string(),
                },
                Severity::Info,
            );
        }
        Some(version)
    }

    /// `description`, with a non-empty string `identifier` (`asString("")`: any other value is
    /// empty).
    fn description(&mut self, payload: &mut Members<'_>, at: &JsonPath) -> Option<Description> {
        let object = payload.take("description")?.as_object()?;
        let identifier = object.get("identifier")?.as_string()?.as_str();
        if identifier.is_empty() {
            return None;
        }
        let at = at.child("description");
        let mut members = Members::new(object);
        members.take("identifier");
        let extra = self.rest(&members, &at);
        Some(Description {
            identifier: RecipeId::parse(identifier.to_owned()),
            extra,
        })
    }

    /// `tags`: a string or a list of strings (other entries skipped); any other value counts as
    /// none.
    fn tags(&mut self, payload: &mut Members<'_>, at: &JsonPath, kind: RecipeKind) -> Option<Tags> {
        let path = at.child("tags");
        let tags = match payload.take("tags") {
            None => None,
            Some(value) => {
                if let Some(tag) = value.as_string() {
                    Some(Tags::One(tag.as_str().to_owned()))
                } else if let Some(entries) = value.as_array() {
                    let mut tags = Vec::new();
                    // The file's index of each tag held, for the link pass.
                    let mut kept = Vec::new();
                    for (index, entry) in entries.iter().enumerate() {
                        match entry.as_string() {
                            Some(tag) => {
                                tags.push(tag.as_str().to_owned());
                                kept.push(index);
                            }
                            None => self.report(
                                &path.index(index),
                                DiagnosticCode::InvalidType { text: None },
                                Severity::Info,
                            ),
                        }
                    }
                    self.diagnostics.left_out(&path, &kept);
                    Some(Tags::Many(tags))
                } else {
                    payload.keep("tags");
                    self.report(
                        &path,
                        DiagnosticCode::InvalidType { text: None },
                        Severity::Info,
                    );
                    None
                }
            }
        };
        let list = tags.as_ref().map_or(&[][..], Tags::as_slice);
        if list.is_empty() {
            // `No tags specified`: the game logs it as an error and goes on; one recipe object
            // is registered per tag, so nothing is registered. A kept error.
            self.report_kept(
                &path,
                DiagnosticCode::NoEffect {
                    text: fixed("No tags specified"),
                },
            );
        }
        let smithing = matches!(
            kind,
            RecipeKind::SmithingTransform | RecipeKind::SmithingTrim
        );
        if !smithing && list.iter().any(|tag| tag == SMITHING_TABLE) {
            // The game logs the misuse as an error and goes on loading: a kept error.
            self.report_kept(
                &path,
                DiagnosticCode::InvalidValue {
                    text: fixed(
                        "smithing_table tag is only supported by recipes of type recipe_smithing_transform and of type recipe_smithing_trim",
                    ),
                },
            );
        }
        tags
    }

    // ---- ingredients ---------------------------------------------------------------------

    /// One ingredient; `None` when it is invalid, with the
    /// ingredient's own finding kept or rejecting as `flags.kept` says.
    fn ingredient(
        &mut self,
        value: Option<&Value>,
        at: &JsonPath,
        flags: Flags,
    ) -> Option<Ingredient> {
        let Some(value) = value.filter(|value| !value.is_null()) else {
            // A null value goes to the item lookup with an empty name.
            self.empty_item(at, flags, true);
            return None;
        };
        if let Some(text) = value.as_string() {
            let text = text.as_str();
            self.item_text(text, at);
            return Some(Ingredient::Name(text.to_owned()));
        }
        let Some(object) = value.as_object() else {
            self.empty_item(at, flags, false);
            return None;
        };
        let mut members = Members::new(object);
        let mut ingredient = IngredientObject::default();
        // Vanilla reads the members in the order `data`, `count`, `item`, `tag`, and the first
        // failure ends the ingredient.
        self.data(&mut members, &mut ingredient, at);
        self.count(&mut members, &mut ingredient, at, flags)?;
        self.item_or_tag(&mut members, &mut ingredient, at, flags)?;
        ingredient.extra = self.rest(&members, at);
        Some(Ingredient::Object(ingredient))
    }

    /// `data`, read only when not `empty()`: a number or Molang, evaluated at load.
    fn data(
        &mut self,
        members: &mut Members<'_>,
        ingredient: &mut IngredientObject,
        at: &JsonPath,
    ) {
        let Some(data) = members.take("data") else {
            return;
        };
        if is_empty(data) {
            members.keep("data");
            return;
        }
        if let Ok(list) = self.data_list(data, &mut at.child("data")) {
            ingredient.data_value = self.evaluate(&list, &at.child("data"));
            ingredient.data = Some(list);
            return;
        }
        members.keep("data");
        // An object `data` makes the game log "Type not convertible to Molang expression: %d"
        // as an error and go on with an empty expression, which evaluates to 0: a kept error.
        // A list with an element that is no value has no line in the game, so the crate's own
        // note is an information.
        if data.is_object() {
            let text = formatted(
                "Type not convertible to Molang expression: {}",
                [OBJECT_VALUE_TYPE.to_owned()],
            );
            self.diagnostics.push_kept_error(
                Diagnostic::error(DiagnosticCode::InvalidType { text }).at(at.child("data")),
            );
        } else {
            self.report(
                &at.child("data"),
                DiagnosticCode::InvalidType { text: None },
                Severity::Info,
            );
        }
        ingredient.data_value = Some(0);
    }

    /// `data` as a value or a list of them, element by element (`molang::read_list`'s shape): a
    /// number beyond `f32` is read as the largest finite float of its sign, with core's warning
    /// at the number. `at` is the path of `value`, and is left as it came.
    fn data_list(&mut self, value: &Value, at: &mut JsonPath) -> Result<MolangList, ReprError> {
        let reader = ReaderKind::ScalarOrArray;
        if let Some(elements) = value.as_array() {
            let mut list = Vec::with_capacity(elements.len());
            for (index, element) in elements.iter().enumerate() {
                at.push_index(index);
                let read = self.data_list(element, at);
                at.pop();
                list.push(read?);
            }
            return Ok(MolangList::Many(list));
        }
        match read_finite(json_scalar(value), reader, self.cx.molang_version)? {
            (ReadField::Value(value), clamped) => {
                if clamped {
                    self.diagnostics.push(beyond_f32_warning(at.clone(), None));
                }
                Ok(MolangList::One(value))
            }
            // `null` here is an element: the member's own `null` is `is_empty`.
            _ => Err(ReprError::UnsupportedType {
                reader,
                found: JsonType::Null,
            }),
        }
    }

    /// `count`: refused where the call site does not allow it, which makes the ingredient
    /// invalid; a non-integer is ignored (stays 1).
    fn count(
        &mut self,
        members: &mut Members<'_>,
        ingredient: &mut IngredientObject,
        at: &JsonPath,
        flags: Flags,
    ) -> Option<()> {
        let Some(count) = members.take("count") else {
            return Some(());
        };
        if is_empty(count) {
            members.keep("count");
            return Some(());
        }
        if !flags.count {
            self.report_ingredient(
                &at.child("count"),
                DiagnosticCode::InvalidValue {
                    text: fixed(
                        "Recipe ingredient is invalid: Cannot specify an item count for input items",
                    ),
                },
                flags,
            );
            return None;
        }
        if let Some(count) = integer(count) {
            ingredient.count = Some(count);
        } else {
            members.keep("count");
            self.report(
                &at.child("count"),
                DiagnosticCode::InvalidType { text: None },
                Severity::Info,
            );
        }
        Some(())
    }

    /// `item`, or where tags are allowed and `item` is not a string, `tag`.
    fn item_or_tag(
        &mut self,
        members: &mut Members<'_>,
        ingredient: &mut IngredientObject,
        at: &JsonPath,
        flags: Flags,
    ) -> Option<()> {
        let item = members.take("item");
        if let Some(text) = item.and_then(Value::as_string) {
            self.item_text(text.as_str(), &at.child("item"));
            ingredient.item = Some(text.as_str().to_owned());
            // `tag` beside a string `item` is not read; kept as written.
            if let Some(tag) = members.take("tag") {
                match tag.as_string() {
                    Some(tag) => ingredient.tag = Some(tag.as_str().to_owned()),
                    None => members.keep("tag"),
                }
            }
            return Some(());
        }
        if item.is_some() {
            members.keep("item");
        }
        if !flags.tag {
            self.report_ingredient(
                at,
                DiagnosticCode::InvalidValue {
                    text: fixed("Recipe ingredient is invalid: Must contain a valid item name"),
                },
                flags,
            );
            return None;
        }
        if let Some(tag) = members.take("tag").and_then(Value::as_string) {
            ingredient.tag = Some(tag.as_str().to_owned());
            return Some(());
        }
        // Neither an item nor a tag: the game's error line for this case.
        self.report_ingredient(
            at,
            DiagnosticCode::InvalidValue {
                text: fixed(
                    "Recipe ingredient is invalid: Must contain either a valid item name or item tag",
                ),
            },
            flags,
        );
        None
    }

    /// An ingredient that is absent, or neither a string nor an object: the item lookup with an
    /// empty name fails ("The Item: %s is missing or invalid, can't make the recipe").
    fn empty_item(&mut self, at: &JsonPath, flags: Flags, missing: bool) {
        let text = formatted(
            "The Item: {} is missing or invalid, can't make the recipe",
            [String::new()],
        );
        let code = if missing {
            DiagnosticCode::MissingRequiredField { text }
        } else {
            DiagnosticCode::InvalidType { text }
        };
        self.report_ingredient(at, code, flags);
    }

    /// What vanilla's item-name parser logs about an item text. The game logs each line as an
    /// error and goes on with the tokens it has (a missing namespace is `minecraft`), so each
    /// is a kept error.
    fn item_text(&mut self, text: &str, at: &JsonPath) {
        if potion(text).is_some() {
            return;
        }
        for problem in parse_item(text).1 {
            self.diagnostics.push_kept_error(
                Diagnostic::error(DiagnosticCode::InvalidValue {
                    text: formatted(problem.template(), [text.to_owned()]),
                })
                .at(at.clone()),
            );
        }
    }

    /// The value `data` evaluates to at load: a constant directly; an expression compiled with
    /// the pack's version and resolved when it folds (a rejected one is 0); a list is no number.
    fn evaluate(&mut self, data: &MolangList, at: &JsonPath) -> Option<i32> {
        match data {
            MolangList::One(bedrock_addon_core::molang::Molang::Expr(source)) => {
                self.expression(source, at)
            }
            MolangList::One(_) => constant_data(data),
            MolangList::Many(_) => {
                #[cfg(feature = "molang")]
                if let Some(check) = self.cx.molang.clone() {
                    let mcx = self.molang_context();
                    let location =
                        bedrock_addon_core::diagnostic::Location::new(None, None, at.clone(), None);
                    let mut found = Diagnostics::default();
                    check.check_list(
                        data,
                        &DATA_SPEC,
                        bedrock_addon_core::molang::Side::Server,
                        &mcx,
                        &location,
                        &mut found,
                    );
                    self.diagnostics.extend(found);
                }
                // An array `data` yields no number here, and the game logs no line for it, so
                // the crate's own note is an information.
                self.report(
                    at,
                    DiagnosticCode::InvalidType { text: None },
                    Severity::Info,
                );
                None
            }
        }
    }

    #[cfg(feature = "molang")]
    fn molang_context(&self) -> bedrock_addon_core::molang::MolangContext {
        let document = bedrock_addon_core::version::FormatVersionInput {
            version: Some(self.version),
            beta_literal: false,
            prerelease: false,
        };
        bedrock_addon_core::molang::MolangContext::new(
            self.cx.min_engine_version.as_ref(),
            self.cx.molang_experiments,
            &document,
            Some(self.cx.molang_version),
        )
    }

    /// One `data` expression: compiled with the `molang` feature and a compile check; a rejected
    /// expression is a kept error (or an information when the game logs no line) and 0, one
    /// that folds gives its value, and any other is not evaluated, because running a Molang
    /// program is out of scope for the loader.
    fn expression(&mut self, source: &MolangSource, at: &JsonPath) -> Option<i32> {
        #[cfg(feature = "molang")]
        if let Some(check) = self.cx.molang.clone() {
            use bedrock_addon_core::molang::{MolangOutcome, ReportRules, Side};
            let mcx = self.molang_context();
            let location =
                bedrock_addon_core::diagnostic::Location::new(None, None, at.clone(), None);
            let mut found = Diagnostics::default();
            let (outcome, checked) = check.check_source_with(
                source,
                source,
                &DATA_SPEC,
                Side::Server,
                &mcx,
                &location,
                &ReportRules::LOAD,
                &mut found,
            );
            let rejected = matches!(outcome, MolangOutcome::Failed(_));
            for mut finding in found {
                // A rejection is an `InvalidMolang`: the parser's line is a kept error
                // (`DATA_SPEC`); one carrying only an info note (an empty text below Molang
                // version 4, which the game rejects without a line) is an information of the
                // crate's own.
                if rejected
                    && finding.severity < Severity::Warning
                    && let DiagnosticCode::MolangLint { expression } = &finding.code
                {
                    finding.code = DiagnosticCode::InvalidMolang {
                        expression: expression.clone(),
                    };
                }
                self.diagnostics.push(finding);
            }
            if rejected {
                return Some(0);
            }
            if let Some(value) = checked.as_ref().and_then(|checked| {
                checked
                    .expr()
                    .and_then(bedrock_addon_core::molang::molang::compile::Expr::as_constant)
            }) {
                return Some(super::ingredient::truncate(value));
            }
        }
        let _ = (source, at);
        self.not_checked += 1;
        None
    }

    /// An output or result list: one ingredient or a list of them (`null` entries skipped).
    fn output(&mut self, value: Option<&Value>, at: &JsonPath, flags: Flags) -> Option<Output> {
        if let Some(entries) = value.and_then(Value::as_array) {
            let mut items = Vec::new();
            // The file's index of each entry held, for the link pass.
            let mut kept = Vec::new();
            for (index, entry) in entries.iter().enumerate() {
                let entry_at = at.index(index);
                if entry.is_null() {
                    self.report(
                        &entry_at,
                        DiagnosticCode::IgnoredMember { text: None },
                        Severity::Info,
                    );
                    continue;
                }
                items.push(self.ingredient(Some(entry), &entry_at, flags)?);
                kept.push(index);
            }
            self.diagnostics.left_out(at, &kept);
            return Some(Output::Many(items));
        }
        self.ingredient(value, at, flags).map(Output::One)
    }

    /// `Recipe result malformed` (also the text of a bad shapeless ingredient and of a bad
    /// furnace input or output).
    fn result_malformed(&mut self, at: &JsonPath) {
        self.report(
            at,
            DiagnosticCode::DocumentRejected {
                text: fixed("Recipe result malformed"),
            },
            Severity::Error,
        );
    }

    /// `result` of shaped and shapeless recipes: count allowed, tags not.
    fn result(&mut self, payload: &mut Members<'_>, at: &JsonPath) -> Output {
        let at = at.child("result");
        match self.output(payload.take("result"), &at, Flags::new(true, false)) {
            None => {
                self.result_malformed(&at);
                Output::default()
            }
            Some(Output::Many(items)) if items.is_empty() => {
                self.report(
                    &at,
                    DiagnosticCode::DocumentRejected {
                        text: fixed("Recipe has no result item(s)"),
                    },
                    Severity::Error,
                );
                Output::Many(items)
            }
            Some(output) => output,
        }
    }

    /// `unlock`: the malformed forms are logged and give the requirement `{None, []}`; only a value that is neither a list nor an object with a string
    /// `context` counts as absent.
    fn unlock(&mut self, payload: &mut Members<'_>, at: &JsonPath) -> Option<Unlock> {
        let at = at.child("unlock");
        let value = payload.take("unlock")?;
        if value.is_null() {
            payload.keep("unlock");
            return None;
        }
        if let Some(entries) = value.as_array() {
            if entries.is_empty() {
                // The game logs this as an error and returns the requirement `{None, []}`: a
                // kept error.
                self.report_kept(
                    &at,
                    DiagnosticCode::InvalidValue {
                        text: formatted(
                            "Recipe: {} has an empty unlocking ingredient array",
                            [self.id.clone()],
                        ),
                    },
                );
                return Some(Unlock::Ingredients(Vec::new()));
            }
            let flags = Flags {
                kept: true,
                ..Flags::new(true, true)
            };
            let mut ingredients = Vec::new();
            for (index, entry) in entries.iter().enumerate() {
                match self.ingredient(Some(entry), &at.index(index), flags) {
                    Some(ingredient) => ingredients.push(ingredient),
                    None => {
                        // The game logs an error, zeroes the requirement and loads the
                        // recipe: a kept error.
                        self.report_kept(
                            &at,
                            DiagnosticCode::InvalidValue {
                                text: formatted(
                                    "Recipe: {} has an invalid unlock ingredient",
                                    [self.id.clone()],
                                ),
                            },
                        );
                        return Some(Unlock::Malformed(value.clone()));
                    }
                }
            }
            return Some(Unlock::Ingredients(ingredients));
        }
        let context = value.as_object().and_then(|object| {
            let name = object.get("context")?.as_string()?.as_str();
            Some((object, name))
        });
        let Some((object, name)) = context else {
            // The game logs an error and then returns normally, so the recipe still loads
            // (the `unlock` gate below decides whether a missing requirement is fatal): a
            // kept error.
            payload.keep("unlock");
            self.report_kept(
                &at,
                DiagnosticCode::InvalidType {
                    text: formatted(
                        "Recipe: {} has an unlocking element but does not contain an ingredient list or and unlocking context",
                        [self.id.clone()],
                    ),
                },
            );
            return None;
        };
        let mut members = Members::new(object);
        members.take("context");
        let Some(context) = UnlockContext::from_name(name) else {
            // The game logs an error and goes on loading: a kept error.
            self.report_kept(
                &at.child("context"),
                DiagnosticCode::InvalidValue {
                    text: formatted(
                        "Recipe: {} has a malformed unlocking context",
                        [self.id.clone()],
                    ),
                },
            );
            return Some(Unlock::Malformed(value.clone()));
        };
        let extra = self.rest(&members, &at);
        Some(Unlock::Context { context, extra })
    }

    /// A shaped or shapeless recipe from 1.20.10 needs `unlock`.
    fn unlock_gate(&mut self, unlock: Option<&Unlock>, at: &JsonPath) {
        if unlock.is_none() && unlock_required(self.version) {
            self.report(
                at,
                DiagnosticCode::MissingRequiredField {
                    text: fixed("1.20+ Recipes require unlock data"),
                },
                Severity::Error,
            );
        }
    }

    /// `priority`: an integer; any other value is not read.
    fn priority(&mut self, payload: &mut Members<'_>, at: &JsonPath) -> Option<i32> {
        let value = payload.take("priority")?;
        if let Some(priority) = integer(value) {
            return Some(priority);
        }
        payload.keep("priority");
        if !is_empty(value) {
            self.report(
                &at.child("priority"),
                DiagnosticCode::InvalidType { text: None },
                Severity::Info,
            );
        }
        None
    }

    /// Whether input ingredients may carry `count` in this pack. A pack without a
    /// `min_engine_version` is read as having vanilla's zeroed version, which is legacy.
    fn input_count(&self) -> bool {
        self.cx
            .min_engine_version
            .as_ref()
            .is_none_or(input_count_allowed)
    }

    // ---- the recipe types ----------------------------------------------------------------

    fn shaped(&mut self, payload: &mut Members<'_>, at: &JsonPath) -> Recipe {
        let priority = self.priority(payload, at);
        let assume_symmetry = match payload.take("assume_symmetry") {
            Some(value) => match value.as_bool() {
                Some(value) => Some(value),
                None => {
                    // A non-bool is ignored (the default is `true`).
                    payload.keep("assume_symmetry");
                    self.report(
                        &at.child("assume_symmetry"),
                        DiagnosticCode::InvalidType { text: None },
                        Severity::Info,
                    );
                    None
                }
            },
            None => None,
        };
        let result = self.result(payload, at);
        let unlock = self.unlock(payload, at);
        self.unlock_gate(unlock.as_ref(), at);
        let pattern = self.pattern(payload, at);
        let key = self.key(payload, at);
        if let (Some(pattern), Some(key)) = (&pattern, &key) {
            self.symbols(pattern, key, at);
        }
        Recipe::Shaped(Shaped {
            priority,
            assume_symmetry,
            pattern: pattern.unwrap_or_default(),
            key: key.unwrap_or_default(),
            result,
            unlock,
            ..Shaped::default()
        })
    }

    /// `pattern`: 1 to 3 rows of 1 to 3 symbols; vanilla drops a missing, empty or mistyped one
    /// silently, so the crate's errors for them carry no game text.
    fn pattern(&mut self, payload: &mut Members<'_>, at: &JsonPath) -> Option<Vec<String>> {
        let at = at.child("pattern");
        let Some(value) = payload.take("pattern") else {
            self.report(
                &at,
                DiagnosticCode::MissingRequiredField { text: None },
                Severity::Error,
            );
            return None;
        };
        let Some(rows) = value.as_array() else {
            self.report(
                &at,
                DiagnosticCode::InvalidType { text: None },
                Severity::Error,
            );
            return None;
        };
        if rows.is_empty() {
            self.report(
                &at,
                DiagnosticCode::InvalidValue { text: None },
                Severity::Error,
            );
            return None;
        }
        let mut pattern = Vec::new();
        for (index, row) in rows.iter().enumerate() {
            match row.as_string() {
                Some(row) => pattern.push(row.as_str().to_owned()),
                None => {
                    self.report(
                        &at.index(index),
                        DiagnosticCode::InvalidType { text: None },
                        Severity::Error,
                    );
                    return None;
                }
            }
        }
        let width = pattern.iter().map(String::len).max().unwrap_or(0);
        let height = pattern.len();
        if width > 3 || height > 3 {
            // The game rejects a pattern larger than 3x3 when it normalises the grid.
            self.report(
                &at,
                DiagnosticCode::InvalidValue {
                    text: formatted(
                        "Adding a recipe larger than 3x3: {} width, {} height",
                        [width.to_string(), height.to_string()],
                    ),
                },
                Severity::Error,
            );
        }
        Some(pattern)
    }

    /// `key`: a non-empty object of input ingredients (count only in legacy packs, tags
    /// allowed).
    fn key(
        &mut self,
        payload: &mut Members<'_>,
        at: &JsonPath,
    ) -> Option<indexmap::IndexMap<String, Ingredient>> {
        let at = at.child("key");
        let Some(value) = payload.take("key") else {
            self.report(
                &at,
                DiagnosticCode::MissingRequiredField { text: None },
                Severity::Error,
            );
            return None;
        };
        let Some(object) = value.as_object() else {
            self.report(
                &at,
                DiagnosticCode::InvalidType { text: None },
                Severity::Error,
            );
            return None;
        };
        if object.is_empty() {
            self.report(
                &at,
                DiagnosticCode::InvalidValue { text: None },
                Severity::Error,
            );
            return None;
        }
        let flags = Flags::new(self.input_count(), true);
        let mut key = indexmap::IndexMap::new();
        for (name, value) in object {
            let name = name.as_str();
            let entry_at = at.child(name);
            if name.is_empty() {
                self.report(
                    &entry_at,
                    DiagnosticCode::InvalidValue { text: None },
                    Severity::Error,
                );
                return None;
            }
            match self.ingredient(Some(value), &entry_at, flags) {
                Some(ingredient) => {
                    key.insert(name.to_owned(), ingredient);
                }
                None => {
                    self.report(
                        &entry_at,
                        DiagnosticCode::DocumentRejected {
                            text: formatted(
                                "Recipe for: {} is missing (unknown) ingredient",
                                [self.id.clone()],
                            ),
                        },
                        Severity::Error,
                    );
                    return None;
                }
            }
        }
        Some(key)
    }

    /// The game's two messages for a pattern it cannot add: one of spaces only, and one none of
    /// whose symbols has a key. The conditions are the plain readings of the messages: a
    /// pattern with no symbol but spaces, and a pattern none of whose symbols is the first byte
    /// of a `key` member name.
    fn symbols(
        &mut self,
        pattern: &[String],
        key: &indexmap::IndexMap<String, Ingredient>,
        at: &JsonPath,
    ) {
        let symbols: HashSet<u8> = key.keys().filter_map(|name| name.bytes().next()).collect();
        let used: Vec<u8> = pattern
            .iter()
            .flat_map(|row| row.bytes())
            .filter(|b| *b != b' ')
            .collect();
        let at = at.child("pattern");
        if used.is_empty() {
            self.report(
                &at,
                DiagnosticCode::InvalidValue {
                    text: fixed("Recipes::addShapedRecipe: adding an empty recipe!"),
                },
                Severity::Error,
            );
        } else if !used.iter().any(|symbol| symbols.contains(symbol)) {
            self.report(
                &at,
                DiagnosticCode::InvalidValue {
                    text: formatted(
                        "Recipes::addShapedRecipe: '{}' doesn't have a single valid key in its pattern!",
                        [self.id.clone()],
                    ),
                },
                Severity::Error,
            );
        }
    }

    fn shapeless(&mut self, payload: &mut Members<'_>, at: &JsonPath) -> Recipe {
        let priority = self.priority(payload, at);
        let result = self.result(payload, at);
        if let Output::Many(items) = &result
            && items.len() > 1
        {
            // The game logs an error and loads the recipe with the first entry: a kept error.
            self.report_kept(
                &at.child("result"),
                DiagnosticCode::IgnoredMember {
                    text: fixed("unsupported multiple output types for result list in shapeless"),
                },
            );
        }
        let unlock = self.unlock(payload, at);
        self.unlock_gate(unlock.as_ref(), at);
        let ingredients_at = at.child("ingredients");
        let mut ingredients = Vec::new();
        match payload.take("ingredients") {
            None => self.report(
                &ingredients_at,
                DiagnosticCode::MissingRequiredField { text: None },
                Severity::Error,
            ),
            Some(value) => match value.as_array() {
                None => self.report(
                    &ingredients_at,
                    DiagnosticCode::InvalidType { text: None },
                    Severity::Error,
                ),
                Some(entries) => {
                    for (index, entry) in entries.iter().enumerate() {
                        let entry_at = ingredients_at.index(index);
                        match self.ingredient(Some(entry), &entry_at, Flags::new(true, true)) {
                            Some(ingredient) => ingredients.push(ingredient),
                            None => {
                                self.result_malformed(&entry_at);
                                break;
                            }
                        }
                    }
                }
            },
        }
        Recipe::Shapeless(Shapeless {
            priority,
            ingredients,
            result,
            unlock,
            ..Shapeless::default()
        })
    }

    fn furnace(&mut self, payload: &mut Members<'_>, at: &JsonPath) -> Recipe {
        let priority = self.priority(payload, at);
        // A furnace input takes no `count` but may be a tag; the output takes `count` but no
        // tag.
        let input = self.part(
            payload,
            at,
            "input",
            Flags::new(false, true),
            PartText::Malformed,
        );
        let output = self.part(
            payload,
            at,
            "output",
            Flags::new(true, false),
            PartText::Malformed,
        );
        let unlock = self.unlock(payload, at);
        Recipe::Furnace(Furnace {
            priority,
            input,
            output,
            unlock,
            ..Furnace::default()
        })
    }

    fn brewing_mix(&mut self, payload: &mut Members<'_>, at: &JsonPath) -> Recipe {
        // The reagent is read first and takes neither `count` nor a tag; the input takes `count`
        // only in legacy packs and may be a tag; the output takes `count` but no tag. The
        // game has no recipe-level messages for a brewing mix.
        let reagent = self.part(
            payload,
            at,
            "reagent",
            Flags::new(false, false),
            PartText::None,
        );
        let input = self.part(
            payload,
            at,
            "input",
            Flags::new(self.input_count(), true),
            PartText::None,
        );
        let output = self.part(
            payload,
            at,
            "output",
            Flags::new(true, false),
            PartText::None,
        );
        Recipe::BrewingMix(BrewingMix {
            input,
            reagent,
            output,
            ..BrewingMix::default()
        })
    }

    fn brewing_container(&mut self, payload: &mut Members<'_>, at: &JsonPath) -> Recipe {
        let texts = |part| {
            PartText::Site(match part {
                0 => "Brewing Container Recipe: {} has an invalid input item",
                1 => "Brewing Container Recipe: {} has an invalid reagent item",
                _ => "Brewing Container Recipe: {} has an invalid output item",
            })
        };
        let input = self.part(
            payload,
            at,
            "input",
            Flags::new(self.input_count(), true),
            texts(0),
        );
        let reagent = self.part(payload, at, "reagent", Flags::new(false, false), texts(1));
        let output = self.part(payload, at, "output", Flags::new(true, false), texts(2));
        Recipe::BrewingContainer(BrewingContainer {
            input,
            reagent,
            output,
            ..BrewingContainer::default()
        })
    }

    fn material_reduction(&mut self, payload: &mut Members<'_>, at: &JsonPath) -> Recipe {
        let input = self.part(
            payload,
            at,
            "input",
            Flags::new(self.input_count(), true),
            PartText::Site("Material Reduction Recipe: {} has an invalid input item"),
        );
        let output_at = at.child("output");
        let output = match payload.take("output") {
            None => None,
            Some(value) if value.is_object() => self
                .reduction_output(Some(value), &output_at)
                .map(Output::One),
            Some(value) => match value.as_array() {
                Some(entries) => {
                    if entries.len() > MAX_REDUCER_OUTPUTS {
                        self.report(
                            &output_at,
                            DiagnosticCode::InvalidValue {
                                text: formatted(
                                    "Material Reduction Recipe: {} has too many outputs, the max is {}.",
                                    [self.id.clone(), MAX_REDUCER_OUTPUTS.to_string()],
                                ),
                            },
                            Severity::Error,
                        );
                    }
                    let mut items = Vec::new();
                    for (index, entry) in entries.iter().enumerate() {
                        match self.reduction_output(Some(entry), &output_at.index(index)) {
                            Some(item) => items.push(item),
                            None => break,
                        }
                    }
                    Some(Output::Many(items))
                }
                None => {
                    // Any other type gives no outputs; the value is kept as written.
                    payload.keep("output");
                    if !value.is_null() {
                        self.report(
                            &output_at,
                            DiagnosticCode::InvalidType { text: None },
                            Severity::Info,
                        );
                    }
                    None
                }
            },
        };
        Recipe::MaterialReduction(MaterialReduction {
            input,
            output,
            ..MaterialReduction::default()
        })
    }

    /// One material-reduction output: count ✓ / tag ✗.
    fn reduction_output(&mut self, value: Option<&Value>, at: &JsonPath) -> Option<Ingredient> {
        let item = self.ingredient(value, at, Flags::new(true, false));
        if item.is_none() {
            self.report(
                at,
                DiagnosticCode::DocumentRejected {
                    text: formatted(
                        "Material Reduction Recipe: {} has an invalid output item",
                        [self.id.clone()],
                    ),
                },
                Severity::Error,
            );
        }
        item
    }

    /// The smithing tag rule: exactly one tag, `smithing_table`.
    fn smithing_tag(&mut self, tags: Option<&Tags>, at: &JsonPath, template: &'static str) {
        let list = tags.map_or(&[][..], Tags::as_slice);
        if list.len() != 1 || list[0] != SMITHING_TABLE {
            self.report(
                &at.child("tags"),
                DiagnosticCode::DocumentRejected {
                    text: formatted(template, [self.id.clone(), SMITHING_TABLE.to_owned()]),
                },
                Severity::Error,
            );
        }
    }

    /// Whether the world is compatible with Trails & Tales, the 1.20 content update that added
    /// smithing trims.
    fn trails_and_tales(&self) -> bool {
        self.cx.world_base_game_version.includes(&TRAILS_AND_TALES)
    }

    fn smithing_transform(
        &mut self,
        payload: &mut Members<'_>,
        at: &JsonPath,
        tags: Option<&Tags>,
    ) -> Recipe {
        self.smithing_tag(
            tags,
            at,
            "Smithing Transform Recipe: {} has an invalid tag. They only supported one is {}.",
        );
        let flags = Flags::new(false, false);
        // `template` is read only in a Trails & Tales world; absent, it is a
        // null template that skips the check.
        let template = match payload.take("template") {
            Some(value) if !self.trails_and_tales() => {
                let _ = value;
                payload.keep("template");
                self.report(
                    &at.child("template"),
                    DiagnosticCode::IgnoredMember { text: None },
                    Severity::Info,
                );
                None
            }
            Some(value) if !value.is_null() => {
                let template = self.ingredient(Some(value), &at.child("template"), flags);
                if template.is_none() {
                    self.smithing_part(
                        "Smithing Transform Recipe: {} has an invalid template item.",
                        at,
                        "template",
                    );
                }
                template
            }
            Some(_) => {
                payload.keep("template");
                None
            }
            None => None,
        };
        let base = self.part(
            payload,
            at,
            "base",
            flags,
            PartText::Site("Smithing Transform Recipe: {} has an invalid base item."),
        );
        let addition = self.part(
            payload,
            at,
            "addition",
            flags,
            PartText::Site("Smithing Transform Recipe: {} has an invalid addition item."),
        );
        let result = self.part(
            payload,
            at,
            "result",
            flags,
            PartText::Site("Smithing Transform Recipe: {} has an invalid result item."),
        );
        Recipe::SmithingTransform(SmithingTransform {
            template,
            base,
            addition,
            result,
            ..SmithingTransform::default()
        })
    }

    fn smithing_trim(
        &mut self,
        payload: &mut Members<'_>,
        at: &JsonPath,
        tags: Option<&Tags>,
    ) -> Recipe {
        self.smithing_tag(
            tags,
            at,
            "Smithing Trim Recipe: {} has an invalid tag. They only supported one is {}.",
        );
        if !self.trails_and_tales() {
            self.report(
                at,
                DiagnosticCode::DocumentRejected {
                    text: formatted(
                        "Smithing Trim Recipe: {} is invalid. Smithing Trim Recipes are only supported from version 1.20 onwards.",
                        [self.id.clone()],
                    ),
                },
                Severity::Error,
            );
        }
        // Template, base and addition take no `count` but may be tags.
        let flags = Flags::new(false, true);
        let template = self.part(payload, at, "template", flags, PartText::None);
        let base = self.part(payload, at, "base", flags, PartText::None);
        let addition = self.part(payload, at, "addition", flags, PartText::None);
        Recipe::SmithingTrim(SmithingTrim {
            template,
            base,
            addition,
            ..SmithingTrim::default()
        })
    }

    fn smithing_part(&mut self, template: &'static str, at: &JsonPath, key: &str) {
        self.report(
            &at.child(key),
            DiagnosticCode::DocumentRejected {
                text: formatted(template, [self.id.clone()]),
            },
            Severity::Error,
        );
    }

    /// A required single-ingredient member; an invalid one raises the site's text after the
    /// ingredient's own, and stands as an empty name in the (rejected) value.
    fn part(
        &mut self,
        payload: &mut Members<'_>,
        at: &JsonPath,
        key: &str,
        flags: Flags,
        text: PartText,
    ) -> Ingredient {
        let part_at = at.child(key);
        if let Some(ingredient) = self.ingredient(payload.take(key), &part_at, flags) {
            return ingredient;
        }
        match text {
            PartText::None => {}
            PartText::Malformed => self.result_malformed(&part_at),
            PartText::Site(template) => self.smithing_part(template, at, key),
        }
        Ingredient::default()
    }
}

/// What a call site logs after an invalid ingredient.
#[derive(Clone, Copy)]
enum PartText {
    /// Nothing more (brewing mix, smithing trim).
    None,
    /// `Recipe result malformed`.
    Malformed,
    /// The site's text, naming the recipe.
    Site(&'static str),
}

/// The spec of ingredient `data`: a value or a list, the pack's version, the
/// `default` query set, every operation, an empty text compiled, a failure kept and logged:
/// the parser's lines are logged by the game as errors and the recipe loads with the node
/// evaluating 0, so they are kept errors.
#[cfg(feature = "molang")]
const DATA_SPEC: bedrock_addon_core::molang::MolangFieldSpec =
    bedrock_addon_core::molang::MolangFieldSpec {
        reader: bedrock_addon_core::molang::ReaderKind::ScalarOrArray,
        version: bedrock_addon_core::molang::MolangVersionSource::Pack,
        admission: bedrock_addon_core::molang::Admission::Default,
        ops: bedrock_addon_core::molang::OpsRule::All,
        empty: bedrock_addon_core::molang::EmptyRule::Compile,
        on_failure: bedrock_addon_core::molang::MolangFailure::KeepLogged,
        error_severity: Severity::Error,
        error_kept: true,
    };

/// jsoncpp's `empty()`: `null`, an empty array or an empty object.
fn is_empty(value: &Value) -> bool {
    value.is_null()
        || value.as_array().is_some_and(facet_value::VArray::is_empty)
        || value
            .as_object()
            .is_some_and(facet_value::VObject::is_empty)
}

/// An integer-tagged number that fits an `i32` (jsoncpp's `isInt`).
fn integer(value: &Value) -> Option<i32> {
    let number = value.as_number().filter(|number| number.is_integer())?;
    i32::try_from(number.to_i64()?).ok()
}
