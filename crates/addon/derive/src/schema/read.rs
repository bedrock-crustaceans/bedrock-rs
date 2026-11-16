//! What `#[derive(Schema)]` emits for the read: `impl FromJson` (core's `schema::FromJson`) and,
//! for a struct, `impl Fields` (the read of its fields from an object it may share with the
//! struct that flattens it). The emitted code is a slot per field, a match arm per JSON name and
//! one call per field that says what absence and failure make of it; the rest is core's
//! `schema::derive_support`.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{DataEnum, Error, Fields, Ident, Member, Type, Variant};

use super::attr::{ContainerAttrs, DefaultDecl, FieldAttrs};
use super::{FieldInfo, Target, core, json_name, ty, unknown_option};

/// The paths the emitted read names.
struct Paths {
    node: TokenStream,
    read: TokenStream,
    from_json: TokenStream,
    fields: TokenStream,
    object: TokenStream,
    reader: TokenStream,
    expected: TokenStream,
}

fn paths() -> Paths {
    let core = core();
    Paths {
        node: quote!(#core::json::kit::Node),
        read: quote!(#core::json::kit::Read),
        from_json: quote!(#core::schema::FromJson),
        fields: quote!(#core::schema::derive_support::Fields),
        object: quote!(#core::schema::derive_support::Object),
        reader: quote!(#core::schema::derive_support),
        expected: quote!(#core::json::kit::Expected),
    }
}

/// Whether the type is named `name` (its last segment).
fn named(ty: &Type, name: &str) -> bool {
    ty::name(ty).is_some_and(|n| n == name)
}

/// The member of field `index` of `fields`.
fn member_of(field: &syn::Field, index: usize) -> Member {
    match &field.ident {
        Some(ident) => Member::Named(ident.clone()),
        None => Member::Unnamed(index.into()),
    }
}

/// The value a field the read does not take is built with: a `skip` field's facet `default`
/// (its type's `Default` without one), the struct's kept unknown members for an
/// `UnknownMembers` field that is not `skip` (a `skip` one holds nothing the read keeps), the
/// object as written for a `Written` one, and for a `Present` one the `names` (the JSON names
/// of the struct's fields) the object wrote, `null` ones included.
fn untaken_value(field: &syn::Field, names: &[&String]) -> TokenStream {
    let mut scratch = Vec::new();
    let attrs = FieldAttrs::read(&field.attrs, &mut scratch);
    let ty = &field.ty;
    if named(ty, "UnknownMembers") && !attrs.skip {
        return quote!(__extra);
    }
    if named(ty, "Present") {
        return quote!(::core::convert::From::from(::core::option::Option::Some(
            [#(#names),*]
                .into_iter()
                .filter(|__name: &&'static str| __object
                    .node()
                    .value()
                    .as_object()
                    .is_some_and(|__members| __members.contains_key(*__name)))
                .collect::<::std::vec::Vec<&'static str>>()
        )));
    }
    if named(ty, "Written") {
        return quote!(::core::convert::From::from(::core::option::Option::Some(
            ::core::clone::Clone::clone(__object.node().value())
        )));
    }
    attrs.default.and_then(|default| default.value).map_or_else(
        || quote!(::core::default::Default::default()),
        |expr| super::typed(&expr, ty),
    )
}

/// Whether the fields keep their unknown members by a mark: a `skip` field of type `Extra`, or
/// a field of type `UnknownMembers`.
fn marked(fields: &Fields) -> bool {
    fields.iter().any(|field| {
        let mut scratch = Vec::new();
        let attrs = FieldAttrs::read(&field.attrs, &mut scratch);
        (attrs.skip && named(&field.ty, "Extra")) || named(&field.ty, "UnknownMembers")
    })
}

/// Whether the fields have an `UnknownMembers` field, which holds the kept unknown members.
fn has_extra(fields: &Fields) -> bool {
    fields
        .iter()
        .any(|field| named(&field.ty, "UnknownMembers"))
}

/// The field that holds the kept unknown members (an `UnknownMembers` field that is not
/// `skip`), by its member.
fn extra_member(infos: &[FieldInfo]) -> Option<TokenStream> {
    infos
        .iter()
        .find(|info| !taken(info) && !info.attrs.skip)
        .map(|info| {
            let member = &info.member;
            quote!(#member)
        })
}

/// Whether `info` is the read's to take: not an `UnknownMembers` field (it holds the unknown
/// members).
fn taken(info: &FieldInfo) -> bool {
    !named(info.ty, "UnknownMembers")
}

/// The JSON names a field is read by: the members it is fed by (`schema::members`), else its
/// own name and its alias.
fn json_names<'a>(info: &'a FieldInfo<'_>) -> Vec<&'a String> {
    match &info.attrs.members {
        Some((names, _)) => names.iter().collect(),
        None => std::iter::once(&info.json)
            .chain(info.attrs.alias.as_ref())
            .collect(),
    }
}

/// `knows`: the JSON names of the fields, a flattened struct's included, of the member
/// `__key`. A read (`dynamic`) asks the flattened types what they know of its object
/// (`__object`), where an internally tagged enum knows the members of the variant it read.
pub(super) fn knows(infos: &[FieldInfo], dynamic: bool) -> TokenStream {
    let p = paths();
    let fields = &p.fields;
    let names: Vec<&String> = infos
        .iter()
        .filter(|info| taken(info) && !info.inline)
        .flat_map(json_names)
        .fold(Vec::new(), |mut names, name| {
            if !names.contains(&name) {
                names.push(name);
            }
            names
        });
    let own = if names.is_empty() {
        quote!(false)
    } else {
        quote!(::core::matches!(__key, #(#names)|*))
    };
    let flattened = infos
        .iter()
        .filter(|info| info.inline && taken(info))
        .map(|info| {
            let ty = info.ty;
            if dynamic {
                quote_spanned!(ty.span()=> || <#ty as #fields>::knows_given(__object, __key))
            } else {
                quote_spanned!(ty.span()=> || <#ty as #fields>::knows(__key))
            }
        });
    quote!(#own #(#flattened)*)
}

/// The refusals of the read's keys: `read_with` on a flattened field or on the field that holds
/// the unknown members; a code or `finite` where the field has no member of its own to read,
/// `missing` on a field that is never missing, `finite` on a field that holds no float; a
/// list's keys on a field that is no `Vec` or that has its own read; `or_kept` on a field that
/// is no `Option`.
pub(super) fn validate(info: &FieldInfo, errors: &mut Vec<Error>) {
    let attrs = &info.attrs;
    let held = info.option.unwrap_or(info.ty);
    if let Some(span) = attrs.iterated {
        if ty::name(held).is_none_or(|name| name != "Vec") {
            errors.push(Error::new(
                span,
                "`schema::iterated` reads a list: a `Vec` field (or an `Option` of one)",
            ));
        } else if attrs.read_with.is_some() {
            errors.push(Error::new(
                span,
                "a field with `schema::read_with` is read by its function, which reads the list \
                 its own way",
            ));
        }
    }
    validate_drop(info, errors);
    validate_keys(info, errors);
    validate_members(info, errors);
    if let Some(span) = attrs.reads_null
        && (info.option.is_none() || attrs.flatten)
    {
        errors.push(Error::new(
            span,
            "`schema::reads_null` reads a `null` as a value: an `Option` field",
        ));
    }
    if let Some((_, span)) = attrs.or_kept
        && (info.option.is_none() || attrs.flatten)
    {
        errors.push(Error::new(
            span,
            "`schema::or_kept` leaves the field `None` where its value fails: an `Option` field",
        ));
    }
    let ruled = [
        &attrs.wrong_type,
        &attrs.missing,
        &attrs.invalid,
        &attrs.negative,
    ]
    .into_iter()
    .find_map(|code| code.as_ref().map(|(_, span)| *span))
    .or(attrs.finite);
    if let Some(span) = ruled.filter(|_| attrs.flatten || !taken(info)) {
        errors.push(Error::new(
            span,
            "a member's code or rule is for a field that reads one member: a flattened field \
             or an `UnknownMembers` one has none",
        ));
    }
    if let Some((_, span)) = &attrs.missing
        && ((info.option.is_some() && attrs.required.is_none()) || attrs.default.is_some())
    {
        errors.push(Error::new(
            *span,
            "`schema::missing` words a member the document must write: an `Option` field \
             (without `schema::required`) or one with a `default` is never missing",
        ));
    }
    if let Some(span) = attrs.finite
        && !ty::name(info.option.unwrap_or(info.ty))
            .is_some_and(|name| name == "f32" || name == "f64")
    {
        errors.push(Error::new(
            span,
            "`schema::finite` applies to an `f32` or `f64` field (or an `Option` of one)",
        ));
    }
    if let Some((_, span)) = &info.attrs.read_with {
        if info.attrs.flatten {
            errors.push(Error::new(
                *span,
                "`schema::read_with` reads one member: a flattened field has none, its struct's \
                 own read takes its members from the owner's object",
            ));
        } else if !taken(info) {
            errors.push(Error::new(
                *span,
                "`schema::read_with` reads one member: an `UnknownMembers` field holds the \
                 members no field takes",
            ));
        }
    }
}

/// The refusals of `schema::members`: a field that shares its owner's object or holds its
/// unknown members, a key that words or shapes one member of the field's own name, and a field
/// with no value for an object that writes none of its members (no `default`, no `Option`).
fn validate_members(info: &FieldInfo, errors: &mut Vec<Error>) {
    let attrs = &info.attrs;
    let Some((_, span)) = &attrs.members else {
        return;
    };
    let one_member = attrs.flatten
        || !taken(info)
        || attrs.alias.is_some()
        || attrs.since_format.is_some()
        || attrs.or_kept.is_some()
        || attrs.reads_null.is_some()
        || attrs.required.is_some()
        || attrs.iterated.is_some()
        || attrs.drop.is_some()
        || attrs.keys.is_some()
        || attrs.finite.is_some()
        || [
            &attrs.wrong_type,
            &attrs.missing,
            &attrs.invalid,
            &attrs.negative,
        ]
        .iter()
        .any(|code| code.is_some());
    if one_member {
        errors.push(Error::new(
            *span,
            "`schema::members` feeds the field from several members, which its type's \
             `FromMembers` (or its `read_with`) reads: a key of one member of the field's own \
             name (`alias`, `since_format`, `or_kept`, `reads_null`, `required`, a list's or a \
             code's), `flatten` and an `UnknownMembers` field have none",
        ));
    } else if info.option.is_none() && attrs.default.is_none() {
        errors.push(Error::new(
            *span,
            "a field fed by `schema::members` is handed its value for a document that writes \
             none of them: give it a `default` (or make it an `Option`)",
        ));
    }
}

/// The refusals of `schema::drop`: a field that is no list, or one with its own read.
fn validate_drop(info: &FieldInfo, errors: &mut Vec<Error>) {
    let attrs = &info.attrs;
    let held = info.option.unwrap_or(info.ty);
    let Some(span) = attrs.drop else {
        return;
    };
    if ty::name(held).is_none_or(|name| name != "Vec") {
        errors.push(Error::new(
            span,
            "`schema::drop` reads a list: a `Vec` field (or an `Option` of one)",
        ));
    } else if attrs.read_with.is_some() {
        errors.push(Error::new(
            span,
            "a field with `schema::read_with` is read by its function, which reads the list its \
             own way",
        ));
    }
}

/// The refusals of `schema::keys`: a field that is no map, or one with its own read.
fn validate_keys(info: &FieldInfo, errors: &mut Vec<Error>) {
    let attrs = &info.attrs;
    let held = info.option.unwrap_or(info.ty);
    if let Some((_, span)) = &attrs.keys {
        if ty::map_values(held).is_none() {
            errors.push(Error::new(
                *span,
                "`schema::keys` is the rule of a map's member names: an `IndexMap<String, V>` \
                 field (or an `Option` of one)",
            ));
        } else if attrs.read_with.is_some() {
            errors.push(Error::new(
                *span,
                "a field with `schema::read_with` is read by its function, which reads the \
                 names its own way",
            ));
        }
    }
}

/// What the read emits for one field it takes by name: its slot, its match arm and the
/// finish that says what absence and failure make of it.
struct FieldRead {
    slot: TokenStream,
    arm: TokenStream,
    /// The keep of a value it could not read (`schema::or_kept`), after the members.
    kept: Option<TokenStream>,
    finish: TokenStream,
}

/// The read of the field `info`, its value bound to `__v{index}`. The calls that need the
/// field's type to have a read are spanned at the type, so a type without one is the error
/// there.
fn field_read(info: &FieldInfo, index: usize, extra_arg: &TokenStream) -> FieldRead {
    let reader = paths().reader;
    let slot = format_ident!("__s{index}");
    let rules = member_rules(info);
    let value = format_ident!("__v{index}");
    let held = info.option.unwrap_or(info.ty);
    let read_call = member_read(info, &slot);
    let json = &info.json;
    let names = info
        .attrs
        .alias
        .iter()
        .fold(quote!(#json), |names, alias| quote!(#names | #alias));
    let finish = if info.option.is_some() && info.attrs.required.is_none() {
        quote!(let #value = #slot.optional(__read, &mut __ok);)
    } else if let Some(default) = &info.attrs.default {
        let value_expr = default.value.as_ref().map_or_else(
            || quote!(::core::default::Default::default()),
            |expr| super::typed(expr, info.ty),
        );
        quote!(let #value = #slot.or_default(__read, &mut __ok, || #value_expr);)
    } else {
        // A plain field, or an `Option` the document must write (`schema::required`): its value
        // is the field's own, `Some(..)` for the `Option`.
        quote!(let #value = #slot.required(__object, #json, __read, &mut __ok);)
    };
    let new_slot = rules.map_or_else(
        || quote!(#reader::Slot::<#held>::new()),
        |rules| {
            quote! {{
                static __RULES: #reader::MemberRules = #rules;
                #reader::Slot::<#held>::ruled(&__RULES)
            }}
        },
    );
    // A required field with one name tells its absence where a read in declared order asks for
    // the member.
    let required = !(info.option.is_some() && info.attrs.required.is_none())
        && info.attrs.default.is_none()
        && info.attrs.alias.is_none();
    let when_absent = if required {
        quote!(#slot.missing_now(__object, #json, __read);)
    } else {
        quote!()
    };
    FieldRead {
        slot: quote!(let mut #slot = #new_slot;),
        // A field that reads a `null` as a value reads it, whatever the reading does with a
        // `null` member of the others (it is not absent for it).
        arm: if info.attrs.reads_null.is_some() {
            quote! {
                #names => {
                    let ::core::option::Option::Some(__member) = __member else {
                        #when_absent
                        return true;
                    };
                    #read_call
                    true
                }
            }
        } else {
            quote! {
                #names => {
                    let ::core::option::Option::Some(__member) = __member else {
                        #when_absent
                        return true;
                    };
                    if !__object.absent(__member, __read) {
                        #read_call
                    }
                    true
                }
            }
        },
        kept: info.attrs.or_kept.map(|_| {
            quote! {
                if let ::core::option::Option::Some(__kept) = #slot.take_kept() {
                    __object.keep(#json, __kept, __read, #extra_arg);
                }
            }
        }),
        finish,
    }
}

/// The read of a field fed by several members of the object (`schema::members`), its value
/// bound to `__v{index}`: read once, where the first of its names is asked for (or met in the
/// file), else after the members, by its `read_with` or its type's `FromMembers`, given each
/// member as written and the field's default. Its arm is not its own: a member may feed several
/// fields, so the arms are by name ([`member_arms`]), each reading every field it feeds that is
/// not read yet; `.1` is the field's slot and its read, for them.
fn members_read(
    info: &FieldInfo,
    index: usize,
    names: &[String],
) -> (FieldRead, (Ident, TokenStream)) {
    let core = core();
    let slot = format_ident!("__s{index}");
    let value = format_ident!("__v{index}");
    let ty = info.ty;
    let count = names.len();
    let default = match &info.attrs.default {
        Some(DefaultDecl {
            value: Some(expr), ..
        }) => super::typed(expr, ty),
        Some(_) => quote!(<#ty as ::core::default::Default>::default()),
        None => quote!(::core::option::Option::None),
    };
    let with = info.attrs.read_with.as_ref().map_or_else(
        || quote_spanned!(ty.span()=> <#ty as #core::schema::FromMembers<#count>>::from_members),
        |(with, _)| quote!(#with),
    );
    let read_now = quote! {{
        let __members = [#(__object.member(#names)),*];
        (#with)(__object.node(), &__members, #default, __read)
    }};
    let read = FieldRead {
        slot: quote! {
            let mut #slot: ::core::option::Option<::core::option::Option<#ty>> =
                ::core::option::Option::None;
        },
        arm: TokenStream::new(),
        kept: None,
        finish: quote! {
            let #value = match #slot {
                ::core::option::Option::Some(__read_value) => __read_value,
                ::core::option::Option::None => #read_now,
            };
            if #value.is_none() {
                __ok = false;
            }
        },
    };
    (read, (slot.clone(), read_now))
}

/// The arms of the members that feed fields (`schema::members`): one per name, reading each
/// field it feeds that is not read yet, in declaration order. `fed` is each such field's names,
/// slot and read.
fn member_arms(fed: &[(&[String], Ident, TokenStream)]) -> Vec<TokenStream> {
    let mut names: Vec<&String> = Vec::new();
    for (own, _, _) in fed {
        for name in *own {
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
        .into_iter()
        .map(|name| {
            let reads =
                fed.iter()
                    .filter(|(own, _, _)| own.contains(name))
                    .map(|(_, slot, read_now)| {
                        quote! {
                            if #slot.is_none() {
                                #slot = ::core::option::Option::Some(#read_now);
                            }
                        }
                    });
            quote! {
                #name => {
                    #(#reads)*
                    true
                }
            }
        })
        .collect()
}

/// The call that reads the member of field `info` into its slot: the field's `read_with`, the
/// read of a list it iterates (`schema::iterated`), or its type's; a value it
/// cannot read kept as written where it says `schema::or_kept`.
fn member_read(info: &FieldInfo, slot: &Ident) -> TokenStream {
    let reader = paths().reader;
    let held = info.option.unwrap_or(info.ty);
    let field_rule = unknown_option(info.attrs.unknown.as_ref());
    let p = paths();
    let (node, read) = (&p.node, &p.read);
    // What the type says of its `null`. A `schema::read_with` fn never sees one (the null rule
    // reads it first), so the type's `HOLDS_NULL` is not for it: a member that fn reads refuses a
    // `null` as every other member does.
    let kind = if info.attrs.read_with.is_some() {
        quote! {{
            #[allow(unused_imports)]
            use #reader::{KindOfNone as _, KindOfRead as _};
            #reader::Kind { holds_null: false, ..(&#reader::Probe::<#held>::NEW).kind() }
        }}
    } else {
        quote! {{
            #[allow(unused_imports)]
            use #reader::{KindOfNone as _, KindOfRead as _};
            (&#reader::Probe::<#held>::NEW).kind()
        }}
    };
    // The field's own read: its `read_with`, a list it iterates, or none (its type's).
    let iterated = info.attrs.iterated.is_some();
    let dropping = info.attrs.drop.is_some();
    let list = (iterated || dropping)
        .then(|| ty::elements(held))
        .flatten()
        .map(|element| {
            quote! {
                |__node: &#node<'_>, __read: &mut #read<'_>| {
                    #reader::list::<#element>(__node, __read, #iterated, #dropping)
                }
            }
        });
    // A map read by its key rule: each name its pattern takes, the value its type's read.
    let keyed = info.attrs.keys.as_ref().and_then(|(keys, _)| {
        let value = ty::map_values(held)?;
        let core = core();
        Some(quote_spanned! {value.span()=>
            |__node: &#node<'_>, __read: &mut #read<'_>| {
                #core::schema::read_keyed(
                    __node,
                    __read,
                    &#keys,
                    <#value as #core::schema::FromJson>::from_json,
                )
            }
        })
    });
    let with = info
        .attrs
        .read_with
        .as_ref()
        .map(|(with, _)| quote!(#with))
        .or(list)
        .or(keyed);
    match (info.attrs.or_kept, with, info.option.is_some()) {
        // A value it cannot read is kept as written.
        (Some((told, _)), with, _) => {
            let with = with.unwrap_or_else(|| {
                quote_spanned!(held.span()=> <#held as ::bedrock_addon_core::schema::FromJson>::from_json)
            });
            let reads_null = info.attrs.reads_null.is_some();
            let ends = info.attrs.ends.is_some();
            quote!(#slot.read_or_kept(__member, __read, #field_rule, #kind, #told, #reads_null, #ends, #with);)
        }
        // An iterated list reads a `null` as a value that is no array, a `reads_null` field as
        // a value.
        (None, Some(with), true) if !iterated && info.attrs.reads_null.is_none() => {
            quote!(#slot.read_option_with(__member, __read, #field_rule, #kind, #with);)
        }
        (None, Some(with), _) => quote!(#slot.read_with(__member, __read, #field_rule, #with);),
        (None, None, true) if info.attrs.reads_null.is_none() => quote_spanned! {held.span()=>
            ::bedrock_addon_core::schema::derive_support::Slot::<#held>::read_option(&mut #slot, __member, __read, #field_rule);
        },
        (None, None, _) => quote_spanned! {held.span()=>
            ::bedrock_addon_core::schema::derive_support::Slot::<#held>::read(&mut #slot, __member, __read, #field_rule);
        },
    }
}

/// A format version's tokens.
fn format_version(version: crate::attr::Version) -> TokenStream {
    let crate::attr::Version {
        major,
        minor,
        patch,
    } = version;
    quote!(::bedrock_addon_core::version::FormatVersion::new(#major, #minor, #patch))
}

/// The field's rules for its member (`schema::wrong_type`, `missing`, `invalid`, `finite`), a
/// `MemberRules` value; `None` when it states none.
fn member_rules(info: &FieldInfo) -> Option<TokenStream> {
    let attrs = &info.attrs;
    if attrs.wrong_type.is_none()
        && attrs.missing.is_none()
        && attrs.invalid.is_none()
        && attrs.negative.is_none()
        && attrs.finite.is_none()
    {
        return None;
    }
    let reader = paths().reader;
    let code = |code: &Option<(syn::Expr, proc_macro2::Span)>| {
        code.as_ref().map_or_else(
            || quote!(::core::option::Option::None),
            |(expr, _)| quote!(::core::option::Option::Some(#reader::MemberCode::of(&#expr))),
        )
    };
    let (wrong_type, missing, invalid, negative) = (
        code(&attrs.wrong_type),
        code(&attrs.missing),
        code(&attrs.invalid),
        code(&attrs.negative),
    );
    let finite = attrs.finite.is_some();
    Some(quote! {
        #reader::MemberRules {
            wrong_type: #wrong_type,
            missing: #missing,
            invalid: #invalid,
            negative: #negative,
            finite: #finite,
        }
    })
}

/// `Some(&mut __extra)` where the fields have an `UnknownMembers` field, else `None`.
fn extra_arg(extra: bool) -> TokenStream {
    if extra {
        quote!(::core::option::Option::Some(&mut __extra))
    } else {
        quote!(::core::option::Option::None)
    }
}

/// The disposal of an unknown member `__key`: the struct's, and for the members it says are
/// `quiet` the same members kept and told nothing.
fn disposal(quiet: &[String]) -> TokenStream {
    if quiet.is_empty() {
        return quote!(__disposal);
    }
    quote!(if ::core::matches!(__key, #(#quiet)|*) {
        __disposal.silenced()
    } else {
        __disposal
    })
}

/// The value a struct's read builds from its fields' values `__v<index>`: each field's own, the
/// kept unknown members, the object as written, or the names the object wrote.
fn built_value(construct: &TokenStream, fields: &Fields, infos: &[FieldInfo]) -> TokenStream {
    let own_names: Vec<&String> = infos
        .iter()
        .filter(|info| taken(info) && !info.inline)
        .map(|info| &info.json)
        .collect();
    let inits = fields.iter().enumerate().map(|(index, field)| {
        let member = member_of(field, index);
        let value = format_ident!("__v{index}");
        let built = match infos.iter().find(|info| info.member == member) {
            Some(info) if !taken(info) => quote!(__extra),
            Some(info) if info.attrs.members.is_some() => quote!(#value?),
            Some(info) if info.option.is_some() => quote!(#value),
            Some(_) => quote!(#value?),
            None => untaken_value(field, &own_names),
        };
        quote!(#member: #built)
    });
    quote!(#construct { #(#inits),* })
}

/// The body of a read of `fields` from `__object`, evaluating to `Option<#construct ..>`, with
/// `__object`, `__read`, `__unknown` and `__tag` in scope (`Fields::read_fields`' parameters);
/// `knows` is the function that names the fields' members.
#[allow(
    clippy::too_many_lines,
    reason = "one arm per kind of field and variant"
)]
fn fields_body(
    construct: &TokenStream,
    fields: &Fields,
    infos: &[FieldInfo],
    rule: Option<&String>,
    quiet: &[String],
    after_members: Option<&syn::Expr>,
) -> TokenStream {
    let p = paths();
    let (fields_trait, reader, node, read) = (&p.fields, &p.reader, &p.node, &p.read);
    let rule = unknown_option(rule);
    let marked = marked(fields);
    let extra = has_extra(fields);
    let index_of = |info: &FieldInfo| {
        fields
            .iter()
            .enumerate()
            .position(|(index, field)| member_of(field, index) == info.member)
            .unwrap_or_default()
    };
    let extra_arg = extra_arg(extra);
    let (mut slots, mut arms, mut finishes, mut flattened, mut flat_skips) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut flat_slots, mut flat_arms) = (Vec::new(), Vec::new());
    let mut kept = Vec::new();
    let mut fed = Vec::new();
    for info in infos.iter().filter(|info| taken(info)) {
        let index = index_of(info);
        if info.inline {
            let (ty, value) = (info.ty, format_ident!("__v{index}"));
            let slot = format_ident!("__f{index}");
            // An internally tagged enum is read where the object has its tag (where a read in
            // declared order asks for it), a struct after the members.
            flat_slots.push(quote_spanned! {ty.span()=>
                let mut #slot: ::core::option::Option<::core::option::Option<#ty>> =
                    ::core::option::Option::None;
            });
            flat_arms.push(quote_spanned! {ty.span()=>
                _ if ::core::option::Option::Some(__key) == <#ty as #fields_trait>::TAG => {
                    if #slot.is_none() {
                        #slot = ::core::option::Option::Some(
                            <#ty as #fields_trait>::read_fields(__object, __read, false, __tag),
                        );
                    }
                    true
                }
            });
            flattened.push(quote_spanned! {ty.span()=>
                let #value = #reader::flattened(
                    match #slot {
                        ::core::option::Option::Some(__read_flat) => __read_flat,
                        ::core::option::Option::None => {
                            <#ty as #fields_trait>::read_fields(__object, __read, false, __tag)
                        }
                    },
                    &mut __ok,
                );
            });
            flat_skips
                .push(quote!(_ if <#ty as #fields_trait>::knows_given(__object, __key) => true,));
            continue;
        }
        let FieldRead {
            slot,
            arm,
            kept: keep,
            finish,
        } = match &info.attrs.members {
            Some((names, _)) => {
                let (read, (slot, read_now)) = members_read(info, index, names);
                fed.push((names.as_slice(), slot, read_now));
                read
            }
            None => field_read(info, index, &extra_arg),
        };
        slots.push(slot);
        arms.push(arm);
        kept.extend(keep);
        finishes.push(finish);
    }
    arms.extend(member_arms(&fed));
    let extra_decl = extra.then(|| {
        let core = core();
        quote!(let mut __extra = #core::document::UnknownMembers::default();)
    });
    let build = built_value(construct, fields, infos);
    let (names, gates) = member_names_and_gates(infos);
    let knows = knows(infos, true);
    let unknown_member = unknown_member(&extra_arg, quiet);
    // A flattened enum whose variant failed (a tag that names none) fails the read of its owner
    // there: the members after it, the unknown ones and the missing ones are not told.
    let abandoned = infos
        .iter()
        .filter(|info| info.inline && taken(info))
        .map(|info| {
            let slot = format_ident!("__f{}", index_of(info));
            quote!(|| ::core::matches!(#slot, ::core::option::Option::Some(::core::option::Option::None)))
        })
        // A member that `ends` the read, once it failed or was told missing.
        .chain(
            infos
                .iter()
                .filter(|info| info.attrs.ends.is_some() && taken(info) && !info.inline)
                .map(|info| {
                    let slot = format_ident!("__s{}", index_of(info));
                    quote!(|| #slot.ended())
                }),
        )
        .collect::<Vec<_>>();
    let (give_up, unknown_last, abandon) = if abandoned.is_empty() {
        (quote!(), quote!(), quote!())
    } else {
        (
            quote!(if false #(#abandoned)* { return true; }),
            quote!(let __abandoned = false #(#abandoned)*;),
            quote!(if __abandoned {
                __object.abandon();
                __ok = false;
            }),
        )
    };
    let unknown_after = if abandoned.is_empty() {
        quote!(#reader::read_unknown_last(__object, __read, __unknown, &__declared, #unknown_member);)
    } else {
        quote! {
            if !__abandoned {
                #reader::read_unknown_last(__object, __read, __unknown, &__declared, #unknown_member);
            }
        }
    };
    let ending = match after_members {
        None => quote! {
            #unknown_after
            // Under the game's schema tree an error raised while the members were read fails the
            // struct, and its missing members are then not told.
            let mut __ok = __object.settle(__read);
            #abandon
            #(#flattened)*
            #(#finishes)*
            if !__ok {
                return ::core::option::Option::None;
            }
            ::core::option::Option::Some(#build)
        },
        Some(rule) => {
            // The value is built and its rule run before the unknown members told after the
            // members, which the value's own field then takes.
            let (later_decl, later_arg, later_take) = match extra_member(infos) {
                Some(member) => {
                    let core = core();
                    (
                        quote!(let mut __later = #core::document::UnknownMembers::default();),
                        quote!(::core::option::Option::Some(&mut __later)),
                        quote! {
                            if let ::core::option::Option::Some(__built) = &mut __built {
                                __built.#member.extend(__later.iter().map(|(__key, __value)| {
                                    (
                                        ::std::borrow::ToOwned::to_owned(__key),
                                        ::core::clone::Clone::clone(__value),
                                    )
                                }));
                            }
                        },
                    )
                }
                None => (quote!(), quote!(::core::option::Option::None), quote!()),
            };
            let later_member = self::unknown_member(&later_arg, quiet);
            let unknown_after = if abandoned.is_empty() {
                quote!(#reader::read_unknown_last(__object, __read, __unknown, &__declared, #later_member);)
            } else {
                quote! {
                    if !__abandoned {
                        #reader::read_unknown_last(__object, __read, __unknown, &__declared, #later_member);
                    }
                }
            };
            quote! {
                let mut __ok = __object.settle(__read);
                #abandon
                #(#flattened)*
                #(#finishes)*
                let mut __built = if __ok {
                    (|| ::core::option::Option::Some(#build))()
                } else {
                    ::core::option::Option::None
                };
                if let ::core::option::Option::Some(__built) = &mut __built {
                    (#rule)(__built, __object.node(), __read);
                }
                #later_decl
                #unknown_after
                #later_take
                // Under the game's schema tree an error raised by an unknown member fails it too.
                if !__object.settle(__read) {
                    return ::core::option::Option::None;
                }
                __built
            }
        }
    };
    quote! {
        let __disposal = __object.disposal(__read, #rule, #marked);
        #extra_decl
        #(#slots)*
        #(#flat_slots)*
        let __declared = #reader::Declared {
            names: &[#(#names),*],
            gates: &[#(#gates),*],
            knows: |__key: &str| -> bool { #knows },
            tag: __tag,
        };
        #reader::read_members(
            __object,
            __read,
            __unknown,
            &__declared,
            |__key: &str, __member: ::core::option::Option<&#node<'_>>, __read: &mut #read<'_>| -> bool {
                #give_up
                match __key {
                    #(#arms)*
                    _ if ::core::option::Option::Some(__key) == __tag => true,
                    #(#flat_arms)*
                    #(#flat_skips)*
                    _ => false,
                }
            },
            #unknown_member,
        );
        // What the members could not read is kept before the unknown members told after them.
        #(#kept)*
        #unknown_last
        #ending
    }
}

/// The tell of one unknown member, given to the read twice: for the unknown members told with the
/// members and for the ones told after the keeps of what the members could not read.
fn unknown_member(extra_arg: &TokenStream, quiet: &[String]) -> TokenStream {
    let p = paths();
    let (node, read) = (&p.node, &p.read);
    let disposal = disposal(quiet);
    quote! {
        |__key: &str, __member: &#node<'_>, __read: &mut #read<'_>| {
            __object.unknown(__key, __member, __read, #disposal, #extra_arg);
        }
    }
}

/// The members a declaration-order read asks for, in declaration order, and the ones a struct
/// reads from a format version on. A flattened type is asked for by its tag, where it declares
/// it (an empty name for a type with none, which is read after the members).
fn member_names_and_gates(infos: &[FieldInfo<'_>]) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let (reader, fields) = (paths().reader, paths().fields);
    let own = || infos.iter().filter(|info| taken(info));
    let names = own()
        .flat_map(|info| {
            if info.inline {
                let ty = info.ty;
                return vec![quote_spanned!(ty.span()=> #reader::tag_name(<#ty as #fields>::TAG))];
            }
            json_names(info)
                .into_iter()
                .map(|name| quote!(#name))
                .collect()
        })
        .collect();
    let gates = own()
        .filter(|info| !info.inline)
        .filter_map(|info| {
            let version = format_version(info.attrs.since_format?);
            let names = std::iter::once(&info.json).chain(info.attrs.alias.as_ref());
            Some(quote!(#((#names, #version)),*))
        })
        .collect();
    (names, gates)
}

/// The type parameters a struct flattens: they share its object, so they are read as `Fields`.
fn flattened_parameters<'a>(infos: &'a [FieldInfo<'_>]) -> Vec<&'a Ident> {
    infos
        .iter()
        .filter(|info| info.attrs.flatten)
        .filter_map(|info| match info.ty {
            Type::Path(path) if path.qself.is_none() => path.path.get_ident(),
            _ => None,
        })
        .collect()
}

/// The call of the struct's `schema::after_read` rule on the value just read, if it has one.
fn after_read(container: &ContainerAttrs) -> Option<TokenStream> {
    let (after, _) = container.after_read.as_ref()?;
    Some(quote! {
        if let ::core::option::Option::Some(__built) = &mut __value {
            (#after)(__built, __node, __read);
        }
    })
}

/// The read of a struct: `FromJson` (the object, then its fields) and `Fields`; a transparent
/// newtype reads as its one field.
#[allow(
    clippy::too_many_lines,
    reason = "one arm per kind of field and variant"
)]
pub(super) fn struct_read(
    target: &Target,
    fields: &Fields,
    container: &ContainerAttrs,
    infos: &[FieldInfo],
) -> TokenStream {
    let Paths {
        node,
        read,
        from_json,
        fields: fields_trait,
        object,
        reader,
        expected,
    } = paths();
    let ident = target.ident;
    let bound = quote!(#from_json);
    let flat = flattened_parameters(infos);
    let from_json_impl = target.impl_flat(&from_json, &bound, &flat, &fields_trait);
    let fields_impl = target.impl_flat(&fields_trait, &bound, &flat, &fields_trait);
    if container.transparent {
        let Some(info) = infos.first() else {
            return TokenStream::new();
        };
        let ty = info.ty;
        // A list the field iterates or drops from is read as its keys say.
        let iterated = info.attrs.iterated.is_some();
        let dropping = info.attrs.drop.is_some();
        let list = (iterated || dropping).then(|| ty::elements(ty)).flatten();
        // A field read by its own function takes no wording or `null` of the type's.
        let (inner, kind) = match (&info.attrs.read_with, list) {
            (None, Some(element)) => (
                quote_spanned!(ty.span()=> #reader::list::<#element>(__node, __read, #iterated, #dropping)),
                // An iterated list takes any value.
                if iterated {
                    quote! {
                        const EXPECTED: ::core::option::Option<#expected> = ::core::option::Option::None;
                    }
                } else {
                    quote_spanned! {ty.span()=>
                        const EXPECTED: ::core::option::Option<#expected> = <#ty as #from_json>::EXPECTED;
                    }
                },
            ),
            (Some((with, _)), _) => (
                quote!((#with)(__node, __read)),
                quote! {
                    const EXPECTED: ::core::option::Option<#expected> = ::core::option::Option::None;
                },
            ),
            (None, None) => (
                quote_spanned!(ty.span()=> <#ty as #from_json>::from_json(__node, __read)),
                quote_spanned! {ty.span()=>
                    const EXPECTED: ::core::option::Option<#expected> = <#ty as #from_json>::EXPECTED;
                    const HOLDS_NULL: bool = <#ty as #from_json>::HOLDS_NULL;
                },
            ),
        };
        // The word of the field (`schema::unknown`) is for the value the newtype holds.
        let inner = if info.attrs.unknown.is_some() {
            let rule = unknown_option(info.attrs.unknown.as_ref());
            quote!(#reader::under_word(__read, #rule, |__read| #inner)?)
        } else {
            quote!(#inner?)
        };
        let build = match &info.member {
            Member::Named(name) => quote!(Self { #name: #inner }),
            Member::Unnamed(_) => quote!(Self(#inner)),
        };
        return quote! {
            #[automatically_derived]
            #from_json_impl {
                #kind

                fn from_json(__node: &#node<'_>, __read: &mut #read<'_>) -> ::core::option::Option<Self> {
                    ::core::option::Option::Some(#build)
                }
            }
        };
    }
    let knows = knows(infos, false);
    // A struct that flattens another asks it what its object's members are (an internally
    // tagged enum knows the variant it read).
    let knows_given = infos.iter().any(|info| info.inline).then(|| {
        let given = self::knows(infos, true);
        quote! {
            fn knows_given(__object: &#object<'_>, __key: &str) -> bool {
                #given
            }
        }
    });
    let after_read = after_read(container);
    let body = fields_body(
        &quote!(#ident),
        fields,
        infos,
        container.unknown.as_ref(),
        &container.quiet,
        container.after_members.as_ref().map(|(rule, _)| rule),
    );
    quote! {
        #[automatically_derived]
        #from_json_impl {
            const EXPECTED: ::core::option::Option<#expected> =
                ::core::option::Option::Some(#expected::Object);

            fn from_json(__node: &#node<'_>, __read: &mut #read<'_>) -> ::core::option::Option<Self> {
                let __object = #object::open(__node, __read)?;
                let mut __value = <Self as #fields_trait>::read_fields(
                    &__object,
                    __read,
                    true,
                    ::core::option::Option::None,
                );
                __object.close(__read);
                #after_read
                __value
            }
        }

        #[automatically_derived]
        #fields_impl {
            fn knows(__key: &str) -> bool {
                #knows
            }

            #knows_given

            #[allow(unused_variables, unused_mut, clippy::too_many_lines)]
            fn read_fields(
                __object: &#object<'_>,
                __read: &mut #read<'_>,
                __unknown: bool,
                __tag: ::core::option::Option<&'static str>,
            ) -> ::core::option::Option<Self> {
                #body
            }
        }
    }
}

/// The refusals of a tuple struct's fields, each read from the element at its place: no field
/// that would move the places (`skip`) or names a member (`flatten`, `rename`, `alias`, an
/// `UnknownMembers` field), none of the keys about a member's presence; under `schema::arity`
/// each field says what a missing element is (an `Option`, or a `default`).
pub(super) fn validate_positional(
    fields: &Fields,
    container: &ContainerAttrs,
    infos: &[FieldInfo],
    errors: &mut Vec<Error>,
) {
    for field in fields {
        let mut scratch = Vec::new();
        if FieldAttrs::read(&field.attrs, &mut scratch).skip {
            errors.push(Error::new_spanned(
                field,
                "a tuple struct reads each field from the element at its place: a `skip` field \
                 would move the places of the ones after it",
            ));
        }
    }
    for info in infos {
        let attrs = &info.attrs;
        let named =
            attrs.flatten || attrs.rename.is_some() || attrs.alias.is_some() || !taken(info);
        let presence = [
            attrs.or_kept.map(|(_, span)| span),
            attrs.reads_null,
            attrs.required,
            attrs.missing.as_ref().map(|(_, span)| *span),
            attrs.stored.filter(|_| attrs.since_format.is_some()),
        ]
        .into_iter()
        .flatten()
        .next();
        if named {
            errors.push(Error::new_spanned(
                info.ty,
                "a tuple struct's field is the element at its place: it has no member name to \
                 rename, alias or flatten, and no unknown members",
            ));
        }
        if let Some(span) = presence {
            errors.push(Error::new(
                span,
                "a tuple struct's element is there by its array's length: `or_kept`, \
                 `reads_null`, `required`, `missing` and `since_format` are about a member",
            ));
        }
        if container.arity.is_some() && info.option.is_none() && attrs.default.is_none() {
            errors.push(Error::new_spanned(
                info.ty,
                "under `schema::arity` an element the array lacks is absent: the field is an \
                 `Option` or has a `default`",
            ));
        }
    }
}

/// The read of a tuple struct: an array (else a wrong type, under the struct's
/// `schema::wrong_type`), each field the element at its place, read as a member of its field
/// is; another length a wrong type, or under `schema::arity` told after the elements and read
/// on. A failed element fails the struct where its field does not reset; the struct's
/// `after_read` runs on the value built.
pub(super) fn tuple_read(
    target: &Target,
    container: &ContainerAttrs,
    infos: &[FieldInfo],
) -> TokenStream {
    let Paths {
        node,
        read,
        from_json,
        fields: fields_trait,
        reader,
        expected,
        ..
    } = paths();
    let from_json_impl = target.impl_flat(&from_json, &quote!(#from_json), &[], &fields_trait);
    let count = infos.len();
    let wrong_type = container.wrong_type.as_ref().map_or_else(
        || quote!(::core::option::Option::None),
        |(code, _)| quote!(::core::option::Option::Some(&#code)),
    );
    let (length, tell_length) = match &container.arity {
        Some((code, _)) => (
            quote!(let __other = __elements.len() != #count;),
            quote! {
                if __other && !__read.stopped() {
                    #reader::tell_at(__node, &#code, __read);
                }
            },
        ),
        None => (
            quote! {
                if __elements.len() != #count {
                    __read.wrong_type(__node);
                    return ::core::option::Option::None;
                }
            },
            TokenStream::new(),
        ),
    };
    let reads = infos.iter().enumerate().map(|(index, info)| {
        let slot = format_ident!("__s{index}");
        let value = format_ident!("__v{index}");
        let held = info.option.unwrap_or(info.ty);
        let new_slot = member_rules(info).map_or_else(
            || quote!(#reader::Slot::<#held>::new()),
            |rules| {
                quote! {{
                    static __RULES: #reader::MemberRules = #rules;
                    #reader::Slot::<#held>::ruled(&__RULES)
                }}
            },
        );
        let read_call = member_read(info, &slot);
        let finish = if info.option.is_some() {
            quote!(let #value = #slot.optional(__read, &mut __ok);)
        } else if let Some(default) = &info.attrs.default {
            let default = default.value.as_ref().map_or_else(
                || quote!(::core::default::Default::default()),
                |expr| super::typed(expr, info.ty),
            );
            quote!(let #value = #slot.or_default(__read, &mut __ok, || #default);)
        } else {
            quote!(let #value = #slot.element(&mut __ok);)
        };
        quote! {
            let mut #slot = #new_slot;
            if let ::core::option::Option::Some(__member) = __elements.element(#index) {
                let __member = &__member;
                #read_call
            }
            #finish
        }
    });
    let build = infos.iter().enumerate().map(|(index, info)| {
        let value = format_ident!("__v{index}");
        if info.option.is_some() {
            quote!(#value)
        } else {
            quote!(#value?)
        }
    });
    let after_read = after_read(container);
    quote! {
        #[automatically_derived]
        #from_json_impl {
            const EXPECTED: ::core::option::Option<#expected> =
                ::core::option::Option::Some(#expected::Array);

            #[allow(unused_mut)]
            fn from_json(__node: &#node<'_>, __read: &mut #read<'_>) -> ::core::option::Option<Self> {
                let __elements = #reader::elements(__node, __read, #wrong_type)?;
                #length
                let mut __ok = true;
                #(#reads)*
                #tell_length
                if !__ok {
                    return ::core::option::Option::None;
                }
                let mut __value = ::core::option::Option::Some(Self(#(#build),*));
                #after_read
                __value
            }
        }
    }
}

/// The read of a string enum: by name, through its `from_name`; from a string and nothing
/// else when `string_only`.
pub(super) fn unit_enum_read(ident: &Ident, string_only: bool) -> TokenStream {
    let Paths {
        node,
        read,
        from_json,
        reader,
        expected,
        ..
    } = paths();
    quote! {
        #[automatically_derived]
        impl #from_json for #ident {
            const EXPECTED: ::core::option::Option<#expected> =
                ::core::option::Option::Some(#expected::String);

            fn from_json(__node: &#node<'_>, __read: &mut #read<'_>) -> ::core::option::Option<Self> {
                #reader::unit_enum(__node, __read, #string_only, Self::from_name)
            }
        }
    }
}

/// One variant of an enum with data, for its read.
pub(super) struct ReadVariant<'a> {
    pub variant: &'a Variant,
    /// The JSON name (tagged enums).
    pub name: String,
    pub fields: &'a [FieldInfo<'a>],
    /// A struct variant's `schema::unknown`.
    pub unknown: Option<String>,
}

/// What a variant's payload is.
enum Payload<'a> {
    Unit,
    /// One value: its type.
    Newtype(&'a Type, Option<&'a syn::Expr>),
    /// Named members.
    Struct,
}

fn payload<'a>(variant: &ReadVariant<'a>) -> Payload<'a> {
    match &variant.variant.fields {
        Fields::Unit => Payload::Unit,
        Fields::Unnamed(_) => match variant.fields.first() {
            Some(info) => {
                Payload::Newtype(info.ty, info.attrs.read_with.as_ref().map(|(with, _)| with))
            }
            None => Payload::Unit,
        },
        Fields::Named(_) => Payload::Struct,
    }
}

/// The local function that reads struct variant `index` of `ident` from an object (a variant
/// has no `Fields` impl, so its `knows` is a closure).
fn variant_fn(target: &Target, index: usize, variant: &ReadVariant) -> TokenStream {
    let Paths {
        read,
        object,
        from_json,
        ..
    } = paths();
    let ident = target.ident;
    let ty = target.ty();
    let (generics, where_clause) = target.fn_generics(&from_json);
    let v = &variant.variant.ident;
    let name = format_ident!("__variant_{index}");
    let body = fields_body(
        &quote!(#ident::#v),
        &variant.variant.fields,
        variant.fields,
        variant.unknown.as_ref(),
        &[],
        None,
    );
    quote! {
        #[allow(unused_variables, unused_mut, clippy::too_many_lines)]
        fn #name #generics(
            __object: &#object<'_>,
            __read: &mut #read<'_>,
            __unknown: bool,
            __tag: ::core::option::Option<&'static str>,
        ) -> ::core::option::Option<#ty> #where_clause {
            #body
        }
    }
}

/// The read of a newtype variant's value at `at`.
fn newtype_value(ty: &Type, with: Option<&syn::Expr>, at: &TokenStream) -> TokenStream {
    let from_json = paths().from_json;
    with.map_or_else(
        || quote_spanned!(ty.span()=> <#ty as #from_json>::from_json(#at, __read)),
        |with| quote!((#with)(#at, __read)),
    )
}

/// The read of a struct variant from the object at `at`: open it, read the fields, close it.
fn struct_value(index: usize, at: &TokenStream) -> TokenStream {
    let object = paths().object;
    let name = format_ident!("__variant_{index}");
    quote! {{
        match #object::open(#at, __read) {
            ::core::option::Option::Some(__object) => {
                let __value = #name(&__object, __read, true, ::core::option::Option::None);
                __object.close(__read);
                __value
            }
            ::core::option::Option::None => ::core::option::Option::None,
        }
    }}
}

/// The read of an untagged enum with data: the arm the reader chooses
/// (`derive_support::untagged_arm`), then that variant's payload.
pub(super) fn untagged_read(
    target: &Target,
    invalid: Option<&(syn::Expr, proc_macro2::Span)>,
    variants: &[ReadVariant],
) -> TokenStream {
    let Paths {
        node,
        read,
        from_json,
        reader,
        ..
    } = paths();
    let core = core();
    let invalid = invalid_code(invalid);
    let fns = variants
        .iter()
        .enumerate()
        .filter(|(_, v)| matches!(payload(v), Payload::Struct))
        .map(|(index, v)| variant_fn(target, index, v));
    // The arm is chosen from the enum's shape, which only an enum facet describes does.
    let ty = target.ty();
    let header = target.impl_where(
        &from_json,
        Some(&from_json),
        Some(&quote!(#ty: #core::__private::Facet<'static>)),
    );
    let units: Vec<(usize, &ReadVariant)> = variants
        .iter()
        .enumerate()
        .filter(|(_, v)| matches!(payload(v), Payload::Unit))
        .collect();
    let unit_names = units.iter().map(|(_, v)| &v.name);
    let unit_arms = units.iter().enumerate().map(|(at, (_, v))| {
        let v = &v.variant.ident;
        quote!(::core::option::Option::Some(#at) => ::core::option::Option::Some(Self::#v),)
    });
    let at = quote!(__node);
    let arms = variants.iter().enumerate().map(|(index, v)| {
        let name = &v.variant.ident;
        let value = match payload(v) {
            Payload::Unit => quote!(::core::option::Option::Some(Self::#name)),
            Payload::Newtype(ty, with) => {
                let value = newtype_value(ty, with, &at);
                quote!(#value.map(Self::#name))
            }
            Payload::Struct => struct_value(index, &at),
        };
        quote!(::core::option::Option::Some(#reader::Arm::Variant(#index)) => #value,)
    });
    quote! {
        #[automatically_derived]
        #header {
            fn from_json(__node: &#node<'_>, __read: &mut #read<'_>) -> ::core::option::Option<Self> {
                #(#fns)*
                match #reader::untagged_arm::<Self>(__node, __read, #invalid) {
                    ::core::option::Option::Some(#reader::Arm::Unit) => match #reader::unit_arm(__node, &[#(#unit_names),*]) {
                        #(#unit_arms)*
                        _ => {
                            __read.wrong_type(__node);
                            ::core::option::Option::None
                        }
                    },
                    #(#arms)*
                    ::core::option::Option::None => ::core::option::Option::None,
                    #[allow(unreachable_patterns)]
                    _ => {
                        __read.wrong_type(__node);
                        ::core::option::Option::None
                    }
                }
            }
        }
    }
}

/// The read of an internally tagged enum (`tag = ".."`): the variant its tag member names (or
/// `tag_with` says), its members beside the tag. `Fields` is the read of the variant from an
/// object the enum may share with the struct that flattens it, `FromJson` opens the object and
/// tells the unknown members of the variant.
#[allow(
    clippy::too_many_lines,
    reason = "one arm per kind of field and variant"
)]
pub(super) fn tagged_read(
    target: &Target,
    tag: &str,
    invalid: Option<&(syn::Expr, proc_macro2::Span)>,
    tag_with: Option<&(syn::Expr, proc_macro2::Span)>,
    after_read: Option<&(syn::Expr, proc_macro2::Span)>,
    variants: &[ReadVariant],
) -> TokenStream {
    let Paths {
        node,
        read,
        fields: fields_trait,
        from_json,
        object,
        reader,
        expected,
    } = paths();
    let invalid = invalid_code(invalid);
    let with = tag_with.map_or_else(
        || quote!(::core::option::Option::None),
        |(with, _)| quote!(::core::option::Option::Some(#with)),
    );
    let fns = variants
        .iter()
        .enumerate()
        .filter(|(_, v)| matches!(payload(v), Payload::Struct | Payload::Unit))
        .map(|(index, v)| variant_fn(target, index, v));
    // The rule over the variant just read, with the object as written.
    let after = after_read.map(|(after, _)| {
        quote! {
            if let ::core::option::Option::Some(__built) = &mut __value {
                (#after)(__built, __object.node(), __read);
            }
        }
    });
    let from_json_header = target.impl_for(&from_json, Some(&from_json));
    let fields_header = target.impl_for(&fields_trait, Some(&from_json));
    // What the enum knows of an object's members: its tag, and those of every variant (the
    // variant read, once it is).
    let knowing: Vec<TokenStream> = variants.iter().map(|v| knows(v.fields, false)).collect();
    let names: Vec<&String> = variants.iter().map(|v| &v.name).collect();
    let arms = variants.iter().enumerate().map(|(index, v)| {
        let json = &v.name;
        let name = &v.variant.ident;
        let fn_name = format_ident!("__variant_{index}");
        let known = &knowing[index];
        let value = match payload(v) {
            Payload::Unit | Payload::Struct => {
                quote!(#fn_name(__object, __read, __unknown, ::core::option::Option::Some(#tag)))
            }
            Payload::Newtype(ty, _) => quote_spanned! {ty.span()=>
                <#ty as #fields_trait>::read_fields(__object, __read, __unknown, ::core::option::Option::Some(#tag))
                    .map(Self::#name)
            },
        };
        quote! {
            #json => {
                __object.choose(|__key: &str| -> bool { __key == #tag || #known });
                #value
            }
        }
    });
    quote! {
        #[automatically_derived]
        #from_json_header {
            const EXPECTED: ::core::option::Option<#expected> =
                ::core::option::Option::Some(#expected::Object);

            fn from_json(__node: &#node<'_>, __read: &mut #read<'_>) -> ::core::option::Option<Self> {
                let __object = #object::open(__node, __read)?;
                let __value = <Self as #fields_trait>::read_fields(
                    &__object,
                    __read,
                    true,
                    ::core::option::Option::None,
                );
                __object.close(__read);
                __value
            }
        }

        #[automatically_derived]
        #fields_header {
            const TAG: ::core::option::Option<&'static str> = ::core::option::Option::Some(#tag);

            const VARIANTS: &'static [&'static str] = &[#(#names),*];

            fn knows(__key: &str) -> bool {
                __key == #tag #(|| #knowing)*
            }

            fn knows_given(__object: &#object<'_>, __key: &str) -> bool {
                match __object.chosen_knows(__key) {
                    ::core::option::Option::Some(__known) => __known,
                    ::core::option::Option::None => <Self as #fields_trait>::knows(__key),
                }
            }

            #[allow(unused_variables, clippy::too_many_lines)]
            fn read_fields(
                __object: &#object<'_>,
                __read: &mut #read<'_>,
                __unknown: bool,
                __tag: ::core::option::Option<&'static str>,
            ) -> ::core::option::Option<Self> {
                #(#fns)*
                let __name = #reader::variant_of(__object, __read, #tag, #with)?;
                let mut __value = match __name.as_str() {
                    #(#arms)*
                    _ => {
                        #reader::unknown_tag(__object, __read, #invalid);
                        ::core::option::Option::None
                    }
                };
                #after
                __value
            }
        }
    }
}

/// The read of an externally tagged enum: a unit variant by its name, a variant with data as
/// `{name: payload}`.
pub(super) fn external_read(
    target: &Target,
    invalid: Option<&(syn::Expr, proc_macro2::Span)>,
    variants: &[ReadVariant],
) -> TokenStream {
    let Paths {
        node,
        read,
        from_json,
        reader,
        ..
    } = paths();
    let invalid = invalid_code(invalid);
    let fns = variants
        .iter()
        .enumerate()
        .filter(|(_, v)| matches!(payload(v), Payload::Struct))
        .map(|(index, v)| variant_fn(target, index, v));
    let header = target.impl_for(&from_json, Some(&from_json));
    let unit_arms = variants
        .iter()
        .filter(|v| matches!(payload(v), Payload::Unit))
        .map(|v| {
            let json = &v.name;
            let name = &v.variant.ident;
            quote!(#json => ::core::option::Option::Some(Self::#name),)
        });
    let at = quote!(&__payload);
    let payload_arms = variants
        .iter()
        .enumerate()
        .filter(|(_, v)| !matches!(payload(v), Payload::Unit))
        .map(|(index, v)| {
            let json = &v.name;
            let name = &v.variant.ident;
            let value = match payload(v) {
                Payload::Newtype(ty, with) => {
                    let value = newtype_value(ty, with, &at);
                    quote!(#value.map(Self::#name))
                }
                _ => struct_value(index, &at),
            };
            quote!(#json => #value,)
        });
    quote! {
        #[automatically_derived]
        #header {
            fn from_json(__node: &#node<'_>, __read: &mut #read<'_>) -> ::core::option::Option<Self> {
                #(#fns)*
                match #reader::external(__node, __read, #invalid)? {
                    #reader::External::Unit(__name) => match __name.as_str() {
                        #(#unit_arms)*
                        _ => {
                            #reader::unknown_variant(__node, __read, #invalid);
                            ::core::option::Option::None
                        }
                    },
                    #reader::External::Payload(__name, __payload) => match __name.as_str() {
                        #(#payload_arms)*
                        _ => {
                            #reader::unknown_variant(&__payload, __read, #invalid);
                            ::core::option::Option::None
                        }
                    },
                }
            }
        }
    }
}

/// The code of a value that names no variant (the enum's `schema::invalid`): `Some(&CODE)`, or
/// `None` for the read's own words.
fn invalid_code(invalid: Option<&(syn::Expr, proc_macro2::Span)>) -> TokenStream {
    invalid.map_or_else(
        || quote!(::core::option::Option::None),
        |(code, _)| quote!(::core::option::Option::Some(&#code)),
    )
}

/// The JSON name of a variant of a tagged enum: its `rename`, else the enum's `rename_all`.
pub(super) fn variant_name(
    variant: &Variant,
    rename: Option<&String>,
    container: &ContainerAttrs,
) -> String {
    json_name(&variant.ident.to_string(), rename, container)
}

/// The variants of a tagged enum with data, with the refusals of their shapes: a tuple of
/// several values has no JSON form; an internally tagged newtype variant holds a struct (its
/// members sit beside the tag), never a plain value.
pub(super) fn check_tagged_variants(data: &DataEnum, internal: bool, errors: &mut Vec<Error>) {
    for variant in &data.variants {
        if let Fields::Unnamed(unnamed) = &variant.fields {
            if unnamed.unnamed.len() != 1 {
                errors.push(Error::new_spanned(
                    unnamed,
                    "a variant of a tagged enum holds one value or named members: a tuple of \
                     several has no JSON form the derive reads",
                ));
            } else if internal && ty::is_plain_value(&unnamed.unnamed[0].ty) {
                errors.push(Error::new_spanned(
                    unnamed,
                    "a newtype variant of an internally tagged enum holds a struct, whose members \
                     sit beside the tag: a plain value has no member to sit there",
                ));
            } else if internal {
                let mut scratch = Vec::new();
                let attrs = FieldAttrs::read(&unnamed.unnamed[0].attrs, &mut scratch);
                if let Some((_, span)) = attrs.read_with {
                    errors.push(Error::new(
                        span,
                        "`schema::read_with` reads one member: the struct of an internally \
                         tagged newtype variant has none, its members sit beside the tag",
                    ));
                }
            }
        }
    }
}
