//! What `#[derive(Schema)]` emits for the read: `impl FromJson` (core's `schema::FromJson`) and,
//! for a struct, `impl Fields` (the read of its fields from an object it may share with the
//! struct that flattens it). The emitted code is a slot per field, a match arm per JSON name and
//! one call per field that says what absence and failure make of it; the rest is core's
//! `schema::__read`.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataEnum, Error, Fields, Ident, Member, Type, Variant};

use super::attr::{ContainerAttrs, FieldAttrs};
use super::{FieldInfo, core, json_name, ty, unknown_option};

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
        fields: quote!(#core::schema::__read::Fields),
        object: quote!(#core::schema::__read::Object),
        reader: quote!(#core::schema::__read),
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
/// `UnknownMembers` field, the object as written for a `Written` one.
fn untaken_value(field: &syn::Field) -> TokenStream {
    let mut scratch = Vec::new();
    let attrs = FieldAttrs::read(&field.attrs, &mut scratch);
    let ty = &field.ty;
    if named(ty, "UnknownMembers") {
        return quote!(__extra);
    }
    if named(ty, "Written") {
        return quote!(::core::convert::From::from(::core::option::Option::Some(
            ::core::clone::Clone::clone(object.node().value())
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

/// Whether `info` is the read's to take: not an `UnknownMembers` field (it holds the unknown
/// members).
fn taken(info: &FieldInfo) -> bool {
    !named(info.ty, "UnknownMembers")
}

/// `knows`: the JSON names of the fields, a flattened struct's included.
pub(super) fn knows(infos: &[FieldInfo]) -> TokenStream {
    let p = paths();
    let fields = &p.fields;
    let names: Vec<&String> = infos
        .iter()
        .filter(|info| taken(info) && !info.inline)
        .map(|info| &info.json)
        .collect();
    let own = if names.is_empty() {
        quote!(false)
    } else {
        quote!(::core::matches!(key, #(#names)|*))
    };
    let flattened = infos
        .iter()
        .filter(|info| info.inline && taken(info))
        .map(|info| {
            let ty = info.ty;
            quote!(|| <#ty as #fields>::knows(key))
        });
    quote!(#own #(#flattened)*)
}

/// The refusals of the read's keys: `read_with` on a flattened field or on the field that holds
/// the unknown members.
pub(super) fn validate(info: &FieldInfo, errors: &mut Vec<Error>) {
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

/// What the read emits for one field it takes by name: its slot, its match arm and the
/// finish that says what absence and failure make of it.
struct FieldRead {
    slot: TokenStream,
    arm: TokenStream,
    finish: TokenStream,
}

/// The read of the field `info`, its value bound to `__v{index}`.
fn field_read(info: &FieldInfo, index: usize) -> FieldRead {
    let reader = paths().reader;
    let slot = format_ident!("__s{index}");
    let value = format_ident!("__v{index}");
    let held = info.option.unwrap_or(info.ty);
    let field_rule = unknown_option(info.attrs.unknown.as_ref());
    let read_call = match (&info.attrs.read_with, info.option.is_some()) {
        (Some((with, _)), _) => quote!(#slot.read_with(&member, read, #field_rule, #with);),
        (None, true) => quote!(#slot.read_option(&member, read, #field_rule);),
        (None, false) => quote!(#slot.read(&member, read, #field_rule);),
    };
    let json = &info.json;
    let finish = if info.option.is_some() && info.attrs.required.is_none() {
        quote!(let #value = #slot.optional(read, &mut __ok);)
    } else if let Some(default) = &info.attrs.default {
        let value_expr = default.value.as_ref().map_or_else(
            || quote!(::core::default::Default::default()),
            |expr| super::typed(expr, info.ty),
        );
        quote!(let #value = #slot.or_default(read, &mut __ok, || #value_expr);)
    } else {
        // A plain field, or an `Option` the document must write (`schema::required`): its value
        // is the field's own, `Some(..)` for the `Option`.
        quote!(let #value = #slot.required(object, #json, read, &mut __ok);)
    };
    FieldRead {
        slot: quote!(let mut #slot = #reader::Slot::<#held>::new();),
        arm: quote! {
            #json => {
                if !object.absent(&member, read) {
                    #read_call
                }
            }
        },
        finish,
    }
}

/// `Some(&mut __extra)` where the fields have an `UnknownMembers` field, else `None`.
fn extra_arg(extra: bool) -> TokenStream {
    if extra {
        quote!(::core::option::Option::Some(&mut __extra))
    } else {
        quote!(::core::option::Option::None)
    }
}

/// The body of a read of `fields` from `object`, evaluating to `Option<#construct ..>`, with
/// `object`, `read`, `unknown` and `tag` in scope (`Fields::read_fields`' parameters).
fn fields_body(
    construct: &TokenStream,
    fields: &Fields,
    infos: &[FieldInfo],
    rule: Option<&String>,
) -> TokenStream {
    let p = paths();
    let (fields_trait, reader) = (&p.fields, &p.reader);
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
    let (mut slots, mut arms, mut finishes, mut flattened, mut flat_skips) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for info in infos.iter().filter(|info| taken(info)) {
        let index = index_of(info);
        if info.inline {
            let (ty, value) = (info.ty, format_ident!("__v{index}"));
            flattened.push(quote! {
                let #value = #reader::flattened(
                    <#ty as #fields_trait>::read_fields(object, read, false, tag),
                    &mut __ok,
                );
            });
            flat_skips.push(quote!(_ if <#ty as #fields_trait>::knows(key) => {}));
            continue;
        }
        let FieldRead { slot, arm, finish } = field_read(info, index);
        slots.push(slot);
        arms.push(arm);
        finishes.push(finish);
    }
    let extra_decl = extra.then(|| {
        let core = core();
        quote!(let mut __extra = #core::document::UnknownMembers::default();)
    });
    let extra_arg = extra_arg(extra);
    let inits = fields.iter().enumerate().map(|(index, field)| {
        let member = member_of(field, index);
        let value = format_ident!("__v{index}");
        let built = match infos.iter().find(|info| info.member == member) {
            Some(info) if !taken(info) => quote!(__extra),
            Some(info) if info.option.is_some() => quote!(#value),
            Some(_) => quote!(#value?),
            None => untaken_value(field),
        };
        quote!(#member: #built)
    });
    let build = quote!(#construct { #(#inits),* });
    let knows = knows(infos);
    quote! {
        let __disposal = object.disposal(read, #rule, #marked);
        #extra_decl
        #(#slots)*
        let __told = unknown
            && object.unknown_first(read, __disposal, tag, |key: &str| -> bool { #knows }, #extra_arg);
        for (key, member) in object.members() {
            if read.stopped() {
                break;
            }
            match key {
                #(#arms)*
                _ if ::core::option::Option::Some(key) == tag => {}
                #(#flat_skips)*
                _ => {
                    if unknown && !__told {
                        object.unknown(key, &member, read, __disposal, #extra_arg);
                    }
                }
            }
        }
        let mut __ok = true;
        #(#flattened)*
        #(#finishes)*
        if !__ok {
            return ::core::option::Option::None;
        }
        ::core::option::Option::Some(#build)
    }
}

/// The read of a struct: `FromJson` (the object, then its fields) and `Fields`; a transparent
/// newtype reads as its one field.
pub(super) fn struct_read(
    ident: &Ident,
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
        expected,
        ..
    } = paths();
    if container.transparent {
        let Some(info) = infos.first() else {
            return TokenStream::new();
        };
        let ty = info.ty;
        let inner = info.attrs.read_with.as_ref().map_or_else(
            || quote!(<#ty as #from_json>::from_json(node, read)?),
            |(with, _)| quote!((#with)(node, read)?),
        );
        let build = match &info.member {
            Member::Named(name) => quote!(Self { #name: #inner }),
            Member::Unnamed(_) => quote!(Self(#inner)),
        };
        return quote! {
            #[automatically_derived]
            impl #from_json for #ident {
                const EXPECTED: ::core::option::Option<#expected> = <#ty as #from_json>::EXPECTED;

                fn from_json(node: &#node<'_>, read: &mut #read<'_>) -> ::core::option::Option<Self> {
                    ::core::option::Option::Some(#build)
                }
            }
        };
    }
    let knows = knows(infos);
    let body = fields_body(&quote!(#ident), fields, infos, container.unknown.as_ref());
    quote! {
        #[automatically_derived]
        impl #from_json for #ident {
            const EXPECTED: ::core::option::Option<#expected> =
                ::core::option::Option::Some(#expected::Object);

            fn from_json(node: &#node<'_>, read: &mut #read<'_>) -> ::core::option::Option<Self> {
                let object = #object::open(node, read)?;
                let value = <Self as #fields_trait>::read_fields(
                    &object,
                    read,
                    true,
                    ::core::option::Option::None,
                );
                object.close(read);
                value
            }
        }

        #[automatically_derived]
        impl #fields_trait for #ident {
            fn knows(key: &str) -> bool {
                #knows
            }

            #[allow(unused_variables, unused_mut, clippy::too_many_lines)]
            fn read_fields(
                object: &#object<'_>,
                read: &mut #read<'_>,
                unknown: bool,
                tag: ::core::option::Option<&'static str>,
            ) -> ::core::option::Option<Self> {
                #body
            }
        }
    }
}

/// The read of a string enum: by name, through its `from_name`.
pub(super) fn unit_enum_read(ident: &Ident) -> TokenStream {
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

            fn from_json(node: &#node<'_>, read: &mut #read<'_>) -> ::core::option::Option<Self> {
                #reader::unit_enum(node, read, Self::from_name)
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

/// The local function that reads struct variant `index` of `ident` from an object.
fn variant_fn(ident: &Ident, index: usize, variant: &ReadVariant) -> TokenStream {
    let Paths { read, object, .. } = paths();
    let v = &variant.variant.ident;
    let name = format_ident!("__variant_{index}");
    let body = fields_body(
        &quote!(#ident::#v),
        &variant.variant.fields,
        variant.fields,
        variant.unknown.as_ref(),
    );
    quote! {
        #[allow(unused_variables, unused_mut, clippy::too_many_lines)]
        fn #name(
            object: &#object<'_>,
            read: &mut #read<'_>,
            unknown: bool,
            tag: ::core::option::Option<&'static str>,
        ) -> ::core::option::Option<#ident> {
            #body
        }
    }
}

/// The read of a newtype variant's value at `at`.
fn newtype_value(ty: &Type, with: Option<&syn::Expr>, at: &TokenStream) -> TokenStream {
    let from_json = paths().from_json;
    with.map_or_else(
        || quote!(<#ty as #from_json>::from_json(#at, read)),
        |with| quote!((#with)(#at, read)),
    )
}

/// The read of a struct variant from the object at `at`: open it, read the fields, close it.
fn struct_value(index: usize, at: &TokenStream) -> TokenStream {
    let object = paths().object;
    let name = format_ident!("__variant_{index}");
    quote! {{
        match #object::open(#at, read) {
            ::core::option::Option::Some(object) => {
                let value = #name(&object, read, true, ::core::option::Option::None);
                object.close(read);
                value
            }
            ::core::option::Option::None => ::core::option::Option::None,
        }
    }}
}

/// The read of an untagged enum with data: the arm the door chooses (`__read::untagged_arm`),
/// then that variant's payload.
pub(super) fn untagged_read(ident: &Ident, variants: &[ReadVariant]) -> TokenStream {
    let Paths {
        node,
        read,
        from_json,
        reader,
        ..
    } = paths();
    let fns = variants
        .iter()
        .enumerate()
        .filter(|(_, v)| matches!(payload(v), Payload::Struct))
        .map(|(index, v)| variant_fn(ident, index, v));
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
    let at = quote!(node);
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
        quote!(#reader::Arm::Variant(#index) => #value,)
    });
    quote! {
        #[automatically_derived]
        impl #from_json for #ident {
            fn from_json(node: &#node<'_>, read: &mut #read<'_>) -> ::core::option::Option<Self> {
                #(#fns)*
                match #reader::untagged_arm::<Self>(node, read)? {
                    #reader::Arm::Unit => match #reader::unit_arm(node, &[#(#unit_names),*]) {
                        #(#unit_arms)*
                        _ => {
                            read.wrong_type(node);
                            ::core::option::Option::None
                        }
                    },
                    #(#arms)*
                    #[allow(unreachable_patterns)]
                    _ => {
                        read.wrong_type(node);
                        ::core::option::Option::None
                    }
                }
            }
        }
    }
}

/// The read of an internally tagged enum (`tag = ".."`): the variant its tag member names, its
/// members beside the tag.
pub(super) fn tagged_read(ident: &Ident, tag: &str, variants: &[ReadVariant]) -> TokenStream {
    let Paths {
        node,
        read,
        fields: fields_trait,
        from_json,
        reader,
        expected,
        ..
    } = paths();
    let fns = variants
        .iter()
        .enumerate()
        .filter(|(_, v)| matches!(payload(v), Payload::Struct | Payload::Unit))
        .map(|(index, v)| variant_fn(ident, index, v));
    let arms = variants.iter().enumerate().map(|(index, v)| {
        let json = &v.name;
        let name = &v.variant.ident;
        let fn_name = format_ident!("__variant_{index}");
        let value = match payload(v) {
            Payload::Unit | Payload::Struct => {
                quote!(#fn_name(&object, read, true, ::core::option::Option::Some(#tag)))
            }
            Payload::Newtype(ty, _) => quote! {
                <#ty as #fields_trait>::read_fields(&object, read, true, ::core::option::Option::Some(#tag))
                    .map(Self::#name)
            },
        };
        quote!(#json => #value,)
    });
    quote! {
        #[automatically_derived]
        impl #from_json for #ident {
            const EXPECTED: ::core::option::Option<#expected> =
                ::core::option::Option::Some(#expected::Object);

            fn from_json(node: &#node<'_>, read: &mut #read<'_>) -> ::core::option::Option<Self> {
                #(#fns)*
                let (object, name) = #reader::tagged(node, read, #tag)?;
                let value = match name.as_str() {
                    #(#arms)*
                    _ => {
                        #reader::unknown_tag(object, read, #tag);
                        return ::core::option::Option::None;
                    }
                };
                object.close(read);
                value
            }
        }
    }
}

/// The read of an externally tagged enum: a unit variant by its name, a variant with data as
/// `{name: payload}`.
pub(super) fn external_read(ident: &Ident, variants: &[ReadVariant]) -> TokenStream {
    let Paths {
        node,
        read,
        from_json,
        reader,
        ..
    } = paths();
    let fns = variants
        .iter()
        .enumerate()
        .filter(|(_, v)| matches!(payload(v), Payload::Struct))
        .map(|(index, v)| variant_fn(ident, index, v));
    let unit_arms = variants
        .iter()
        .filter(|v| matches!(payload(v), Payload::Unit))
        .map(|v| {
            let json = &v.name;
            let name = &v.variant.ident;
            quote!(#json => ::core::option::Option::Some(Self::#name),)
        });
    let at = quote!(&payload);
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
        impl #from_json for #ident {
            fn from_json(node: &#node<'_>, read: &mut #read<'_>) -> ::core::option::Option<Self> {
                #(#fns)*
                match #reader::external(node, read)? {
                    #reader::External::Unit(name) => match name.as_str() {
                        #(#unit_arms)*
                        _ => {
                            #reader::unknown_variant(node, read);
                            ::core::option::Option::None
                        }
                    },
                    #reader::External::Payload(name, payload) => match name.as_str() {
                        #(#payload_arms)*
                        _ => {
                            #reader::unknown_variant(&payload, read);
                            ::core::option::Option::None
                        }
                    },
                }
            }
        }
    }
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
            }
        }
    }
}
