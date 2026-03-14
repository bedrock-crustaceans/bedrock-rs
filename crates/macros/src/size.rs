use crate::attr::{ProtoCodecEndianness, extract_inner_type_from_vec, get_attrs};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, DataEnum, DataStruct, Field, Fields, Type};

fn build_size_instance(
    endianness: Option<ProtoCodecEndianness>,
    f_type: &Type,
    f_name: TokenStream,
) -> TokenStream {
    match endianness {
        None => {
            quote! { <#f_type as ::bedrockrs_proto_core::ProtoCodec>::size_hint(&#f_name) }
        }
        Some(ProtoCodecEndianness::Le) => {
            quote! { <#f_type as ::bedrockrs_proto_core::ProtoCodecLE>::size_hint(&#f_name) }
        }
        Some(ProtoCodecEndianness::Be) => {
            quote! { <#f_type as ::bedrockrs_proto_core::ProtoCodecBE>::size_hint(&#f_name) }
        }
        Some(ProtoCodecEndianness::Var) => {
            quote! { <#f_type as ::bedrockrs_proto_core::ProtoCodecVAR>::size_hint(&#f_name) }
        }
    }
}

fn build_size_field(
    fields: &[&Field],
    f_prefix: Option<TokenStream>,
    vec_by_ref: bool,
) -> TokenStream {
    let code = fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let name = f.ident.clone().unwrap_or(Ident::new(&format!("e{i}"), Span::call_site()));
            let final_name = if let Some(prefix) = &f_prefix {
                quote! { #prefix.#name }
            } else {
                quote! { #name }
            };

            let ty = f.ty.clone();
            let flags = get_attrs(f.attrs.as_slice()).expect("Error while getting attrs");

            if let Some(repr) = flags.vec_repr {
                let vec_size = build_size_instance(flags.vec_endianness, &repr, quote! { len });
                let inner_ty = extract_inner_type_from_vec(&ty).expect("Failed to get inner Vec type").clone();
                let size = build_size_instance(flags.endianness, &inner_ty, quote! { i });

                let vec_prefix = if vec_by_ref {
                    quote! { & }
                } else {
                    quote! {}
                };

                return quote! {
                    {
                        let len: #repr = #final_name.len() as #repr;

                        size += #vec_size;

                        for i in #vec_prefix #final_name {
                            size += #size;
                        };
                    };
                };
            }

            if flags.nbt {
                return quote! {
                    size += 1;
                };
            }

            if flags.str {
                return quote! {
                    size += <String as ::bedrockrs_proto_core::ProtoCodec>::size_hint(&ToString::to_string(&#final_name));
                };
            }

            let size = build_size_instance(flags.endianness, &ty, final_name);

            quote! {
                size += #size;
            }
        });

    quote! {
        #(#code)*
    }
}

fn build_size_fields(
    fields: Fields,
    f_prefix: Option<TokenStream>,
    vec_by_ref: bool,
) -> (TokenStream, Option<TokenStream>) {
    let i_fields = match fields {
        Fields::Named(ref v) => Some(v.named.iter().clone()),
        Fields::Unnamed(ref v) => Some(v.unnamed.iter().clone()),
        Fields::Unit => None,
    };

    let ser = if let Some(i_fields) = i_fields {
        build_size_field(Vec::from_iter(i_fields).as_slice(), f_prefix, vec_by_ref)
    } else {
        quote! {}
    };

    let fields = match fields {
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

    (ser, fields)
}

pub fn build_size_struct(data_struct: &DataStruct) -> TokenStream {
    let (ser, _) = build_size_fields(data_struct.fields.clone(), Some(quote! { self }), true);

    quote! {
        let mut size: usize = 0;
        #ser;
        size
    }
}

pub fn build_size_enum(data_enum: &DataEnum, attrs: &[Attribute]) -> TokenStream {
    let flags = get_attrs(attrs).expect("Error while getting attrs");

    if let (Some(repr), endian) = (flags.enum_repr, flags.enum_endianness) {
        let variants = data_enum.variants.iter().map(|var| {
            let desc = var
                .discriminant
                .clone()
                .unwrap_or_else(|| panic!("Missing discriminant for {:?}", var.ident))
                .1;

            let enum_type_size = build_size_instance(endian.clone(), &repr, quote! {#desc});
            let name = var.ident.clone();
            let (size, fields) = build_size_fields(var.fields.clone(), None, false);

            if let Some(fields) = fields {
                quote! {
                    Self::#name #fields => {
                        size += #enum_type_size;
                        #size
                    }
                }
            } else {
                quote! {
                    Self::#name => {
                        size += #enum_type_size;
                        #size
                    }
                }
            }
        });

        quote! {
            let mut size: usize = 0;
            match self {
                #(#variants),*
            }
            size
        }
    } else {
        panic!("Missing attr `enum_repr` or `enum_endianness` on enum")
    }
}
