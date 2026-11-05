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
    pub molang: Option<(Expr, Span)>,
    pub molang_with: Option<(Expr, Span)>,
    pub foreign_event: Option<Span>,
    pub lowercase: Option<Span>,
}

/// What a struct or enum declares.
#[derive(Default)]
pub(crate) struct ContainerAttrs {
    pub rename_all: Option<RenameRule>,
    pub transparent: bool,
    pub unknown: Option<String>,
    pub expected: Option<LitStr>,
    pub case_insensitive: bool,
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
    "molang",
    "molang_with",
    "foreign_event",
    "lowercase",
];
const ENUM_KEYS: &[&str] = &["expected", "case_insensitive"];

/// Where the attributes stand, for the placement errors.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Place {
    Struct,
    Enum,
    Field,
    Variant,
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
            Place::Struct | Place::Enum if field_key => {
                format!("`schema::{key}` is a field's key: put it on the field it is about")
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
            .or_else(|| self.molang.as_ref().map(|(_, s)| *s))
            .or_else(|| self.molang_with.as_ref().map(|(_, s)| *s))
            .or(self.foreign_event)
            .or(self.lowercase)
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
                _ => Ok(()),
            };
            if let Err(error) = result {
                errors.push(error);
            }
        }
        out
    }
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
