//! The checks of `BlockCullingParser::prepareCullingData` on a parsed culling-rules document
//! the identifier, then the rules. Any
//! failure skips the file.

use std::collections::HashSet;

use super::texts::{
    CUBE_WITHOUT_FACE, DUPLICATE_PART, FACE_WITHOUT_CUBE, IDENTIFIER_FAILED, ILLEGAL_CHARACTERS,
    INVALID_IDENTIFIER, MINECRAFT_NAMESPACE, NAME_INVALID, NAMESPACE_INVALID, NEEDS_NAMESPACE,
    formatted,
};
use super::{CullingRules, Facing, Rule};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};

/// Why vanilla's `Util::validateIdentifier` refuses `identifier`: its messages, in order (none
/// for an empty identifier, which fails without a reason). `Ok` when it passes.
// The checks run on a lower-cased copy, the first
// failure returns.
pub(super) fn identifier(identifier: &str, base_game: bool) -> Result<(), Vec<&'static str>> {
    let illegal = |chunk: &str| chunk.contains([' ', '\t', '\n', '\r']);
    if identifier.is_empty() {
        return Err(Vec::new());
    }
    let mut parts = identifier.split(':');
    let (first, second, third) = (parts.next(), parts.next(), parts.next());
    let (namespace, name) = match (first, second, third) {
        // No `:` at all: a valid format, but an identifier here needs a namespace.
        (Some(_), None, _) => return Err(vec![NEEDS_NAMESPACE]),
        (Some(namespace), Some(name), None) if !namespace.is_empty() && !name.is_empty() => {
            (namespace, name)
        }
        _ => return Err(vec![INVALID_IDENTIFIER]),
    };
    if namespace.eq_ignore_ascii_case("minecraft") && !base_game {
        return Err(vec![MINECRAFT_NAMESPACE]);
    }
    if illegal(namespace) {
        return Err(vec![ILLEGAL_CHARACTERS, NAMESPACE_INVALID]);
    }
    if illegal(name) {
        return Err(vec![ILLEGAL_CHARACTERS, NAME_INVALID]);
    }
    Ok(())
}

/// One failed rule check: the rule's index, the member the finding points at, and vanilla's
/// text.
pub(super) struct RuleProblem {
    /// The rule's index.
    pub(super) index: usize,
    /// The member of `geometry_part` the finding points at; `None` for the part as a whole.
    pub(super) member: Option<&'static str>,
    /// Vanilla's template and its arguments.
    pub(super) text: (&'static str, Vec<String>),
}

/// The rule checks: every rule with a cube and no face, then with a face and no cube, then
/// every rule whose part (bone, cube, face; the direction and condition do not count) an
/// earlier rule already has.
// The game's culling-data preparation makes two passes over all rules; the duplicate key is
// `format("{}:{}:{}", bone, cube_or_-1, face_or_6)`.
pub(super) fn rules(rules: &[Rule]) -> Vec<RuleProblem> {
    let mut problems = Vec::new();
    for (index, rule) in rules.iter().enumerate() {
        let part = &rule.geometry_part;
        let index_text = index.to_string();
        match (part.cube, part.face) {
            (Some(_), None) => problems.push(RuleProblem {
                index,
                member: Some("cube"),
                text: (CUBE_WITHOUT_FACE, vec![index_text, part.bone.clone()]),
            }),
            (None, Some(_)) => problems.push(RuleProblem {
                index,
                member: Some("face"),
                text: (FACE_WITHOUT_CUBE, vec![index_text, part.bone.clone()]),
            }),
            _ => {}
        }
    }
    let mut seen: HashSet<(&str, i32, u8)> = HashSet::new();
    for (index, rule) in rules.iter().enumerate() {
        let part = &rule.geometry_part;
        let cube = part.cube.unwrap_or(-1);
        let face = part.face.map_or(6, Facing::number);
        if !seen.insert((part.bone.as_str(), cube, face)) {
            problems.push(RuleProblem {
                index,
                member: None,
                text: (
                    DUPLICATE_PART,
                    vec![
                        index.to_string(),
                        part.bone.clone(),
                        cube.to_string(),
                        part.face.map_or("Unknown facing", Facing::title).to_owned(),
                    ],
                ),
            });
        }
    }
    problems
}

/// Runs both checks on `document`, reporting below `at` (the payload). An identifier that
/// fails skips the rule checks.
pub(super) fn check(document: &CullingRules, base_game: bool, at: &JsonPath, d: &mut Diagnostics) {
    let error = |code: DiagnosticCode, path: JsonPath| {
        Diagnostic::new(
            code,
            Severity::Error,
            Phase::Validate,
            Location::new(None, None, path, None),
        )
    };
    let id = document.description.identifier.as_str();
    if let Err(reasons) = identifier(id, base_game) {
        let mut path = at.clone();
        path.push("description");
        path.push("identifier");
        for reason in reasons {
            d.push(error(
                DiagnosticCode::NamespaceRule {
                    text: Some(formatted(reason, &[])),
                },
                path.clone(),
            ));
        }
        d.push(error(
            DiagnosticCode::MissingIdentifier {
                text: Some(formatted(IDENTIFIER_FAILED, &[id])),
            },
            path,
        ));
        return;
    }
    for problem in rules(&document.rules) {
        let mut path = at.clone();
        path.push("rules");
        path.push_index(problem.index);
        path.push("geometry_part");
        if let Some(member) = problem.member {
            path.push(member);
        }
        let (template, args) = problem.text;
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let text = Some(formatted(template, &args));
        let code = if template == DUPLICATE_PART {
            DiagnosticCode::DuplicateIdentifier { text }
        } else {
            DiagnosticCode::InvalidValue { text }
        };
        d.push(error(code, path));
    }
}
