use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::token::Token;
use syn::{
    braced, bracketed, parenthesized, parse::Parse, punctuated::Punctuated, LitInt, LitStr, Path,
    Token,
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(packets);
    custom_keyword!(types);
    custom_keyword!(enums);
}

struct DefineVersionsInput {
    versions: Punctuated<DefineVersionsEntry, Token![,]>,
}

struct DefineVersionsEntry {
    version: u32,
    branch: LitStr,
    game_version: LitStr,
    packets: Option<DefineVersionsDiffList>,
    types: Option<DefineVersionsDiffList>,
    enums: Option<DefineVersionsDiffList>,
}

struct DefineVersionsDiffList {
    pub entries: Punctuated<DefineVersionsDiffEntry, Token![,]>,
    pub path: Path,
}

enum DefineVersionsDiffEntry {
    Added {
        ident: Ident,
        path: Path,
        versioned: bool,
    },
    Removed {
        ident: Ident,
    },
    Modified {
        ident: Ident,
        path: Path,
        versioned: bool,
    },
}

impl Parse for DefineVersionsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(DefineVersionsInput {
            versions: Punctuated::parse_terminated(input)?,
        })
    }
}

impl Parse for DefineVersionsEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let paren;
        let brace;
        parenthesized!(paren in input);

        let version = paren.parse::<LitInt>()?.base10_parse()?;
        paren.parse::<Token![,]>()?;
        let branch = paren.parse::<LitStr>()?;
        paren.parse::<Token![,]>()?;
        let game_version = paren.parse::<LitStr>()?;

        input.parse::<Token![:]>()?;
        braced!(brace in input);

        let mut packets = None;
        let mut types = None;
        let mut enums = None;

        while !brace.is_empty() {
            if brace.peek(kw::packets) {
                brace.parse::<kw::packets>()?;
                if packets.is_some() {
                    return Err(brace.error("duplicate `packets` section"));
                }
                packets = Some(brace.parse()?);
            } else if brace.peek(kw::types) {
                brace.parse::<kw::types>()?;
                if types.is_some() {
                    return Err(brace.error("duplicate `types` section"));
                }
                types = Some(brace.parse()?);
            } else if brace.peek(kw::enums) {
                brace.parse::<kw::enums>()?;
                if enums.is_some() {
                    return Err(brace.error("duplicate `enums` section"));
                }
                enums = Some(brace.parse()?);
            } else {
                return Err(brace.error("expected `packets`, `types`, or `enums`"));
            }

            if !brace.is_empty() {
                brace.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            version,
            branch,
            game_version,
            packets,
            types,
            enums,
        })
    }
}

impl Parse for DefineVersionsDiffList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![:]>()?;

        let content;
        bracketed!(content in input);
        let entries = Punctuated::parse_terminated(&content)?;

        input.parse::<Token![in]>()?;
        let path: Path = input.parse()?;

        Ok(Self { entries, path })
    }
}

impl Parse for DefineVersionsDiffEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![+]) {
            input.parse::<Token![+]>()?;

            let ident: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            Ok(Self::Added {
                ident,
                path: input.parse()?,
                versioned: input.parse::<Option<Token![^]>>()?.is_some(),
            })
        } else if input.peek(Token![%]) {
            input.parse::<Token![%]>()?;

            let ident: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            Ok(Self::Modified {
                ident,
                path: input.parse()?,
                versioned: input.parse::<Option<Token![^]>>()?.is_some(),
            })
        } else if input.peek(Token![-]) {
            input.parse::<Token![-]>()?;

            let ident: Ident = input.parse()?;

            Ok(Self::Removed { ident })
        } else {
            Err(input.error("expected one of +, %, or -"))
        }
    }
}

pub fn define_versions_internal(input: TokenStream) -> TokenStream {
    let DefineVersionsInput { versions } = syn::parse_macro_input!(input as DefineVersionsInput);

    let mut versions_vec = versions.into_iter().collect::<Vec<_>>();

    versions_vec.sort_by_key(|v| v.version);

    let all_packets = match versions_vec
        .iter()
        .try_fold(HashSet::<Ident>::new(), |mut acc, v| {
            if let Some(packets) = &v.packets {
                for entry in &packets.entries {
                    if let DefineVersionsDiffEntry::Added { ident, .. } = entry {
                        if let Some(prev) = acc.get(ident) {
                            let mut err =
                                syn::Error::new(ident.span(), "packet added more than once");

                            err.combine(syn::Error::new(ident.span(), "did you mean to use `%`?"));
                            err.combine(syn::Error::new(prev.span(), "previously added here"));

                            return Err(err);
                        } else {
                            acc.insert(ident.clone());
                        }
                    }
                }
            }
            Ok(acc)
        }) {
        Ok(acc) => acc,
        Err(e) => return e.into_compile_error().into(),
    };

    let all_types = match versions_vec
        .iter()
        .try_fold(HashSet::<Ident>::new(), |mut acc, v| {
            if let Some(types) = &v.types {
                for entry in &types.entries {
                    if let DefineVersionsDiffEntry::Added { ident, .. } = entry {
                        if let Some(prev) = acc.get(ident) {
                            let mut err =
                                syn::Error::new(ident.span(), "type added more than once");

                            err.combine(syn::Error::new(ident.span(), "did you mean to use `%`?"));
                            err.combine(syn::Error::new(prev.span(), "previously added here"));

                            return Err(err);
                        } else {
                            acc.insert(ident.clone());
                        }
                    }
                }
            }
            Ok(acc)
        }) {
        Ok(acc) => acc,
        Err(e) => return e.into_compile_error().into(),
    };

    let all_enums = match versions_vec
        .iter()
        .try_fold(HashSet::<Ident>::new(), |mut acc, v| {
            if let Some(enums) = &v.enums {
                for entry in &enums.entries {
                    if let DefineVersionsDiffEntry::Added { ident, .. } = entry {
                        if let Some(prev) = acc.get(ident) {
                            let mut err =
                                syn::Error::new(ident.span(), "enum added more than once");

                            err.combine(syn::Error::new(ident.span(), "did you mean to use `%`?"));
                            err.combine(syn::Error::new(prev.span(), "previously added here"));

                            return Err(err);
                        } else {
                            acc.insert(ident.clone());
                        }
                    }
                }
            }
            Ok(acc)
        }) {
        Ok(acc) => acc,
        Err(e) => return e.into_compile_error().into(),
    };

    let proto_version_packets = all_packets
        .iter()
        .map(|p| quote!(type #p: bedrockrs_proto_core::ProtoCodec + Clone + std::fmt::Debug;))
        .collect::<Vec<_>>();

    let proto_version_types = all_types
        .iter()
        .map(|p| quote!(type #p: bedrockrs_proto_core::ProtoCodec + Clone + std::fmt::Debug;))
        .collect::<Vec<_>>();

    let proto_version_enums = all_enums
        .iter()
        .map(|p| quote!(type #p: bedrockrs_proto_core::ProtoCodec + Clone + std::fmt::Debug;))
        .collect::<Vec<_>>();

    let proto_version = quote! {
        pub trait ProtoVersionPackets {
            #(#proto_version_packets)*
        }

        pub trait ProtoVersionTypes {
            #(#proto_version_types)*
        }

        pub trait ProtoVersionEnums {
            #(#proto_version_enums)*
        }

        pub trait ProtoVersion: ProtoVersionPackets + ProtoVersionTypes + ProtoVersionEnums {
            const PROTOCOL_VERSION: u32;
            const PROTOCOL_BRANCH: &str;
            const GAME_VERSION: &str;
        }
    };

    let mut cumulative_packets = HashMap::<Ident, proc_macro2::TokenStream>::new();
    let mut cumulative_types = HashMap::<Ident, proc_macro2::TokenStream>::new();
    let mut cumulative_enums = HashMap::<Ident, proc_macro2::TokenStream>::new();

    let mut versions_stream = proc_macro2::TokenStream::new();
    for entry in &versions_vec {
        if let Err(e) = collapse(&entry.packets, &mut cumulative_packets) {
            return e.into_compile_error().into();
        }
        if let Err(e) = collapse(&entry.types, &mut cumulative_types) {
            return e.into_compile_error().into();
        }
        if let Err(e) = collapse(&entry.enums, &mut cumulative_enums) {
            return e.into_compile_error().into();
        }

        let proto_version_packets_impl = all_packets
            .iter()
            .map(|k| {
                if let Some(v) = cumulative_packets.get(k) {
                    quote!(type #k = #v;)
                } else {
                    quote!(type #k = ();)
                }
            })
            .collect::<Vec<_>>();

        let proto_version_types_impl = all_types
            .iter()
            .map(|k| {
                if let Some(v) = cumulative_types.get(k) {
                    quote!(type #k = #v;)
                } else {
                    quote!(type #k = ();)
                }
            })
            .collect::<Vec<_>>();

        let proto_version_enums_impl = all_enums
            .iter()
            .map(|k| {
                if let Some(v) = cumulative_enums.get(k) {
                    quote!(type #k = #v;)
                } else {
                    quote!(type #k = ();)
                }
            })
            .collect::<Vec<_>>();

        let version = entry.version;
        let branch = entry.branch.clone();
        let game_version = entry.game_version.clone();

        let struct_ident = Ident::new(format!("V{}", version).as_str(), Span::call_site());

        versions_stream.extend(quote! {
            #[derive(Clone, std::fmt::Debug)]
            pub struct #struct_ident;

            impl ProtoVersionPackets for #struct_ident {
                #(#proto_version_packets_impl)*
            }

            impl ProtoVersionTypes for #struct_ident {
                #(#proto_version_types_impl)*
            }

            impl ProtoVersionEnums for #struct_ident {
                #(#proto_version_enums_impl)*
            }

            impl ProtoVersion for #struct_ident {
                const PROTOCOL_VERSION: u32 = #version;
                const PROTOCOL_BRANCH: &str = #branch;
                const GAME_VERSION: &str = #game_version;
            }
        })
    }

    quote! {
        #proto_version

        #versions_stream
    }
    .into()
}

fn collapse(
    list: &Option<DefineVersionsDiffList>,
    map: &mut HashMap<Ident, proc_macro2::TokenStream>,
) -> syn::Result<()> {
    if let Some(diff_list) = list {
        for entry in &diff_list.entries {
            let base_path = &diff_list.path;
            match entry {
                DefineVersionsDiffEntry::Added {
                    ident,
                    path,
                    versioned,
                } => {
                    let tokens = if *versioned {
                        quote!(#base_path::#path<Self>)
                    } else {
                        quote!(#base_path::#path)
                    };
                    map.insert(ident.clone(), tokens);
                }
                DefineVersionsDiffEntry::Removed { ident } => {
                    if !map.contains_key(ident) {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!("cannot remove {} because it was never added", ident),
                        ));
                    }

                    map.remove(ident);
                }
                DefineVersionsDiffEntry::Modified {
                    ident,
                    path,
                    versioned,
                } => {
                    if !map.contains_key(ident) {
                        let mut err = syn::Error::new(
                            ident.span(),
                            format!("cannot modify `{}` because it was never added", ident),
                        );

                        err.combine(syn::Error::new(ident.span(), "did you mean to use `+`?"));

                        return Err(err);
                    }

                    let tokens = if *versioned {
                        quote!(#base_path::#path<Self>)
                    } else {
                        quote!(#base_path::#path)
                    };
                    map.insert(ident.clone(), tokens);
                }
            }
        }
    }
    Ok(())
}
