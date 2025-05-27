//! Procedural macros for deriving SimpleSerialize trait for Causality types.
//!
//! This module provides the #[derive(SimpleSerialize)] macro which automatically
//! implements the SimpleSerialize trait as well as its prerequisites (Encode and Decode).

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Index};

/// Derives the SimpleSerialize trait for a struct or enum.
///
/// This macro will automatically implement the SimpleSerialize trait as well as its prerequisites (Encode and Decode).
/// It will skip implementation for primitive types and types that already have implementations.
#[proc_macro_derive(SimpleSerialize, attributes(ssz, ssz_skip, ssz_size))]
pub fn derive_simple_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate Encode implementation
    let encode_impl = generate_encode_impl(&input);
    
    // Generate Decode implementation  
    let decode_impl = generate_decode_impl(&input);

    let expanded = quote! {
        impl #impl_generics crate::serialization::Encode for #name #ty_generics #where_clause {
            fn as_ssz_bytes(&self) -> Vec<u8> {
                #encode_impl
            }
        }

        impl #impl_generics crate::serialization::Decode for #name #ty_generics #where_clause {
            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, crate::serialization::DecodeError> {
                #decode_impl
            }
        }

        impl #impl_generics crate::serialization::SimpleSerialize for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

fn generate_encode_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let field_encodings = fields.named.iter().map(|field| {
                        let field_name = &field.ident;
                        quote! {
                            result.extend(self.#field_name.as_ssz_bytes());
                        }
                    });
                    
                    quote! {
                        let mut result = Vec::new();
                        #(#field_encodings)*
                        result
                    }
                }
                Fields::Unnamed(fields) => {
                    let field_encodings = fields.unnamed.iter().enumerate().map(|(i, _)| {
                        let index = Index::from(i);
                        quote! {
                            result.extend(self.#index.as_ssz_bytes());
                        }
                    });
                    
                    quote! {
                        let mut result = Vec::new();
                        #(#field_encodings)*
                        result
                    }
                }
                Fields::Unit => {
                    quote! {
                        Vec::new()
                    }
                }
            }
        }
        Data::Enum(data_enum) => {
            let variant_encodings = data_enum.variants.iter().enumerate().map(|(i, variant)| {
                let variant_name = &variant.ident;
                let discriminant = i as u8;
                
                match &variant.fields {
                    Fields::Named(fields) => {
                        let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                        let field_encodings = field_names.iter().map(|name| {
                            quote! {
                                result.extend(#name.as_ssz_bytes());
                            }
                        });
                        
                        quote! {
                            Self::#variant_name { #(#field_names),* } => {
                                let mut result = vec![#discriminant];
                                #(#field_encodings)*
                                result
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let field_names: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| format_ident!("field_{}", i))
                            .collect();
                        let field_encodings = field_names.iter().map(|name| {
                            quote! {
                                result.extend(#name.as_ssz_bytes());
                            }
                        });
                        
                        quote! {
                            Self::#variant_name(#(#field_names),*) => {
                                let mut result = vec![#discriminant];
                                #(#field_encodings)*
                                result
                            }
                        }
                    }
                    Fields::Unit => {
                        quote! {
                            Self::#variant_name => vec![#discriminant]
                        }
                    }
                }
            });
            
            quote! {
                match self {
                    #(#variant_encodings)*
                }
            }
        }
        Data::Union(_) => {
            quote! {
                compile_error!("SimpleSerialize cannot be derived for unions")
            }
        }
    }
}

fn generate_decode_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let field_decodings = fields.named.iter().map(|field| {
                        let field_name = &field.ident;
                        let field_type = &field.ty;
                        quote! {
                            let #field_name = <#field_type as crate::serialization::Decode>::from_ssz_bytes(&bytes[offset..])?;
                            let field_bytes = #field_name.as_ssz_bytes();
                            offset += field_bytes.len();
                        }
                    });
                    
                    let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                    
                    quote! {
                        let mut offset = 0;
                        #(#field_decodings)*
                        Ok(Self {
                            #(#field_names),*
                        })
                    }
                }
                Fields::Unnamed(fields) => {
                    let field_decodings = fields.unnamed.iter().enumerate().map(|(i, field)| {
                        let field_name = format_ident!("field_{}", i);
                        let field_type = &field.ty;
                        quote! {
                            let #field_name = <#field_type as crate::serialization::Decode>::from_ssz_bytes(&bytes[offset..])?;
                            let field_bytes = #field_name.as_ssz_bytes();
                            offset += field_bytes.len();
                        }
                    });
                    
                    let field_names: Vec<_> = (0..fields.unnamed.len())
                        .map(|i| format_ident!("field_{}", i))
                        .collect();
                    
                    quote! {
                        let mut offset = 0;
                        #(#field_decodings)*
                        Ok(Self(#(#field_names),*))
                    }
                }
                Fields::Unit => {
                    quote! {
                        Ok(Self)
                    }
                }
            }
        }
        Data::Enum(data_enum) => {
            let variant_decodings = data_enum.variants.iter().enumerate().map(|(i, variant)| {
                let variant_name = &variant.ident;
                let discriminant = i as u8;
                
                match &variant.fields {
                    Fields::Named(fields) => {
                        let field_decodings = fields.named.iter().map(|field| {
                            let field_name = &field.ident;
                            let field_type = &field.ty;
                            quote! {
                                let #field_name = <#field_type as crate::serialization::Decode>::from_ssz_bytes(&bytes[offset..])?;
                                let field_bytes = #field_name.as_ssz_bytes();
                                offset += field_bytes.len();
                            }
                        });
                        
                        let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                        
                        quote! {
                            #discriminant => {
                                let mut offset = 1;
                                #(#field_decodings)*
                                Ok(Self::#variant_name { #(#field_names),* })
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let field_decodings = fields.unnamed.iter().enumerate().map(|(i, field)| {
                            let field_name = format_ident!("field_{}", i);
                            let field_type = &field.ty;
                            quote! {
                                let #field_name = <#field_type as crate::serialization::Decode>::from_ssz_bytes(&bytes[offset..])?;
                                let field_bytes = #field_name.as_ssz_bytes();
                                offset += field_bytes.len();
                            }
                        });
                        
                        let field_names: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| format_ident!("field_{}", i))
                            .collect();
                        
                        quote! {
                            #discriminant => {
                                let mut offset = 1;
                                #(#field_decodings)*
                                Ok(Self::#variant_name(#(#field_names),*))
                            }
                        }
                    }
                    Fields::Unit => {
                        quote! {
                            #discriminant => Ok(Self::#variant_name)
                        }
                    }
                }
            });
            
            quote! {
                if bytes.is_empty() {
                    return Err(crate::serialization::DecodeError {
                        message: "Empty bytes for enum".to_string(),
                    });
                }
                
                match bytes[0] {
                    #(#variant_decodings)*
                    _ => Err(crate::serialization::DecodeError {
                        message: format!("Invalid discriminant: {}", bytes[0]),
                    })
                }
            }
        }
        Data::Union(_) => {
            quote! {
                compile_error!("SimpleSerialize cannot be derived for unions")
            }
        }
    }
} 