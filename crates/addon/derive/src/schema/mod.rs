//! What `#[derive(Schema)]` emits: for a struct, `impl Schema` (`FIELDS`, `check`,
//! `trigger_members`), `impl MolangFields`, `impl Default` when no field is required, and a
//! getter per `Option` field with a `schema::fallback`; for a unit-only enum, `ALL`, `NAMES`,
//! `EXPECTED`, `name`, `from_name`, `parse_name`, `Display`, an empty `impl Schema` and an
//! empty Molang walk; for an untagged enum with data, `impl Schema` and `impl MolangFields`
//! over the active variant and `VARIANT_FIELDS`.

mod attr;
mod ty;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    Data, DataEnum, DeriveInput, Error, Expr, Fields, Ident, Lit, Member, Result, Type, Visibility,
};

use self::attr::{BoundDecl, ContainerAttrs, FieldAttrs, LimitDecl, Op, Place};
use crate::attr::Version;

/// The derive's root path for the emitted code.
fn core() -> TokenStream {
    quote!(::bedrock_addon_core)
}

/// Expands the derive, or every error found.
pub(crate) fn derive(input: &DeriveInput) -> Result<TokenStream> {
    let mut errors = Vec::new();
    if !input.generics.params.is_empty() {
        errors.push(Error::new_spanned(
            &input.generics,
            "`#[derive(Schema)]` takes no generic parameters",
        ));
    }
    let out = match &input.data {
        Data::Struct(data) => derive_struct(input, &data.fields, &mut errors),
        Data::Enum(data) => derive_enum(input, data, &mut errors),
        Data::Union(data) => {
            return Err(Error::new(
                data.union_token.span,
                "`#[derive(Schema)]` applies to a struct or a string enum, not a union",
            ));
        }
    };
    match errors.into_iter().reduce(|mut all, next| {
        all.combine(next);
        all
    }) {
        Some(error) => Err(error),
        None => Ok(out),
    }
}

/// One field of the struct, read.
struct FieldInfo<'a> {
    /// `self.<member>`.
    member: Member,
    /// The Rust name.
    rust: String,
    /// The JSON key; empty for an inline field.
    json: String,
    /// A transparent newtype's field or a flattened one: it shares its owner's object, so the
    /// walks take no step into it.
    inline: bool,
    ty: &'a Type,
    /// `T` of an `Option<T>` field.
    option: Option<&'a Type>,
    vis: &'a Visibility,
    ident: Option<&'a Ident>,
    attrs: FieldAttrs,
    /// The place expression of the field's value: `self.x` in a struct, `(*__f0)` for a
    /// variant's bound field.
    access: TokenStream,
}

impl FieldInfo<'_> {
    /// The type a bound compares: `T` of an `Option<T>`, or the field's.
    fn value_ty(&self) -> &Type {
        self.option.unwrap_or(self.ty)
    }

    fn required(&self) -> bool {
        self.option.is_none() && self.attrs.default.is_none()
    }
}

/// `expr` as a value of `ty`: a string literal through `From` (`String` from `&str`), any
/// other expression as written, so a literal of another type is the compiler's error at it.
fn typed(expr: &Expr, ty: &Type) -> TokenStream {
    if let Expr::Lit(syn::ExprLit {
        lit: Lit::Str(_), ..
    }) = expr
    {
        quote!({ let __value: #ty = ::core::convert::From::from(#expr); __value })
    } else {
        quote!({ let __value: #ty = #expr; __value })
    }
}

/// The `Lit` of the row table for an attribute's value.
fn row_lit(expr: &Expr) -> TokenStream {
    let core = core();
    let negated = |lit: &Lit, neg: bool| -> Option<TokenStream> {
        match lit {
            Lit::Int(int) => {
                let value: i64 = int.base10_parse().ok()?;
                let value = if neg { -value } else { value };
                Some(quote!(#core::schema::Lit::Int(#value)))
            }
            Lit::Float(float) => {
                let value: f64 = float.base10_parse().ok()?;
                let value = if neg { -value } else { value };
                Some(quote!(#core::schema::Lit::Float(#value)))
            }
            Lit::Bool(b) if !neg => {
                let value = b.value;
                Some(quote!(#core::schema::Lit::Bool(#value)))
            }
            Lit::Str(s) if !neg => Some(quote!(#core::schema::Lit::Str(#s))),
            _ => None,
        }
    };
    let found = match expr {
        Expr::Lit(lit) => negated(&lit.lit, false),
        Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => match &*unary.expr {
            Expr::Lit(lit) => negated(&lit.lit, true),
            _ => None,
        },
        _ => None,
    };
    found.unwrap_or_else(|| {
        let text = attr::text(expr);
        quote!(#core::schema::Lit::Expr(#text))
    })
}

/// `Option<FormatVersion>` tokens.
fn version_option(version: Option<Version>) -> TokenStream {
    let core = core();
    let Some(Version {
        major,
        minor,
        patch,
    }) = version
    else {
        return quote!(::core::option::Option::None);
    };
    quote!(::core::option::Option::Some(#core::version::FormatVersion::new(#major, #minor, #patch)))
}

/// `Option<UnknownRule>` tokens.
fn unknown_option(word: Option<&String>) -> TokenStream {
    let core = core();
    let Some(word) = word else {
        return quote!(::core::option::Option::None);
    };
    let variant = format_ident!(
        "{}",
        match word.as_str() {
            "keep" => "Keep",
            "report" => "Report",
            "reject" => "Reject",
            _ => "Ignore",
        }
    );
    quote!(::core::option::Option::Some(#core::schema::UnknownRule::#variant))
}

/// The JSON key of a field or variant named `rust`: facet's `rename`, else `rename_all`.
fn json_name(rust: &str, rename: Option<&String>, container: &ContainerAttrs) -> String {
    let rust = rust.strip_prefix("r#").unwrap_or(rust);
    match (rename, container.rename_all) {
        (Some(rename), _) => rename.clone(),
        (None, Some(rule)) => rule.apply(rust),
        (None, None) => rust.to_owned(),
    }
}

/// The fields of a struct or a variant, `skip` ones left out (with an error for a `schema::`
/// key on one): each named by `container`'s rules, inline where `inline_all` (a transparent
/// newtype, a newtype variant) or flattened, its value at `access(member, index)`.
fn field_infos<'a>(
    fields: &'a Fields,
    container: &ContainerAttrs,
    inline_all: bool,
    access: impl Fn(&Member, usize) -> TokenStream,
    errors: &mut Vec<Error>,
) -> Vec<FieldInfo<'a>> {
    let mut infos = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let attrs = FieldAttrs::read(&field.attrs, errors);
        let member = match &field.ident {
            Some(ident) => Member::Named(ident.clone()),
            None => Member::Unnamed(index.into()),
        };
        if attrs.skip {
            if let Some(span) = attrs.any_schema_key() {
                errors.push(Error::new(
                    span,
                    "a `skip` field is not read or written: it takes no `schema::` key",
                ));
            }
            continue;
        }
        let rust = match &field.ident {
            Some(ident) => ident.to_string(),
            None => index.to_string(),
        };
        let inline = inline_all || attrs.flatten;
        let json = if inline {
            String::new()
        } else {
            json_name(&rust, attrs.rename.as_ref(), container)
        };
        let access = access(&member, index);
        infos.push(FieldInfo {
            member,
            rust,
            json,
            inline,
            ty: &field.ty,
            option: ty::option_inner(&field.ty),
            vis: &field.vis,
            ident: field.ident.as_ref(),
            attrs,
            access,
        });
    }
    infos
}

fn derive_struct(input: &DeriveInput, fields: &Fields, errors: &mut Vec<Error>) -> TokenStream {
    let container = ContainerAttrs::read(&input.attrs, Place::Struct, errors);
    if matches!(fields, Fields::Unnamed(_)) && !container.transparent {
        errors.push(Error::new_spanned(
            fields,
            "`#[derive(Schema)]` applies to a struct with named fields or a \
             `#[facet(transparent)]` newtype",
        ));
    }
    let infos = field_infos(
        fields,
        &container,
        container.transparent,
        |member, _| quote!(self.#member),
        errors,
    );
    for info in &infos {
        validate_field(info, &infos, errors);
    }

    let core = core();
    let ident = &input.ident;
    let rows = infos.iter().map(|info| field_row(info));
    let checks = infos.iter().map(|info| field_check(info, &infos)).chain(
        infos
            .iter()
            .filter_map(|info| field_cross_check(info, &infos)),
    );
    let molang = infos.iter().map(field_molang);
    let triggers = trigger_calls(&infos);
    let unknown = container.unknown.as_ref().map(|word| {
        let rule = unknown_option(Some(word));
        quote!(const UNKNOWN: ::core::option::Option<#core::schema::UnknownRule> = #rule;)
    });
    let default =
        (!infos.iter().any(FieldInfo::required)).then(|| default_impl(ident, fields, &infos));
    let getters = getters(ident, &infos);

    quote! {
        #[automatically_derived]
        impl #core::schema::Schema for #ident {
            const FIELDS: &'static [#core::schema::FieldRow] = &[#(#rows),*];
            #unknown

            #[allow(unused_variables)]
            fn check(&mut self, cx: &mut #core::schema::SchemaCx<'_>) {
                #(#checks)*
            }

            #[allow(unused_variables)]
            fn trigger_members(prefix: &str, out: &mut ::std::vec::Vec<::std::string::String>) {
                #(#triggers)*
            }
        }

        #[automatically_derived]
        impl #core::molang::MolangFields for #ident {
            #[allow(unused_variables)]
            fn visit_molang(&mut self, visitor: &mut #core::molang::MolangVisitor<'_>) {
                #(#molang)*
            }
        }

        #default
        #getters
    }
}

/// `trigger_members`' calls: one per field that is no `foreign_event`, below the field's key
/// (or at the owner's for an inline field).
fn trigger_calls(infos: &[FieldInfo]) -> Vec<TokenStream> {
    let core = core();
    infos
        .iter()
        .filter(|info| info.attrs.foreign_event.is_none())
        .map(|info| {
            let ty = info.ty;
            if info.inline {
                quote!(<#ty as #core::schema::SchemaFields>::trigger_members(prefix, out);)
            } else {
                let json = &info.json;
                quote!(<#ty as #core::schema::SchemaFields>::trigger_members(&#core::schema::member(prefix, #json), out);)
            }
        })
        .collect()
}

/// The forced declarations of one field.
fn validate_field(info: &FieldInfo, all: &[FieldInfo], errors: &mut Vec<Error>) {
    let attrs = &info.attrs;
    if info.option.is_some()
        && let Some(default) = &attrs.default
    {
        let message = if default.value.is_some() {
            "an `Option` field keeps absence: facet's `default = ..` would fill `Some(..)` on read \
             and write it back; drop it, and state what absence stands for with \
             `schema::fallback = ..`"
        } else {
            "an `Option` field is `None` when absent without facet's `default`: drop it \
             (`schema::fallback = ..` states what absence stands for)"
        };
        errors.push(Error::new(default.span, message));
    }
    if info.option.is_none()
        && let Some((_, span)) = &attrs.fallback
    {
        errors.push(Error::new(
            *span,
            "`schema::fallback` is what an absent `Option` stands for: a plain field states its \
             default with facet's `default = ..`",
        ));
    }
    if let Some(span) = attrs.lowercase
        && !ty::is_string(info.ty)
    {
        errors.push(Error::new(
            span,
            "`schema::lowercase` applies to a `String` field or an `Option<String>`",
        ));
    }
    if let Some(span) = attrs.foreign_event
        && !ty::is_event_trigger(info.ty)
    {
        errors.push(Error::new(
            span,
            "`schema::foreign_event` marks a field holding an `EventTrigger` (or an `Option`, \
             `Vec` or map of them)",
        ));
    }
    match (&attrs.molang, &attrs.molang_with) {
        (Some(_), Some((_, span))) => errors.push(Error::new(
            *span,
            "`schema::molang` and `schema::molang_with` both say how the field is walked: keep one",
        )),
        (Some((_, span)), None) if ty::is_plain_value(info.ty) => errors.push(Error::new(
            *span,
            "`schema::molang` names the spec of a Molang field, or the default spec of a nested \
             walk, and this field holds a plain value: a hand walk is `schema::molang_with`",
        )),
        _ => {}
    }
    for bound in &attrs.bounds {
        if let LimitDecl::Field(other) = &bound.limit {
            if *other == info.rust {
                errors.push(Error::new(
                    other.span(),
                    format!(
                        "`schema::{}` names the field itself: a bound compares with a number or \
                         another field",
                        bound.op.key()
                    ),
                ));
            } else if !all.iter().any(|f| *other == f.rust) {
                errors.push(Error::new(
                    other.span(),
                    format!(
                        "no field `{other}` in this struct for `schema::{}` to compare with",
                        bound.op.key()
                    ),
                ));
            }
        }
    }
}

/// One `FieldRow` literal.
fn field_row(info: &FieldInfo) -> TokenStream {
    let core = core();
    let FieldInfo {
        rust,
        json,
        ty,
        attrs,
        ..
    } = info;
    let ty_text = attr::type_text(ty);
    let kind = format_ident!(
        "{}",
        if info.option.is_some() {
            "Optional"
        } else if attrs.default.is_some() {
            "Filled"
        } else {
            "Required"
        }
    );
    let mut parts = Vec::new();
    if let Some(default) = &attrs.default {
        let lit = default
            .value
            .as_ref()
            .map_or_else(|| quote!(#core::schema::Lit::TypeDefault), row_lit);
        parts.push(quote!(default: ::core::option::Option::Some(#lit)));
    }
    if let Some((fallback, _)) = &attrs.fallback {
        let lit = row_lit(fallback);
        parts.push(quote!(fallback: ::core::option::Option::Some(#lit)));
    }
    if !attrs.bounds.is_empty() {
        let bounds = attrs.bounds.iter().map(|BoundDecl { op, limit, .. }| {
            let op = op.variant();
            let limit = match limit {
                LimitDecl::Num(expr) => {
                    let lit = row_lit(expr);
                    // A number limit's row value is its `f64`.
                    quote!(#core::schema::Limit::Num(match #lit {
                        #core::schema::Lit::Int(value) => value as f64,
                        #core::schema::Lit::Float(value) => value,
                        _ => f64::NAN,
                    }))
                }
                LimitDecl::Field(other) => {
                    let other = other.to_string();
                    quote!(#core::schema::Limit::Field(#other))
                }
            };
            quote!(#core::schema::Bound { op: #core::schema::BoundOp::#op, limit: #limit })
        });
        parts.push(quote!(bounds: &[#(#bounds),*]));
    }
    if let Some((step, _)) = attrs.multiple_of {
        parts.push(quote!(multiple_of: ::core::option::Option::Some(#step)));
    }
    if attrs.since_format.is_some() {
        let v = version_option(attrs.since_format);
        parts.push(quote!(since_format: #v));
    }
    if attrs.since_engine.is_some() {
        let v = version_option(attrs.since_engine);
        parts.push(quote!(since_engine: #v));
    }
    if attrs.unknown.is_some() {
        let v = unknown_option(attrs.unknown.as_ref());
        parts.push(quote!(unknown: #v));
    }
    if attrs.lowercase.is_some() {
        parts.push(quote!(lowercase: true));
    }
    if attrs.foreign_event.is_some() {
        parts.push(quote!(foreign_event: true));
    }
    if attrs.flatten {
        parts.push(quote!(flatten: true));
    }
    quote! {
        #core::schema::FieldRow {
            #(#parts,)*
            ..#core::schema::FieldRow::plain(#rust, #json, #ty_text, #core::schema::FieldKind::#kind)
        }
    }
}

/// The value a failed member takes from its own declaration, `None` when it keeps its value
/// (a required field): an `Option` becomes absent (its fallback is what absence stands for and
/// is never written into the data), a plain field takes its facet `default`.
fn own_reset(info: &FieldInfo) -> Option<TokenStream> {
    if info.option.is_some() {
        return Some(quote!(::core::option::Option::None));
    }
    let default = info.attrs.default.as_ref()?;
    Some(default.value.as_ref().map_or_else(
        || quote!(::core::default::Default::default()),
        |expr| typed(expr, info.value_ty()),
    ))
}

/// The bound on the lower side (`min`, `above`) or the upper one (`max`, `below`).
fn side_bound<'a>(info: &'a FieldInfo<'_>, low: bool) -> Option<&'a BoundDecl> {
    info.attrs
        .bounds
        .iter()
        .find(|b| matches!(b.op, Op::Min | Op::Above) == low)
}

/// Whether a bound of the field names another field: its check runs after every field's own.
fn is_cross(info: &FieldInfo) -> bool {
    info.attrs
        .bounds
        .iter()
        .any(|b| matches!(b.limit, LimitDecl::Field(_)))
}

/// The limit of a bound as an `Option<T>`, typed by the field: a number, or the sibling's value
/// (what it stands for where it is an absent `Option` with a fallback; no side where it is
/// absent without one).
fn limit_expr(bound: Option<&BoundDecl>, value_ty: &Type, all: &[FieldInfo]) -> TokenStream {
    let Some(bound) = bound else {
        return quote!({ let __limit: ::core::option::Option<#value_ty> = ::core::option::Option::None; __limit });
    };
    let limit = match &bound.limit {
        LimitDecl::Num(expr) => quote!(::core::option::Option::Some(#expr)),
        LimitDecl::Field(other) => match all.iter().find(|f| *other == f.rust) {
            None => quote!(::core::option::Option::None),
            Some(sibling) => {
                let access = &sibling.access;
                match (&sibling.option, &sibling.attrs.fallback) {
                    (None, _) => quote!(::core::option::Option::Some(#access)),
                    (Some(inner), Some((fallback, _))) => {
                        let fallback = typed(fallback, inner);
                        quote!(::core::option::Option::Some(::core::option::Option::unwrap_or(#access, #fallback)))
                    }
                    (Some(_), None) => quote!(#access),
                }
            }
        },
    };
    // The limit typed by the field: a literal of another type is the compiler's error at it.
    let span = bound.span;
    quote_spanned!(span=> { let __limit: ::core::option::Option<#value_ty> = #limit; __limit })
}

/// What the member takes when `side` failed under `DefaultAndTell`: the other member where the
/// side names one, else its own reset; nothing for a required field.
fn side_reset(info: &FieldInfo, bound: Option<&BoundDecl>, limit: &TokenStream) -> TokenStream {
    let access = &info.access;
    if bound.is_some_and(|b| matches!(b.limit, LimitDecl::Field(_))) {
        let value = if info.option.is_some() {
            quote!(::core::option::Option::Some(__other))
        } else {
            quote!(__other)
        };
        return quote! {
            if let ::core::option::Option::Some(__other) = #limit {
                #access = #value;
            }
        };
    }
    match own_reset(info) {
        Some(value) => quote!(#access = #value;),
        None => TokenStream::new(),
    }
}

/// A field's bounds and its step: one range, one finding, whichever side the value broke, and
/// the step (`schema::multiple_of`) checked only for a value inside the range; empty where the
/// field states neither.
fn field_bounds(info: &FieldInfo, all: &[FieldInfo]) -> TokenStream {
    let range = field_range(info, all);
    let Some(multiple) = field_multiple(info) else {
        return range.map_or_else(TokenStream::new, |range| quote!(let _ = #range;));
    };
    match range {
        Some(range) => quote! {
            if !#range {
                #multiple
            }
        },
        None => multiple,
    }
}

/// A field's step (`schema::multiple_of`): a value that is no multiple of it is one finding,
/// graded and reset as a broken bound is.
fn field_multiple(info: &FieldInfo) -> Option<TokenStream> {
    let (step, _) = info.attrs.multiple_of?;
    let core = core();
    let access = &info.access;
    let value = if info.option.is_some() {
        quote!(#access)
    } else {
        quote!(::core::option::Option::Some(#access))
    };
    let reset = own_reset(info);
    let resettable = reset.is_some();
    let apply = reset.map_or_else(
        || quote!(let _ = __verdict;),
        |reset| {
            quote! {
                if __verdict == #core::schema::Verdict::Reset {
                    #access = #reset;
                }
            }
        },
    );
    Some(quote! {
        if let ::core::option::Option::Some(__value) = #value {
            let __value = #core::schema::Number::to_f64(__value);
            if __value % #step != 0.0 {
                let __verdict = cx.multiple(__value, #step, #resettable);
                #apply
            }
        }
    })
}

/// A field's range, as an expression that is whether the value broke it: one finding,
/// whichever side the value broke; `None` where the field states no bound.
fn field_range(info: &FieldInfo, all: &[FieldInfo]) -> Option<TokenStream> {
    if info.attrs.bounds.is_empty() {
        return None;
    }
    let core = core();
    let access = &info.access;
    let value_ty = info.value_ty();
    let (low, high) = (side_bound(info, true), side_bound(info, false));
    let exclusive =
        |bound: Option<&BoundDecl>| bound.is_some_and(|b| matches!(b.op, Op::Above | Op::Below));
    let (low_exclusive, high_exclusive) = (exclusive(low), exclusive(high));
    let (low_limit, high_limit) = (
        limit_expr(low, value_ty, all),
        limit_expr(high, value_ty, all),
    );
    let low_reset = side_reset(info, low, &quote!(__sides.low));
    let high_reset = side_reset(info, high, &quote!(__sides.high));
    // A side resets where it names another member or the field has its own reset.
    let resettable = |bound: Option<&BoundDecl>| {
        bound.is_some_and(|b| matches!(b.limit, LimitDecl::Field(_))) || own_reset(info).is_some()
    };
    let (low_resettable, high_resettable) = (resettable(low), resettable(high));
    let value = if info.option.is_some() {
        quote!(#access)
    } else {
        quote!(::core::option::Option::Some(#access))
    };
    let apply = if low_reset.is_empty() && high_reset.is_empty() {
        quote!(let _ = (__verdict, __side);)
    } else {
        quote! {
            if __verdict == #core::schema::Verdict::Reset {
                match __side {
                    #core::schema::Side::Low => { #low_reset }
                    #core::schema::Side::High => { #high_reset }
                }
            }
        }
    };
    Some(quote! {{
        let __sides = #core::schema::Sides::<#value_ty> {
            low: #low_limit,
            low_exclusive: #low_exclusive,
            high: #high_limit,
            high_exclusive: #high_exclusive,
        };
        let mut __broken = false;
        if let ::core::option::Option::Some(__value) = #value {
            if let ::core::option::Option::Some(__side) = __sides.broken(__value) {
                __broken = true;
                let __resettable = match __side {
                    #core::schema::Side::Low => #low_resettable,
                    #core::schema::Side::High => #high_resettable,
                };
                let __verdict = cx.range(__sides.range(), __resettable);
                #apply
            }
        }
        __broken
    }})
}

/// `body` at the field's path: inside `enter`/`leave`, or at the owner's for an inline field.
fn at_field(info: &FieldInfo, body: TokenStream) -> TokenStream {
    if info.inline {
        body
    } else {
        let json = &info.json;
        quote! {
            cx.enter(#json);
            #body
            cx.leave();
        }
    }
}

/// A field's own part of `check`: its bounds (unless one names another field), `lowercase`
/// (the ASCII lower-casing the game applies in silence), then the walk into it.
fn field_check(info: &FieldInfo, all: &[FieldInfo]) -> TokenStream {
    let core = core();
    let access = &info.access;
    let bounds = if is_cross(info) {
        TokenStream::new()
    } else {
        field_bounds(info, all)
    };
    // The game lower-cases the member as it reads it and tells nothing.
    let lowercase = info.attrs.lowercase.map(|_| {
        if info.option.is_some() {
            quote! {
                if let ::core::option::Option::Some(__value) = &mut #access {
                    __value.make_ascii_lowercase();
                }
            }
        } else {
            quote!(#access.make_ascii_lowercase();)
        }
    });
    let walk = quote!(#core::schema::SchemaFields::check_fields(&mut #access, cx););
    at_field(info, quote!(#bounds #lowercase #walk))
}

/// A field's cross-field bounds, which run after every field's own check, so they compare with
/// the other member as that check left it.
fn field_cross_check(info: &FieldInfo, all: &[FieldInfo]) -> Option<TokenStream> {
    is_cross(info).then(|| at_field(info, field_bounds(info, all)))
}

/// A field's part of the Molang walk.
fn field_molang(info: &FieldInfo) -> TokenStream {
    let core = core();
    let access = &info.access;
    let json = &info.json;
    if let Some((walk, _)) = &info.attrs.molang_with {
        return if info.inline {
            quote!((#walk)(&mut #access, visitor);)
        } else {
            quote! {
                visitor.enter(#core::molang::Step::Key(#json));
                (#walk)(&mut #access, visitor);
                visitor.leave();
            }
        };
    }
    if ty::is_molang_slot(info.ty) {
        let spec = info.attrs.molang.as_ref().map(|(spec, _)| spec);
        return match (info.inline, spec) {
            (false, Some(spec)) => quote!(visitor.field(#json, &#spec, &mut #access);),
            (false, None) => quote!(visitor.field_default(#json, &mut #access);),
            (true, Some(spec)) => {
                quote!(#core::molang::MolangSlot::visit_slot(&mut #access, &#spec, visitor);)
            }
            (true, None) => quote! {
                let __spec = *visitor.default_spec();
                #core::molang::MolangSlot::visit_slot(&mut #access, &__spec, visitor);
            },
        };
    }
    // A spec on a nested walk is that walk's default.
    match (
        info.inline,
        info.attrs.molang.as_ref().map(|(spec, _)| spec),
    ) {
        (true, None) => quote!(#core::molang::MolangFields::visit_molang(&mut #access, visitor);),
        (false, None) => quote!(visitor.nested(#json, &mut #access);),
        (true, Some(spec)) => quote! {
            visitor.under(&#spec, |visitor| {
                #core::molang::MolangFields::visit_molang(&mut #access, visitor);
            });
        },
        (false, Some(spec)) => quote!(visitor.nested_under(#json, &#spec, &mut #access);),
    }
}

/// `impl Default`: an `Option` field `None`, a plain field its facet `default`, a `skip`
/// field its type's `Default`.
fn default_impl(ident: &Ident, fields: &Fields, infos: &[FieldInfo]) -> TokenStream {
    let value = |member: &Member| -> TokenStream {
        let info = infos.iter().find(|info| info.member == *member);
        match info {
            Some(info) if info.option.is_some() => quote!(::core::option::Option::None),
            Some(FieldInfo {
                ty,
                attrs:
                    FieldAttrs {
                        default:
                            Some(attr::DefaultDecl {
                                value: Some(expr), ..
                            }),
                        ..
                    },
                ..
            }) => typed(expr, ty),
            _ => quote!(::core::default::Default::default()),
        }
    };
    let members = fields
        .iter()
        .enumerate()
        .map(|(index, field)| match &field.ident {
            Some(name) => Member::Named(name.clone()),
            None => Member::Unnamed(index.into()),
        });
    let body = match fields {
        Fields::Named(_) => {
            let inits = members.map(|member| {
                let value = value(&member);
                quote!(#member: #value)
            });
            quote!(Self { #(#inits),* })
        }
        Fields::Unnamed(_) => {
            let inits = members.map(|member| value(&member));
            quote!(Self(#(#inits),*))
        }
        Fields::Unit => quote!(Self),
    };
    quote! {
        #[automatically_derived]
        impl ::core::default::Default for #ident {
            fn default() -> Self {
                #body
            }
        }
    }
}

/// A getter per named `Option` field with a `schema::fallback`.
fn getters(ident: &Ident, infos: &[FieldInfo]) -> TokenStream {
    let getters: Vec<TokenStream> = infos
        .iter()
        .filter_map(|info| {
            let (fallback, _) = info.attrs.fallback.as_ref()?;
            let name = info.ident?;
            let inner = info.option?;
            let vis = info.vis;
            let value = typed(fallback, inner);
            // A named fallback is often a private const of the family, which the doc of a
            // public getter cannot name: it says what the const is instead.
            let doc = if matches!(fallback, syn::Expr::Path(_)) {
                format!("`{name}`, or the family's default where the document has none.")
            } else {
                format!(
                    "`{name}`, or `{}` where the document has none.",
                    attr::text(fallback)
                )
            };
            Some(quote! {
                #[doc = #doc]
                #[must_use]
                #[allow(clippy::clone_on_copy)]
                #vis fn #name(&self) -> #inner {
                    match &self.#name {
                        ::core::option::Option::Some(value) => ::core::clone::Clone::clone(value),
                        ::core::option::Option::None => #value,
                    }
                }
            })
        })
        .collect();
    if getters.is_empty() {
        return TokenStream::new();
    }
    quote! {
        #[automatically_derived]
        impl #ident {
            #(#getters)*
        }
    }
}

fn derive_enum(input: &DeriveInput, data: &DataEnum, errors: &mut Vec<Error>) -> TokenStream {
    let has_data = data
        .variants
        .iter()
        .any(|variant| !matches!(variant.fields, Fields::Unit));
    if has_data && attr::is_untagged(&input.attrs) {
        return derive_untagged(input, data, errors);
    }
    let container = ContainerAttrs::read(&input.attrs, Place::Enum, errors);
    let mut variants = Vec::new();
    for variant in &data.variants {
        let rename = attr::variant_rename(&variant.attrs, errors);
        if !matches!(variant.fields, Fields::Unit) {
            errors.push(Error::new_spanned(
                &variant.fields,
                "`#[derive(Schema)]` on an enum applies to a string enum, whose variants are all \
                 units, or to an untagged one (`#[facet(untagged)]`): a tagged variant with data \
                 has no one name",
            ));
            continue;
        }
        let name = json_name(&variant.ident.to_string(), rename.as_ref(), &container);
        variants.push((&variant.ident, name));
    }
    let core = core();
    let ident = &input.ident;
    // An enum in the game's order states its own `ALL`.
    let all = (!container.order_game).then(|| {
        let all = variants.iter().map(|(v, _)| quote!(Self::#v));
        quote! {
            /// Every value, in declaration order.
            pub const ALL: &'static [Self] = &[#(#all),*];
        }
    });
    let all_check = container
        .order_game
        .then(|| game_order_check(ident, &variants));
    let names: Vec<&String> = variants.iter().map(|(_, n)| n).collect();
    let expected = container.expected.as_ref().map_or_else(
        || {
            let list = names
                .iter()
                .map(|n| n.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("one of: {list}")
        },
        syn::LitStr::value,
    );
    let name_arms = variants.iter().map(|(v, n)| quote!(Self::#v => #n,));
    let from_name = from_name_body(&variants, container.case_insensitive);
    let case = if container.case_insensitive {
        ", in any case"
    } else {
        ""
    };
    let from_doc = format!("The value spelt `name`{case}.");
    quote! {
        #[automatically_derived]
        impl #ident {
            #all
            /// The JSON names, in declaration order (an enum in the game's order states its
            /// own `ALL` in that order; `NAMES` and `EXPECTED` keep the declaration's).
            pub const NAMES: &'static [&'static str] = &[#(#names),*];
            /// What the enum takes, in words: the error of a name it does not know.
            pub const EXPECTED: &'static str = #expected;

            /// The JSON name.
            pub const fn name(&self) -> &'static str {
                match self {
                    #(#name_arms)*
                }
            }

            #[doc = #from_doc]
            pub fn from_name(name: &str) -> ::core::option::Option<Self> {
                #from_name
            }

            /// The value spelt `name`, or the error naming what the enum takes.
            pub fn parse_name(name: &str) -> ::core::result::Result<Self, #core::schema::UnknownName> {
                Self::from_name(name).ok_or_else(|| #core::schema::UnknownName {
                    expected: Self::EXPECTED,
                    found: ::std::borrow::ToOwned::to_owned(name),
                })
            }
        }

        #all_check

        #[automatically_derived]
        impl ::core::fmt::Display for #ident {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.name())
            }
        }

        #[automatically_derived]
        impl #core::schema::Schema for #ident {
            const FIELDS: &'static [#core::schema::FieldRow] = &[];

            fn check(&mut self, _: &mut #core::schema::SchemaCx<'_>) {}

            fn trigger_members(_: &str, _: &mut ::std::vec::Vec<::std::string::String>) {}
        }

        #[automatically_derived]
        impl #core::molang::MolangFields for #ident {
            fn visit_molang(&mut self, _: &mut #core::molang::MolangVisitor<'_>) {}
        }
    }
}

/// The body of a string enum's `from_name`: the variant spelt `name`, in any case where the
/// enum says `case_insensitive`.
fn from_name_body(variants: &[(&Ident, String)], case_insensitive: bool) -> TokenStream {
    if case_insensitive {
        let tests = variants.iter().map(|(v, n)| {
            quote!(if name.eq_ignore_ascii_case(#n) { return ::core::option::Option::Some(Self::#v); })
        });
        quote!(#(#tests)* ::core::option::Option::None)
    } else {
        let arms = variants
            .iter()
            .map(|(v, n)| quote!(#n => ::core::option::Option::Some(Self::#v),));
        quote!(match name { #(#arms)* _ => ::core::option::Option::None })
    }
}

/// The compile-time check of an enum's own `ALL` (`schema::order = "game"`): a list of `Self`
/// naming every variant once.
fn game_order_check(ident: &Ident, variants: &[(&Ident, String)]) -> TokenStream {
    let count = variants.len();
    let indices = variants
        .iter()
        .enumerate()
        .map(|(index, (v, _))| quote!(#ident::#v => #index,));
    quote! {
        #[automatically_derived]
        const _: () = {
            let all: &[#ident] = &#ident::ALL;
            let mut seen = [false; #count];
            let mut at = 0;
            while at < all.len() {
                let index = match &all[at] {
                    #(#indices)*
                };
                assert!(
                    !seen[index],
                    "`schema::order = \"game\"`: the enum's `ALL` names a variant twice"
                );
                seen[index] = true;
                at += 1;
            }
            assert!(
                all.len() == #count,
                "`schema::order = \"game\"`: the enum's `ALL` leaves out a variant"
            );
        };
    }
}

/// One variant of an untagged enum, read.
struct VariantInfo<'a> {
    ident: &'a Ident,
    /// `Self::V { a: __f0, .. }`, `Self::V(__f0)` or `Self::V`.
    pattern: TokenStream,
    fields: Vec<FieldInfo<'a>>,
}

/// The binding of a variant's field `index` in its arm.
fn binding(index: usize) -> Ident {
    format_ident!("__f{index}")
}

/// An untagged enum with data: `impl Schema` (no `FIELDS` of its own; `check` and the Molang
/// walk of the active variant's fields at the owner's path, every variant's trigger members)
/// and `VARIANT_FIELDS`, the rows per variant. A newtype variant's value is the enum's (no
/// step); a struct variant's members are the object's. No `Default`: no variant is the
/// default one.
fn derive_untagged(input: &DeriveInput, data: &DataEnum, errors: &mut Vec<Error>) -> TokenStream {
    let _ = ContainerAttrs::read(&input.attrs, Place::Untagged, errors);
    let variants = untagged_variants(data, errors);
    untagged_impls(&input.ident, &variants)
}

/// The variants of an untagged enum, each with its arm's pattern and its fields bound in it.
fn untagged_variants<'a>(data: &'a DataEnum, errors: &mut Vec<Error>) -> Vec<VariantInfo<'a>> {
    let mut variants: Vec<VariantInfo> = Vec::new();
    for variant in &data.variants {
        let named = matches!(variant.fields, Fields::Named(_));
        let rename_all = attr::untagged_variant(&variant.attrs, named, errors);
        let naming = ContainerAttrs {
            rename_all,
            ..ContainerAttrs::default()
        };
        let v = &variant.ident;
        let access = |_: &Member, index: usize| {
            let binding = binding(index);
            quote!((*#binding))
        };
        let (pattern, fields) = match &variant.fields {
            Fields::Unit => (quote!(Self::#v), Vec::new()),
            Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
                let fields = field_infos(&variant.fields, &naming, true, access, errors);
                let pattern = if fields.is_empty() {
                    quote!(Self::#v(..))
                } else {
                    quote!(Self::#v(__f0))
                };
                (pattern, fields)
            }
            Fields::Unnamed(unnamed) => {
                errors.push(Error::new_spanned(
                    unnamed,
                    "a variant of an untagged enum holds one value or named members: a tuple of \
                     several has no JSON form the derive walks",
                ));
                continue;
            }
            Fields::Named(_) => {
                let fields = field_infos(&variant.fields, &naming, false, access, errors);
                let binds = variant
                    .fields
                    .iter()
                    .enumerate()
                    .filter_map(|(index, field)| {
                        let name = field.ident.as_ref()?;
                        fields.iter().any(|info| info.ident == Some(name)).then(|| {
                            let binding = binding(index);
                            quote!(#name: #binding)
                        })
                    });
                (quote!(Self::#v { #(#binds,)* .. }), fields)
            }
        };
        for info in &fields {
            validate_field(info, &fields, errors);
        }
        variants.push(VariantInfo {
            ident: v,
            pattern,
            fields,
        });
    }
    variants
}

/// The impls of an untagged enum over its read `variants`.
fn untagged_impls(ident: &Ident, variants: &[VariantInfo]) -> TokenStream {
    let core = core();
    let variant_rows = variants.iter().map(|variant| {
        let name = variant.ident.to_string();
        let rows = variant.fields.iter().map(field_row);
        quote!((#name, &[#(#rows),*]))
    });
    let check_arms = variants.iter().map(|variant| {
        let pattern = &variant.pattern;
        let checks = variant
            .fields
            .iter()
            .map(|info| field_check(info, &variant.fields))
            .chain(
                variant
                    .fields
                    .iter()
                    .filter_map(|info| field_cross_check(info, &variant.fields)),
            );
        quote!(#pattern => { #(#checks)* })
    });
    let molang_arms = variants.iter().map(|variant| {
        let pattern = &variant.pattern;
        let molang = variant.fields.iter().map(field_molang);
        quote!(#pattern => { #(#molang)* })
    });
    let triggers: Vec<TokenStream> = variants
        .iter()
        .flat_map(|variant| trigger_calls(&variant.fields))
        .collect();

    quote! {
        #[automatically_derived]
        impl #core::schema::Schema for #ident {
            /// An untagged enum is no one object: its rows are per variant (`VARIANT_FIELDS`).
            const FIELDS: &'static [#core::schema::FieldRow] = &[];

            #[allow(unused_variables)]
            fn check(&mut self, cx: &mut #core::schema::SchemaCx<'_>) {
                match self {
                    #(#check_arms)*
                }
            }

            #[allow(unused_variables)]
            fn trigger_members(prefix: &str, out: &mut ::std::vec::Vec<::std::string::String>) {
                // Every variant's, a member two of them share once.
                let mut __all = ::std::vec::Vec::new();
                {
                    let out = &mut __all;
                    #(#triggers)*
                }
                let __start = out.len();
                for __member in __all {
                    if !out[__start..].contains(&__member) {
                        out.push(__member);
                    }
                }
            }
        }

        #[automatically_derived]
        impl #core::molang::MolangFields for #ident {
            #[allow(unused_variables)]
            fn visit_molang(&mut self, visitor: &mut #core::molang::MolangVisitor<'_>) {
                match self {
                    #(#molang_arms)*
                }
            }
        }

        #[automatically_derived]
        impl #ident {
            /// The rows of each variant's fields, by variant, in declaration order: a newtype's
            /// one row has no JSON key (its value is the enum's), a unit's none.
            pub const VARIANT_FIELDS: &'static [(&'static str, &'static [#core::schema::FieldRow])] =
                &[#(#variant_rows),*];
        }
    }
}
