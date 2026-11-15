//! Reading the jigsaw documents: each struct read by the derived reader
//! (`versioning::read_payload`; the types derive `Schema`), then the bind constraints the game's
//! parse checks: the bounds the types state, and the few rules no attribute states (a file
//! reference's length, an object `limit` of the other form, the block specifiers' findings).
//!
//! Every finding here is a parse finding: an error fails the file, as a parse error of the game does, and the
//! loader then adds the family's "Failed to parse …" line.

use super::codes;
use super::history::{MetadataFamily, MetadataHistory, v1_21_50};
use super::{
    BlockEntityModifier, BlockPredicate, CappedDelegate, IntProvider, IntProviderType,
    JigsawMetadata, JigsawStructure, MetadataRegistry, PoolElement, Processor, ProcessorList,
    ProcessorRule, StructureSet, TemplatePool, V1_21_50,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Class, Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{Document, Origin, Rejected, VersionInfo};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::schema::{Schema, SchemaCx};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::FormatVersionInput;
use bedrock_addon_core::versioning::{
    ParseCx, VersionStruct, Versioned, read_payload, write_payload,
};
use bedrock_addon_core::versioning::{Selection, select};

/// The version of the four JSON documents' only struct.
pub(crate) const V1_21_20: FormatVersion = FormatVersion::new(1, 21, 20);
/// The version of the newest metadata registry.
pub(crate) const V1_21_80: FormatVersion = FormatVersion::new(1, 21, 80);

/// The longest `FileReference`.
const MAX_FILE_REFERENCE: usize = 256;

/// The grade of an unknown member of a shared type the documents hold: the families read every
/// slice under [`JSON_READING`](crate::JSON_READING), whose mask warns.
const UNKNOWN: Severity = crate::JSON_READING.grade(Class::Unknown);

/// The bind constraints `value` states as its bounds (`Schema::check`), at `at`: each member
/// outside its bound is an `InvalidValue` error, which fails the file as the game's parse does.
/// The slice's `Refuse` resets nothing, so the value is as read afterwards.
fn bounds<T: Schema>(value: &mut T, at: &JsonPath, d: &mut Diagnostics) {
    value.check(&mut SchemaCx::new(crate::JSON_READING, at.clone(), d));
}

/// A bind constraint that failed: a member outside its bound, which fails the file as the game's parse does.
fn violation(path: JsonPath, d: &mut Diagnostics) {
    d.push(Diagnostic::of(&core_codes::INVALID_VALUE).at(path));
}

// ---- minecraft:jigsaw ---------------------------------------------------------------------

impl Versioned for JigsawStructure {
    const VERSION: FormatVersion = V1_21_20;
}

impl VersionStruct for JigsawStructure {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut structure: Self = read_payload(payload, cx, d)?;
        structure.check(&cx.at, d);
        Some(structure)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl JigsawStructure {
    /// The bind constraints and the filter's findings.
    fn check(&mut self, at: &JsonPath, d: &mut Diagnostics) {
        // `max_depth` is bound to 0 to 20, and `max_distance_from_center`'s `horizontal` to 1
        // to 128 and `vertical` to 1 or more: the struct's own bounds and its enum's.
        bounds(self, at, d);
        if let Some(filter) = &self.biome_filters {
            d.extend(filter.check(&at.child("biome_filters"), UNKNOWN));
        }
    }
}

// ---- minecraft:template_pool --------------------------------------------------------------

impl Versioned for TemplatePool {
    const VERSION: FormatVersion = V1_21_20;
}

impl VersionStruct for TemplatePool {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut pool: Self = read_payload(payload, cx, d)?;
        pool.check(&cx.at, d);
        Some(pool)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl TemplatePool {
    /// The bind constraints.
    fn check(&mut self, at: &JsonPath, d: &mut Diagnostics) {
        let elements = at.child("elements");
        for (i, entry) in self.elements.iter_mut().enumerate() {
            let at = elements.index(i);
            // `weight` is bound to 1 to 200: the entry's own bounds.
            bounds(entry, &at, d);
            if let PoolElement::Single { location, .. } = &entry.element
                && location.as_str().len() > MAX_FILE_REFERENCE
            {
                violation(at.child("element").child("location"), d);
            }
        }
    }
}

// ---- minecraft:structure_set --------------------------------------------------------------

impl Versioned for StructureSet {
    const VERSION: FormatVersion = V1_21_20;
}

impl VersionStruct for StructureSet {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut set: Self = read_payload(payload, cx, d)?;
        set.check(&cx.at, d);
        Some(set)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl StructureSet {
    /// The bind constraints: `spacing` at least 1, each weight 1 to 200 (the struct's own
    /// bounds).
    fn check(&mut self, at: &JsonPath, d: &mut Diagnostics) {
        bounds(self, at, d);
    }
}

// ---- minecraft:processor_list -------------------------------------------------------------

impl Versioned for ProcessorList {
    const VERSION: FormatVersion = V1_21_20;
}

impl VersionStruct for ProcessorList {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut list: Self = read_payload(payload, cx, d)?;
        list.check(&cx.at, d);
        Some(list)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl ProcessorList {
    /// The bind constraints (the probabilities, the chances and distances of the rules), then
    /// the rules the bounds do not state: an object `limit` of the other form, a loot table too
    /// long, the block specifiers' findings.
    fn check(&mut self, at: &JsonPath, d: &mut Diagnostics) {
        bounds(self, at, d);
        let processors = at.child("processors");
        for (i, processor) in self.processors.iter().enumerate() {
            let at = processors.index(i);
            match processor {
                Processor::Rule { rules, .. } => check_rules(rules, &at.child("rules"), d),
                Processor::Capped {
                    delegate, limit, ..
                } => {
                    check_limit(limit, &at.child("limit"), d);
                    if let CappedDelegate::Rule { rules, .. } = delegate {
                        check_rules(rules, &at.child("delegate").child("rules"), d);
                    }
                }
                _ => {}
            }
        }
    }
}

/// An object `limit` whose `type` names the other form: vanilla's variant bind picks the form
/// by `type`, so the members of the one written are missing (`InvalidEnumValue` here, at
/// `type`).
fn check_limit(limit: &IntProvider, at: &JsonPath, d: &mut Diagnostics) {
    let mismatch = match limit {
        IntProvider::Constant { provider_type, .. } => *provider_type != IntProviderType::Constant,
        IntProvider::Uniform { provider_type, .. } => *provider_type != IntProviderType::Uniform,
        IntProvider::Value(_) => false,
    };
    if mismatch {
        d.push(Diagnostic::of(&core_codes::INVALID_ENUM_VALUE).at(at.child("type")));
    }
}

/// The rules of each rule of a `minecraft:rule` processor at `at` that are no bound.
fn check_rules(rules: &[ProcessorRule], at: &JsonPath, d: &mut Diagnostics) {
    for (i, rule) in rules.iter().enumerate() {
        let at = at.index(i);
        for (key, predicate) in [
            ("input_predicate", &rule.input_predicate),
            ("location_predicate", &rule.location_predicate),
        ] {
            if let Some(
                BlockPredicate::BlockstateMatch { block_state, .. }
                | BlockPredicate::RandomBlockstateMatch { block_state, .. },
            ) = predicate
            {
                d.extend(block_state.check(&at.child(key).child("block_state"), UNKNOWN));
            }
        }
        if let Some(BlockEntityModifier::AppendLoot { loot_table, .. }) =
            &rule.block_entity_modifier
            && loot_table.as_str().len() > MAX_FILE_REFERENCE
        {
            violation(at.child("block_entity_modifier").child("loot_table"), d);
        }
        d.extend(rule.output_state.check(&at.child("output_state"), UNKNOWN));
    }
}

// ---- the metadata registry (the typed model of the `.dat` body) ---------------------------

impl Versioned for MetadataRegistry {
    const VERSION: FormatVersion = V1_21_80;
}

impl VersionStruct for MetadataRegistry {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read_payload(payload, cx, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl VersionStruct for v1_21_50::MetadataRegistry {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read_payload(payload, cx, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

// ---- one metadata file ---------------------------------------------------------------------

/// The leading string of a metadata file and the bytes after it: an unsigned LEB128 length,
/// then that many bytes of UTF-8. `None` when the stream
/// ends early or the bytes are not UTF-8.
fn leading_string(bytes: &[u8]) -> Option<(&str, &[u8])> {
    let mut length: u64 = 0;
    let mut shift = 0;
    let mut at = 0;
    loop {
        let byte = *bytes.get(at)?;
        at += 1;
        length |= u64::from(byte & 0x7f).checked_shl(shift)?;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 35 {
            return None;
        }
    }
    let end = at.checked_add(usize::try_from(length).ok()?)?;
    let text = std::str::from_utf8(bytes.get(at..end)?).ok()?;
    Some((text, bytes.get(end..)?))
}

/// Loads one `worldgen/metadata/*.dat` file: the leading version string checked as vanilla
/// checks it, the body kept undecoded with a `SkippedCacheDocument` information (the crate's
/// own).
pub(crate) fn parse_metadata(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
) -> Result<Document<JigsawMetadata>, Rejected> {
    let origin = Origin::file(pack, side, path.clone());
    let mut diagnostics = Diagnostics::default();
    let reject = |mut diagnostics: Diagnostics, finding: Diagnostic| {
        diagnostics.push(finding);
        Err(Rejected::located(origin.clone(), diagnostics))
    };
    let Some((written, body)) = leading_string(bytes) else {
        return reject(diagnostics, bad_leading_string(path));
    };
    // An empty string is not parsed as a version, so the game ends up with no version at all.
    if written.is_empty() {
        return reject(diagnostics, Diagnostic::of(&core_codes::NO_VALID_VERSION));
    }
    let Some(input) = FormatVersionInput::parse(written) else {
        return reject(diagnostics, bad_leading_string(path));
    };
    let Selection::Struct(index) = select::<MetadataFamily>(&input) else {
        return reject(
            diagnostics,
            Diagnostic::of(core_codes::NO_PARSER_FOR_VERSION.with([written.to_string()])),
        );
    };
    let effective = input.version.unwrap_or(V1_21_50);
    let parsed_as =
        <MetadataHistory as bedrock_addon_core::versioning::VersionChain<MetadataFamily>>::VERSIONS
            .get(index)
            .copied()
            .unwrap_or(effective);
    diagnostics.push(Diagnostic::of(&core_codes::SKIPPED_CACHE_DOCUMENT));
    diagnostics.locate(&origin);
    let value = JigsawMetadata {
        format_version: written.to_owned(),
        body: body.to_vec(),
    };
    let version = VersionInfo::new(input.version, false, effective, parsed_as, false);
    Ok(Document::new(value, origin, version, diagnostics))
}

/// "Failed to parse format version string of file %s; it will be skipped."
fn bad_leading_string(path: &PackPath) -> Diagnostic {
    Diagnostic::of(codes::BAD_FORMAT_VERSION_STRING.with([path.as_str().to_owned()]))
}
