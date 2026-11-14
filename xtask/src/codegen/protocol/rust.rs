//! The Rust files the generator writes: the traits, the packet enum of a version with its
//! impls, the facade's re-exports, and the `lib.rs` a new version crate starts from.

use crate::codegen::protocol::diff::{Item, crate_name};
use crate::codegen::protocol::{Definition, Version};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use syn::LitStr;

/// The tokens as the text of a file.
fn unparse(tokens: TokenStream) -> syn::Result<String> {
    Ok(prettyplease::unparse(&syn::parse2(tokens)?))
}

/// The `type X: ...;` declarations of one `ProtoVersion*` trait.
fn assoc_type_decls(names: &BTreeSet<Ident>) -> Vec<TokenStream> {
    names
        .iter()
        .map(|p| quote!(type #p: bedrock_protocol_core::ProtoCodec + Clone + std::fmt::Debug + Send + Sync + 'static;))
        .collect()
}

/// The `type X = ...;` definitions of one `ProtoVersion*` impl in the crate of the version
/// `module`; items the version does not have yet are `()`.
fn assoc_type_impls(
    names: &BTreeSet<Ident>,
    current: &BTreeMap<Ident, Item>,
    module: &Ident,
) -> Vec<TokenStream> {
    names
        .iter()
        .map(|k| {
            if let Some(item) = current.get(k) {
                let path = item.path_in(module);
                quote!(type #k = #path;)
            } else {
                quote!(type #k = ();)
            }
        })
        .collect()
}

/// `src/generated.rs` of the traits crate: the four `ProtoVersion*` traits, with one associated
/// type for every item some version adds.
pub fn traits(definition: &Definition) -> syn::Result<String> {
    let proto_version_packets = assoc_type_decls(&definition.packets);
    let proto_version_types = assoc_type_decls(&definition.types);
    let proto_version_enums = assoc_type_decls(&definition.enums);

    unparse(quote! {
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
    })
}

/// Renders the `Packets` impl of one version: one match arm per packet.
fn packets_trait_tokens(
    struct_ident: &Ident,
    previous_packets: &BTreeMap<Ident, Item>,
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
fn dyn_impl_tokens(struct_ident: &Ident, previous_packets: &BTreeMap<Ident, Item>) -> TokenStream {
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

/// `src/generated.rs` of one version crate: the packet enum and its trait impls. The crate's
/// `lib.rs` puts the module behind the `version` feature.
pub fn version(definition: &Definition, version: &Version) -> syn::Result<String> {
    let Version {
        module,
        enum_ident: struct_ident,
        number,
        branch,
        game_version,
        raknet_version,
        packets,
        types,
        enums,
        ..
    } = version;

    let packet_variants = packets
        .keys()
        .map(|k| {
            quote! { #k(Box<<Self as ProtoVersionPackets>::#k>), }
        })
        .collect::<Vec<_>>();

    let packets_trait = packets_trait_tokens(struct_ident, packets);
    let dyn_impls = dyn_impl_tokens(struct_ident, packets);

    let proto_version_packets_impl = assoc_type_impls(&definition.packets, packets, module);
    let proto_version_types_impl = assoc_type_impls(&definition.types, types, module);
    let proto_version_enums_impl = assoc_type_impls(&definition.enums, enums, module);

    // The generated modules hold one giant match arm per packet, so `too_many_lines`
    // cannot apply to them.
    unparse(quote! {
        #![allow(unused, clippy::too_many_lines)]

        use bedrock_protocol_version::ProtoVersion;
        use bedrock_protocol_version::ProtoVersionPackets;
        use bedrock_protocol_version::ProtoVersionTypes;
        use bedrock_protocol_version::ProtoVersionEnums;

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
            const PROTOCOL_VERSION: u32 = #number;
            const PROTOCOL_BRANCH: &str = #branch;
            const GAME_VERSION: &str = #game_version;
            const RAKNET_VERSION: u8 = #raknet_version;
        }

        #dyn_impls
    })
}

/// `src/generated.rs` of the facade: the traits, every version crate under its module ident,
/// and the packet enum of each behind the feature of its version.
pub fn facade(definition: &Definition) -> syn::Result<String> {
    let versions = definition.versions.iter().map(|version| {
        let Version {
            module, enum_ident, ..
        } = version;
        let name = format_ident!("{}", crate_name(module));
        let feature = LitStr::new(&module.to_string(), module.span());
        // Inlined: rustdoc then documents the crate as a module of the facade, not as one
        // line of re-export.
        quote! {
            #[doc(inline)]
            pub use #name as #module;
            #[cfg(feature = #feature)]
            pub use #name::#enum_ident;
        }
    });

    unparse(quote! {
        pub use bedrock_protocol_version::*;

        #(#versions)*
    })
}

/// The `src/lib.rs` a version crate without one starts from: the branch as its doc line, the
/// modules the definition puts its items in, and the generated module behind `version`.
pub fn lib_scaffold(version: &Version) -> String {
    let mut out = format!("//! `{}`\n\n", version.branch.value());
    for module in &version.modules {
        let _ = writeln!(out, "pub mod {module};");
    }
    if !version.modules.is_empty() {
        out.push('\n');
    }
    out.push_str(
        "#[cfg(feature = \"version\")]\nmod generated;\n#[cfg(feature = \"version\")]\npub use generated::*;\n",
    );
    out
}
