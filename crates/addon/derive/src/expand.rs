//! What `#[derive(Components)]` emits for a component enum.

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, DataEnum, DeriveInput, Error, Fields, Ident, Result, Type, Variant};

use crate::attr::{self, EnumAttrs, Row, Version};

/// A typed variant, `Friction(Friction)`, with its table row (`None` in a `from_only` enum).
struct Typed<'a> {
    variant: &'a Ident,
    ty: &'a Type,
    row: Option<Row>,
}

/// A `custom`, `unknown` or `dynamic` variant and the payload type the version-chain model gives it.
struct Special<'a> {
    variant: &'a Ident,
    ty: &'a Type,
    expected: TokenStream,
}

/// The variants of the enum, sorted by role.
struct Variants<'a> {
    typed: Vec<Typed<'a>>,
    special: Vec<Special<'a>>,
    dynamic_rows: Vec<Row>,
}

/// Expands the derive, or the errors that stop it. Every error of an enum is reported at
/// once; a target that is not an enum is the one early return.
pub(crate) fn derive(input: &DeriveInput) -> Result<TokenStream> {
    let data = component_enum(input)?;
    let mut errors: Vec<Error> = Vec::new();
    if !input.generics.params.is_empty() {
        errors.push(Error::new_spanned(
            &input.generics,
            "a component enum has no generic parameters",
        ));
    }
    let enum_attrs = EnumAttrs::parse(&input.attrs, &mut errors);
    match (&enum_attrs.from_only, &enum_attrs.family) {
        (None, None) => errors.push(Error::new(
            input.ident.span(),
            "missing `#[component(family = …)]` on the component enum: the family whose \
             `ComponentTable` it fills",
        )),
        (Some(_), Some(family)) => errors.push(Error::new_spanned(
            family,
            "a `from_only` enum has no component table, so no `family`",
        )),
        _ => {}
    }
    let variants = variants(data, &enum_attrs, &mut errors);
    if let Some(error) = errors.into_iter().reduce(|mut all, next| {
        all.combine(next);
        all
    }) {
        return Err(error);
    }
    let enum_ident = &input.ident;
    let checks = payload_checks(enum_ident, &variants.special);
    let from = from_impls(enum_ident, &variants.typed);
    match &enum_attrs.family {
        Some(family) if enum_attrs.from_only.is_none() => {
            let components = emit(enum_ident, family, &variants);
            Ok(quote!(#checks #from #components))
        }
        _ => Ok(quote!(#checks #from)),
    }
}

/// What a variant of the component enum is: a typed component, or one of the three the enum
/// attributes name.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Custom,
    Unknown,
    Dynamic,
    Component,
}

impl Role {
    /// The word the enum attributes and the error messages use.
    fn name(self) -> &'static str {
        match self {
            Self::Custom => "custom",
            Self::Unknown => "unknown",
            Self::Dynamic => "dynamic",
            Self::Component => "component",
        }
    }
}

/// The enum the derive is on; a struct or a union is an error.
fn component_enum(input: &DeriveInput) -> Result<&DataEnum> {
    match &input.data {
        Data::Enum(data) => Ok(data),
        Data::Struct(data) => Err(Error::new(
            data.struct_token.span,
            "`#[derive(Components)]` applies to a family's component enum, not a struct",
        )),
        Data::Union(data) => Err(Error::new(
            data.union_token.span,
            "`#[derive(Components)]` applies to a family's component enum, not a union",
        )),
    }
}

/// Sorts the variants by role. Every malformed variant, unknown variant name and duplicate
/// component name adds an error.
fn variants<'a>(
    data: &'a DataEnum,
    enum_attrs: &EnumAttrs,
    errors: &mut Vec<Error>,
) -> Variants<'a> {
    let roles = [
        (
            Role::Custom,
            &enum_attrs.custom,
            quote!(::bedrock_addon_core::document::CustomComponent),
        ),
        (
            Role::Unknown,
            &enum_attrs.unknown,
            quote!(::bedrock_addon_core::json::Value),
        ),
        (
            Role::Dynamic,
            &enum_attrs.dynamic,
            quote!(::bedrock_addon_core::json::Value),
        ),
    ];
    for named in roles.iter().filter_map(|(_, named, _)| named.as_ref()) {
        if !data.variants.iter().any(|v| v.ident == *named) {
            errors.push(Error::new(
                named.span(),
                format!("no variant `{named}` in this enum"),
            ));
        }
    }

    let from_only = enum_attrs.from_only.is_some();
    let mut out = Variants {
        typed: Vec::new(),
        special: Vec::new(),
        dynamic_rows: Vec::new(),
    };
    for variant in &data.variants {
        let ident = &variant.ident;
        if from_only && let Some(attr) = attr::component_attrs(&variant.attrs).next() {
            let message = if attr::names_key(attr, "molang") {
                "a `from_only` enum gets no `MolangFields` impl, so no `molang`: implement \
                 `MolangFields` for it by hand"
            } else {
                "a `from_only` enum takes no `#[component]` attribute on its variants"
            };
            errors.push(Error::new_spanned(attr, message));
            continue;
        }
        let role = roles
            .iter()
            .find(|(_, named, _)| named.as_ref() == Some(ident));
        let result = match role {
            Some((role, _, expected)) => special_variant(variant, *role).map(|(ty, rows)| {
                out.special.push(Special {
                    variant: ident,
                    ty,
                    expected: expected.clone(),
                });
                out.dynamic_rows.extend(rows);
            }),
            None => typed_variant(variant, from_only).map(|t| out.typed.push(t)),
        };
        if let Err(error) = result {
            errors.push(error);
        }
    }

    let mut seen = HashSet::new();
    let rows = out.typed.iter().filter_map(|t| t.row.as_ref());
    for row in rows.chain(&out.dynamic_rows) {
        let name = row.name.value();
        if !seen.insert(name.clone()) {
            errors.push(Error::new(
                row.name.span(),
                format!("duplicate component name `{name}`"),
            ));
        }
    }
    out
}

/// A `custom`, `unknown` or `dynamic` variant: one unnamed field (its type is checked by the
/// emitted code); only the `dynamic` one may carry `#[component]` rows.
fn special_variant(variant: &Variant, role: Role) -> Result<(&Type, Vec<Row>)> {
    let ty = newtype_field(variant, role)?;
    if role == Role::Dynamic {
        return Ok((ty, attr::dynamic_rows(&variant.attrs)?));
    }
    match attr::component_attrs(&variant.attrs).next() {
        Some(attr) => Err(Error::new_spanned(
            attr,
            format!(
                "the `{}` variant takes no `#[component]` attribute",
                role.name()
            ),
        )),
        None => Ok((ty, Vec::new())),
    }
}

/// A typed variant: one unnamed field, the component type, and (unless `from_only`) a
/// `#[component]` row.
fn typed_variant(variant: &Variant, from_only: bool) -> Result<Typed<'_>> {
    let ident = &variant.ident;
    let ty = newtype_field(variant, Role::Component)?;
    if from_only {
        return Ok(Typed {
            variant: ident,
            ty,
            row: None,
        });
    }
    let Some(row) = attr::typed_row(&variant.attrs)? else {
        return Err(Error::new(
            ident.span(),
            "missing `#[component(name = \"minecraft:…\", parse = …)]`: every variant other than \
             the `custom`, `unknown` and `dynamic` ones is a typed component",
        ));
    };
    Ok(Typed {
        variant: ident,
        ty,
        row: Some(row),
    })
}

/// The one unnamed field of `variant`; `role` names the variant in the error.
fn newtype_field(variant: &Variant, role: Role) -> Result<&Type> {
    let ident = &variant.ident;
    match &variant.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => Ok(&fields.unnamed[0].ty),
        fields => {
            let message = match role {
                Role::Custom => format!(
                    "the `custom` variant has exactly one unnamed field: `{ident}(CustomComponent)`"
                ),
                Role::Unknown | Role::Dynamic => {
                    format!(
                        "the `{}` variant has exactly one unnamed field: `{ident}(Value)`",
                        role.name()
                    )
                }
                Role::Component => format!(
                    "a component variant has exactly one unnamed field, the component type: \
                     `{ident}({ident})`"
                ),
            };
            Err(match fields {
                Fields::Unit => Error::new(ident.span(), message),
                fields => Error::new_spanned(fields, message),
            })
        }
    }
}

/// For each special variant, a constant that only compiles when its payload is the type the
/// version-chain model gives it (`CustomComponent` for `custom`, `json::Value` for `unknown`
/// and `dynamic`); a mismatch is reported at the payload type.
fn payload_checks(enum_ident: &Ident, special: &[Special]) -> TokenStream {
    let checks = special.iter().map(
        |Special {
             variant,
             ty,
             expected,
         }| {
            // Every token of the check carries the payload's span, so a mismatch points there.
            let span = ty.span();
            let enum_ident = Ident::new(&enum_ident.to_string(), span);
            let variant = Ident::new(&variant.to_string(), span);
            quote_spanned! {span=>
                const _: fn(#expected) -> #enum_ident = #enum_ident::#variant;
            }
        },
    );
    quote!(#(#checks)*)
}

/// `impl From<Type> for <Enum>` per typed variant.
fn from_impls(enum_ident: &Ident, typed: &[Typed]) -> TokenStream {
    let impls = typed.iter().map(|Typed { variant, ty, .. }| {
        quote! {
            #[automatically_derived]
            impl ::core::convert::From<#ty> for #enum_ident {
                fn from(component: #ty) -> Self {
                    Self::#variant(component)
                }
            }
        }
    });
    quote!(#(#impls)*)
}

/// The `ComponentSet` impl, a `Component` impl per typed variant, and `TABLE`.
fn emit(enum_ident: &Ident, family: &Type, variants: &Variants) -> TokenStream {
    let typed: Vec<(&Typed, &Row)> = variants
        .typed
        .iter()
        .filter_map(|t| t.row.as_ref().map(|row| (t, row)))
        .collect();

    let name_arms = typed.iter().map(|(t, row)| {
        let variant = t.variant;
        let name = &row.name;
        quote!(Self::#variant(_) => ::core::option::Option::Some(#name),)
    });

    let impls = typed.iter().map(|(t, row)| {
        let Typed { variant, ty, .. } = t;
        let name = &row.name;
        quote! {
            #[automatically_derived]
            #[allow(unreachable_patterns, clippy::match_wildcard_for_single_variants)]
            impl ::bedrock_addon_core::document::Component for #ty {
                type Set = #enum_ident;
                const NAME: &'static str = #name;

                fn from_set(set: &#enum_ident) -> ::core::option::Option<&Self> {
                    match set {
                        #enum_ident::#variant(component) => ::core::option::Option::Some(component),
                        _ => ::core::option::Option::None,
                    }
                }

                fn from_set_mut(set: &mut #enum_ident) -> ::core::option::Option<&mut Self> {
                    match set {
                        #enum_ident::#variant(component) => ::core::option::Option::Some(component),
                        _ => ::core::option::Option::None,
                    }
                }

                fn try_from_set(set: #enum_ident) -> ::core::result::Result<Self, #enum_ident> {
                    match set {
                        #enum_ident::#variant(component) => ::core::result::Result::Ok(component),
                        other => ::core::result::Result::Err(other),
                    }
                }

                fn into_set(self) -> #enum_ident {
                    #enum_ident::#variant(self)
                }
            }
        }
    });

    // The Molang walk forwards to the types of the variants marked `molang`; every other
    // variant holds no typed Molang field.
    let molang_arms = typed.iter().filter(|(_, row)| row.molang).map(|(t, _)| {
        let variant = t.variant;
        quote! {
            Self::#variant(component) => {
                ::bedrock_addon_core::molang::MolangFields::visit_molang(component, visitor);
            }
        }
    });

    // Sorted here, in the byte order `str::cmp` gives, so the table can be binary-searched.
    let mut rows: Vec<&Row> = typed
        .iter()
        .map(|(_, row)| *row)
        .chain(&variants.dynamic_rows)
        .collect();
    rows.sort_by_key(|row| row.name.value());
    let metas = rows.into_iter().map(meta);

    quote! {
        #[automatically_derived]
        #[allow(unreachable_patterns, clippy::match_wildcard_for_single_variants)]
        impl ::bedrock_addon_core::document::ComponentSet for #enum_ident {
            fn name(&self) -> ::core::option::Option<&'static str> {
                match self {
                    #(#name_arms)*
                    _ => ::core::option::Option::None,
                }
            }
        }

        #(#impls)*

        #[automatically_derived]
        #[allow(unreachable_patterns, clippy::match_wildcard_for_single_variants)]
        impl ::bedrock_addon_core::molang::MolangFields for #enum_ident {
            #[allow(unused_variables)]
            fn visit_molang(&mut self, visitor: &mut ::bedrock_addon_core::molang::MolangVisitor<'_>) {
                #[allow(clippy::match_single_binding)]
                match self {
                    #(#molang_arms)*
                    _ => {}
                }
            }
        }

        #[automatically_derived]
        impl #enum_ident {
            /// The family's component table: one row per typed component and dynamic key,
            /// sorted by name for binary search.
            #[doc(hidden)]
            pub const TABLE: &'static ::bedrock_addon_core::versioning::ComponentTable<#family> =
                &::bedrock_addon_core::versioning::ComponentTable(&[#(#metas),*]);
        }
    }
}

/// One `ComponentMeta` literal.
fn meta(row: &Row) -> TokenStream {
    let name = &row.name;
    let released = option(row.released.map(version));
    let deprecated_since = option(row.deprecated_since.map(version));
    let deprecated_message = option(row.deprecated_message.as_ref().map(|lit| quote!(#lit)));
    let supersedes = option(row.supersedes.as_ref().map(|lit| quote!(#lit)));
    let requires = &row.requires;
    let area = &row.area;
    let fields = row.fields.iter().map(|field| {
        let field_name = &field.name;
        let since_engine = version(field.since_engine);
        quote! {
            ::bedrock_addon_core::versioning::FieldMeta {
                name: #field_name,
                since_engine: #since_engine,
            }
        }
    });
    let parse = &row.parse;
    quote! {
        ::bedrock_addon_core::versioning::ComponentMeta {
            name: #name,
            released: #released,
            deprecated_since: #deprecated_since,
            deprecated_message: #deprecated_message,
            supersedes: #supersedes,
            requires: &[#requires],
            area: ::bedrock_addon_core::versioning::ContextArea::#area,
            fields: &[#(#fields),*],
            parse: #parse,
        }
    }
}

/// `Some(tokens)` or `None`, fully qualified.
fn option(value: Option<TokenStream>) -> TokenStream {
    value.map_or_else(
        || quote!(::core::option::Option::None),
        |value| quote!(::core::option::Option::Some(#value)),
    )
}

/// A `FormatVersion` literal.
fn version(v: Version) -> TokenStream {
    let Version {
        major,
        minor,
        patch,
    } = v;
    quote!(::bedrock_addon_core::version::FormatVersion::new(#major, #minor, #patch))
}
