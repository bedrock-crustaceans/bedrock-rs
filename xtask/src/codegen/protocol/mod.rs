use crate::codegen::protocol::diff::DefineVersionsDiffEntry;
use crate::codegen::protocol::input::DefineVersionsInput;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use syn::LitStr;

pub mod diff;
pub mod entry;
pub mod input;

mod kw {
    use syn::custom_keyword;

    custom_keyword!(versions);
    custom_keyword!(packets);
    custom_keyword!(types);
    custom_keyword!(enums);
    custom_keyword!(raknet_version);
}

pub fn build(input: TokenStream, path: &std::path::Path) -> syn::Result<TokenStream> {
    let DefineVersionsInput { versions } = syn::parse2::<DefineVersionsInput>(input)?;

    let mut versions_vec = versions.into_iter().collect::<Vec<_>>();

    versions_vec.sort_by_key(|v| v.version);

    let all_packets = versions_vec
        .iter()
        .try_fold(BTreeSet::<Ident>::new(), |mut acc, v| {
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
        })?;

    let all_types = versions_vec
        .iter()
        .try_fold(BTreeSet::<Ident>::new(), |mut acc, v| {
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
        })?;

    let all_enums = versions_vec
        .iter()
        .try_fold(BTreeSet::<Ident>::new(), |mut acc, v| {
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
        })?;

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
    let mut previous_packets = BTreeMap::<Ident, TokenStream>::new();
    let mut previous_types = BTreeMap::<Ident, TokenStream>::new();
    let mut previous_enums = BTreeMap::<Ident, TokenStream>::new();

    let mut versions_stream = TokenStream::new();
    for entry in &versions_vec {
        if let Some(packets) = &entry.packets {
            packets.collapse(&mut previous_packets)?;
        }
        if let Some(types) = &entry.types {
            types.collapse(&mut previous_types)?;
        }
        if let Some(enums) = &entry.enums {
            enums.collapse(&mut previous_enums)?;
        }

        if let Some(raknet_version) = entry.raknet_version {
            previous_raknet_version = Some(raknet_version);
        }

        let Some(raknet_version) = previous_raknet_version else {
            return Err(syn::Error::new(
                Span::call_site(),
                "raknet_version not defined",
            ));
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

        let packet_id = previous_packets.keys().map(|name| {
            quote! {
                #struct_ident::#name(_) => <<#struct_ident as ProtoVersionPackets>::#name as bedrock_protocol_core::Packet>::ID,
            }
        });

        let packet_size_hint = previous_packets.keys().map(|name| {
            quote! { #struct_ident::#name(pk) => <<#struct_ident as ProtoVersionPackets>::#name as bedrock_protocol_core::ProtoCodec>::size_hint(pk.as_ref()), }
        });

        let packet_dyn_as_ref = previous_packets.keys().map(|name| {
            quote! { #struct_ident::#name(pk) => pk.as_ref(), }
        });

        let packet_dyn_into = previous_packets.keys().map(|name| {
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
                fn id(&self) -> u16 {
                    match self {
                        #(#packet_id)*
                        #struct_ident::Unknown(pk) => pk.id,
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

            #[cfg(feature = "packet-dyn")]
            impl AsRef<dyn bedrock_protocol_core::PacketDyn> for #struct_ident {
                fn as_ref(&self) -> &dyn bedrock_protocol_core::PacketDyn {
                    match self {
                        #(#packet_dyn_as_ref)*
                        #struct_ident::Unknown(pk) => pk.as_ref(),
                    }
                }
            }

            #[cfg(feature = "packet-dyn")]
            impl From<#struct_ident> for Box<dyn bedrock_protocol_core::PacketDyn> {
                fn from(val: #struct_ident) -> Box<dyn bedrock_protocol_core::PacketDyn> {
                    match val {
                        #(#packet_dyn_into)*
                        #struct_ident::Unknown(pk) => pk,
                    }
                }
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

        let file = syn::parse2(version_mod_tokens)?;

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

    Ok(quote! {
        #![allow(unused)]

        #proto_version

        #versions_stream
    })
}
