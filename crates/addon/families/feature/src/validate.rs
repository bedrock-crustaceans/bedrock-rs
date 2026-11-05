//! The schema rules a typed parse cannot express (node ranges, array lengths, engine gates) and
//! the readers' own findings of the shared types, walked member by member.

use super::read::{
    ARRAY_TOO_LARGE, ARRAY_TOO_SMALL, FLOAT_OUT_OF_RANGE, INT_OUT_OF_RANGE, RANGE_MAX_OUT_OF_RANGE,
    RANGE_MIN_OUT_OF_RANGE, formatted, not_valid_here,
};
use super::values::{BlockList, Chance, ChanceList, JsonInt};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Phase};
use bedrock_addon_core::shared::{BlockDescriptor, IntRange};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{FieldMeta, ValidateCx, check_fields_since_engine};

/// The walk of one payload's rules over the validation context: where it is (`at`), the
/// pack's load context (the engine version the gates compare against), the file's
/// `format_version` (a rule on a member an upgrade step moved names the member where a file
/// of that version has it), and where findings go (an error rejects the document).
pub(crate) trait Check {
    /// Runs `f` with the path continued by `key`.
    fn nested(&mut self, key: &str, f: impl FnOnce(&mut Self));

    /// Runs `f` for `value` at `key` when present.
    fn opt<T>(&mut self, key: &str, value: Option<&T>, f: impl FnOnce(&mut Self, &T));

    /// Runs `f` for each element of `values` at `key`, with the element's index.
    fn each<T>(&mut self, key: &str, values: &[T], f: impl FnMut(&mut Self, &T));

    /// The path of the member `key` of the current one.
    fn path(&self, key: &str) -> JsonPath;

    /// An error at `key`.
    fn error(&mut self, key: &str, code: DiagnosticCode);

    /// A kept error at `key` (`""`: the current member): a setter's or a builder's line the
    /// game logs at level 3 while the feature stays registered.
    fn kept(&mut self, key: &str, code: DiagnosticCode);

    /// An integer node with the range `[min, max]`.
    fn check_int(&mut self, key: &str, value: Option<JsonInt>, min: i32, max: i32);

    /// An integer node with a lower bound only.
    fn int_min(&mut self, key: &str, value: Option<JsonInt>, min: i32);

    /// A float node with the range `[min, max]`.
    fn check_float(&mut self, key: &str, value: Option<f32>, min: f32, max: f32);

    /// An `IntRange` node whose ends must lie in `[min, max]`; also the range's own finding.
    fn range(&mut self, key: &str, value: Option<&IntRange>, min: i32, max: i32);

    /// An `IntRange` node with a lower bound only.
    fn range_min(&mut self, key: &str, value: Option<&IntRange>, min: i32);

    /// An array node of at least `min` (and at most `max`) elements.
    fn check_len(&mut self, key: &str, len: Option<usize>, min: usize, max: Option<usize>);

    /// A chance: a ratio's `numerator ≥ 0` and `denominator ≥ 1`.
    fn chance(&mut self, key: &str, value: Option<&Chance>);

    /// One chance or several.
    fn chances(&mut self, key: &str, value: Option<&ChanceList>);

    /// The chance at the current path.
    fn chance_here(&mut self, chance: &Chance);

    /// A block descriptor's own findings (a mixed or invalid shape, unknown members: warnings).
    fn descriptor(&mut self, key: &str, value: Option<&BlockDescriptor>);

    /// The findings of every descriptor of an array at `key`.
    fn descriptors(&mut self, key: &str, values: Option<&[BlockDescriptor]>);

    /// The findings of a descriptor or an array of them.
    fn list(&mut self, key: &str, value: Option<&BlockList>);

    /// An engine-gated member: on an engine older than `since` vanilla's schema has no such
    /// member, so a document that uses it fails with "child '%s' not valid here."; on the
    /// engine, a pack whose `min_engine_version` is older than `since` gets `FieldSinceEngine`
    /// (a warning, `check_fields_since_engine`).
    fn since_engine(&mut self, key: &'static str, present: bool, since: FormatVersion);
}

impl Check for ValidateCx<'_> {
    fn nested(&mut self, key: &str, f: impl FnOnce(&mut Self)) {
        self.at.push(key);
        f(self);
        self.at.pop();
    }

    fn opt<T>(&mut self, key: &str, value: Option<&T>, f: impl FnOnce(&mut Self, &T)) {
        if let Some(value) = value {
            self.at.push(key);
            f(self, value);
            self.at.pop();
        }
    }

    fn each<T>(&mut self, key: &str, values: &[T], mut f: impl FnMut(&mut Self, &T)) {
        self.at.push(key);
        for (index, value) in values.iter().enumerate() {
            self.at.push_index(index);
            f(self, value);
            self.at.pop();
        }
        self.at.pop();
    }

    fn path(&self, key: &str) -> JsonPath {
        self.at.child(key)
    }

    fn error(&mut self, key: &str, code: DiagnosticCode) {
        let at = self.path(key);
        self.diagnostics
            .push(Diagnostic::error(code).at(at).phase(Phase::Validate));
    }

    fn kept(&mut self, key: &str, code: DiagnosticCode) {
        let at = if key.is_empty() {
            self.at.clone()
        } else {
            self.path(key)
        };
        self.diagnostics
            .push_kept_error(Diagnostic::error(code).at(at).phase(Phase::Validate));
    }

    fn check_int(&mut self, key: &str, value: Option<JsonInt>, min: i32, max: i32) {
        if let Some(JsonInt(v, _)) = value
            && !(min..=max).contains(&v)
        {
            let text = formatted(INT_OUT_OF_RANGE, &[&v, &min, &max]);
            self.error(key, DiagnosticCode::InvalidValue { text: Some(text) });
        }
    }

    fn int_min(&mut self, key: &str, value: Option<JsonInt>, min: i32) {
        self.check_int(key, value, min, i32::MAX);
    }

    fn check_float(&mut self, key: &str, value: Option<f32>, min: f32, max: f32) {
        if let Some(v) = value
            && !(min..=max).contains(&v)
        {
            let text = formatted(FLOAT_OUT_OF_RANGE, &[&v, &min, &max]);
            self.error(key, DiagnosticCode::InvalidValue { text: Some(text) });
        }
    }

    fn range(&mut self, key: &str, value: Option<&IntRange>, min: i32, max: i32) {
        let Some(range) = value else {
            return;
        };
        let at = self.path(key);
        self.diagnostics.extend(range.check(&at));
        for (end, template) in [
            (range.min, RANGE_MIN_OUT_OF_RANGE),
            (range.max, RANGE_MAX_OUT_OF_RANGE),
        ] {
            if !(min..=max).contains(&end) {
                let text = formatted(template, &[&end, &min, &max]);
                self.error(key, DiagnosticCode::InvalidValue { text: Some(text) });
                return;
            }
        }
    }

    fn range_min(&mut self, key: &str, value: Option<&IntRange>, min: i32) {
        self.range(key, value, min, i32::MAX);
    }

    fn check_len(&mut self, key: &str, len: Option<usize>, min: usize, max: Option<usize>) {
        let Some(len) = len else {
            return;
        };
        let text = if len < min {
            formatted(ARRAY_TOO_SMALL, &[&len, &min])
        } else if let Some(max) = max.filter(|&max| len > max) {
            formatted(ARRAY_TOO_LARGE, &[&len, &max])
        } else {
            return;
        };
        self.error(key, DiagnosticCode::InvalidValue { text: Some(text) });
    }

    fn chance(&mut self, key: &str, value: Option<&Chance>) {
        if let Some(&Chance::Ratio {
            numerator,
            denominator,
        }) = value
        {
            self.nested(key, |check| {
                check.check_int("numerator", Some(JsonInt::new(numerator)), 0, i32::MAX);
                check.check_int("denominator", Some(JsonInt::new(denominator)), 1, i32::MAX);
            });
        }
    }

    fn chances(&mut self, key: &str, value: Option<&ChanceList>) {
        match value {
            Some(ChanceList::One(chance)) => self.chance(key, Some(chance)),
            Some(ChanceList::Many(chances)) => self.each(key, chances, |check, chance| {
                check.chance_here(chance);
            }),
            None => {}
        }
    }

    fn chance_here(&mut self, chance: &Chance) {
        if let &Chance::Ratio {
            numerator,
            denominator,
        } = chance
        {
            self.check_int("numerator", Some(JsonInt::new(numerator)), 0, i32::MAX);
            self.check_int("denominator", Some(JsonInt::new(denominator)), 1, i32::MAX);
        }
    }

    fn descriptor(&mut self, key: &str, value: Option<&BlockDescriptor>) {
        if let Some(descriptor) = value {
            let at = self.path(key);
            self.diagnostics.extend(descriptor.check(&at));
        }
    }

    fn descriptors(&mut self, key: &str, values: Option<&[BlockDescriptor]>) {
        if let Some(values) = values {
            self.each(key, values, |check, descriptor| {
                let at = check.at.clone();
                check.diagnostics.extend(descriptor.check(&at));
            });
        }
    }

    fn list(&mut self, key: &str, value: Option<&BlockList>) {
        match value {
            Some(BlockList::One(one)) => self.descriptor(key, Some(one)),
            Some(BlockList::Many(many)) => self.descriptors(key, Some(many)),
            None => {}
        }
    }

    fn since_engine(&mut self, key: &'static str, present: bool, since: FormatVersion) {
        if !present {
            return;
        }
        if self.load.engine_version < since {
            let at = self.path(key);
            self.diagnostics.push(not_valid_here(at));
            return;
        }
        let field = FieldMeta {
            name: key,
            since_engine: since,
        };
        check_fields_since_engine(&[field], |_| true, &self.at, self.load, self.diagnostics);
    }
}
