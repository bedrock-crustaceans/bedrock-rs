//! What `#[derive(Schema)]` emits: for a struct, `impl Schema` (`FIELDS`, `check`,
//! `trigger_members`), `impl MolangFields`, `impl Default` when no field is required, and a
//! getter per `Option` field with a `schema::fallback`; for a unit-only enum, `ALL`, `NAMES`,
//! `EXPECTED`, `name`, `from_name`, `parse_name`, `Display`, an empty `impl Schema` and an
//! empty Molang walk.

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

fn derive_struct(input: &DeriveInput, fields: &Fields, errors: &mut Vec<Error>) -> TokenStream {
    let container = ContainerAttrs::read(&input.attrs, Place::Struct, errors);
    if matches!(fields, Fields::Unnamed(_)) && !container.transparent {
        errors.push(Error::new_spanned(
            fields,
            "`#[derive(Schema)]` applies to a struct with named fields or a \
             `#[facet(transparent)]` newtype",
        ));
    }
    let mut infos: Vec<FieldInfo> = Vec::new();
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
        let inline = container.transparent || attrs.flatten;
        let json = if inline {
            String::new()
        } else {
            json_name(&rust, attrs.rename.as_ref(), &container)
        };
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
        });
    }
    for info in &infos {
        validate_field(info, &infos, errors);
    }

    let core = core();
    let ident = &input.ident;
    let rows = infos.iter().map(|info| field_row(info));
    let checks = infos.iter().map(|info| field_check(info, &infos));
    let molang = infos.iter().map(field_molang);
    let triggers = infos
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
        });
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
        (Some((_, span)), None) if !ty::is_molang_slot(info.ty) => errors.push(Error::new(
            *span,
            "`schema::molang` names the spec of a Molang field, and this field holds no \
             `Molang`, `MolangSource` or `MolangList`: a hand walk is `schema::molang_with`",
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
    let ty_text = attr::text(ty);
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

/// The value a failed member resets to, `None` when it keeps its value (a required field).
fn reset(info: &FieldInfo, limit_is_field: bool) -> Option<TokenStream> {
    let value_ty = info.value_ty();
    if limit_is_field {
        return Some(if info.option.is_some() {
            quote!(::core::option::Option::Some(__limit))
        } else {
            quote!(__limit)
        });
    }
    if info.option.is_some() {
        return Some(info.attrs.fallback.as_ref().map_or_else(
            || quote!(::core::option::Option::None),
            |(fallback, _)| {
                let value = typed(fallback, value_ty);
                quote!(::core::option::Option::Some(#value))
            },
        ));
    }
    let default = info.attrs.default.as_ref()?;
    Some(default.value.as_ref().map_or_else(
        || quote!(::core::default::Default::default()),
        |expr| typed(expr, value_ty),
    ))
}

/// The limit of a bound as an `Option<T>`: a number, or the sibling's value (its fallback where
/// it is an absent `Option` with one; no check where it is absent without one).
fn limit_expr(bound: &BoundDecl, all: &[FieldInfo]) -> TokenStream {
    match &bound.limit {
        LimitDecl::Num(expr) => quote!(::core::option::Option::Some(#expr)),
        LimitDecl::Field(other) => {
            let Some(sibling) = all.iter().find(|f| *other == f.rust) else {
                return quote!(::core::option::Option::None);
            };
            let member = &sibling.member;
            match (&sibling.option, &sibling.attrs.fallback) {
                (None, _) => quote!(::core::option::Option::Some(self.#member)),
                (Some(inner), Some((fallback, _))) => {
                    let fallback = typed(fallback, inner);
                    quote!(::core::option::Option::Some(::core::option::Option::unwrap_or(self.#member, #fallback)))
                }
                (Some(_), None) => quote!(self.#member),
            }
        }
    }
}

/// A field's part of `check`: its bounds and `lowercase` at its path, then the walk into it.
fn field_check(info: &FieldInfo, all: &[FieldInfo]) -> TokenStream {
    let core = core();
    let member = &info.member;
    let value_ty = info.value_ty();
    let bounds = info.attrs.bounds.iter().map(|bound| {
        let compare = format_ident!(
            "{}",
            match bound.op {
                Op::Min => "lt",
                Op::Max => "gt",
                Op::Above => "le",
                Op::Below => "ge",
            }
        );
        let op = bound.op.variant();
        let limit = limit_expr(bound, all);
        let apply = reset(info, matches!(bound.limit, LimitDecl::Field(_))).map_or_else(
            || quote!(let _ = __verdict;),
            |value| {
                quote! {
                    if __verdict == #core::schema::Verdict::Reset {
                        self.#member = #value;
                    }
                }
            },
        );
        let value = if info.option.is_some() {
            quote!(__value)
        } else {
            quote!(self.#member)
        };
        let test = quote! {
            if ::core::cmp::PartialOrd::#compare(&#value, &__limit) {
                let __verdict = cx.bound(
                    #core::schema::BoundOp::#op,
                    #core::schema::Number::to_f64(__limit),
                );
                #apply
            }
        };
        // The limit typed by the field: a literal of another type is the compiler's error at it.
        let span = bound.span;
        let typed_limit = quote_spanned!(span=> let __limit: ::core::option::Option<#value_ty> = #limit;);
        if info.option.is_some() {
            quote! {{
                #typed_limit
                if let (::core::option::Option::Some(__limit), ::core::option::Option::Some(__value)) = (__limit, self.#member) {
                    #test
                }
            }}
        } else {
            quote! {{
                #typed_limit
                if let ::core::option::Option::Some(__limit) = __limit {
                    #test
                }
            }}
        }
    });
    let lowercase = info.attrs.lowercase.map(|_| {
        let fix = quote! {
            if __value.chars().any(char::is_uppercase)
                && cx.not_lowercase() == #core::schema::Verdict::Reset
            {
                *__value = __value.to_lowercase();
            }
        };
        if info.option.is_some() {
            quote!(if let ::core::option::Option::Some(__value) = &mut self.#member { #fix })
        } else {
            quote!({ let __value = &mut self.#member; #fix })
        }
    });
    let walk = quote!(#core::schema::SchemaFields::check_fields(&mut self.#member, cx););
    if info.inline {
        quote!(#(#bounds)* #lowercase #walk)
    } else {
        let json = &info.json;
        quote! {
            cx.enter(#json);
            #(#bounds)*
            #lowercase
            #walk
            cx.leave();
        }
    }
}

/// A field's part of the Molang walk.
fn field_molang(info: &FieldInfo) -> TokenStream {
    let core = core();
    let member = &info.member;
    let json = &info.json;
    if let Some((walk, _)) = &info.attrs.molang_with {
        return if info.inline {
            quote!((#walk)(&mut self.#member, visitor);)
        } else {
            quote! {
                visitor.enter(#core::molang::Step::Key(#json));
                (#walk)(&mut self.#member, visitor);
                visitor.leave();
            }
        };
    }
    if ty::is_molang_slot(info.ty) {
        let spec = info.attrs.molang.as_ref().map(|(spec, _)| spec);
        return match (info.inline, spec) {
            (false, Some(spec)) => quote!(visitor.field(#json, &#spec, &mut self.#member);),
            (false, None) => quote!(visitor.field_default(#json, &mut self.#member);),
            (true, Some(spec)) => {
                quote!(#core::molang::MolangSlot::visit_slot(&mut self.#member, &#spec, visitor);)
            }
            (true, None) => quote! {
                let __spec = *visitor.default_spec();
                #core::molang::MolangSlot::visit_slot(&mut self.#member, &__spec, visitor);
            },
        };
    }
    if info.inline {
        quote!(#core::molang::MolangFields::visit_molang(&mut self.#member, visitor);)
    } else {
        quote!(visitor.nested(#json, &mut self.#member);)
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
            let doc = format!(
                "`{name}`, or `{}` where the document has none.",
                attr::text(fallback)
            );
            Some(quote! {
                #[doc = #doc]
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
    let container = ContainerAttrs::read(&input.attrs, Place::Enum, errors);
    let mut variants = Vec::new();
    for variant in &data.variants {
        let rename = attr::variant_rename(&variant.attrs, errors);
        if !matches!(variant.fields, Fields::Unit) {
            errors.push(Error::new_spanned(
                &variant.fields,
                "`#[derive(Schema)]` on an enum applies to a string enum, whose variants are all \
                 units: a variant with data has no one name",
            ));
            continue;
        }
        let name = json_name(&variant.ident.to_string(), rename.as_ref(), &container);
        variants.push((&variant.ident, name));
    }
    let core = core();
    let ident = &input.ident;
    let all = variants.iter().map(|(v, _)| quote!(Self::#v));
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
    let from_name = if container.case_insensitive {
        let tests = variants.iter().map(|(v, n)| {
            quote!(if name.eq_ignore_ascii_case(#n) { return ::core::option::Option::Some(Self::#v); })
        });
        quote!(#(#tests)* ::core::option::Option::None)
    } else {
        let arms = variants
            .iter()
            .map(|(v, n)| quote!(#n => ::core::option::Option::Some(Self::#v),));
        quote!(match name { #(#arms)* _ => ::core::option::Option::None })
    };
    let case = if container.case_insensitive {
        ", in any case"
    } else {
        ""
    };
    let from_doc = format!("The value spelt `name`{case}.");
    quote! {
        #[automatically_derived]
        impl #ident {
            /// Every value, in declaration order.
            pub const ALL: &'static [Self] = &[#(#all),*];
            /// The JSON names, in declaration order.
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
