use crate::de::{build_de_enum, build_de_struct};
use crate::ser::{build_ser_enum, build_ser_struct};
use crate::size::{build_size_enum, build_size_struct};
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Data, DeriveInput, LitInt, Token, parse_macro_input};

mod attr;
mod de;
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
        impl #impl_generics ::bedrock_protocol_core::ProtoCodec for #name #ty_generics #where_clause {
            fn serialize<W: ::std::io::Write>(&self, stream: &mut W) -> Result<(), ::bedrock_protocol_core::error::ProtoCodecError> where Self: Sized {
                #[cfg(debug_assertions)]
                ::tracing::trace!("ProtoSerialize: {}", stringify!(#name));
                #ser
                Ok(())
            }

            fn deserialize<R: ::std::io::Read>(stream: &mut R) -> Result<Self, ::bedrock_protocol_core::error::ProtoCodecError> where Self: Sized {
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
    id: LitInt,
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(id);
}

impl Parse for PacketInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::id>()?;
        input.parse::<Token![=]>()?;

        Ok(Self {
            id: input.parse::<LitInt>()?,
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

    let item = proc_macro2::TokenStream::from(item);

    let generics = derive.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        #item

        impl #impl_generics ::bedrock_protocol_core::Packet for #name #ty_generics #where_clause {
            const ID: u16 = #id;
        }
    };

    TokenStream::from(expanded)
}
