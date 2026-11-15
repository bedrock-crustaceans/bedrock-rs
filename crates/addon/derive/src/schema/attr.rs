//! The `#[facet(..)]` attributes `#[derive(Schema)]` reads: facet's own `default`, `rename`,
//! `rename_all`, `transparent`, `flatten` and `skip`, and the `schema::` keys. Every other
//! entry is facet's business and is passed over.

use facet_macro_types::RenameRule;
use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{Attribute, Error, Expr, Ident, Lit, LitStr, Result};

use crate::attr::Version;

/// One entry of a `#[facet(..)]` list: its path (`default`, `schema::min`) and the tokens after
/// its `=`, if any.
struct Entry {
    path: Vec<Ident>,
    value: Option<TokenStream>,
    span: Span,
}

impl Entry {
    /// `schema::key` → `Some("key")`.
    fn schema_key(&self) -> Option<String> {
        match self.path.as_slice() {
            [ns, key] if ns == "schema" => Some(key.to_string()),
            _ => None,
        }
    }

    /// A facet builtin, `default` → `Some("default")`.
    fn builtin(&self) -> Option<String> {
        match self.path.as_slice() {
            [key] => Some(key.to_string()),
            _ => None,
        }
    }

    /// The value as an expression; a key without one is facet's error, so `None` here.
    fn expr(&self) -> Option<Result<Expr>> {
        self.value.clone().map(syn::parse2)
    }
}

/// The entries of every `#[facet(..)]` among `attrs`, split at the commas of the list's top
/// level (a comma inside an unbracketed generic value splits it, as it does for facet).
fn entries(attrs: &[Attribute]) -> Vec<Entry> {
    let mut out = Vec::new();
    for attr in attrs.iter().filter(|a| a.path().is_ident("facet")) {
        let Ok(list) = attr.meta.require_list() else {
            continue;
        };
        let mut current: Vec<TokenTree> = Vec::new();
        for token in list.tokens.clone() {
            if matches!(&token, TokenTree::Punct(p) if p.as_char() == ',') {
                out.extend(entry(std::mem::take(&mut current)));
            } else {
                current.push(token);
            }
        }
        out.extend(entry(current));
    }
    out
}

/// One entry from its tokens: `a`, `a::b`, then optionally `= value` or a `(..)` group.
fn entry(tokens: Vec<TokenTree>) -> Option<Entry> {
    let mut tokens = tokens.into_iter().peekable();
    let first = tokens.next()?;
    let span = first.span();
    let TokenTree::Ident(first) = first else {
        return None;
    };
    let mut path = vec![first];
    loop {
        match tokens.peek() {
            Some(TokenTree::Punct(p)) if p.as_char() == ':' => {
                tokens.next();
                if !matches!(tokens.next(), Some(TokenTree::Punct(p)) if p.as_char() == ':') {
                    return None;
                }
                match tokens.next() {
                    Some(TokenTree::Ident(ident)) => path.push(ident),
                    _ => return None,
                }
            }
            _ => break,
        }
    }
    let value = match tokens.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == '=' => Some(tokens.collect()),
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => Some(g.stream()),
        _ => None,
    };
    Some(Entry { path, value, span })
}

/// The comparison of a bound key.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Op {
    Min,
    Max,
    Above,
    Below,
}

impl Op {
    fn from_key(key: &str) -> Option<Self> {
        match key {
            "min" => Some(Self::Min),
            "max" => Some(Self::Max),
            "above" => Some(Self::Above),
            "below" => Some(Self::Below),
            _ => None,
        }
    }

    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Min => "min",
            Self::Max => "max",
            Self::Above => "above",
            Self::Below => "below",
        }
    }

    /// The `BoundOp` variant.
    pub(crate) fn variant(self) -> Ident {
        let name = match self {
            Self::Min => "Min",
            Self::Max => "Max",
            Self::Above => "Above",
            Self::Below => "Below",
        };
        Ident::new(name, Span::call_site())
    }

    /// Whether `self` and `other` bound the same side.
    fn same_side(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Min | Self::Above, Self::Min | Self::Above)
                | (Self::Max | Self::Below, Self::Max | Self::Below)
        )
    }
}

/// What a bound compares with: a number literal (a leading `-` allowed) or a sibling field.
pub(crate) enum LimitDecl {
    Num(Expr),
    Field(Ident),
}

/// One bound key on a field.
pub(crate) struct BoundDecl {
    pub op: Op,
    pub limit: LimitDecl,
    pub span: Span,
}

/// facet's `default` on a field: bare (the type's `Default`) or `= expr`.
pub(crate) struct DefaultDecl {
    pub value: Option<Expr>,
    pub span: Span,
}

/// What a field declares.
#[derive(Default)]
pub(crate) struct FieldAttrs {
    pub default: Option<DefaultDecl>,
    pub rename: Option<String>,
    /// facet's `alias`: a second JSON name the member is read by.
    pub alias: Option<String>,
    pub skip: bool,
    pub flatten: bool,
    pub since_format: Option<Version>,
    pub since_engine: Option<Version>,
    pub unknown: Option<String>,
    pub fallback: Option<(Expr, Span)>,
    pub bounds: Vec<BoundDecl>,
    /// `schema::multiple_of`, as the `f64` it compares with.
    pub multiple_of: Option<(f64, Span)>,
    pub molang: Option<(Expr, Span)>,
    pub molang_with: Option<(Expr, Span)>,
    /// `schema::read_with = path`: the field's read, `fn(&Node, &mut Read) -> Option<T>`.
    pub read_with: Option<(Expr, Span)>,
    /// `schema::check_with = path`: the field's check, `fn(&mut T, &mut SchemaCx)`, in place of
    /// the walk into its type.
    pub check_with: Option<(Expr, Span)>,
    /// `schema::min_len`, `schema::max_len`: the number of elements the field holds.
    pub min_len: Option<(usize, Span)>,
    pub max_len: Option<(usize, Span)>,
    pub foreign_event: Option<Span>,
    pub lowercase: Option<Span>,
    /// `schema::required`: an `Option` field the document must write.
    pub required: Option<Span>,
    /// The read's codes: `schema::wrong_type`, `schema::missing`, `schema::invalid`,
    /// `schema::negative`.
    pub wrong_type: Option<(Expr, Span)>,
    pub missing: Option<(Expr, Span)>,
    pub invalid: Option<(Expr, Span)>,
    pub negative: Option<(Expr, Span)>,
    /// `schema::finite`: a float beyond its width reads as its largest finite value.
    pub finite: Option<Span>,
    /// `schema::low_code`, `schema::high_code`: what a value outside the field's low or high
    /// bound is told as, an expression of type `Code`, made when it is told.
    pub low_code: Option<(Expr, Span)>,
    pub high_code: Option<(Expr, Span)>,
    /// `schema::kept`: the game logs a value outside the field's bounds and keeps it.
    pub kept: Option<Span>,
    /// `schema::key_with = path`: the rule of a map's keys, `fn(&str) -> Option<Code>`.
    pub key_with: Option<(Expr, Span)>,
    /// `schema::keys = KEYS`: the rule a map's member names are read by, a `schema::Keys`.
    pub keys: Option<(Expr, Span)>,
    /// `schema::iterated`: the list is iterated as jsoncpp iterates a value.
    pub iterated: Option<Span>,
    /// `schema::reads_null`: a `null` member of an `Option` field is read as a value.
    pub reads_null: Option<Span>,
    /// `schema::elements`: `true` for `"drop"`, `false` for `"fail"`.
    pub elements: Option<(bool, Span)>,
    /// `schema::or_kept`: `true` for `"told"`, `false` for `"silent"`.
    pub or_kept: Option<(bool, Span)>,
    /// `schema::skip_code`: what a list element left out is told as, in place of its own
    /// findings, an expression of type `Code`.
    pub skip_code: Option<(Expr, Span)>,
    /// Where the first stored key (`since_format`, `since_engine`, `unknown`) is written.
    pub stored: Option<Span>,
}

/// What a struct or enum declares.
#[derive(Default)]
#[allow(clippy::struct_excessive_bools, reason = "one flag per attribute read")]
pub(crate) struct ContainerAttrs {
    pub rename_all: Option<RenameRule>,
    pub transparent: bool,
    pub unknown: Option<String>,
    pub expected: Option<LitStr>,
    pub case_insensitive: bool,
    /// `schema::order = "game"`: the enum keeps its own `ALL`.
    pub order_game: bool,
    /// `schema::string_only`: the enum reads a string and nothing else.
    pub string_only: bool,
    /// `schema::rule = path`: a rule across the fields of the struct.
    pub rule: Option<(Expr, Span)>,
    /// `schema::trigger`: the struct is an event trigger itself.
    pub trigger: bool,
    /// `schema::molang = SPEC` on a struct: the default spec of the Molang walk below it.
    pub molang: Option<(Expr, Span)>,
    /// `schema::molang_with = path` on a struct: its Molang walk, in place of the walk over its
    /// fields.
    pub molang_with: Option<(Expr, Span)>,
    /// `schema::invalid = CODE` on an enum with data: the code of a value that names no variant.
    pub invalid: Option<(Expr, Span)>,
}

/// The keys of a field, the keys of a string enum's container, and the one key of both.
const FIELD_KEYS: &[&str] = &[
    "since_format",
    "since_engine",
    "fallback",
    "min",
    "max",
    "above",
    "below",
    "multiple_of",
    "molang",
    "molang_with",
    "read_with",
    "check_with",
    "min_len",
    "max_len",
    "foreign_event",
    "lowercase",
    "required",
    "wrong_type",
    "missing",
    "invalid",
    "finite",
    "iterated",
    "reads_null",
    "elements",
    "or_kept",
    "low_code",
    "high_code",
    "kept",
    "key_with",
    "skip_code",
    "negative",
    "keys",
];
const ENUM_KEYS: &[&str] = &["expected", "case_insensitive", "order", "string_only"];
/// The keys of a struct alone.
const STRUCT_KEYS: &[&str] = &["rule", "trigger"];

/// Where the attributes stand, for the placement errors.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Place {
    Struct,
    Enum,
    /// An untagged enum with data.
    Untagged,
    /// An internally (`tag = ".."`) or externally tagged enum with data.
    Tagged,
    Field,
    Variant,
    /// A variant of an untagged enum with data.
    UntaggedVariant,
    /// A variant of a tagged enum with data.
    TaggedVariant,
}

/// Records `value` in `slot`, or the duplicate's error.
fn once<T>(slot: &mut Option<T>, value: T, key: &str, span: Span, errors: &mut Vec<Error>) {
    if slot.is_some() {
        errors.push(Error::new(span, format!("duplicate `schema::{key}`")));
    } else {
        *slot = Some(value);
    }
}

/// The schema keys at `place` that do not belong there.
fn placement(entries: &[Entry], place: Place, errors: &mut Vec<Error>) {
    for entry in entries {
        // Facet takes a key written with a longer path (`crate::schema::min`) as it takes
        // `schema::min`; the derive reads the one spelling, and would drop the other in silence.
        if entry.path.len() > 2
            && let Some(key) = entry.path.last()
            && entry.path[..entry.path.len() - 1]
                .iter()
                .any(|s| s == "schema")
        {
            errors.push(Error::new(
                entry.span,
                format!("write the key as `schema::{key}`: the derive reads no other path to it"),
            ));
            continue;
        }
        let Some(key) = entry.schema_key() else {
            continue;
        };
        let field_key = FIELD_KEYS.contains(&key.as_str());
        let enum_key = ENUM_KEYS.contains(&key.as_str());
        let struct_key = STRUCT_KEYS.contains(&key.as_str());
        let message = match place {
            Place::Field | Place::Enum | Place::Untagged | Place::Tagged | Place::Variant
                if struct_key =>
            {
                format!("`schema::{key}` is a struct's key: put it on the struct it is about")
            }
            Place::UntaggedVariant | Place::TaggedVariant if struct_key => {
                format!("a variant takes no `schema::{key}`: it is a struct's key")
            }
            Place::Struct if key == "molang" || key == "molang_with" => continue,
            Place::Untagged | Place::Tagged if key == "invalid" => continue,
            Place::Struct | Place::Enum | Place::Untagged | Place::Tagged if field_key => {
                format!("`schema::{key}` is a field's key: put it on the field it is about")
            }
            Place::Enum if key == "unknown" => {
                "`schema::unknown` is about the members of an object: a string enum has none"
                    .to_owned()
            }
            Place::Struct if enum_key => {
                format!("`schema::{key}` is a string enum's key: a struct has no names to read")
            }
            Place::Field if enum_key => {
                format!("`schema::{key}` is a string enum's key: put it on the enum")
            }
            _ if key == "otherwise" && place != Place::UntaggedVariant => {
                "`schema::otherwise` marks the newtype variant of an untagged enum that a value \
                 takes when no other arm reads it"
                    .to_owned()
            }
            Place::Variant if field_key || enum_key || key == "unknown" => {
                format!(
                    "a variant of a string enum takes no `schema::` key, `schema::{key}` included"
                )
            }
            Place::Untagged if enum_key => format!(
                "`schema::{key}` is a string enum's key: an untagged enum with data has no names \
                 to read"
            ),
            Place::Untagged | Place::Tagged if key == "unknown" => {
                "`schema::unknown` is about the \
                 members of an object: put it on the struct variant whose members it is about"
                    .to_owned()
            }
            Place::UntaggedVariant if field_key || enum_key => format!(
                "a variant of an untagged enum takes no `schema::{key}`: put it on the field it \
                 is about"
            ),
            Place::Tagged if enum_key => format!(
                "`schema::{key}` is a string enum's key: a tagged enum with data names its \
                 variants by facet's `rename`"
            ),
            Place::TaggedVariant if field_key || enum_key => format!(
                "a variant of a tagged enum takes no `schema::{key}`: put it on the field it is \
                 about"
            ),
            _ => continue,
        };
        errors.push(Error::new(entry.span, message));
    }
}

/// A stored key's string literal.
fn string_value(entry: &Entry, key: &str) -> Result<LitStr> {
    let Some(value) = &entry.value else {
        return Err(Error::new(
            entry.span,
            format!("`schema::{key}` takes a string literal"),
        ));
    };
    syn::parse2::<LitStr>(value.clone()).map_err(|_| {
        Error::new_spanned(
            value,
            format!("`schema::{key}` takes a string literal, as in `schema::{key} = \"..\"`"),
        )
    })
}

/// `schema::unknown`'s word, checked.
fn unknown_word(entry: &Entry) -> Result<String> {
    let lit = string_value(entry, "unknown")?;
    let word = lit.value();
    if ["keep", "report", "reject", "ignore", "component", "silent"].contains(&word.as_str()) {
        Ok(word)
    } else {
        Err(Error::new(
            lit.span(),
            format!(
                "`{word}` is not an unknown-member rule: expected `keep`, `report`, `reject`, \
                 `ignore` or `component`"
            ),
        ))
    }
}

/// A bound's value: a number literal, a negated one, or one identifier.
fn limit(entry: &Entry, op: Op) -> Result<LimitDecl> {
    let wrong = || {
        Error::new(
            entry.span,
            format!(
                "`schema::{}` takes a number or the name of another field of the struct",
                op.key()
            ),
        )
    };
    let Some(Ok(expr)) = entry.expr() else {
        return Err(wrong());
    };
    match &expr {
        Expr::Lit(lit) if matches!(lit.lit, Lit::Int(_) | Lit::Float(_)) => {
            Ok(LimitDecl::Num(expr))
        }
        Expr::Unary(unary)
            if matches!(unary.op, syn::UnOp::Neg(_))
                && matches!(&*unary.expr, Expr::Lit(lit) if matches!(lit.lit, Lit::Int(_) | Lit::Float(_))) =>
        {
            Ok(LimitDecl::Num(expr))
        }
        Expr::Path(path) if path.qself.is_none() => match path.path.get_ident() {
            Some(ident) => Ok(LimitDecl::Field(ident.clone())),
            None => Err(Error::new_spanned(&expr, wrong().to_string())),
        },
        _ => Err(Error::new_spanned(&expr, wrong().to_string())),
    }
}

/// `schema::multiple_of`'s step: a positive number literal.
fn step(entry: &Entry) -> Result<f64> {
    let wrong = |span: Span| {
        Error::new(
            span,
            "`schema::multiple_of` takes a positive number literal, as in `schema::multiple_of = 16`",
        )
    };
    let Some(Ok(expr)) = entry.expr() else {
        return Err(wrong(entry.span));
    };
    let step = match &expr {
        Expr::Lit(syn::ExprLit {
            lit: Lit::Int(int), ..
        }) => int.base10_parse::<f64>().ok(),
        Expr::Lit(syn::ExprLit {
            lit: Lit::Float(float),
            ..
        }) => float.base10_parse::<f64>().ok(),
        _ => None,
    };
    match step {
        Some(step) if step.is_finite() && step > 0.0 => Ok(step),
        _ => Err(Error::new_spanned(&expr, wrong(entry.span).to_string())),
    }
}

/// A key whose value is one of two words: `true` for `yes`, `false` for `no`.
fn either_word(entry: &Entry, key: &str, yes: &str, no: &str) -> Result<bool> {
    let lit = string_value(entry, key)?;
    let word = lit.value();
    if word == yes || word == no {
        Ok(word == yes)
    } else {
        Err(Error::new(
            lit.span(),
            format!("`schema::{key}` takes \"{yes}\" or \"{no}\""),
        ))
    }
}

/// `schema::min_len`'s or `max_len`'s count: an integer literal.
fn length(entry: &Entry, key: &str) -> Result<usize> {
    let wrong = |span: Span| {
        Error::new(
            span,
            format!("`schema::{key}` takes a whole number literal, as in `schema::{key} = 1`"),
        )
    };
    let Some(Ok(expr)) = entry.expr() else {
        return Err(wrong(entry.span));
    };
    match &expr {
        Expr::Lit(syn::ExprLit {
            lit: Lit::Int(int), ..
        }) => int.base10_parse::<usize>().map_err(|_| wrong(entry.span)),
        _ => Err(Error::new_spanned(&expr, wrong(entry.span).to_string())),
    }
}

/// A key's value as an expression; facet already refuses a valued key written without one.
fn value_expr(entry: &Entry, key: &str) -> Result<Expr> {
    match entry.expr() {
        Some(expr) => expr,
        None => Err(Error::new(
            entry.span,
            format!("`schema::{key}` requires a value"),
        )),
    }
}

/// Records the bound `entry` states on `out`, or the error of a second one on a side.
fn push_bound(out: &mut FieldAttrs, entry: &Entry, op: Op) -> Result<()> {
    let span = entry.span;
    let limit = limit(entry, op)?;
    if let Some(previous) = out.bounds.iter().find(|b| b.op.same_side(op)) {
        let message = if previous.op == op {
            format!("duplicate `schema::{}`", op.key())
        } else {
            format!(
                "`schema::{}` and `schema::{}` bound the same side: keep one",
                previous.op.key(),
                op.key()
            )
        };
        return Err(Error::new(span, message));
    }
    out.bounds.push(BoundDecl { op, limit, span });
    Ok(())
}

impl FieldAttrs {
    /// Reads a field's attributes; each malformed key adds an error and is left out.
    pub(crate) fn read(attrs: &[Attribute], errors: &mut Vec<Error>) -> Self {
        let entries = entries(attrs);
        placement(&entries, Place::Field, errors);
        let mut out = FieldAttrs::default();
        for entry in &entries {
            if let Some(key) = entry.builtin() {
                out.read_builtin(entry, &key, errors);
                continue;
            }
            let Some(key) = entry.schema_key() else {
                continue;
            };
            let span = entry.span;
            if matches!(key.as_str(), "since_format" | "since_engine" | "unknown") {
                out.stored.get_or_insert(span);
            }
            let result: Result<()> = (|| {
                match key.as_str() {
                    "since_format" | "since_engine" => {
                        let version = Version::parse(&string_value(entry, &key)?)?;
                        let slot = if key == "since_format" {
                            &mut out.since_format
                        } else {
                            &mut out.since_engine
                        };
                        once(slot, version, &key, span, errors);
                    }
                    "unknown" => {
                        let word = unknown_word(entry)?;
                        once(&mut out.unknown, word, &key, span, errors);
                    }
                    "fallback" | "molang" | "molang_with" | "read_with" | "check_with"
                    | "wrong_type" | "missing" | "invalid" | "low_code" | "high_code"
                    | "key_with" | "skip_code" | "negative" | "keys" => {
                        let expr = value_expr(entry, &key)?;
                        let slot = match key.as_str() {
                            "fallback" => &mut out.fallback,
                            "molang" => &mut out.molang,
                            "molang_with" => &mut out.molang_with,
                            "wrong_type" => &mut out.wrong_type,
                            "missing" => &mut out.missing,
                            "invalid" => &mut out.invalid,
                            "negative" => &mut out.negative,
                            "low_code" => &mut out.low_code,
                            "high_code" => &mut out.high_code,
                            "key_with" => &mut out.key_with,
                            "skip_code" => &mut out.skip_code,
                            "check_with" => &mut out.check_with,
                            "keys" => &mut out.keys,
                            _ => &mut out.read_with,
                        };
                        once(slot, (expr, span), &key, span, errors);
                    }
                    "min_len" | "max_len" => {
                        let count = length(entry, &key)?;
                        let slot = if key == "min_len" {
                            &mut out.min_len
                        } else {
                            &mut out.max_len
                        };
                        once(slot, (count, span), &key, span, errors);
                    }
                    "multiple_of" => {
                        let step = step(entry)?;
                        once(&mut out.multiple_of, (step, span), &key, span, errors);
                    }
                    "foreign_event" => once(&mut out.foreign_event, span, &key, span, errors),
                    "lowercase" => once(&mut out.lowercase, span, &key, span, errors),
                    "required" => once(&mut out.required, span, &key, span, errors),
                    "kept" => once(&mut out.kept, span, &key, span, errors),
                    "finite" => once(&mut out.finite, span, &key, span, errors),
                    "iterated" => once(&mut out.iterated, span, &key, span, errors),
                    "reads_null" => once(&mut out.reads_null, span, &key, span, errors),
                    "elements" => {
                        let drop = either_word(entry, &key, "drop", "fail")?;
                        once(&mut out.elements, (drop, span), &key, span, errors);
                    }
                    "or_kept" => {
                        let told = either_word(entry, &key, "told", "silent")?;
                        once(&mut out.or_kept, (told, span), &key, span, errors);
                    }
                    other => {
                        if let Some(op) = Op::from_key(other) {
                            push_bound(&mut out, entry, op)?;
                        }
                    }
                }
                Ok(())
            })();
            if let Err(error) = result {
                errors.push(error);
            }
        }
        out
    }

    /// Reads one of facet's own keys the derive takes: `default`, `rename`, `alias`, `skip` and
    /// `flatten`.
    fn read_builtin(&mut self, entry: &Entry, key: &str, errors: &mut Vec<Error>) {
        let string = || match entry.expr() {
            Some(Ok(Expr::Lit(syn::ExprLit {
                lit: Lit::Str(lit), ..
            }))) => Some(lit.value()),
            _ => None,
        };
        match key {
            "default" => {
                let value = match entry.expr() {
                    Some(Ok(expr)) => Some(expr),
                    Some(Err(error)) => {
                        errors.push(error);
                        return;
                    }
                    None => None,
                };
                self.default = Some(DefaultDecl {
                    value,
                    span: entry.span,
                });
            }
            "rename" => self.rename = string().or(self.rename.take()),
            "alias" => self.alias = string().or(self.alias.take()),
            "skip" => self.skip = true,
            "flatten" => self.flatten = true,
            _ => {}
        }
    }

    /// Whether the field states any `schema::` key.
    pub(crate) fn any_schema_key(&self) -> Option<Span> {
        self.fallback
            .as_ref()
            .map(|(_, s)| *s)
            .or_else(|| self.bounds.first().map(|b| b.span))
            .or_else(|| self.multiple_of.as_ref().map(|(_, s)| *s))
            .or_else(|| self.molang.as_ref().map(|(_, s)| *s))
            .or_else(|| self.molang_with.as_ref().map(|(_, s)| *s))
            .or_else(|| self.read_with.as_ref().map(|(_, s)| *s))
            .or_else(|| {
                [
                    &self.wrong_type,
                    &self.missing,
                    &self.invalid,
                    &self.negative,
                ]
                .into_iter()
                .find_map(|code| code.as_ref().map(|(_, s)| *s))
            })
            .or(self.finite)
            .or(self.iterated)
            .or(self.reads_null)
            .or_else(|| self.elements.map(|(_, s)| s))
            .or_else(|| self.or_kept.map(|(_, s)| s))
            .or_else(|| self.check_with.as_ref().map(|(_, s)| *s))
            .or_else(|| self.low_code.as_ref().map(|(_, s)| *s))
            .or_else(|| self.high_code.as_ref().map(|(_, s)| *s))
            .or_else(|| self.key_with.as_ref().map(|(_, s)| *s))
            .or_else(|| self.keys.as_ref().map(|(_, s)| *s))
            .or_else(|| self.skip_code.as_ref().map(|(_, s)| *s))
            .or(self.kept)
            .or_else(|| self.min_len.as_ref().map(|(_, s)| *s))
            .or_else(|| self.max_len.as_ref().map(|(_, s)| *s))
            .or(self.foreign_event)
            .or(self.lowercase)
            .or(self.required)
            .or(self.stored)
    }
}

impl ContainerAttrs {
    /// Reads a struct's or enum's attributes.
    #[allow(clippy::too_many_lines, reason = "one arm per container key")]
    pub(crate) fn read(attrs: &[Attribute], place: Place, errors: &mut Vec<Error>) -> Self {
        let entries = entries(attrs);
        placement(&entries, place, errors);
        let mut out = ContainerAttrs::default();
        for entry in &entries {
            if let Some(key) = entry.builtin() {
                match key.as_str() {
                    "rename_all" => {
                        if let Some(Ok(Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(lit), ..
                        }))) = entry.expr()
                        {
                            match RenameRule::parse(&lit.value()) {
                                Some(rule) => out.rename_all = Some(rule),
                                // facet refuses it too, with its own list.
                                None => errors.push(Error::new(
                                    lit.span(),
                                    format!("unknown `rename_all` rule `{}`", lit.value()),
                                )),
                            }
                        }
                    }
                    "transparent" => out.transparent = true,
                    "default" => errors.push(Error::new(
                        entry.span,
                        "facet's container `default` fills a missing member from the field \
                         type's `Default`, not from the field's own: drop it and state each \
                         plain field's default with `default = ..` on the field",
                    )),
                    _ => {}
                }
                continue;
            }
            let result: Result<()> = match entry.schema_key().as_deref() {
                Some("unknown") => unknown_word(entry).map(|word| {
                    once(&mut out.unknown, word, "unknown", entry.span, errors);
                }),
                Some("rule") if place == Place::Struct => value_expr(entry, "rule").map(|expr| {
                    once(
                        &mut out.rule,
                        (expr, entry.span),
                        "rule",
                        entry.span,
                        errors,
                    );
                }),
                Some("trigger") if place == Place::Struct => {
                    out.trigger = true;
                    Ok(())
                }
                Some("molang_with") if place == Place::Struct => value_expr(entry, "molang_with")
                    .map(|expr| {
                        once(
                            &mut out.molang_with,
                            (expr, entry.span),
                            "molang_with",
                            entry.span,
                            errors,
                        );
                    }),
                Some("molang") if place == Place::Struct => {
                    value_expr(entry, "molang").map(|expr| {
                        once(
                            &mut out.molang,
                            (expr, entry.span),
                            "molang",
                            entry.span,
                            errors,
                        );
                    })
                }
                Some("invalid") if matches!(place, Place::Untagged | Place::Tagged) => {
                    value_expr(entry, "invalid").map(|expr| {
                        once(
                            &mut out.invalid,
                            (expr, entry.span),
                            "invalid",
                            entry.span,
                            errors,
                        );
                    })
                }
                Some("expected") if place == Place::Enum => {
                    string_value(entry, "expected").map(|lit| {
                        once(&mut out.expected, lit, "expected", entry.span, errors);
                    })
                }
                Some("case_insensitive") if place == Place::Enum => {
                    out.case_insensitive = true;
                    Ok(())
                }
                Some("string_only") if place == Place::Enum => {
                    out.string_only = true;
                    Ok(())
                }
                Some("order") if place == Place::Enum => {
                    string_value(entry, "order").and_then(|lit| {
                        if lit.value() == "game" {
                            out.order_game = true;
                            Ok(())
                        } else {
                            Err(Error::new(
                                lit.span(),
                                "`schema::order` takes \"game\": the enum states its own `ALL` \
                                 in the game's order",
                            ))
                        }
                    })
                }
                _ => Ok(()),
            };
            if let Err(error) = result {
                errors.push(error);
            }
        }
        out
    }
}

/// Whether the item's attributes say facet's `untagged`.
pub(crate) fn is_untagged(attrs: &[Attribute]) -> bool {
    entries(attrs)
        .iter()
        .any(|entry| entry.builtin().as_deref() == Some("untagged"))
}

/// What a variant of an untagged enum declares: facet's `rename_all` (its members' names), and
/// a checked `schema::unknown` (stored, the door's), which only a struct variant (`named`)
/// takes: the door reads the word on the object's arm alone. An error for any other
/// `schema::` key.
pub(crate) fn untagged_variant(
    attrs: &[Attribute],
    named: bool,
    errors: &mut Vec<Error>,
) -> Option<RenameRule> {
    let entries = entries(attrs);
    placement(&entries, Place::UntaggedVariant, errors);
    let mut rename_all = None;
    for entry in &entries {
        match (entry.builtin().as_deref(), entry.schema_key().as_deref()) {
            (Some("rename_all"), _) => {
                if let Some(Ok(Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(lit), ..
                }))) = entry.expr()
                {
                    rename_all = RenameRule::parse(&lit.value());
                }
            }
            (_, Some("unknown")) if !named => errors.push(Error::new(
                entry.span,
                "`schema::unknown` is about the members of an object: put it on a struct \
                 variant, whose members it is about",
            )),
            (_, Some("unknown")) => {
                if let Err(error) = unknown_word(entry) {
                    errors.push(error);
                }
            }
            _ => {}
        }
    }
    rename_all
}

/// facet's `rename` among `attrs`, read without a word about any other key.
pub(crate) fn rename_of(attrs: &[Attribute]) -> Option<String> {
    entries(attrs).iter().find_map(|entry| {
        (entry.builtin().as_deref() == Some("rename")).then(|| match entry.expr() {
            Some(Ok(Expr::Lit(syn::ExprLit {
                lit: Lit::Str(lit), ..
            }))) => Some(lit.value()),
            _ => None,
        })?
    })
}

/// The `tag = ".."` of an internally tagged enum.
pub(crate) fn tag_of(attrs: &[Attribute]) -> Option<String> {
    entries(attrs).iter().find_map(|entry| {
        (entry.builtin().as_deref() == Some("tag")).then(|| match entry.expr() {
            Some(Ok(Expr::Lit(syn::ExprLit {
                lit: Lit::Str(lit), ..
            }))) => Some(lit.value()),
            _ => None,
        })?
    })
}

/// Where an adjacently tagged enum (`content = ".."` beside `tag`) says so: no read takes it.
pub(crate) fn content_of(attrs: &[Attribute]) -> Option<Span> {
    entries(attrs)
        .iter()
        .find(|entry| entry.builtin().as_deref() == Some("content"))
        .map(|entry| entry.span)
}

/// What a variant of a tagged enum with data declares: facet's `rename` (its name) and
/// `rename_all` (its members' names), and a checked `schema::unknown`, which only a struct
/// variant (`named`) takes. An error for any other `schema::` key.
pub(crate) fn tagged_variant(
    attrs: &[Attribute],
    named: bool,
    errors: &mut Vec<Error>,
) -> (Option<String>, Option<RenameRule>, Option<String>) {
    let entries = entries(attrs);
    placement(&entries, Place::TaggedVariant, errors);
    let mut rename = None;
    let mut rename_all = None;
    let mut unknown = None;
    for entry in &entries {
        match (entry.builtin().as_deref(), entry.schema_key().as_deref()) {
            (Some("rename"), _) => {
                if let Some(Ok(Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(lit), ..
                }))) = entry.expr()
                {
                    rename = Some(lit.value());
                }
            }
            (Some("rename_all"), _) => {
                if let Some(Ok(Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(lit), ..
                }))) = entry.expr()
                {
                    rename_all = RenameRule::parse(&lit.value());
                }
            }
            (_, Some("unknown")) if !named => errors.push(Error::new(
                entry.span,
                "`schema::unknown` is about the members of an object: put it on a struct \
                 variant, whose members it is about",
            )),
            (_, Some("unknown")) => match unknown_word(entry) {
                Ok(word) => unknown = Some(word),
                Err(error) => errors.push(error),
            },
            _ => {}
        }
    }
    (rename, rename_all, unknown)
}

/// Whether a variant of an untagged enum says `schema::otherwise`.
pub(crate) fn variant_otherwise(attrs: &[Attribute]) -> bool {
    entries(attrs)
        .iter()
        .any(|entry| entry.schema_key().as_deref() == Some("otherwise"))
}

/// The checked `schema::unknown` of a variant of an untagged enum with data (a struct
/// variant's), read again for its object's read; errors are [`untagged_variant`]'s to tell.
pub(crate) fn variant_unknown(attrs: &[Attribute]) -> Option<String> {
    entries(attrs).iter().find_map(|entry| {
        (entry.schema_key().as_deref() == Some("unknown"))
            .then(|| unknown_word(entry).ok())
            .flatten()
    })
}

/// A variant's `rename`, and an error for any `schema::` key on it.
pub(crate) fn variant_rename(attrs: &[Attribute], errors: &mut Vec<Error>) -> Option<String> {
    let entries = entries(attrs);
    placement(&entries, Place::Variant, errors);
    entries.iter().find_map(|entry| {
        (entry.builtin().as_deref() == Some("rename")).then(|| match entry.expr() {
            Some(Ok(Expr::Lit(syn::ExprLit {
                lit: Lit::Str(lit), ..
            }))) => Some(lit.value()),
            _ => None,
        })?
    })
}

/// The tokens of `value` as one line, for the row table.
pub(crate) fn text(value: &impl ToTokens) -> String {
    value.to_token_stream().to_string()
}

/// The tokens of a type as Rust writes it (`Option<Vec<String>>`, `IndexMap<String, Inner>`,
/// `&'static str`): the printer's spaces dropped, except between two words and after a `,` or
/// a `;`.
pub(crate) fn type_text(ty: &impl ToTokens) -> String {
    let printed = text(ty);
    let word = |c: char| c.is_alphanumeric() || c == '_' || c == '\'';
    let mut out = String::with_capacity(printed.len());
    let mut chars = printed.chars().peekable();
    while let Some(c) = chars.next() {
        if c != ' ' {
            out.push(c);
            continue;
        }
        let before = out.chars().next_back();
        let after = chars.peek().copied();
        let keep = matches!(before, Some(',' | ';'))
            || (before.is_some_and(word) && after.is_some_and(word));
        if keep {
            out.push(' ');
        }
    }
    out
}
