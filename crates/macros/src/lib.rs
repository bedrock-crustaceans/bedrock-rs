use crate::de::{build_de_enum, build_de_struct};
use crate::ser::{build_ser_enum, build_ser_struct};
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};
use syn::token::Token;
use syn::{parse_macro_input, Data, DeriveInput, Lit, Token};

mod attr;
mod de;
mod proto;
mod ser;
mod size;

#[proc_macro_derive(
    ProtoCodec,
    attributes(
        endianness,
        vec_endianness,
        vec_repr,
        enum_endianness,
        enum_repr,
        nbt,
        str
    )
)]
pub fn proto_codec_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = input.ident;

    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (ser, de) = match input.data {
        Data::Struct(v) => (build_ser_struct(&v), build_de_struct(&v)),
        Data::Enum(v) => (
            build_ser_enum(&v, input.attrs.as_slice()),
            build_de_enum(&v, input.attrs.as_slice(), name.clone()),
        ),
        Data::Union(_) => {
            return proc_macro::TokenStream::from(quote! {
                compile_error!("ProtoCodec derive macro only supports structs and enums")
            })
        }
    };

    let expanded = quote! {
        impl #impl_generics ::bedrockrs_proto_core::ProtoCodec for #name #ty_generics #where_clause {
            fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ::bedrockrs_proto_core::error::ProtoCodecError> where Self: Sized {
                #[cfg(debug_assertions)]
                ::log::trace!("ProtoSerialize: {}", stringify!(#name));
                #ser
                Ok(())
            }

            fn proto_deserialize(stream: &mut ::std::io::Cursor<&[u8]>) -> Result<Self, ::bedrockrs_proto_core::error::ProtoCodecError> where Self: Sized {
                #[cfg(debug_assertions)]
                ::log::trace!("ProtoDeserialize: {}", stringify!(#name));
                #de
                Ok(val)
            }

            fn get_size_prediction(&self) -> usize {
                1
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

struct GamepacketInput {
    id: Lit,
    compress: Option<Lit>,
    encrypt: Option<Lit>,
}

impl Parse for GamepacketInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut map = HashMap::new();

        loop {
            if !input.peek(syn::Ident) {
                break;
            }

            let param_name = input.parse::<syn::Ident>()?.to_string();
            input.parse::<Token![=]>()?;
            let param_value = input.parse::<syn::Lit>()?;

            map.insert(param_name, param_value);

            if !input.peek(Token![,]) {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        let id = map.remove(&String::from("id")).unwrap_or_else(|| {
            panic!("Missing id");
        });

        Ok(Self {
            id: id.clone(),
            compress: map.remove("compress"),
            encrypt: map.remove("encrypt"),
        })
    }
}

#[proc_macro_attribute]
pub fn gamepacket(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the arguments passed to the attribute
    let args = parse_macro_input!(args as GamepacketInput);
    let item_de = item.clone();
    let derive = parse_macro_input!(item_de as DeriveInput);
    let name = derive.ident;

    let id = args.id;

    let compress = match args.compress {
        Some(v) => quote! {#v},
        None => quote! {true},
    };

    let encrypt = match args.encrypt {
        Some(v) => quote! {#v},
        None => quote! {true},
    };

    let item = proc_macro2::TokenStream::from(item);

    let generics = derive.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        #item

        impl #impl_generics ::bedrockrs_proto_core::GamePacket for #name #ty_generics #where_clause {
            const ID: u16 = #id;
            const COMPRESS: bool = #compress;
            const ENCRYPT: bool = #encrypt;
        }
    };

    proc_macro::TokenStream::from(expanded)
}

struct GamepacketsInput {
    packets: Vec<(proc_macro2::Ident, Option<syn::Type>)>,
}

impl Parse for GamepacketsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut vec = vec![];

        loop {
            if !input.peek(syn::Ident) {
                break;
            }

            let param_name = input.parse::<syn::Ident>()?;
            input.parse::<Token![:]>()?;

            let param_value = if input.peek(Token![_]) {
                input.parse::<Token![_]>()?;
                None
            } else {
                Some(input.parse::<syn::Type>()?)
            };

            vec.push((param_name, param_value));

            if !input.peek(Token![,]) {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        Ok(Self { packets: vec })
    }
}

#[proc_macro]
pub fn gamepackets(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = parse_macro_input!(input as GamepacketsInput);

    let variants = args.packets.clone();
    let variants = variants.iter().map(|(name, value)| {
        if let Some(value) = value {
            quote! { #name(#value), }
        } else {
            quote! { #name(), }
        }
    });

    let id = args.packets.clone();
    let id = id.iter().map(|(name, value)| {
        if let Some(v) = value {
            quote! { GamePackets::#name(_) => { return <#v as ::bedrockrs_proto_core::GamePacket>::ID; }, }
        } else {
            quote! { GamePackets::#name() => { todo!("impl GamePackets::{}", stringify!(name)); }, }
        }
    });

    let compress = args.packets.clone();
    let compress = compress.iter().map(|(name, value)| {
        if let Some(v) = value {
            quote! { GamePackets::#name(_) => { return <#v as ::bedrockrs_proto_core::GamePacket>::COMPRESS; }, }
        } else {
            quote! { GamePackets::#name() => { todo!("impl GamePackets::{}", stringify!(name)); }, }
        }
    });

    let encrypt = args.packets.clone();
    let encrypt = encrypt.iter().map(|(name, value)| {
        if let Some(v) = value {
            quote! { GamePackets::#name(_) => { return <#v as ::bedrockrs_proto_core::GamePacket>::ENCRYPT; }, }
        } else {
            quote! { GamePackets::#name() => { todo!("impl GamePackets::{}", stringify!(name)); }, }
        }
    });

    let size_prediction = args.packets.clone();
    let size_prediction = size_prediction.iter().map(|(name, value)| {
        if let Some(v) = value {
            quote! { GamePackets::#name(pk) => <#v as ::bedrockrs_proto_core::GamePacket>::get_size_prediction(pk), }
        } else {
            quote! { GamePackets::#name() => { todo!("impl GamePackets::{}", stringify!(name)); }, }
        }
    });

    let ser = args.packets.clone();
    let ser = ser.iter().map(|(name, value)| {
        if let Some(v) = value {
            quote! {
                GamePackets::#name(pk) => {
                    let mut buf = Vec::new();

                    match <#v as bedrockrs_proto_core::ProtoCodec>::proto_serialize(pk, &mut buf) {
                        Ok(_) => {},
                        Err(err) => return Err(err),
                    };

                    let len: u32 = match buf.len().try_into() {
                        Ok(len) => len,
                        Err(err) => return Err(::bedrockrs_proto_core::error::ProtoCodecError::FromIntError(err)),
                    };

                    match write_gamepacket_header(stream, len, <#v as ::bedrockrs_proto_core::GamePacket>::ID, subclient_sender_id, subclient_target_id) {
                        Ok(_) => {},
                        Err(err) => return Err(err),
                    };

                    match stream.write_all(buf.as_slice()) {
                        Ok(_) => {},
                        Err(err) => return Err(::bedrockrs_proto_core::error::ProtoCodecError::IOError(err)),
                    };
                },
            }
        } else {
            quote! {
                GamePackets::#name() => { todo!("impl GamePackets::{}", stringify!(name)) },
            }
        }
    });

    let de = args.packets.clone();
    let de = de.iter().map(|(name, value)| {
        if let Some(v) = value {
            quote! {
                <#v as ::bedrockrs_proto_core::GamePacket>::ID => {
                    match <#v as ::bedrockrs_proto_core::ProtoCodec>::proto_deserialize(stream) {
                        Ok(pk) => GamePackets::#name(pk),
                        Err(e) => return Err(e),
                    }
                },
            }
        } else {
            quote! {}
        }
    });

    let expanded = quote! {
        #[repr(u16)]
        #[derive(Debug, Clone)]
        pub enum GamePackets {
            #(#variants)*
        }

        impl ::bedrockrs_proto_core::GamePacketsAll for GamePackets {
            #[inline]
            fn id(&self) -> u16 {
                match self {
                    #(#id)*
                };
            }

            #[inline]
            fn compress(&self) -> bool {
                match self {
                    #(#compress)*
                };
            }

            #[inline]
            fn encrypt(&self) -> bool {
                match self {
                    #(#encrypt)*
                };
            }

            #[inline]
            fn pk_serialize(&self, stream: &mut Vec<u8>, subclient_sender_id: SubClientID, subclient_target_id: SubClientID) -> Result<(), ::bedrockrs_proto_core::error::ProtoCodecError> {
                match self {
                    #(#ser)*
                };

                Ok(())
            }

            #[inline]
            fn pk_deserialize(stream: &mut Cursor<&[u8]>) -> Result<(GamePackets, SubClientID, SubClientID), ::bedrockrs_proto_core::error::ProtoCodecError> {
                let (_length, gamepacket_id, subclient_sender_id, subclient_target_id) = match read_gamepacket_header(stream) {
                    Ok(val) => val,
                    Err(err) => return Err(err),
                };

                let gamepacket = match gamepacket_id {
                    #(#de)*
                    other => {
                        return Err(::bedrockrs_proto_core::error::ProtoCodecError::InvalidGamePacketID(other));
                    },
                };

                Ok((gamepacket, subclient_sender_id, subclient_target_id))
            }

            #[inline]
            fn get_size_prediction(&self) -> usize {
                let len = match self {
                    #(#size_prediction)*
                };

                len + get_gamepacket_header_size_prediction()
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn define_versions(input: TokenStream) -> TokenStream {
    proto::define_versions_internal(input)
}
