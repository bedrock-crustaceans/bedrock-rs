use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, io};
use syn::parse::ParseStream;
use syn::{
    File, LitInt, LitStr, Path, Token, braced, bracketed, parenthesized, parse::Parse,
    punctuated::Punctuated,
};

fn main() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let protocol = project_root.join("crates/protocol/");

    let text =
        fs::read_to_string(protocol.join("def/versions.def.rs")).expect("versions file not found");
    let tokens = TokenStream::from_str(&text).unwrap();

    clear_directory(protocol.join("src/generated").to_str().unwrap()).unwrap();

    let version_tokens = define_versions_internal(tokens, protocol.as_path());
    let file = syn::parse2::<File>(version_tokens).unwrap();

    fs::write(
        protocol.join("src/generated/mod.rs"),
        prettyplease::unparse(&file),
    )
    .unwrap();
}

fn clear_directory(path: &str) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let path = entry.path();

        if meta.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(versions);
    custom_keyword!(packets);
    custom_keyword!(types);
    custom_keyword!(enums);
    custom_keyword!(raknet_version);
}

struct DefineVersionsInput {
    versions: Punctuated<DefineVersionsEntry, Token![,]>,
}

struct DefineVersionsEntry {
    version: u32,
    branch: LitStr,
    game_version: LitStr,
    raknet_version: Option<u8>,
    packets: Option<DefineVersionsDiffList>,
    types: Option<DefineVersionsDiffList>,
    enums: Option<DefineVersionsDiffList>,
    ident: Option<Ident>,
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
        input.parse::<kw::versions>()?;
        input.parse::<Token![!]>()?;

        let bracket;
        bracketed!(bracket in input);

        let versions = Punctuated::parse_terminated(&bracket)?;

        input.parse::<Token![;]>()?;

        Ok(DefineVersionsInput { versions })
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

        let mut raknet_version = None;
        let mut packets = None;
        let mut types = None;
        let mut enums = None;

        while !brace.is_empty() {
            if brace.peek(kw::raknet_version) {
                brace.parse::<kw::raknet_version>()?;
                brace.parse::<Token![:]>()?;
                if raknet_version.is_some() {
                    return Err(brace.error("duplicate `raknet_version` definition"));
                }
                raknet_version = Some(brace.parse::<LitInt>()?.base10_parse()?);
            } else if brace.peek(kw::packets) {
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
                return Err(
                    brace.error("expected `raknet_version`, `packets`, `types`, or `enums`")
                );
            }

            if !brace.is_empty() {
                brace.parse::<Token![,]>()?;
            }
        }

        let ident = if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            Some(input.parse::<Ident>()?)
        } else {
            None
        };

        Ok(Self {
            version,
            branch,
            raknet_version,
            game_version,
            packets,
            types,
            enums,
            ident,
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

pub fn define_versions_internal(input: TokenStream, path: &std::path::Path) -> TokenStream {
    let DefineVersionsInput { versions } = syn::parse2::<DefineVersionsInput>(input).unwrap();

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
        Err(e) => return e.into_compile_error(),
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
        Err(e) => return e.into_compile_error(),
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
        Err(e) => return e.into_compile_error(),
    };

    let proto_version_packets = all_packets
        .iter()
        .map(|p| quote!(type #p: bedrock_protocol_core::ProtoCodec + Clone + std::fmt::Debug + Send + Sync + 'static;))
        .collect::<Vec<_>>();

    let proto_version_types = all_types
        .iter()
        .map(|p| quote!(type #p: bedrock_protocol_core::ProtoCodec + Clone + std::fmt::Debug + Send + Sync + 'static;))
        .collect::<Vec<_>>();

    let proto_version_enums = all_enums
        .iter()
        .map(|p| quote!(type #p: bedrock_protocol_core::ProtoCodec + Clone + std::fmt::Debug + Send + Sync + 'static;))
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

        pub trait ProtoVersion: ProtoVersionPackets + ProtoVersionTypes + ProtoVersionEnums + std::fmt::Debug + Send + Sync + 'static {
            const PROTOCOL_VERSION: u32;
            const PROTOCOL_BRANCH: &str;
            const GAME_VERSION: &str;
            const RAKNET_VERSION: u8;
        }
    };

    let mut previous_raknet_version: Option<u8> = None;
    let mut previous_packets = HashMap::<Ident, TokenStream>::new();
    let mut previous_types = HashMap::<Ident, TokenStream>::new();
    let mut previous_enums = HashMap::<Ident, TokenStream>::new();

    let mut versions_stream = TokenStream::new();
    for entry in &versions_vec {
        if let Err(e) = collapse(&entry.packets, &mut previous_packets) {
            return e.into_compile_error();
        }
        if let Err(e) = collapse(&entry.types, &mut previous_types) {
            return e.into_compile_error();
        }
        if let Err(e) = collapse(&entry.enums, &mut previous_enums) {
            return e.into_compile_error();
        }

        if let Some(raknet_version) = entry.raknet_version {
            previous_raknet_version = Some(raknet_version);
        }

        let Some(raknet_version) = previous_raknet_version else {
            return syn::Error::new(Span::call_site(), "raknet_version not defined")
                .into_compile_error();
        };

        let version = entry.version;
        let branch = entry.branch.clone();
        let game_version = entry.game_version.clone();

        let struct_ident = entry.ident.clone().unwrap_or(Ident::new(
            format!("V{}", version).as_str(),
            Span::call_site(),
        ));

        let mod_ident = Ident::new(
            struct_ident.to_string().to_lowercase().as_str(),
            Span::call_site(),
        );

        let proto_version_packets_impl = all_packets
            .iter()
            .map(|k| {
                if let Some(v) = previous_packets.get(k) {
                    quote!(type #k = #v;)
                } else {
                    quote!(type #k = ();)
                }
            })
            .collect::<Vec<_>>();

        let proto_version_types_impl = all_types
            .iter()
            .map(|k| {
                if let Some(v) = previous_types.get(k) {
                    quote!(type #k = #v;)
                } else {
                    quote!(type #k = ();)
                }
            })
            .collect::<Vec<_>>();

        let proto_version_enums_impl = all_enums
            .iter()
            .map(|k| {
                if let Some(v) = previous_enums.get(k) {
                    quote!(type #k = #v;)
                } else {
                    quote!(type #k = ();)
                }
            })
            .collect::<Vec<_>>();

        let packet_variants = previous_packets
            .keys()
            .map(|k| {
                quote! { #k(Box<<Self as ProtoVersionPackets>::#k>), }
            })
            .collect::<Vec<_>>();

        let packet_id = previous_packets.keys().map(|name| {
            quote! { #struct_ident::#name(_) => <<#struct_ident as ProtoVersionPackets>::#name as bedrock_protocol_core::Packet>::ID, }
        });

        let packet_serialize = previous_packets.keys().map(|name| {
            quote! {
                #struct_ident::#name(pk) => {
                    match <<#struct_ident as ProtoVersionPackets>::#name as bedrock_protocol_core::ProtoCodec>::serialize(pk.as_ref(), stream) {
                        Ok(_) => {},
                        Err(err) => return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(#name),
                            packet_id: <<#struct_ident as ProtoVersionPackets>::#name as bedrock_protocol_core::Packet>::ID,
                            error: err
                        }),
                    };
                },
            }
        });

        let packet_deserialize = previous_packets.keys().map(|name| {
            quote! {
                <<#struct_ident as ProtoVersionPackets>::#name as bedrock_protocol_core::Packet>::ID => {
                    match <<#struct_ident as ProtoVersionPackets>::#name as bedrock_protocol_core::ProtoCodec>::deserialize(stream) {
                        Ok(pk) => #struct_ident::#name(Box::new(pk)),
                        Err(err) => return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: stringify!(#name),
                            packet_id: <<#struct_ident as ProtoVersionPackets>::#name as bedrock_protocol_core::Packet>::ID,
                            error: err
                        }),
                    }
                },
            }
        });

        let packet_size_hint = previous_packets.keys().map(|name| {
            quote! { #struct_ident::#name(pk) => <<#struct_ident as ProtoVersionPackets>::#name as bedrock_protocol_core::ProtoCodec>::size_hint(pk.as_ref()), }
        });

        let packet_inner = previous_packets.keys().map(|name| {
            quote! { #struct_ident::#name(pk) => pk.as_ref(), }
        });

        let packet_into_inner = previous_packets.keys().map(|name| {
            quote! { #struct_ident::#name(pk) => pk, }
        });

        let version_tokens = quote! {
            use crate::ProtoVersion;
            use crate::ProtoVersionPackets;
            use crate::ProtoVersionTypes;
            use crate::ProtoVersionEnums;

            #[derive(Clone, std::fmt::Debug)]
            pub enum #struct_ident {
                #(#packet_variants)*
                Unknown(Box<bedrock_protocol_core::UnknownPacket>),
            }

            impl bedrock_protocol_core::DynPacket for #struct_ident {
                #[inline]
                fn id(&self) -> u16 {
                    match self {
                        #(#packet_id)*
                        #struct_ident::Unknown(pk) => pk.id,
                    }
                }
            }

            impl bedrock_protocol_core::Packets for #struct_ident {
                #[inline]
                fn serialize<W: std::io::Write>(&self, header: &bedrock_protocol_core::PacketHeader, stream: &mut W) -> Result<(), bedrock_protocol_core::error::PacketCodecError> {
                    <bedrock_protocol_core::PacketHeader as bedrock_protocol_core::ProtoCodec>::serialize(header, stream)
                        .map_err(bedrock_protocol_core::error::PacketCodecError::InvalidHeader)?;

                    match self {
                        #(#packet_serialize)*
                        #struct_ident::Unknown(pk) => stream.write_all(pk.buf.as_ref())
                            .map_err(|e| bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                packet_name: "Unknown",
                                packet_id: header.packet_id,
                                error: e.into()
                            })?,
                    };

                    Ok(())
                }

                #[inline]
                fn deserialize<R: std::io::Read>(stream: &mut R) -> Result<(Self, bedrock_protocol_core::PacketHeader), bedrock_protocol_core::error::PacketCodecError> {
                    let header = <bedrock_protocol_core::PacketHeader as bedrock_protocol_core::ProtoCodec>::deserialize(stream)
                        .map_err(bedrock_protocol_core::error::PacketCodecError::InvalidHeader)?;

                    let packet = match header.packet_id {
                        #(#packet_deserialize)*
                        unknown => {
                            let mut buf = Vec::new();
                            stream.read_to_end(&mut buf)
                                .map_err(|e| bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                                    packet_name: "Unknown",
                                    packet_id: header.packet_id,
                                    error: e.into(),
                                })?;
                            #struct_ident::Unknown(Box::new(bedrock_protocol_core::UnknownPacket {
                                id: unknown,
                                buf: buf.into_boxed_slice()
                            }))
                        },
                    };
                    Ok((packet, header))
                }

                #[inline]
                fn size_hint(&self, header: &bedrock_protocol_core::PacketHeader) -> usize {
                    <bedrock_protocol_core::PacketHeader as bedrock_protocol_core::ProtoCodec>::size_hint(header) + match self {
                        #(#packet_size_hint)*
                        #struct_ident::Unknown(pk) => pk.buf.len(),
                    }
                }

                #[inline]
                fn inner(&self) -> &dyn bedrock_protocol_core::DynPacket {
                    match self {
                        #(#packet_inner)*
                        #struct_ident::Unknown(pk) => pk.as_ref(),
                    }
                }

                #[inline]
                fn into_inner(self) -> Box<dyn bedrock_protocol_core::DynPacket> {
                    match self {
                        #(#packet_into_inner)*
                        #struct_ident::Unknown(pk) => pk,
                    }
                }
            }

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
                const RAKNET_VERSION: u8 = #raknet_version;
            }
        };

        let feature_str = LitStr::new(&mod_ident.to_string(), mod_ident.span());

        let version_mod_tokens = quote! {
            #![allow(unused)]

            #[cfg(feature = #feature_str)]
            mod inner {
                #version_tokens
            }
            #[cfg(feature = #feature_str)]
            pub use inner::*;
        };

        let file = syn::parse2(version_mod_tokens).unwrap();

        fs::write(
            path.join(format!("src/generated/{}.rs", mod_ident)),
            prettyplease::unparse(&file),
        )
        .unwrap();

        versions_stream.extend(quote! {
            mod #mod_ident;
            pub use #mod_ident::*;
        })
    }

    quote! {
        #![allow(unused)]

        #proto_version

        #versions_stream
    }
}

fn collapse(
    list: &Option<DefineVersionsDiffList>,
    map: &mut HashMap<Ident, TokenStream>,
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
