//! What the schema keys of `#[derive(Components)]` add to a family's component enum: the
//! second table (`SCHEMA_ROWS`, `schema_row`, `typed_since`), `GoalMeta`, the `VERSIONED`
//! table and `visit_goals` for the goal rows, and on request the write dispatch, `triggers()` and a test holding
//! `typed` to the generated game table.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::attr::{EnumAttrs, Row, Version};
use crate::expand::{Typed, Variants};

fn version(v: Version) -> TokenStream {
    let Version {
        major,
        minor,
        patch,
    } = v;
    quote!(::bedrock_addon_core::version::FormatVersion::new(#major, #minor, #patch))
}

fn option(value: Option<TokenStream>) -> TokenStream {
    value.map_or_else(
        || quote!(::core::option::Option::None),
        |value| quote!(::core::option::Option::Some(#value)),
    )
}

/// One `ComponentRow` literal.
fn component_row(row: &Row) -> TokenStream {
    let name = &row.name;
    let typed = option(row.typed.map(version));
    let shorthand = option(
        row.shorthand
            .then(|| quote!(::bedrock_addon_core::schema::Shorthand::Scalar)),
    );
    let goal = row.goal.is_some();
    let triggers = &row.triggers;
    quote! {
        ::bedrock_addon_core::schema::ComponentRow {
            name: #name,
            typed: #typed,
            shorthand: #shorthand,
            goal: #goal,
            triggers: &[#(#triggers),*],
        }
    }
}

/// Everything the schema keys add to a family's enum.
pub(crate) fn emit(enum_ident: &Ident, attrs: &EnumAttrs, variants: &Variants) -> TokenStream {
    let typed: Vec<(&Typed, &Row)> = variants
        .typed
        .iter()
        .filter_map(|t| t.row.as_ref().map(|row| (t, row)))
        .collect();
    let mut rows: Vec<&Row> = typed
        .iter()
        .map(|(_, row)| *row)
        .chain(&variants.dynamic_rows)
        .collect();
    rows.sort_by_key(|row| row.name.value());
    let literals = rows.iter().map(|row| component_row(row));

    let goals = typed.iter().filter(|(_, row)| row.goal.is_some());
    let goal_meta = goals.clone().filter_map(|(t, row)| {
        let ty = t.ty;
        let typed = version(row.typed?);
        Some(quote! {
            // The goal's `priority` declares its fallback (`schema::fallback` on the goal's
            // `Schema` struct); the build fails here where it does not.
            const _: i64 = ::bedrock_addon_core::schema::int_fallback(
                <#ty as ::bedrock_addon_core::schema::Schema>::FIELDS,
                "priority",
            );

            #[automatically_derived]
            impl ::bedrock_addon_core::schema::GoalMeta for #ty {
                const TYPED: ::bedrock_addon_core::version::FormatVersion = #typed;

                #[allow(clippy::cast_possible_truncation, reason = "the fallback is the field's own `i32`")]
                fn priority(&self) -> i32 {
                    match self.priority {
                        ::core::option::Option::Some(priority) => priority,
                        ::core::option::Option::None => const {
                            ::bedrock_addon_core::schema::int_fallback(
                                <#ty as ::bedrock_addon_core::schema::Schema>::FIELDS,
                                "priority",
                            ) as i32
                        },
                    }
                }
            }
        })
    });
    let versioned = attrs.versioned.as_ref().map(|parse| {
        let mut entries: Vec<(String, TokenStream)> = goals
            .clone()
            .map(|(t, row)| {
                let ty = t.ty;
                let name = &row.name;
                (row.name.value(), quote!((#name, #parse::<#ty>)))
            })
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let entries = entries.into_iter().map(|(_, tokens)| tokens);
        // A goal row states `typed` (the derive refuses one without), as for `GoalMeta`.
        let mut visits: Vec<(String, TokenStream)> = goals
            .clone()
            .filter_map(|(t, row)| {
                let ty = t.ty;
                let name = &row.name;
                let typed = version(row.typed?);
                Some((row.name.value(), quote!(visitor.goal::<#ty>(#name, #typed);)))
            })
            .collect();
        visits.sort_by(|a, b| a.0.cmp(&b.0));
        let visits = visits.into_iter().map(|(_, tokens)| tokens);
        quote! {
            /// `(key, parse by version)` of every goal row, sorted by key.
            #[doc(hidden)]
            pub const VERSIONED: &'static [(&'static str, ::bedrock_addon_core::schema::VersionedParseFn<Self>)] =
                &[#(#entries),*];

            /// Hands `visitor` every goal row's key, typed version and type, sorted by key.
            #[doc(hidden)]
            pub fn visit_goals(visitor: &mut impl ::bedrock_addon_core::schema::GoalVisitor<Self>) {
                #(#visits)*
            }
        }
    });
    let write = write_dispatch(attrs, &typed, variants);
    let triggers = attrs.triggers.map(|_| triggers(&typed, &rows));
    let typed_test = attrs
        .typed_table
        .as_ref()
        .map(|table| typed_test(enum_ident, table));

    quote! {
        #[automatically_derived]
        #[allow(dead_code)]
        impl #enum_ident {
            /// Per component row, the facts the version chain's table does not carry, sorted by
            /// name.
            #[doc(hidden)]
            pub const SCHEMA_ROWS: &'static [::bedrock_addon_core::schema::ComponentRow] =
                &[#(#literals),*];

            /// The `SCHEMA_ROWS` row of the key `name`.
            #[doc(hidden)]
            pub fn schema_row(name: &str) -> ::core::option::Option<&'static ::bedrock_addon_core::schema::ComponentRow> {
                ::bedrock_addon_core::schema::find_row(Self::SCHEMA_ROWS, name)
            }

            /// The format version from which the game reads the key `name` with its typed
            /// struct (the row's `typed`).
            #[doc(hidden)]
            pub fn typed_since(name: &str) -> ::core::option::Option<::bedrock_addon_core::version::FormatVersion> {
                Self::schema_row(name).and_then(|row| row.typed)
            }

            #versioned
            #write
            #triggers
        }

        #(#goal_meta)*
        #typed_test
    }
}

/// `write_component`: each typed variant through `write_payload` (or `write_with`), the custom
/// one as its parameters, the unknown and dynamic ones as written.
fn write_dispatch(
    attrs: &EnumAttrs,
    typed: &[(&Typed, &Row)],
    variants: &Variants,
) -> Option<TokenStream> {
    let writer = match (&attrs.write, &attrs.write_with) {
        (_, Some(path)) => quote!(#path),
        (Some(_), None) => quote!(::bedrock_addon_core::versioning::write_payload),
        (None, None) => return None,
    };
    let typed_arms = typed.iter().map(|(t, _)| {
        let variant = t.variant;
        quote!(Self::#variant(component) => #writer(component),)
    });
    let special_arms = variants.special.iter().map(|special| {
        let variant = special.variant;
        if attrs.custom.as_ref() == Some(variant) {
            quote!(Self::#variant(custom) => ::core::result::Result::Ok(::core::clone::Clone::clone(&custom.parameters)),)
        } else {
            quote!(Self::#variant(json) => ::core::result::Result::Ok(::core::clone::Clone::clone(json)),)
        }
    });
    Some(quote! {
        /// One component as JSON: a typed component written from its value, a custom one as its
        /// parameters, an unknown or dynamic one as written.
        #[doc(hidden)]
        pub fn write_component(&self) -> ::core::result::Result<::bedrock_addon_core::json::Value, ::bedrock_addon_core::json::EncodeError> {
            match self {
                #(#typed_arms)*
                #(#special_arms)*
            }
        }
    })
}

/// `triggers()`: `(key, member)` for every trigger member of every row, sorted: the payload's
/// `trigger_members` and the row's `triggers = [..]`.
fn triggers(typed: &[(&Typed, &Row)], rows: &[&Row]) -> TokenStream {
    let from_types = typed.iter().map(|(t, row)| {
        let ty = t.ty;
        let name = &row.name;
        quote! {
            let mut members = ::std::vec::Vec::new();
            <#ty as ::bedrock_addon_core::schema::SchemaFields>::trigger_members("", &mut members);
            out.extend(members.into_iter().map(|member| (#name, member)));
        }
    });
    let listed = rows.iter().flat_map(|row| {
        let name = &row.name;
        row.triggers
            .iter()
            .map(move |member| quote!(out.push((#name, ::std::string::String::from(#member)));))
    });
    quote! {
        /// `(component key, member)` of every trigger member, sorted: the members holding an
        /// `EventTrigger` of each typed component, and each row's `triggers = [..]`.
        #[doc(hidden)]
        pub fn triggers() -> ::std::vec::Vec<(&'static str, ::std::string::String)> {
            let mut out = ::std::vec::Vec::new();
            #(#from_types)*
            #(#listed)*
            out.sort();
            out.dedup();
            out
        }
    }
}

/// The test holding every row's `typed` to the generated game table: a key the table lists
/// has the table's typed version (or none where the table has none).
fn typed_test(enum_ident: &Ident, table: &syn::Path) -> TokenStream {
    let module = format_ident!(
        "__{}_typed_matches_the_game_table",
        enum_ident.to_string().to_lowercase()
    );
    quote! {
        #[cfg(test)]
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #module {
            // The table's path as written: a bare name or `self::` resolves through the glob,
            // `crate::` and `::` as they are.
            #[allow(unused_imports)]
            use super::*;

            #[test]
            fn every_rows_typed_version_is_the_game_tables() {
                let table: fn(&str) -> ::core::option::Option<::core::option::Option<::bedrock_addon_core::version::FormatVersion>> = #table;
                let mut differ = ::std::vec::Vec::new();
                for row in super::#enum_ident::SCHEMA_ROWS {
                    if let ::core::option::Option::Some(game) = table(row.name) {
                        if game != row.typed {
                            differ.push((row.name, row.typed, game));
                        }
                    }
                }
                assert!(differ.is_empty(), "rows whose `typed` is not the game table's (key, row, table): {differ:?}");
            }
        }
    }
}
