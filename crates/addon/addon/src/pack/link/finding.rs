//! The findings of the link pass: one constructor per kind of miss.
//!
//! A finding is told at the referencing document: its pack and file, and the JSON path of the
//! member that holds the reference **as the file has it** (the key the author wrote, at the
//! place the author wrote it, whatever an upgrade step renamed or moved since). The name is in
//! the detail as written, never changed by the lookup's normalisation. A finding of the
//! crate's own has no text of its own; one that vanilla words carries vanilla's text.
// There is one code per kind of miss, and the detail and the hint carry the kind and the name.

#![allow(
    dead_code,
    reason = "the frame of the rule packages: each uses the constructors its rows need"
)]

use bedrock_addon_core::diagnostic::builders;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, JsonPath, Location, Phase, ReferenceHint,
    Severity, VanillaText,
};
use bedrock_addon_core::document::Origin;

use super::kinds;

/// Where and how bad a finding is: the referencing document, the member, the row's severity
/// and vanilla's text when vanilla words the miss.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct At<'r> {
    /// The referencing document.
    pub(crate) origin: &'r Origin,
    /// The member that holds the reference, as the file has it.
    pub(crate) path: JsonPath,
    /// The severity of the rule's row.
    pub(crate) severity: Severity,
    /// Vanilla's text; `None` for a finding of the crate's own.
    pub(crate) text: Option<VanillaText>,
}

impl<'r> At<'r> {
    /// A finding of the crate's own at `path` of the document of `origin`.
    pub(crate) fn new(origin: &'r Origin, path: JsonPath, severity: Severity) -> Self {
        Self {
            origin,
            path,
            severity,
            text: None,
        }
    }

    /// The same with vanilla's text.
    pub(crate) fn with_text(mut self, text: VanillaText) -> Self {
        self.text = Some(text);
        self
    }

    /// The same with the linked-asset check's line for `name`, its breadcrumb the finding's
    /// path from the segment `from` on (the payload key left out where the game leaves it
    /// out).
    ///
    /// The game writes the line to the content log at the error level and drops nothing: the
    /// finding is an error, save an information a partial view or a lint
    /// made (the crate's own uncertainty).
    pub(crate) fn validator(mut self, says: Validator, from: usize, name: &str) -> Self {
        let text = validator_text(says, &self.path, from, name);
        if self.severity != Severity::Info {
            self.severity = Severity::Error;
        }
        self.with_text(text)
    }

    /// A finding of any `code` here, without a detail: for a code outside the six this module
    /// has a constructor for, whose miss names nothing (the place says it all).
    pub(crate) fn diagnostic(self, code: DiagnosticCode) -> Diagnostic {
        let location = Location::new(
            Some(self.origin.pack.clone()),
            Some(self.origin.path.clone()),
            self.path,
            None,
        );
        Diagnostic::new(code, self.severity, Phase::Link, location)
    }

    /// A finding of any `code` here with the `kind` named `id` (as written) and the lookup's
    /// near miss in its detail: for a code outside the six this module has a constructor for
    /// (`ScriptImportNotFound`, `FontFileMissing`). The six constructors are this with their
    /// code.
    pub(crate) fn reference(
        self,
        code: DiagnosticCode,
        kind: &'static str,
        id: &str,
        hint: Option<ReferenceHint>,
    ) -> Diagnostic {
        let mut finding = self.diagnostic(code);
        finding.detail = Some(Box::new(DiagnosticDetail::Reference {
            kind,
            id: id.to_owned(),
            hint,
        }));
        finding
    }
}

/// `UnresolvedReference`: the `kind` named `id` (as written) resolves to nothing. `hint` is the
/// lookup's near miss (`Target::hint`).
pub(crate) fn unresolved(
    at: At<'_>,
    kind: &'static str,
    id: &str,
    hint: Option<ReferenceHint>,
) -> Diagnostic {
    let code = DiagnosticCode::from(builders::worded(
        &core_codes::UNRESOLVED_REFERENCE,
        &core_codes::UNRESOLVED_REFERENCE_WORDED,
        at.text.as_ref(),
    ));
    at.reference(code, kind, id, hint)
}

/// `InvalidReference`: the `kind` named `id` resolves to something the use cannot accept.
pub(crate) fn invalid(at: At<'_>, kind: &'static str, id: &str) -> Diagnostic {
    let code = DiagnosticCode::from(builders::worded(
        &core_codes::INVALID_REFERENCE,
        &core_codes::INVALID_REFERENCE_WORDED,
        at.text.as_ref(),
    ));
    at.reference(code, kind, id, None)
}

/// `ReferenceCycle`: following the reference to the `kind` named `id` comes back to where it
/// started.
pub(crate) fn cycle(at: At<'_>, kind: &'static str, id: &str) -> Diagnostic {
    let code = DiagnosticCode::from(builders::worded(
        &core_codes::REFERENCE_CYCLE,
        &core_codes::REFERENCE_CYCLE_WORDED,
        at.text.as_ref(),
    ));
    at.reference(code, kind, id, None)
}

/// `TextureNotFound`: no pack of the stack has a texture at `path` (as written).
pub(crate) fn texture_not_found(at: At<'_>, path: &str) -> Diagnostic {
    let code = DiagnosticCode::from(builders::worded(
        &core_codes::TEXTURE_NOT_FOUND,
        &core_codes::TEXTURE_NOT_FOUND_WORDED,
        at.text.as_ref(),
    ));
    at.reference(code, kinds::TEXTURE, path, None)
}

/// `UnknownSound`: a sound name of `kind` (a sound event of the definitions, a level sound
/// event, a block sound type, a named sound) that nothing provides.
pub(crate) fn unknown_sound(at: At<'_>, kind: &'static str, name: &str) -> Diagnostic {
    let code = DiagnosticCode::from(builders::worded(
        &core_codes::UNKNOWN_SOUND,
        &core_codes::UNKNOWN_SOUND_WORDED,
        at.text.as_ref(),
    ));
    at.reference(code, kind, name, None)
}

/// `UnresolvedLangKey`: no `.lang` file defines `key`. The code has no text of vanilla's.
pub(crate) fn unresolved_lang_key(at: At<'_>, key: &str) -> Diagnostic {
    at.reference(
        DiagnosticCode::from(&core_codes::UNRESOLVED_LANG_KEY),
        kinds::LANG_KEY,
        key,
        None,
    )
}

// ---- the game's linked-asset check ------------------------------------------------------

// The check's texts are core vocabulary: content modules give them too.
pub(crate) use bedrock_addon_core::diagnostic::validator::{Validator, validator_text};

/// A second finding of a miss the game logs twice (its family's line and the validator's):
/// `UnresolvedReference` of `kind` named `name` at `at`, with the validator's line.
pub(crate) fn validator_miss(
    at: At<'_>,
    says: Validator,
    from: usize,
    kind: &'static str,
    name: &str,
) -> Diagnostic {
    unresolved(at.validator(says, from, name), kind, name, None)
}
