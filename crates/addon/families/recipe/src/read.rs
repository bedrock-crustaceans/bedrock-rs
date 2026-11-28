//! Reading a recipe file the way vanilla's hand-written walker does. The file is the typed
//! structs of `types.rs`: a root of `format_version` and one payload, chosen by its key
//! ([`payload_kind`]), each payload's members declared in the order vanilla reads them. Their
//! derived reads run on the reader kit ([`Read`]) in that order; the functions here are the
//! members the walker reads by hand: an ingredient (its members read in vanilla's order, the
//! first failure ending it, what it may hold set by where it stands), `data` as Molang evaluated
//! at load, and the members vanilla reads when absent or tells of in its own words. Unknown
//! members go to the document's table at their object's path with an information.

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use facet::Facet;

use super::codes;
use super::ingredient::{constant_data, potion_in, split_item};
use super::types::MISSING_VERSION;
use super::{
    Ingredient, IngredientObject, Output, Recipe, RecipeKind, Shaped, Tags, Unlock, UnlockContext,
    input_count_allowed, unlock_required,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, CodeDef, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, JsonPathSegment, Severity,
};
use bedrock_addon_core::document::VersionInfo;
use bedrock_addon_core::id::RecipeId;
use bedrock_addon_core::json::Prepared;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{self, Node, Profile, Read, UnknownAt};
use bedrock_addon_core::json::parse::Options;
use bedrock_addon_core::json::{Coerce, Coercing, IsIntRule, WideInt};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{
    JsonType, MolangList, MolangSource, ReadField, ReaderKind, ReprError, beyond_f32_warning,
    json_scalar, read_finite,
};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::schema::derive_support::{Fields, ignored_member, present_member};
use bedrock_addon_core::schema::{self, Schema, UnknownRule, read_document};
use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion, SemVersion};
use bedrock_addon_core::versioning::{MemberFailure, Reading};

/// jsoncpp's `isInt` as the recipe reader asks it: an integer token that fits an `i32`; a real
/// token never is one.
const IS_INT: Coercing = Coercing::new(IsIntRule::Token, WideInt::LowWord);

/// How the walker reads a member vanilla ignores when it is of the wrong type: the member keeps
/// its default and the crate says so in an information (the game says nothing).
const LENIENT: Reading = Reading::OWN.with_on_member_failure(MemberFailure::DefaultAndTell);

/// How the walker reads the members the crate refuses the recipe over (`pattern`, `key`,
/// `ingredients`): a missing one or one of the wrong type is an error.
const REFUSING: Reading = Reading::OWN;

/// What the file's read puts in scope for the members that tell it: the file's path, the
/// recipe's identifier, and the count of `data` expressions not evaluated at load.
const FILE: &str = "recipe file";
const IDENTIFIER: &str = "recipe identifier";
const NOT_CHECKED: &str = "molang not checked";

/// What the loader learnt of a file before deciding on it: its type and identifier.
#[derive(Clone, Debug, Default)]
pub(super) struct Parsed {
    /// The payload's type, when the file has a payload key.
    pub(super) kind: Option<RecipeKind>,
    /// The identifier, when the payload has a non-empty one.
    pub(super) identifier: Option<RecipeId>,
}

/// A recipe file: `format_version` and the payload, a member of the root keyed by its type.
#[derive(Facet, Schema)]
struct RecipeFile {
    /// Read whether written or not: without a readable one the game logs `Missing version tag`.
    #[facet(schema::reads_absent, schema::reads_null, schema::or_kept = "told")]
    #[facet(schema::read_with = format_version)]
    format_version: Option<FormatVersion>,
    #[facet(flatten)]
    recipe: Recipe,
}

/// `unlock` as an object whose `context` names a context.
#[derive(Facet, Schema)]
struct UnlockObject {
    #[allow(
        dead_code,
        reason = "read for its unknown members: the context is checked first"
    )]
    context: String,
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
    let mut diagnostics = Diagnostics::default();
    let (parsed, value, written) = read_file(bytes, path, cx, &mut diagnostics);
    let effective = written.unwrap_or(MISSING_VERSION);
    let version = VersionInfo::new(written, false, effective, effective, false);
    // A recipe has no upgrade to finish its load: the places of the elements the reader
    // kept are settled here, before anything else is told of the file.
    diagnostics.name_as_the_file_has_it();
    (parsed, value, version, diagnostics)
}

/// The file's text read as a [`RecipeFile`], with the text's own refusals in vanilla's words.
fn read_file(
    bytes: &[u8],
    path: &PackPath,
    cx: &LoadContext,
    diagnostics: &mut Diagnostics,
) -> (Parsed, Option<Recipe>, Option<FormatVersion>) {
    let mut parsed = Parsed::default();
    let root_path = JsonPath::new();
    let Ok(prepared) = Prepared::new(bytes) else {
        diagnostics.push(
            Diagnostic::error(core_codes::SYNTAX.with(["not UTF-8 JSON within the nesting limit"]))
                .at(root_path),
        );
        return (parsed, None, None);
    };
    // Of a member written twice the last stands.
    let cleaned = bedrock_addon_core::json::last_wins(prepared.as_str());
    // The game's jsoncpp reader accepts comments.
    let document = match bedrock_addon_core::json::parse::parse(&cleaned, Options::default()) {
        Ok(document) => document,
        Err(error) => {
            let message = error.kind.to_string();
            // The game's text is `JSON: %s has an error: \n %s`.
            diagnostics.push(
                Diagnostic::of(codes::JSON_ERROR.with([path.as_str().to_owned(), message.clone()]))
                    .detail(DiagnosticDetail::Syntax(message))
                    .at(root_path),
            );
            return (parsed, None, None);
        }
    };
    let root = Node::parsed(&document, root_path.clone());
    let Some(object) = root.value().as_object() else {
        // The game's text is `JSON: %s root value is not a object`.
        diagnostics.push(
            Diagnostic::of(codes::ROOT_NOT_OBJECT.with([path.as_str()]))
                .detail(DiagnosticDetail::Syntax(
                    "the root is not an object".to_owned(),
                ))
                .at(root_path),
        );
        return (parsed, None, None);
    };
    let payload = present_member(&root, <Recipe as Fields>::VARIANTS);
    parsed.kind = payload.and_then(RecipeKind::from_payload_key);
    let identifier = payload.and_then(|key| identifier_of(object.get(key)?));
    parsed.identifier = identifier.map(|id| RecipeId::parse(id.to_owned()));
    let written = object.get("format_version").and_then(version_of);
    let not_checked = Arc::new(AtomicU32::new(0));
    let mut read = Read::new(LENIENT, diagnostics)
        .profile(Profile::DECLARED_ORDER.with_unknown(UnknownAt::Last))
        .unknown_severity(Severity::Info)
        .keeps_unknown(true)
        .load(cx)
        .format(Some(written.unwrap_or(MISSING_VERSION)));
    let (file, identifier) = (path.as_str().to_owned(), identifier.unwrap_or_default());
    let read_file = read.scoped(FILE, file, |read| {
        read.scoped(IDENTIFIER, identifier.to_owned(), |read| {
            read.scoped(NOT_CHECKED, Arc::clone(&not_checked), |read| {
                read_document::<RecipeFile>(&root, read)
            })
        })
    });
    drop(read);
    in_file_order(diagnostics, root.value());
    let Some(RecipeFile {
        format_version,
        mut recipe,
    }) = read_file
    else {
        return (parsed, None, written);
    };
    super::types::each!(&mut recipe, r => r.format_version = format_version);
    let not_checked = not_checked.load(Ordering::Relaxed);
    if not_checked > 0 {
        diagnostics.push(
            Diagnostic::error(core_codes::MOLANG_NOT_CHECKED.with([not_checked.to_string()]))
                .severity(Severity::Info)
                .at(root_path),
        );
    }
    (parsed, Some(recipe), format_version)
}

/// Puts the members each object keeps in the order its file has them, as vanilla's walker kept
/// them when it had read the object: a member kept for the value it could not read and an
/// unknown one are kept as they come, not the first before the second.
fn in_file_order(diagnostics: &mut Diagnostics, root: &Value) {
    let unknown = diagnostics.unknown_mut();
    let owners: Vec<JsonPath> = unknown.owners().cloned().collect();
    for owner in owners {
        let mut at = Some(root);
        for segment in owner.segments() {
            at = at.and_then(|value| match segment {
                JsonPathSegment::Key(key) => value.as_object()?.get(key),
                JsonPathSegment::Index(index) => value.as_array()?.get(index),
            });
        }
        let Some(object) = at.and_then(Value::as_object) else {
            continue;
        };
        let place = |key: &str| object.keys().position(|written| written.as_str() == key);
        let mut members = unknown.take_all(&owner);
        if members.iter().all(|(key, _)| place(key).is_some()) {
            members.sort_by_key(|(key, _)| place(key));
        }
        for (key, value) in members {
            unknown.put(&owner, &key, value);
        }
    }
}

/// A `format_version`: a string `a.b[.c]` or three unsigned integers, but not `0.0.0`.
fn version_of(value: &Value) -> Option<FormatVersion> {
    SemVersion::from_json(value)
        .filter(|version| !version.is_any())
        .map(|version| version.format_version())
}

/// A payload's identifier: `description.identifier` when a non-empty string (`asString("")`:
/// any other value is empty).
fn identifier_of(payload: &Value) -> Option<&str> {
    let identifier = payload
        .as_object()?
        .get("description")?
        .as_object()?
        .get("identifier")?
        .as_string()?
        .as_str();
    (!identifier.is_empty()).then_some(identifier)
}

// ---- what the leaves read with --------------------------------------------------------------

/// The identifier, for the texts that name it.
fn id(read: &Read<'_>) -> String {
    read.scope::<String>(IDENTIFIER)
        .cloned()
        .unwrap_or_default()
}

/// The effective `format_version`.
fn version(read: &Read<'_>) -> FormatVersion {
    read.known_format().unwrap_or(MISSING_VERSION)
}

fn report(read: &mut Read<'_>, at: &JsonPath, code: Code, severity: Severity) {
    read.push(Diagnostic::error(code).severity(severity).at(at.clone()));
}

/// A kept error: a line the game logs at error level and then goes on, so the recipe still
/// loads. It is an error finding that does not reject the document.
fn report_kept(read: &mut Read<'_>, at: &JsonPath, code: Code) {
    read.push(Diagnostic::error(code).at(at.clone()).into_kept());
}

/// An ingredient's own finding: kept or rejecting as `flags` says.
fn report_ingredient(read: &mut Read<'_>, at: &JsonPath, code: Code, flags: Flags) {
    if flags.kept {
        report_kept(read, at, code);
    } else {
        report(read, at, code, Severity::Error);
    }
}

/// The member at `node` kept as written in its object's table, its finding already told.
fn keep(node: &Node<'_>, read: &mut Read<'_>) {
    let path = node.path();
    let owner = path.prefix(path.len().saturating_sub(1));
    let key = path.last_key().unwrap_or_default().to_owned();
    read.findings_mut()
        .unknown_mut()
        .put(&owner, &key, node.value().clone());
}

/// `f` under the reading of the members the crate refuses the recipe over.
fn refusing<T>(read: &mut Read<'_>, f: impl FnOnce(&mut Read<'_>) -> Option<T>) -> Option<T> {
    f(&mut Read::new(REFUSING, read.findings_mut()))
}

/// `JSON: %s has no identifier`.
fn no_identifier(at: &JsonPath, read: &mut Read<'_>) {
    let file = read.scope::<String>(FILE).cloned().unwrap_or_default();
    report(read, at, codes::NO_IDENTIFIER.with([file]), Severity::Error);
}

/// The payload of the root `node`: the first present payload key, `null` counting as absent;
/// every other one is ignored. A file whose payload has no identifier is skipped: the game reads
/// the identifier from the payload first, so a file with no payload, or whose payload is no
/// object, is told as one without an identifier.
pub(crate) fn payload_kind(node: &Node<'_>, read: &mut Read<'_>) -> Option<&'static str> {
    let names = <Recipe as Fields>::VARIANTS;
    let chosen = present_member(node, names);
    for name in names.iter().filter(|name| Some(**name) != chosen) {
        ignored_member(node, name, read);
    }
    let Some(chosen) = chosen else {
        no_identifier(node.path(), read);
        return None;
    };
    let payload = node.value().as_object().and_then(|root| root.get(chosen));
    if payload.and_then(identifier_of).is_none() {
        no_identifier(&node.path().child(chosen), read);
        return None;
    }
    Some(chosen)
}

/// `format_version`: anything but a version, or none, is 1.0.0 with `Missing version tag` (a
/// kept error: the game logs an error and the recipe still loads); a member written is then kept
/// as written. Above the engine version: an info, there is no ceiling.
fn format_version(node: &Node<'_>, read: &mut Read<'_>) -> Option<FormatVersion> {
    let Some(version) = version_of(node.value()) else {
        report_kept(read, node.path(), Code::from(&codes::MISSING_VERSION_TAG));
        return None;
    };
    let engine = read
        .load_context()
        .map_or(ENGINE_VERSION, |cx| cx.engine_version);
    if version > engine {
        report(
            read,
            node.path(),
            core_codes::FORMAT_VERSION_ABOVE_ENGINE
                .with([version.to_string(), ENGINE_VERSION.to_string()]),
            Severity::Info,
        );
    }
    Some(version)
}

// ---- tags --------------------------------------------------------------------------------------

/// What a recipe type's walker tells of its tags once it has read them.
pub(crate) trait TagRule {
    /// Tells what the type's walker tells of `tags` (at `at`, the payload's `tags`).
    fn after(tags: &[String], at: &JsonPath, read: &mut Read<'_>);
}

/// Every type but the smithing ones: a `smithing_table` tag is a misuse.
pub(crate) struct Crafting;

/// A smithing transform: exactly one tag, `smithing_table`.
pub(crate) struct Transform;

/// A smithing trim: exactly one tag, `smithing_table`, and a Trails & Tales world.
pub(crate) struct Trim;

impl TagRule for Crafting {
    fn after(tags: &[String], at: &JsonPath, read: &mut Read<'_>) {
        if tags.iter().any(|tag| tag == SMITHING_TABLE) {
            // The game logs the misuse as an error and goes on loading: a kept error.
            report_kept(read, at, Code::from(&codes::SMITHING_TABLE_TAG_MISUSED));
        }
    }
}

impl TagRule for Transform {
    fn after(tags: &[String], at: &JsonPath, read: &mut Read<'_>) {
        smithing_tag(tags, at, &codes::TRANSFORM_INVALID_TAG, read);
    }
}

impl TagRule for Trim {
    fn after(tags: &[String], at: &JsonPath, read: &mut Read<'_>) {
        smithing_tag(tags, at, &codes::TRIM_INVALID_TAG, read);
        if !trails_and_tales(read) {
            let payload = at.prefix(at.len().saturating_sub(1));
            report(
                read,
                &payload,
                codes::TRIM_BEFORE_TRAILS_AND_TALES.with([id(read)]),
                Severity::Error,
            );
        }
    }
}

/// The smithing tag rule: exactly one tag, `smithing_table`.
fn smithing_tag(tags: &[String], at: &JsonPath, def: &'static CodeDef<2>, read: &mut Read<'_>) {
    if tags.len() != 1 || tags[0] != SMITHING_TABLE {
        report(
            read,
            at,
            def.with([id(read), SMITHING_TABLE.to_owned()]),
            Severity::Error,
        );
    }
}

/// `tags`: a string or a list of strings (other entries skipped); any other value counts as
/// none and is kept as written.
pub(crate) fn tags<K: TagRule>(node: &Node<'_>, read: &mut Read<'_>) -> Option<Tags> {
    let tags = if node.is_absent() {
        None
    } else if let Some(tag) = node.value().as_string() {
        Some(Tags::One(tag.as_str().to_owned()))
    } else if node.value().is_array() {
        let entries = read.array(node)?;
        let mut tags = Vec::new();
        // The file's index of each tag held, for the link pass.
        let mut kept = Vec::new();
        for (index, entry) in entries.iter().enumerate() {
            if let Some(tag) = read.string(&entry) {
                tags.push(tag.to_owned());
                kept.push(index);
            }
        }
        read.findings_mut().left_out(node.path(), &kept);
        Some(Tags::Many(tags))
    } else {
        read.wrong_type(node);
        None
    };
    let list = tags.as_ref().map_or(&[][..], Tags::as_slice);
    if list.is_empty() {
        // `No tags specified`: the game logs it as an error and goes on; one recipe object
        // is registered per tag, so nothing is registered. A kept error.
        report_kept(read, node.path(), Code::from(&codes::NO_TAGS));
    }
    K::after(list, node.path(), read);
    tags
}

// ---- ingredients -------------------------------------------------------------------------------

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
/// the ones kept as written with a finding already raised, and the rest, unknown.
struct Members<'v> {
    object: kit::Members<'v>,
    read: HashSet<String>,
    quiet: HashSet<String>,
}

impl<'v> Members<'v> {
    /// The members of the object `node`.
    fn of(node: &Node<'v>, read: &mut Read<'_>) -> Option<Self> {
        Some(Self {
            object: read.object(node)?,
            read: HashSet::new(),
            quiet: HashSet::new(),
        })
    }

    /// Member `key`, now read.
    fn take(&mut self, key: &str) -> Option<Node<'v>> {
        let node = self.object.get(key)?;
        self.read.insert(key.to_owned());
        Some(node)
    }

    /// Member `key` is kept in `extra` after all, its finding already raised.
    fn keep(&mut self, key: &str) {
        self.read.remove(key);
        self.quiet.insert(key.to_owned());
    }

    /// The members nobody read, kept in the document's table at the object's path; the unknown
    /// ones raise `UnrecognizedField` at info (vanilla reads no other key and says nothing).
    fn rest(&self, read: &mut Read<'_>) {
        let at = self.object.node().path().clone();
        for (key, node) in self.object.iter() {
            if self.read.contains(key) {
                continue;
            }
            if !self.quiet.contains(key) {
                read.unknown_member(&node, UnknownRule::Keep);
            }
            read.findings_mut()
                .unknown_mut()
                .put(&at, key, node.value().clone());
        }
    }
}

/// One ingredient at `node` (`null` where its member is absent); `None` when it is invalid,
/// with the ingredient's own finding kept or rejecting as `flags.kept` says.
fn ingredient(node: &Node<'_>, read: &mut Read<'_>, flags: Flags) -> Option<Ingredient> {
    let at = node.path();
    let value = node.value();
    if value.is_null() {
        // A null value goes to the item lookup with an empty name.
        empty_item(read, at, flags, true);
        return None;
    }
    if let Some(text) = value.as_string() {
        let text = text.as_str();
        item_text(read, text, at);
        return Some(Ingredient::Name(text.to_owned()));
    }
    if !value.is_object() {
        empty_item(read, at, flags, false);
        return None;
    }
    let mut members = Members::of(node, read)?;
    let mut ingredient = IngredientObject::default();
    // Vanilla reads the members in the order `data`, `count`, `item`, `tag`, and the first
    // failure ends the ingredient.
    data(read, &mut members, &mut ingredient);
    count(read, &mut members, &mut ingredient, at, flags)?;
    item_or_tag(read, &mut members, &mut ingredient, at, flags)?;
    members.rest(read);
    Some(Ingredient::Object(ingredient))
}

/// `data`, read only when not `empty()`: a number or Molang, evaluated at load.
fn data(read: &mut Read<'_>, members: &mut Members<'_>, ingredient: &mut IngredientObject) {
    let Some(data) = members.take("data") else {
        return;
    };
    if is_empty(data.value()) {
        members.keep("data");
        return;
    }
    if let Ok(list) = data_list(read, data.value(), &mut data.path().clone()) {
        ingredient.data_value = evaluate(read, &list, &data);
        ingredient.data = Some(list);
        return;
    }
    members.keep("data");
    // An object `data` makes the game log "Type not convertible to Molang expression: %d"
    // as an error and go on with an empty expression, which evaluates to 0: a kept error.
    // A list with an element that is no value has no line in the game, so the crate's own
    // note is an information.
    if data.value().is_object() {
        read.push(
            Diagnostic::of(codes::DATA_NOT_MOLANG.with([OBJECT_VALUE_TYPE]))
                .at(data.path().clone())
                .into_kept(),
        );
    } else {
        read.wrong_type(&data);
    }
    ingredient.data_value = Some(0);
}

/// `data` as a value or a list of them, element by element (`molang::read_list`'s shape): a
/// number beyond `f32` is read as the largest finite float of its sign, with core's warning
/// at the number. `at` is the path of `value`, and is left as it came.
fn data_list(
    read: &mut Read<'_>,
    value: &Value,
    at: &mut JsonPath,
) -> Result<MolangList, ReprError> {
    let reader = ReaderKind::ScalarOrArray;
    if let Some(elements) = value.as_array() {
        let mut list = Vec::with_capacity(elements.len());
        for (index, element) in elements.iter().enumerate() {
            at.push_index(index);
            let element = data_list(read, element, at);
            at.pop();
            list.push(element?);
        }
        return Ok(MolangList::Many(list));
    }
    let molang_version = read.load_context().map_or_else(
        || bedrock_addon_core::molang::pack_molang_version(None),
        |cx| cx.molang_version,
    );
    match read_finite(json_scalar(value), reader, molang_version)? {
        (ReadField::Value(value), clamped) => {
            if clamped {
                read.push(beyond_f32_warning(at.clone(), None));
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
    read: &mut Read<'_>,
    members: &mut Members<'_>,
    ingredient: &mut IngredientObject,
    at: &JsonPath,
    flags: Flags,
) -> Option<()> {
    let Some(count) = members.take("count") else {
        return Some(());
    };
    if is_empty(count.value()) {
        members.keep("count");
        return Some(());
    }
    if !flags.count {
        report_ingredient(
            read,
            &at.child("count"),
            Code::from(&codes::COUNT_NOT_ALLOWED),
            flags,
        );
        return None;
    }
    match read.int(&count, IS_INT) {
        Some(count) => ingredient.count = Some(count),
        None => members.keep("count"),
    }
    Some(())
}

/// `item`, or where tags are allowed and `item` is not a string, `tag`.
fn item_or_tag(
    read: &mut Read<'_>,
    members: &mut Members<'_>,
    ingredient: &mut IngredientObject,
    at: &JsonPath,
    flags: Flags,
) -> Option<()> {
    let item = members.take("item");
    if let Some(text) = item.as_ref().and_then(|item| item.value().as_string()) {
        item_text(read, text.as_str(), &at.child("item"));
        ingredient.item = Some(text.as_str().to_owned());
        // `tag` beside a string `item` is not read; kept as written.
        if let Some(tag) = members.take("tag") {
            match tag.value().as_string() {
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
        report_ingredient(read, at, Code::from(&codes::NO_ITEM_NAME), flags);
        return None;
    }
    if let Some(tag) = members.take("tag")
        && let Some(tag) = tag.value().as_string()
    {
        ingredient.tag = Some(tag.as_str().to_owned());
        return Some(());
    }
    // Neither an item nor a tag: the game's error line for this case.
    report_ingredient(read, at, Code::from(&codes::NO_ITEM_OR_TAG), flags);
    None
}

/// An ingredient that is absent, or neither a string nor an object: the item lookup with an
/// empty name fails ("The Item: %s is missing or invalid, can't make the recipe").
fn empty_item(read: &mut Read<'_>, at: &JsonPath, flags: Flags, missing: bool) {
    let def = if missing {
        &codes::ITEM_MISSING
    } else {
        &codes::ITEM_INVALID
    };
    report_ingredient(read, at, def.with([""]), flags);
}

/// What vanilla's item-name parser logs about an item text. The game logs each line as an
/// error and goes on with the tokens it has (a missing namespace is `minecraft`), so each
/// is a kept error. A potion text (the context's provider names the potion) logs nothing;
/// one the provider does not know is split as an item, with that split's lines.
fn item_text(read: &mut Read<'_>, text: &str, at: &JsonPath) {
    let potion = read
        .load_context()
        .and_then(|cx| potion_in(text, cx.built_ins()));
    if potion.is_some() {
        return;
    }
    for problem in split_item(text).1 {
        read.push(
            Diagnostic::of(problem.code().with([text]))
                .at(at.clone())
                .into_kept(),
        );
    }
}

/// The value `data` evaluates to at load: a constant directly; an expression compiled with
/// the pack's version and resolved when it folds (a rejected one is 0); a list is no number.
/// `node` is the member as written.
fn evaluate(read: &mut Read<'_>, data: &MolangList, node: &Node<'_>) -> Option<i32> {
    match data {
        MolangList::One(bedrock_addon_core::molang::Molang::Expr(source)) => {
            expression(read, source, node.path())
        }
        MolangList::One(_) => constant_data(data),
        MolangList::Many(_) => {
            #[cfg(feature = "molang")]
            if let Some(cx) = read.load_context()
                && let Some(check) = cx.molang.clone()
            {
                let mcx = molang_context(cx, version(read));
                let location = bedrock_addon_core::diagnostic::Location::new(
                    None,
                    None,
                    node.path().clone(),
                    None,
                );
                let mut found = Diagnostics::default();
                check.check_list(
                    data,
                    &DATA_SPEC,
                    bedrock_addon_core::molang::Side::Server,
                    &mcx,
                    &location,
                    &mut found,
                );
                for finding in found {
                    read.push(finding);
                }
            }
            // An array `data` yields no number here, and the game logs no line for it, so
            // the crate's own note is an information.
            read.wrong_type(node);
            None
        }
    }
}

#[cfg(feature = "molang")]
fn molang_context(
    cx: &LoadContext,
    version: FormatVersion,
) -> bedrock_addon_core::molang::MolangContext {
    let document = bedrock_addon_core::version::FormatVersionInput {
        version: Some(version),
        beta_literal: false,
        prerelease: false,
    };
    bedrock_addon_core::molang::MolangContext::new(
        cx.min_engine_version.as_ref(),
        cx.molang_experiments,
        &document,
        Some(cx.molang_version),
    )
}

/// One `data` expression: compiled with the `molang` feature and a compile check; a rejected
/// expression is a kept error (or an information when the game logs no line) and 0, one
/// that folds gives its value, and any other is not evaluated, because running a Molang
/// program is out of scope for the loader.
fn expression(read: &mut Read<'_>, source: &MolangSource, at: &JsonPath) -> Option<i32> {
    #[cfg(feature = "molang")]
    if let Some(cx) = read.load_context()
        && let Some(check) = cx.molang.clone()
    {
        use bedrock_addon_core::molang::{MolangOutcome, ReportRules, Side};
        let mcx = molang_context(cx, version(read));
        let location = bedrock_addon_core::diagnostic::Location::new(None, None, at.clone(), None);
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
                && let Some(expression) = finding
                    .code
                    .args_of(&bedrock_addon_core::diagnostic::codes::MOLANG_LINT)
            {
                finding.code = bedrock_addon_core::diagnostic::codes::INVALID_MOLANG
                    .with([expression[0].clone()]);
            }
            read.push(finding);
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
    if let Some(not_checked) = read.scope::<Arc<AtomicU32>>(NOT_CHECKED) {
        not_checked.fetch_add(1, Ordering::Relaxed);
    }
    None
}

/// An output or result list at `node`: one ingredient or a list of them (`null` entries
/// skipped).
fn output(read: &mut Read<'_>, node: &Node<'_>, flags: Flags) -> Option<Output> {
    if !node.value().is_array() {
        return ingredient(node, read, flags).map(Output::One);
    }
    let entries = read.array(node)?;
    let mut items = Vec::new();
    // The file's index of each entry held, for the link pass.
    let mut kept = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        if entry.value().is_null() {
            report(
                read,
                entry.path(),
                Code::from(&core_codes::IGNORED_MEMBER),
                Severity::Info,
            );
            continue;
        }
        items.push(ingredient(&entry, read, flags)?);
        kept.push(index);
    }
    read.findings_mut().left_out(node.path(), &kept);
    Some(Output::Many(items))
}

/// `Recipe result malformed` (also the text of a bad shapeless ingredient and of a bad
/// furnace input or output).
fn result_malformed(read: &mut Read<'_>, at: &JsonPath) {
    report(
        read,
        at,
        Code::from(&codes::RESULT_MALFORMED),
        Severity::Error,
    );
}

/// `result` of shaped and shapeless recipes: count allowed, tags not.
fn crafting_result(node: &Node<'_>, read: &mut Read<'_>) -> Output {
    match output(read, node, Flags::new(true, false)) {
        None => {
            result_malformed(read, node.path());
            Output::default()
        }
        Some(Output::Many(items)) if items.is_empty() => {
            report(
                read,
                node.path(),
                Code::from(&codes::NO_RESULT_ITEMS),
                Severity::Error,
            );
            Output::Many(items)
        }
        Some(output) => output,
    }
}

/// A shaped recipe's `result`.
#[allow(
    clippy::unnecessary_wraps,
    reason = "a `schema::read_with` function's signature"
)]
pub(crate) fn result(node: &Node<'_>, read: &mut Read<'_>) -> Option<Output> {
    Some(crafting_result(node, read))
}

/// A shapeless recipe's `result`: one item is made.
#[allow(
    clippy::unnecessary_wraps,
    reason = "a `schema::read_with` function's signature"
)]
pub(crate) fn shapeless_result(node: &Node<'_>, read: &mut Read<'_>) -> Option<Output> {
    let result = crafting_result(node, read);
    if let Output::Many(items) = &result
        && items.len() > 1
    {
        // The game logs an error and loads the recipe with the first entry: a kept error.
        report_kept(read, node.path(), Code::from(&codes::MULTIPLE_OUTPUTS));
    }
    Some(result)
}

/// `unlock`: the malformed forms are logged and give the requirement `{None, []}`; only a value
/// that is neither a list nor an object with a string `context` counts as absent, kept as
/// written.
pub(crate) fn unlock(node: &Node<'_>, read: &mut Read<'_>) -> Option<Unlock> {
    let value = node.value();
    if value.is_null() {
        return None;
    }
    let at = node.path();
    if value.is_array() {
        let entries = read.array(node)?;
        if entries.is_empty() {
            // The game logs this as an error and returns the requirement `{None, []}`: a
            // kept error.
            report_kept(read, at, codes::EMPTY_UNLOCK.with([id(read)]));
            return Some(Unlock::Ingredients(Vec::new()));
        }
        let flags = Flags {
            kept: true,
            ..Flags::new(true, true)
        };
        let mut ingredients = Vec::new();
        for entry in entries.iter() {
            let Some(ingredient) = ingredient(&entry, read, flags) else {
                // The game logs an error, zeroes the requirement and loads the recipe: a kept
                // error. The list is kept as written, so what the elements before put in the
                // table goes.
                read.findings_mut().unknown_mut().forget(at);
                report_kept(read, at, codes::INVALID_UNLOCK_INGREDIENT.with([id(read)]));
                return Some(Unlock::Malformed(value.clone()));
            };
            ingredients.push(ingredient);
        }
        return Some(Unlock::Ingredients(ingredients));
    }
    let context = value
        .as_object()
        .and_then(|object| object.get("context")?.as_string());
    let Some(name) = context else {
        // The game logs an error and then returns normally, so the recipe still loads
        // (the `unlock` gate decides whether a missing requirement is fatal): a kept error.
        report_kept(
            read,
            at,
            codes::UNLOCK_WITHOUT_LIST_OR_CONTEXT.with([id(read)]),
        );
        return None;
    };
    let Some(context) = UnlockContext::from_name(name.as_str()) else {
        // The game logs an error and goes on loading: a kept error.
        report_kept(
            read,
            &at.child("context"),
            codes::MALFORMED_UNLOCK_CONTEXT.with([id(read)]),
        );
        return Some(Unlock::Malformed(value.clone()));
    };
    <UnlockObject as schema::FromJson>::from_json(node, read)?;
    Some(Unlock::Context { context })
}

/// The `unlock` of a shaped or shapeless recipe, which from 1.20.10 must have one.
pub(crate) fn gated_unlock(node: &Node<'_>, read: &mut Read<'_>) -> Option<Unlock> {
    let unlock = if node.is_absent() {
        None
    } else {
        unlock(node, read)
    };
    if unlock.is_none() && unlock_required(version(read)) {
        let payload = node.path().prefix(node.path().len().saturating_sub(1));
        report(
            read,
            &payload,
            Code::from(&codes::UNLOCK_REQUIRED),
            Severity::Error,
        );
    }
    unlock
}

/// `priority`: an integer; any other value is not read.
pub(crate) fn priority(node: &Node<'_>, read: &mut Read<'_>) -> Option<i32> {
    if let Some(priority) = node.value().is_int(IS_INT) {
        return Some(priority);
    }
    if !is_empty(node.value()) {
        read.wrong_type(node);
    }
    None
}

/// A bool member vanilla ignores when it is none (`assume_symmetry`, default `true`).
pub(crate) fn boolean(node: &Node<'_>, read: &mut Read<'_>) -> Option<bool> {
    read.bool(node)
}

/// Whether input ingredients may carry `count` in this pack. A pack without a
/// `min_engine_version` is read as having vanilla's zeroed version, which is legacy.
fn input_count(read: &Read<'_>) -> bool {
    read.load_context()
        .and_then(|cx| cx.min_engine_version.as_ref())
        .is_none_or(input_count_allowed)
}

/// Whether the world is compatible with Trails & Tales, the 1.20 content update that added
/// smithing trims.
fn trails_and_tales(read: &Read<'_>) -> bool {
    read.load_context()
        .is_some_and(|cx| cx.world_base_game_version.includes(&TRAILS_AND_TALES))
}

// ---- the members of the recipe types -----------------------------------------------------------

/// `pattern`: 1 to 3 rows of 1 to 3 symbols; vanilla drops a missing, empty or mistyped one
/// silently, so the crate's errors for them carry no game text.
pub(crate) fn pattern(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    let at = node.path();
    let rows = refusing(read, |read| {
        if node.is_absent() {
            read.missing_absent(node);
            return None;
        }
        let rows = read.array(node)?;
        if rows.is_empty() {
            report(
                read,
                at,
                Code::from(&core_codes::INVALID_VALUE),
                Severity::Error,
            );
            return None;
        }
        let mut pattern = Vec::new();
        for row in rows.iter() {
            pattern.push(read.string(&row)?.to_owned());
        }
        Some(pattern)
    })?;
    let width = rows.iter().map(String::len).max().unwrap_or(0);
    let height = rows.len();
    if width > 3 || height > 3 {
        // The game rejects a pattern larger than 3x3 when it normalises the grid.
        report(
            read,
            at,
            codes::PATTERN_TOO_LARGE.with([width.to_string(), height.to_string()]),
            Severity::Error,
        );
    }
    Some(rows)
}

/// `key`: a non-empty object of input ingredients (count only in legacy packs, tags
/// allowed).
pub(crate) fn key(
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<indexmap::IndexMap<String, Ingredient>> {
    let object = refusing(read, |read| {
        if node.is_absent() {
            read.missing_absent(node);
            return None;
        }
        read.object(node)
    })?;
    if object.is_empty() {
        report(
            read,
            node.path(),
            Code::from(&core_codes::INVALID_VALUE),
            Severity::Error,
        );
        return None;
    }
    let flags = Flags::new(input_count(read), true);
    let mut key = indexmap::IndexMap::new();
    for (name, entry) in object.iter() {
        if name.is_empty() {
            report(
                read,
                entry.path(),
                Code::from(&core_codes::INVALID_VALUE),
                Severity::Error,
            );
            return None;
        }
        let Some(ingredient) = ingredient(&entry, read, flags) else {
            report(
                read,
                entry.path(),
                codes::KEY_INGREDIENT_MISSING.with([id(read)]),
                Severity::Error,
            );
            return None;
        };
        key.insert(name.to_owned(), ingredient);
    }
    Some(key)
}

/// The game's two messages for a pattern it cannot add, once `pattern` and `key` are read:
/// one of spaces only, and one none of whose symbols has a key. The conditions are the plain
/// readings of the messages: a pattern with no symbol but spaces, and a pattern none of whose
/// symbols is the first byte of a `key` member name.
pub(crate) fn symbols(shaped: &mut Shaped, node: &Node<'_>, read: &mut Read<'_>) {
    // An empty pattern or key is one the reader refused.
    if shaped.pattern.is_empty() || shaped.key.is_empty() {
        return;
    }
    let symbols: HashSet<u8> = shaped
        .key
        .keys()
        .filter_map(|name| name.bytes().next())
        .collect();
    let used: Vec<u8> = shaped
        .pattern
        .iter()
        .flat_map(|row| row.bytes())
        .filter(|b| *b != b' ')
        .collect();
    let at = node.path().child("pattern");
    if used.is_empty() {
        report(read, &at, Code::from(&codes::EMPTY_RECIPE), Severity::Error);
    } else if !used.iter().any(|symbol| symbols.contains(symbol)) {
        let id = shaped.description.identifier.as_str().to_owned();
        report(read, &at, codes::NO_VALID_KEY.with([id]), Severity::Error);
    }
}

/// A shapeless recipe's `ingredients`: a list of ingredients, the first invalid one ending it.
pub(crate) fn ingredients(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Ingredient>> {
    let entries = refusing(read, |read| {
        if node.is_absent() {
            read.missing_absent(node);
            return None;
        }
        read.array(node)
    })?;
    let mut ingredients = Vec::new();
    for entry in entries.iter() {
        let Some(ingredient) = ingredient(&entry, read, Flags::new(true, true)) else {
            result_malformed(read, entry.path());
            break;
        };
        ingredients.push(ingredient);
    }
    Some(ingredients)
}

/// Whether an input ingredient may carry `count` where it stands.
#[derive(Clone, Copy)]
pub(crate) enum Count {
    /// Never (a reagent, a furnace input, the smithing parts).
    Never,
    /// Always (an output).
    Always,
    /// In a legacy pack only (an input).
    Legacy,
}

/// What a call site logs after an invalid ingredient.
#[derive(Clone, Copy)]
pub(crate) enum PartText {
    /// Nothing more (brewing mix, smithing trim).
    None,
    /// `Recipe result malformed`.
    Malformed,
    /// The site's line, naming the recipe.
    Site(&'static CodeDef<1>),
}

/// Where a single-ingredient member stands: what it may hold, and what an invalid one is told.
pub(crate) trait Site {
    /// Whether it may carry `count`.
    const COUNT: Count;
    /// Whether it may be a tag.
    const TAG: bool;
    /// What follows its own finding when it is invalid.
    const TEXT: PartText;
}

/// The sites of the single-ingredient members, one per recipe type and member.
macro_rules! sites {
    ($($(#[$doc:meta])* $site:ident: $count:ident, $tag:expr, $text:expr;)+) => {$(
        $(#[$doc])*
        pub(crate) struct $site;

        impl Site for $site {
            const COUNT: Count = Count::$count;
            const TAG: bool = $tag;
            const TEXT: PartText = $text;
        }
    )+};
}

sites! {
    /// A furnace `input`: no `count`, a tag allowed.
    FurnaceInput: Never, true, PartText::Malformed;
    /// A furnace `output`: `count`, no tag.
    FurnaceOutput: Always, false, PartText::Malformed;
    /// A brewing mix `reagent`, read first: neither `count` nor a tag. The game has no
    /// recipe-level messages for a brewing mix.
    MixReagent: Never, false, PartText::None;
    /// A brewing mix `input`: `count` in a legacy pack, a tag allowed.
    MixInput: Legacy, true, PartText::None;
    /// A brewing mix `output`: `count`, no tag.
    MixOutput: Always, false, PartText::None;
    /// A brewing container `input`.
    ContainerInput: Legacy, true, PartText::Site(&codes::BREWING_INVALID_INPUT);
    /// A brewing container `reagent`.
    ContainerReagent: Never, false, PartText::Site(&codes::BREWING_INVALID_REAGENT);
    /// A brewing container `output`.
    ContainerOutput: Always, false, PartText::Site(&codes::BREWING_INVALID_OUTPUT);
    /// A material reduction `input`.
    ReductionInput: Legacy, true, PartText::Site(&codes::REDUCTION_INVALID_INPUT);
    /// A smithing transform `base`.
    TransformBase: Never, false, PartText::Site(&codes::TRANSFORM_INVALID_BASE);
    /// A smithing transform `addition`.
    TransformAddition: Never, false, PartText::Site(&codes::TRANSFORM_INVALID_ADDITION);
    /// A smithing transform `result`.
    TransformResult: Never, false, PartText::Site(&codes::TRANSFORM_INVALID_RESULT);
    /// A smithing trim `template`, `base` or `addition`: no `count`, a tag allowed.
    TrimPart: Never, true, PartText::None;
}

/// A required single-ingredient member at site `S`; an invalid one raises the site's text after
/// the ingredient's own, and stands as an empty name in the (rejected) value.
#[allow(
    clippy::unnecessary_wraps,
    reason = "a `schema::read_with` function's signature"
)]
pub(crate) fn part<S: Site>(node: &Node<'_>, read: &mut Read<'_>) -> Option<Ingredient> {
    let count = match S::COUNT {
        Count::Never => false,
        Count::Always => true,
        Count::Legacy => input_count(read),
    };
    if let Some(ingredient) = ingredient(node, read, Flags::new(count, S::TAG)) {
        return Some(ingredient);
    }
    match S::TEXT {
        PartText::None => {}
        PartText::Malformed => result_malformed(read, node.path()),
        PartText::Site(def) => report(read, node.path(), def.with([id(read)]), Severity::Error),
    }
    Some(Ingredient::default())
}

/// A smithing transform's `template`: read only in a Trails & Tales world (else kept as
/// written); `null`, a template that skips the check.
pub(crate) fn template(node: &Node<'_>, read: &mut Read<'_>) -> Option<Ingredient> {
    if !trails_and_tales(read) {
        keep(node, read);
        report(
            read,
            node.path(),
            Code::from(&core_codes::IGNORED_MEMBER),
            Severity::Info,
        );
        return None;
    }
    if node.value().is_null() {
        keep(node, read);
        return None;
    }
    let template = ingredient(node, read, Flags::new(false, false));
    if template.is_none() {
        let code = codes::TRANSFORM_INVALID_TEMPLATE.with([id(read)]);
        report(read, node.path(), code, Severity::Error);
    }
    template
}

/// A material reduction's `output`: one object or a list of at most nine, the first invalid one
/// ending it; any other value gives no outputs and is kept as written.
pub(crate) fn reduction_output(node: &Node<'_>, read: &mut Read<'_>) -> Option<Output> {
    let value = node.value();
    if value.is_object() {
        return reduction_item(node, read).map(Output::One);
    }
    if !value.is_array() {
        keep(node, read);
        if !value.is_null() {
            read.wrong_type(node);
        }
        return None;
    }
    let entries = read.array(node)?;
    if entries.len() > MAX_REDUCER_OUTPUTS {
        report(
            read,
            node.path(),
            codes::REDUCTION_TOO_MANY_OUTPUTS.with([id(read), MAX_REDUCER_OUTPUTS.to_string()]),
            Severity::Error,
        );
    }
    let mut items = Vec::new();
    for entry in entries.iter() {
        match reduction_item(&entry, read) {
            Some(item) => items.push(item),
            None => break,
        }
    }
    Some(Output::Many(items))
}

/// One material-reduction output: count ✓ / tag ✗.
fn reduction_item(node: &Node<'_>, read: &mut Read<'_>) -> Option<Ingredient> {
    let item = ingredient(node, read, Flags::new(true, false));
    if item.is_none() {
        let code = codes::REDUCTION_INVALID_OUTPUT.with([id(read)]);
        report(read, node.path(), code, Severity::Error);
    }
    item
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
