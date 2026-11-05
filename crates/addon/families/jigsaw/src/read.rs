//! Reading the jigsaw documents: each struct typed from its slice of the text, then the bind
//! constraints the game's parse checks.
//!
//! Every finding here is a parse finding: an error fails the file, as a parse error of the game does, and the
//! loader then adds the family's "Failed to parse …" line.

use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;

use super::history::{MetadataAny, MetadataFamily, v1_21_50};
use super::{
    BlockEntityModifier, BlockPredicate, CappedDelegate, IntProvider, IntProviderType,
    JigsawMetadata, JigsawStructure, MaxDistanceFromCenter, MetadataRegistry, PoolElement,
    PositionPredicate, Processor, ProcessorList, ProcessorRule, StructureSet, TemplatePool,
    V1_21_50,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, VanillaText,
};
use bedrock_addon_core::document::{Document, Extra, Origin, Rejected, VersionInfo};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, NullRule, StrictFloat, Value, keep_unknown_members, parse_at,
    reject_nulls,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::FormatVersionInput;
use bedrock_addon_core::versioning::{
    ParseCx, VersionStruct, Versioned, parse_payload, write_payload,
};
use bedrock_addon_core::versioning::{Selection, select};

/// The version of the four JSON documents' only struct.
pub(crate) const V1_21_20: FormatVersion = FormatVersion::new(1, 21, 20);
/// The version of the newest metadata registry.
pub(crate) const V1_21_80: FormatVersion = FormatVersion::new(1, 21, 80);

/// The longest `FileReference`.
const MAX_FILE_REFERENCE: usize = 256;

/// The text facet gives an internally tagged enum whose tag names no variant.
const UNKNOWN_TAG: &str = "No variant found";

/// Parses `T` from `payload`: the typed parse, its unknown members kept in the `extra` of the
/// struct they belong to (each an `UnrecognizedField` warning), and a failed parse as one error.
/// An internally tagged enum's unknown tag is `InvalidEnumValue`, as vanilla's enum binds
/// report it (facet calls it a type mismatch). `check` sees the JSON before the typed parse and
/// returns `false` to refuse it (the type pre-check of an untagged member).
fn parse_typed<'a, T: Facet<'a>>(
    payload: JsonSlice<'a>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    check: impl FnOnce(&Value, &mut Diagnostics) -> bool,
) -> Option<T> {
    let json = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text).ok();
    if let Some(json) = &json
        && !check(json, d)
    {
        return None;
    }
    match parse_at::<T>(payload) {
        // A `null` where the struct reads a value is an
        // `InvalidType` error that rejects the document.
        Ok(value)
            if cx.load.null_rule == Some(NullRule::Rejected)
                && reject_nulls(&value, payload, &cx.at, d) =>
        {
            None
        }
        Ok(mut value) => {
            if let Some(json) = &json {
                for (path, _) in keep_unknown_members(&mut value, json) {
                    // The game logs an unknown member at the content log's warning level and goes on.
                    d.push(
                        Diagnostic::warn(DiagnosticCode::UnrecognizedField { text: None })
                            .at(cx.at.join(&path)),
                    );
                }
            }
            Some(value)
        }
        Err(error) => {
            d.push(error.into_diagnostic(&cx.at, |code, error| {
                if matches!(code, DiagnosticCode::InvalidType { .. })
                    && error.describes(UNKNOWN_TAG)
                {
                    DiagnosticCode::InvalidEnumValue { text: None }
                } else {
                    code
                }
            }));
            None
        }
    }
}

/// A bind constraint that failed: a member outside its bound, which fails the file as the game's parse does.
fn violation(path: JsonPath, d: &mut Diagnostics) {
    d.push(Diagnostic::error(DiagnosticCode::InvalidValue { text: None }).at(path));
}

/// The member `key` of an object.
fn member<'v>(object: &'v Value, key: &str) -> Option<&'v Value> {
    object.as_object()?.get(key)
}

/// A member bound as `integer | object` must be one of the two: the untagged scalar arm is a
/// plain integer, which facet would also fill from a string or a real.
fn integer_or_object(value: Option<&Value>, at: JsonPath, d: &mut Diagnostics) -> bool {
    let Some(value) = value else {
        return true;
    };
    let integer = value
        .as_number()
        .is_some_and(facet_value::VNumber::is_integer);
    if integer || value.is_object() {
        return true;
    }
    d.push(Diagnostic::error(DiagnosticCode::InvalidType { text: None }).at(at));
    false
}

/// `v` is in `[min, max]`.
fn within(v: f64, min: f64, max: f64) -> bool {
    (min..=max).contains(&v)
}

// ---- minecraft:jigsaw ---------------------------------------------------------------------

impl Versioned for JigsawStructure {
    const VERSION: FormatVersion = V1_21_20;
}

impl VersionStruct for JigsawStructure {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let structure: Self = parse_typed(payload, cx, d, |json, d| {
            let padding = integer_or_object(
                member(json, "dimension_padding"),
                cx.at.child("dimension_padding"),
                d,
            );
            let distance = integer_or_object(
                member(json, "max_distance_from_center"),
                cx.at.child("max_distance_from_center"),
                d,
            );
            padding && distance
        })?;
        structure.check(&cx.at, d);
        Some(structure)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl JigsawStructure {
    /// The bind constraints and the filter's findings.
    fn check(&self, at: &JsonPath, d: &mut Diagnostics) {
        // `max_depth` is bound to 0 to 20.
        if self.max_depth.0 > 20 {
            violation(at.child("max_depth"), d);
        }
        // `horizontal` is bound to 1 to 128 and `vertical` to 1 or more.
        let distance = at.child("max_distance_from_center");
        match &self.max_distance_from_center {
            Some(MaxDistanceFromCenter::Horizontal(h)) if !(1..=128).contains(h) => {
                violation(distance, d);
            }
            Some(MaxDistanceFromCenter::Object {
                horizontal,
                vertical,
                ..
            }) => {
                if !(1..=128).contains(&horizontal.0) {
                    violation(distance.child("horizontal"), d);
                }
                if vertical.is_some_and(|v| v.0 < 1) {
                    violation(distance.child("vertical"), d);
                }
            }
            _ => {}
        }
        if let Some(filter) = &self.biome_filters {
            d.extend(filter.check(&at.child("biome_filters")));
        }
    }
}

// ---- minecraft:template_pool --------------------------------------------------------------

impl Versioned for TemplatePool {
    const VERSION: FormatVersion = V1_21_20;
}

impl VersionStruct for TemplatePool {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let pool: Self = parse_typed(payload, cx, d, |_, _| true)?;
        pool.check(&cx.at, d);
        Some(pool)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl TemplatePool {
    /// The bind constraints.
    fn check(&self, at: &JsonPath, d: &mut Diagnostics) {
        let elements = at.child("elements");
        for (i, entry) in self.elements.iter().enumerate() {
            let at = elements.index(i);
            // `weight` is bound to 1 to 200.
            if entry.weight.is_some_and(|w| !(1..=200).contains(&w.0)) {
                violation(at.child("weight"), d);
            }
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
        let set: Self = parse_typed(payload, cx, d, |_, _| true)?;
        set.check(&cx.at, d);
        Some(set)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl StructureSet {
    /// The bind constraints: `spacing` at least 1, each weight 1 to 200.
    fn check(&self, at: &JsonPath, d: &mut Diagnostics) {
        // `spacing` is bound to 1 or more.
        if self.placement.spacing.0 < 1 {
            violation(at.child("placement").child("spacing"), d);
        }
        let structures = at.child("structures");
        for (i, entry) in self.structures.iter().enumerate() {
            if !(1..=200).contains(&entry.weight.0) {
                violation(structures.index(i).child("weight"), d);
            }
        }
    }
}

// ---- minecraft:processor_list -------------------------------------------------------------

impl Versioned for ProcessorList {
    const VERSION: FormatVersion = V1_21_20;
}

impl VersionStruct for ProcessorList {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let list: Self = parse_typed(payload, cx, d, |json, d| {
            let mut ok = true;
            let processors = member(json, "processors").and_then(Value::as_array);
            for (i, processor) in processors.into_iter().flatten().enumerate() {
                let at = cx.at.child("processors").index(i).child("limit");
                ok &= integer_or_object(member(processor, "limit"), at, d);
            }
            ok
        })?;
        list.check(&cx.at, d);
        Some(list)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl ProcessorList {
    /// The bind constraints of every rule, and the block specifiers' findings.
    fn check(&self, at: &JsonPath, d: &mut Diagnostics) {
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
        d.push(
            Diagnostic::error(DiagnosticCode::InvalidEnumValue { text: None }).at(at.child("type")),
        );
    }
}

/// The constraints of each rule of a `minecraft:rule` processor at `at`.
fn check_rules(rules: &[ProcessorRule], at: &JsonPath, d: &mut Diagnostics) {
    for (i, rule) in rules.iter().enumerate() {
        let at = at.index(i);
        for (key, predicate) in [
            ("input_predicate", &rule.input_predicate),
            ("location_predicate", &rule.location_predicate),
        ] {
            if let Some(predicate) = predicate {
                check_block_predicate(predicate, &at.child(key), d);
            }
        }
        if let Some(PositionPredicate::AxisAlignedLinearPos {
            min_chance,
            max_chance,
            min_dist,
            max_dist,
            ..
        }) = &rule.position_predicate
        {
            let at = at.child("position_predicate");
            // The chances are bound to `[0, 1)` and the distances to 0 or more.
            for (key, chance) in [("min_chance", min_chance), ("max_chance", max_chance)] {
                if chance.is_some_and(|c| !(0.0..1.0).contains(&c.0)) {
                    violation(at.child(key), d);
                }
            }
            for (key, distance) in [("min_dist", min_dist), ("max_dist", max_dist)] {
                if distance.is_some_and(|v| v.0 < 0) {
                    violation(at.child(key), d);
                }
            }
        }
        if let Some(BlockEntityModifier::AppendLoot { loot_table, .. }) =
            &rule.block_entity_modifier
            && loot_table.as_str().len() > MAX_FILE_REFERENCE
        {
            violation(at.child("block_entity_modifier").child("loot_table"), d);
        }
        d.extend(rule.output_state.check(&at.child("output_state")));
    }
}

/// The constraints of a block predicate at `at`: `probability` in `[0, 1]`.
fn check_block_predicate(predicate: &BlockPredicate, at: &JsonPath, d: &mut Diagnostics) {
    let probability = |p: &StrictFloat<f32>, d: &mut Diagnostics| {
        // The probability is bound to `[0, 1]`.
        if !within(f64::from(p.0), 0.0, 1.0) {
            violation(at.child("probability"), d);
        }
    };
    match predicate {
        BlockPredicate::RandomBlockMatch { probability: p, .. } => probability(p, d),
        BlockPredicate::BlockstateMatch { block_state, .. } => {
            d.extend(block_state.check(&at.child("block_state")));
        }
        BlockPredicate::RandomBlockstateMatch {
            block_state,
            probability: p,
            ..
        } => {
            probability(p, d);
            d.extend(block_state.check(&at.child("block_state")));
        }
        _ => {}
    }
}

// ---- the metadata registry (the typed model of the `.dat` body) ---------------------------

/// What a registry payload is read into: its entries as raw slices, each parsed from its own
/// slice so that the unknown members inside an entry are kept.
#[derive(Facet)]
struct RegistryShell<'a> {
    metadata: IndexMap<String, RawJson<'a>>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

/// The entries and the payload's unknown members of a registry at `payload`.
fn parse_registry<T: for<'f> Facet<'f>>(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<(IndexMap<String, T>, Extra)> {
    let shell: RegistryShell<'_> = parse_payload(payload, cx, d)?;
    let mut entries = IndexMap::with_capacity(shell.metadata.len());
    for (key, raw) in &shell.metadata {
        let slice = payload.nested(raw)?;
        let entry_cx = ParseCx {
            at: cx.at.child("metadata").child(key),
            ..cx.clone()
        };
        let entry: T = parse_typed(slice, &entry_cx, d, |_, _| true)?;
        entries.insert(key.clone(), entry);
    }
    Some((entries, shell.extra))
}

impl Versioned for MetadataRegistry {
    const VERSION: FormatVersion = V1_21_80;
}

impl VersionStruct for MetadataRegistry {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let (metadata, extra) = parse_registry(payload, cx, d)?;
        Some(Self { metadata, extra })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl VersionStruct for v1_21_50::MetadataRegistry {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let (metadata, extra) = parse_registry(payload, cx, d)?;
        Some(Self { metadata, extra })
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
        return reject(
            diagnostics,
            Diagnostic::error(DiagnosticCode::MissingFormatVersion {
                text: Some(VanillaText::NoValidVersion),
            }),
        );
    }
    let Some(input) = FormatVersionInput::parse(written) else {
        return reject(diagnostics, bad_leading_string(path));
    };
    let Selection::Struct(index) = select::<MetadataFamily>(&input) else {
        return reject(
            diagnostics,
            Diagnostic::error(DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion {
                    version: written.to_owned(),
                }),
            }),
        );
    };
    let effective = input.version.unwrap_or(V1_21_50);
    let parsed_as =
        <MetadataAny as bedrock_addon_core::versioning::VersionChain<MetadataFamily>>::VERSIONS
            .get(index)
            .copied()
            .unwrap_or(effective);
    diagnostics.push(Diagnostic::info(DiagnosticCode::SkippedCacheDocument));
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
    Diagnostic::error(DiagnosticCode::MalformedFormatVersion {
        text: Some(VanillaText::Formatted {
            template: "Failed to parse format version string of file {}; it will be skipped.",
            args: vec![path.as_str().to_owned()],
        }),
    })
}
