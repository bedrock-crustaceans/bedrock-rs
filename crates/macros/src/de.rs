use crate::attr::{ProtoCodecEndianness, extract_inner_type_from_vec, get_attrs};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, DataEnum, DataStruct, Field, Fields, Type};

fn build_de_instance(endianness: Option<ProtoCodecEndianness>, f_type: &Type) -> TokenStream {
    match endianness {
        None => {
            quote! { <#f_type as ::bedrockrs_proto_core::ProtoCodec>::deserialize(stream)? }
        }
        Some(ProtoCodecEndianness::Le) => {
            quote! { <#f_type as ::bedrockrs_proto_core::ProtoCodecLE>::deserialize(stream)? }
        }
        Some(ProtoCodecEndianness::Be) => {
            quote! { <#f_type as ::bedrockrs_proto_core::ProtoCodecBE>::deserialize(stream)? }
        }
        Some(ProtoCodecEndianness::Var) => {
            quote! { <#f_type as ::bedrockrs_proto_core::ProtoCodecVAR>::deserialize(stream)? }
        }
    }
}

fn build_de_field(fields: &[&Field]) -> TokenStream {
    let code = fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let name = f.ident.clone().unwrap_or(Ident::new(&format!("e{i}"), Span::call_site()));
            let ty = f.ty.clone();
            let flags = get_attrs(f.attrs.as_slice()).expect("Error while getting attrs");

            if let Some(repr) = flags.vec_repr {
                let vec_des = build_de_instance(flags.vec_endianness, &repr);
                let inner_ty = extract_inner_type_from_vec(&ty).expect("Failed to get inner Vec type").clone();
                let des = build_de_instance(flags.endianness, &inner_ty);

                return quote! {
                    let #name = {
                        let len: #repr = #vec_des;

                        let mut vec = Vec::with_capacity(len.try_into()?);

                        for _ in 0..len {
                            vec.push(#des);
                        };

                        vec
                    };
                };
            }

            if flags.nbt {
                return quote! {
                    let #name: #ty = ::nbtx::from_bytes::<::nbtx::NetworkLittleEndian, _>(stream)?;
                };
            }

            if flags.str {
                return quote! {
                    let #name: #ty = {
                        let string = <String as ::bedrockrs_proto_core::ProtoCodec>::deserialize(stream)?;
                        <#ty as std::str::FromStr>::from_str(string.as_str())?
                    };
                };
            }

            let des = build_de_instance(flags.endianness, &ty);

            quote! {
                let #name: #ty = #des;
            }
        });

    quote! {
        #(#code)*
    }
}

fn build_de_fields(fields: Fields) -> (TokenStream, Option<TokenStream>) {
    let i_fields = match fields {
        Fields::Named(ref v) => Some(v.named.iter().clone()),
        Fields::Unnamed(ref v) => Some(v.unnamed.iter().clone()),
        Fields::Unit => None,
    };

    let de = if let Some(i_fields) = i_fields {
        build_de_field(Vec::from_iter(i_fields).as_slice())
    } else {
        quote! {}
    };

    let ctor_fields = match fields {
        Fields::Named(ref v) => {
            let ctor = v.named.iter().enumerate().map(|(i, f)| {
                f.ident
                    .clone()
                    .unwrap_or(Ident::new(&format!("e{i}"), Span::call_site()))
            });

            Some(quote! { {#(#ctor),*} })
        }
        Fields::Unnamed(ref v) => {
            let ctor = v.unnamed.iter().enumerate().map(|(i, f)| {
                f.ident
                    .clone()
                    .unwrap_or(Ident::new(&format!("e{i}"), Span::call_site()))
            });

            Some(quote! { (#(#ctor),*) })
        }
        Fields::Unit => None,
    };

    (de, ctor_fields)
}

pub fn build_de_struct(data_struct: &DataStruct) -> TokenStream {
    let (de, ctor_fields) = build_de_fields(data_struct.fields.clone());

    if let Some(ctor) = ctor_fields {
        quote! {
            #de
            let val = Self #ctor;
        }
    } else {
        quote! {
            #de
            let val = Self;
        }
    }
}

pub fn build_de_enum(data_enum: &DataEnum, attrs: &[Attribute], name: Ident) -> TokenStream {
    let flags = get_attrs(attrs).expect("Error while getting attrs");

    if let (Some(repr), endian) = (flags.enum_repr, flags.enum_endianness) {
        let enum_type_de = build_de_instance(endian, &repr);

        let variants = data_enum.variants.iter().map(|var| {
            let desc = var
                .discriminant
                .clone()
                .unwrap_or_else(|| panic!("Missing discriminant for {:?}", var.ident))
                .1;

            let name = var.ident.clone();
            let (de, ctor_fields) = build_de_fields(var.fields.clone());

            if let Some(ctor) = ctor_fields {
                quote! {
                    #desc => {
                        #de

                        Self::#name #ctor
                    }
                }
            } else {
                quote! {
                    #desc => {
                        #de

                        Self::#name
                    }
                }
            }
        });

        // We need to find a solution for what happens when an enum type is not found
        quote! {
            let enum_type = #enum_type_de;

            let val = match enum_type {
                #(#variants),*
                _ => { return Err(bedrockrs_proto_core::error::ProtoCodecError::InvalidEnumID(format!("{enum_type:?}"), stringify!(#name))) },
            };
        }
    } else {
        panic!("Missing attr `enum_repr` or `enum_endianness` on enum")
    }
}
