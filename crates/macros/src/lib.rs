use crate::de::{build_de_enum, build_de_struct};
use crate::ser::{build_ser_enum, build_ser_struct};
use crate::size::{build_size_enum, build_size_struct};
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};
use syn::{Data, DeriveInput, Lit, Token, parse_macro_input};

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
pub fn proto_codec_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = input.ident;

    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (ser, de, size) = match input.data {
        Data::Struct(v) => (
            build_ser_struct(&v),
            build_de_struct(&v),
            build_size_struct(&v),
        ),
        Data::Enum(v) => (
            build_ser_enum(&v, input.attrs.as_slice()),
            build_de_enum(&v, input.attrs.as_slice(), name.clone()),
            build_size_enum(&v, input.attrs.as_slice()),
        ),
        Data::Union(_) => {
            return TokenStream::from(quote! {
                compile_error!("ProtoCodec derive macro only supports structs and enums")
            });
        }
    };

    let expanded = quote! {
        impl #impl_generics ::bedrockrs_proto_core::ProtoCodec for #name #ty_generics #where_clause {
            fn serialize<W: ::std::io::Write>(&self, stream: &mut W) -> Result<(), ::bedrockrs_proto_core::error::ProtoCodecError> where Self: Sized {
                #[cfg(debug_assertions)]
                ::tracing::trace!("ProtoSerialize: {}", stringify!(#name));
                #ser
                Ok(())
            }

            fn deserialize<R: ::std::io::Read>(stream: &mut R) -> Result<Self, ::bedrockrs_proto_core::error::ProtoCodecError> where Self: Sized {
                #[cfg(debug_assertions)]
                ::tracing::trace!("ProtoDeserialize: {}", stringify!(#name));
                #de
                Ok(val)
            }

            fn size_hint(&self) -> usize {
                #size
            }
        }
    };

    TokenStream::from(expanded)
}

struct PacketInput {
    id: Lit,
    compress: Option<Lit>,
    encrypt: Option<Lit>,
}

impl Parse for PacketInput {
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
pub fn packet(args: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the arguments passed to the attribute
    let args = parse_macro_input!(args as PacketInput);
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

        impl #impl_generics ::bedrockrs_proto_core::Packet for #name #ty_generics #where_clause {
            const ID: u16 = #id;
            const COMPRESS: bool = #compress;
            const ENCRYPT: bool = #encrypt;
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn define_versions(input: TokenStream) -> TokenStream {
    proto::define_versions_internal(input)
}
