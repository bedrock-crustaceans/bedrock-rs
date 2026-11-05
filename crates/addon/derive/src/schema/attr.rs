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
    pub foreign_event: Option<Span>,
    pub lowercase: Option<Span>,
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
    "foreign_event",
    "lowercase",
];
const ENUM_KEYS: &[&str] = &["expected", "case_insensitive", "order"];

/// Where the attributes stand, for the placement errors.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Place {
    Struct,
    Enum,
    /// An untagged enum with data.
    Untagged,
    Field,
    Variant,
    /// A variant of an untagged enum with data.
    UntaggedVariant,
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
        let Some(key) = entry.schema_key() else {
            continue;
        };
        let field_key = FIELD_KEYS.contains(&key.as_str());
        let enum_key = ENUM_KEYS.contains(&key.as_str());
        let message = match place {
            Place::Struct | Place::Enum | Place::Untagged if field_key => {
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
            Place::Variant if field_key || enum_key || key == "unknown" => {
                format!(
                    "a variant of a string enum takes no `schema::` key, `schema::{key}` included"
                )
            }
            Place::Untagged if enum_key => format!(
                "`schema::{key}` is a string enum's key: an untagged enum with data has no names \
                 to read"
            ),
            Place::Untagged if key == "unknown" => "`schema::unknown` is about the members of an \
                 object: put it on the struct variant whose members it is about"
                .to_owned(),
            Place::UntaggedVariant if field_key || enum_key => format!(
                "a variant of an untagged enum takes no `schema::{key}`: put it on the field it \
                 is about"
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
    if ["keep", "report", "reject", "ignore"].contains(&word.as_str()) {
        Ok(word)
    } else {
        Err(Error::new(
            lit.span(),
            format!(
                "`{word}` is not an unknown-member rule: expected `keep`, `report`, `reject` or \
                 `ignore`"
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

impl FieldAttrs {
    /// Reads a field's attributes; each malformed key adds an error and is left out.
    pub(crate) fn read(attrs: &[Attribute], errors: &mut Vec<Error>) -> Self {
        let entries = entries(attrs);
        placement(&entries, Place::Field, errors);
        let mut out = FieldAttrs::default();
        for entry in &entries {
            if let Some(key) = entry.builtin() {
                match key.as_str() {
                    "default" => {
                        let value = match entry.expr() {
                            Some(Ok(expr)) => Some(expr),
                            Some(Err(error)) => {
                                errors.push(error);
                                continue;
                            }
                            None => None,
                        };
                        out.default = Some(DefaultDecl {
                            value,
                            span: entry.span,
                        });
                    }
                    "rename" => {
                        if let Some(Ok(Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(lit), ..
                        }))) = entry.expr()
                        {
                            out.rename = Some(lit.value());
                        }
                    }
                    "skip" => out.skip = true,
                    "flatten" => out.flatten = true,
                    _ => {}
                }
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
                    "fallback" => {
                        let expr = value_expr(entry, &key)?;
                        once(&mut out.fallback, (expr, span), &key, span, errors);
                    }
                    "molang" => {
                        let expr = value_expr(entry, &key)?;
                        once(&mut out.molang, (expr, span), &key, span, errors);
                    }
                    "molang_with" => {
                        let expr = value_expr(entry, &key)?;
                        once(&mut out.molang_with, (expr, span), &key, span, errors);
                    }
                    "multiple_of" => {
                        let step = step(entry)?;
                        once(&mut out.multiple_of, (step, span), &key, span, errors);
                    }
                    "foreign_event" => once(&mut out.foreign_event, span, &key, span, errors),
                    "lowercase" => once(&mut out.lowercase, span, &key, span, errors),
                    other => {
                        if let Some(op) = Op::from_key(other) {
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

    /// Whether the field states any `schema::` key.
    pub(crate) fn any_schema_key(&self) -> Option<Span> {
        self.fallback
            .as_ref()
            .map(|(_, s)| *s)
            .or_else(|| self.bounds.first().map(|b| b.span))
            .or_else(|| self.multiple_of.as_ref().map(|(_, s)| *s))
            .or_else(|| self.molang.as_ref().map(|(_, s)| *s))
            .or_else(|| self.molang_with.as_ref().map(|(_, s)| *s))
            .or(self.foreign_event)
            .or(self.lowercase)
            .or(self.stored)
    }
}

impl ContainerAttrs {
    /// Reads a struct's or enum's attributes.
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
                Some("expected") if place == Place::Enum => {
                    string_value(entry, "expected").map(|lit| {
                        once(&mut out.expected, lit, "expected", entry.span, errors);
                    })
                }
                Some("case_insensitive") if place == Place::Enum => {
                    out.case_insensitive = true;
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
