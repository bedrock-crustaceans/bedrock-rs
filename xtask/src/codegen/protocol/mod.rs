use crate::codegen::protocol::diff::{DefineVersionsDiffEntry, DefineVersionsDiffList};
use crate::codegen::protocol::entry::DefineVersionsEntry;
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

/// Collects every identifier that some version adds with `+`, rejecting an identifier that is
/// added twice. `what` names the kind of item for the error message.
fn collect_added(
    versions: &[DefineVersionsEntry],
    what: &str,
    select: impl Fn(&DefineVersionsEntry) -> Option<&DefineVersionsDiffList>,
) -> syn::Result<BTreeSet<Ident>> {
    versions
        .iter()
        .try_fold(BTreeSet::<Ident>::new(), |mut acc, v| {
            if let Some(list) = select(v) {
                for entry in &list.entries {
                    if let DefineVersionsDiffEntry::Added { ident, .. } = entry {
                        if let Some(prev) = acc.get(ident) {
                            let mut err = syn::Error::new(
                                ident.span(),
                                format!("{what} added more than once"),
                            );

                            err.combine(syn::Error::new(ident.span(), "did you mean to use `%`?"));
                            err.combine(syn::Error::new(prev.span(), "previously added here"));

                            return Err(err);
                        }
                        acc.insert(ident.clone());
                    }
                }
            }
            Ok(acc)
        })
}

/// The `type X: ...;` declarations of one `ProtoVersion*` trait.
fn assoc_type_decls(names: &BTreeSet<Ident>) -> Vec<TokenStream> {
    names
        .iter()
        .map(|p| quote!(type #p: bedrock_protocol_core::ProtoCodec + Clone + std::fmt::Debug + Send + Sync + 'static;))
        .collect()
}

/// The `type X = ...;` definitions of one `ProtoVersion*` impl; items the version does not have
/// yet are `()`.
fn assoc_type_impls(
    names: &BTreeSet<Ident>,
    current: &BTreeMap<Ident, TokenStream>,
) -> Vec<TokenStream> {
    names
        .iter()
        .map(|k| {
            if let Some(v) = current.get(k) {
                quote!(type #k = #v;)
            } else {
                quote!(type #k = ();)
            }
        })
        .collect()
}

/// Everything one version module needs to render its `ProtoVersion` implementation.
struct VersionImpl<'a> {
    struct_ident: &'a Ident,
    version: u32,
    branch: &'a LitStr,
    game_version: &'a LitStr,
    raknet_version: u8,
    packets: &'a BTreeMap<Ident, TokenStream>,
    packets_impl: Vec<TokenStream>,
    types_impl: Vec<TokenStream>,
    enums_impl: Vec<TokenStream>,
}

/// Renders the `Packets` impl of one version: one match arm per packet.
fn packets_trait_tokens(
    struct_ident: &Ident,
    previous_packets: &BTreeMap<Ident, TokenStream>,
) -> TokenStream {
    let packet_serialize = previous_packets.keys().map(|name| {
        quote! {
            #struct_ident::#name(pk) => {
                match <<#struct_ident as ProtoVersionPackets>::#name as bedrock_protocol_core::ProtoCodec>::serialize(pk.as_ref(), stream) {
                    Ok(()) => {},
                    Err(err) => return Err(bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                        packet_name: stringify!(#name),
                        packet_id: <<#struct_ident as ProtoVersionPackets>::#name as bedrock_protocol_core::Packet>::ID,
                        error: err
                    }),
                }
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

    quote! {
        impl bedrock_protocol_core::Packets for #struct_ident {
            #[inline]
            fn serialize<W: std::io::Write>(&self, header: &bedrock_protocol_core::PacketHeader, stream: &mut W) -> Result<(), bedrock_protocol_core::error::PacketCodecError> {
                <bedrock_protocol_core::PacketHeader as bedrock_protocol_core::ProtoCodec>::serialize(header, stream)
                    .map_err(bedrock_protocol_core::error::PacketCodecError::InvalidHeader)?;

                match self {
                    #(#packet_serialize)*
                    #struct_ident::Unknown(pk) => {
                        stream.write_all(pk.buf.as_ref())
                        .map_err(|e| bedrock_protocol_core::error::PacketCodecError::InvalidPacket {
                            packet_name: "Unknown",
                            packet_id: header.packet_id,
                            error: e.into()
                        })?;
                    }
                }

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
    }
}

/// Renders the `packet-dyn` conversions of one version.
fn dyn_impl_tokens(
    struct_ident: &Ident,
    previous_packets: &BTreeMap<Ident, TokenStream>,
) -> TokenStream {
    let packet_dyn_as_ref = previous_packets.keys().map(|name| {
        quote! { #struct_ident::#name(pk) => pk.as_ref(), }
    });

    let packet_dyn_into = previous_packets.keys().map(|name| {
        quote! { #struct_ident::#name(pk) => pk, }
    });

    quote! {
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
    }
}

/// Renders the packet enum and the trait impls of one version.
fn version_tokens(v: VersionImpl<'_>) -> TokenStream {
    let VersionImpl {
        struct_ident,
        version,
        branch,
        game_version,
        raknet_version,
        packets: previous_packets,
        packets_impl: proto_version_packets_impl,
        types_impl: proto_version_types_impl,
        enums_impl: proto_version_enums_impl,
    } = v;

    let packet_variants = previous_packets
        .keys()
        .map(|k| {
            quote! { #k(Box<<Self as ProtoVersionPackets>::#k>), }
        })
        .collect::<Vec<_>>();

    let packets_trait = packets_trait_tokens(struct_ident, previous_packets);
    let dyn_impls = dyn_impl_tokens(struct_ident, previous_packets);

    quote! {
        use crate::ProtoVersion;
        use crate::ProtoVersionPackets;
        use crate::ProtoVersionTypes;
        use crate::ProtoVersionEnums;

        #[derive(Clone, std::fmt::Debug)]
        pub enum #struct_ident {
            #(#packet_variants)*
            Unknown(Box<bedrock_protocol_core::UnknownPacket>),
        }

        #packets_trait

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

        #dyn_impls
    }
}

/// Writes `src/generated/<mod_ident>.rs`: the version tokens behind the version's feature.
fn write_version_module(
    path: &std::path::Path,
    mod_ident: &Ident,
    version_tokens: &TokenStream,
) -> syn::Result<()> {
    let feature_str = LitStr::new(&mod_ident.to_string(), mod_ident.span());

    // The generated modules hold one giant match arm per packet, so `too_many_lines`
    // cannot apply to them.
    let version_mod_tokens = quote! {
        #![allow(unused, clippy::too_many_lines)]

        #[cfg(feature = #feature_str)]
        mod inner {
            #version_tokens
        }
        #[cfg(feature = #feature_str)]
        pub use inner::*;
    };

    let file = syn::parse2(version_mod_tokens)?;

    fs::write(
        path.join(format!("src/generated/{mod_ident}.rs")),
        prettyplease::unparse(&file),
    )
    .unwrap();

    Ok(())
}

/// Expands the version definitions into `src/generated/` and returns the `mod.rs` tokens.
///
/// # Errors
///
/// Returns an error if the definitions are inconsistent: an item added twice, removed or
/// modified before it was added, or a version without a `raknet_version`.
///
/// # Panics
///
/// Panics if a generated module cannot be written to `path`.
pub fn build(input: TokenStream, path: &std::path::Path) -> syn::Result<TokenStream> {
    let DefineVersionsInput { versions } = syn::parse2::<DefineVersionsInput>(input)?;

    let mut versions_vec = versions.into_iter().collect::<Vec<_>>();

    versions_vec.sort_by_key(|v| v.version);

    let all_packets = collect_added(&versions_vec, "packet", |v| v.packets.as_ref())?;
    let all_types = collect_added(&versions_vec, "type", |v| v.types.as_ref())?;
    let all_enums = collect_added(&versions_vec, "enum", |v| v.enums.as_ref())?;

    let proto_version_packets = assoc_type_decls(&all_packets);
    let proto_version_types = assoc_type_decls(&all_types);
    let proto_version_enums = assoc_type_decls(&all_enums);

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
            format!("V{version}").as_str(),
            Span::call_site(),
        ));

        let mod_ident = Ident::new(
            struct_ident.to_string().to_lowercase().as_str(),
            Span::call_site(),
        );

        let proto_version_packets_impl = assoc_type_impls(&all_packets, &previous_packets);
        let proto_version_types_impl = assoc_type_impls(&all_types, &previous_types);
        let proto_version_enums_impl = assoc_type_impls(&all_enums, &previous_enums);

        let version_tokens = version_tokens(VersionImpl {
            struct_ident: &struct_ident,
            version,
            branch: &branch,
            game_version: &game_version,
            raknet_version,
            packets: &previous_packets,
            packets_impl: proto_version_packets_impl,
            types_impl: proto_version_types_impl,
            enums_impl: proto_version_enums_impl,
        });

        write_version_module(path, &mod_ident, &version_tokens)?;

        versions_stream.extend(quote! {
            mod #mod_ident;
            pub use #mod_ident::*;
        });
    }

    Ok(quote! {
        #![allow(unused)]

        #proto_version

        #versions_stream
    })
}
