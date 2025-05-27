//! Procedural macros for deriving SimpleSerialize trait for Causality types.
//!
//! This module provides the #[derive(SimpleSerialize)] macro which automatically
//! implements the SimpleSerialize trait as well as its prerequisites (Encode and Decode).
//! Also provides #[derive(TypeSchema)] for automatic schema generation.

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Index, Type, PathArguments, GenericArgument};

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

/// Derives TypeSchema for a struct or enum, automatically generating TypeExpr
#[proc_macro_derive(TypeSchema)]
pub fn derive_type_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let type_expr_impl = generate_type_expr_impl(&input);

    let expanded = quote! {
        impl #impl_generics crate::expression::type_::AsSchema for #name #ty_generics #where_clause {
            fn schema_id(&self) -> crate::primitive::ids::TypeExprId {
                self.type_expr().id()
            }
        }

        impl #impl_generics #name #ty_generics #where_clause {
            /// Get the TypeExpr representation of this type
            pub fn type_expr() -> crate::expression::type_::TypeExpr {
                #type_expr_impl
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_type_expr_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let field_mappings = fields.named.iter().map(|field| {
                        let field_name = field.ident.as_ref().unwrap().to_string();
                        let field_type = &field.ty;
                        let type_expr = rust_type_to_type_expr(field_type);
                        quote! {
                            fields.insert(
                                crate::primitive::string::Str::from(#field_name),
                                #type_expr
                            );
                        }
                    });
                    
                    quote! {
                        {
                            use std::collections::BTreeMap;
                            use crate::expression::type_::{TypeExpr, TypeExprMap};
                            let mut fields = BTreeMap::new();
                            #(#field_mappings)*
                            TypeExpr::Record(TypeExprMap(fields))
                        }
                    }
                }
                Fields::Unnamed(fields) => {
                    let field_types = fields.unnamed.iter().map(|field| {
                        rust_type_to_type_expr(&field.ty)
                    });
                    
                    quote! {
                        {
                            use crate::expression::type_::{TypeExpr, TypeExprVec};
                            let types = vec![#(#field_types),*];
                            TypeExpr::Tuple(TypeExprVec(types))
                        }
                    }
                }
                Fields::Unit => {
                    quote! {
                        crate::expression::type_::TypeExpr::Unit
                    }
                }
            }
        }
        Data::Enum(data_enum) => {
            let variant_types = data_enum.variants.iter().map(|variant| {
                match &variant.fields {
                    Fields::Named(_) => {
                        // For named fields, create a record type for this variant
                        quote! { crate::expression::type_::TypeExpr::Any }
                    }
                    Fields::Unnamed(fields) => {
                        if fields.unnamed.len() == 1 {
                            let field_type = &fields.unnamed[0].ty;
                            rust_type_to_type_expr(field_type)
                        } else {
                            quote! { crate::expression::type_::TypeExpr::Any }
                        }
                    }
                    Fields::Unit => {
                        quote! { crate::expression::type_::TypeExpr::Unit }
                    }
                }
            });
            
            quote! {
                {
                    use crate::expression::type_::{TypeExpr, TypeExprVec};
                    let variants = vec![#(#variant_types),*];
                    TypeExpr::Union(TypeExprVec(variants))
                }
            }
        }
        Data::Union(_) => {
            quote! {
                compile_error!("TypeSchema cannot be derived for unions")
            }
        }
    }
}

fn rust_type_to_type_expr(ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                match segment.ident.to_string().as_str() {
                    "bool" => quote! { crate::expression::type_::TypeExpr::Bool },
                    "String" => quote! { crate::expression::type_::TypeExpr::String },
                    "i8" | "i16" | "i32" | "i64" | "isize" => quote! { crate::expression::type_::TypeExpr::Integer },
                    "u8" | "u16" | "u32" | "u64" | "usize" => quote! { crate::expression::type_::TypeExpr::Integer },
                    "f32" | "f64" => quote! { crate::expression::type_::TypeExpr::Number },
                    "Str" => quote! { crate::expression::type_::TypeExpr::String },
                    "Vec" => {
                        if let PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                                let inner_expr = rust_type_to_type_expr(inner_type);
                                return quote! {
                                    crate::expression::type_::TypeExpr::List(
                                        crate::expression::type_::TypeExprBox(Box::new(#inner_expr))
                                    )
                                };
                            }
                        }
                        quote! { crate::expression::type_::TypeExpr::List(
                            crate::expression::type_::TypeExprBox(Box::new(crate::expression::type_::TypeExpr::Any))
                        ) }
                    }
                    "Option" => {
                        if let PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                                let inner_expr = rust_type_to_type_expr(inner_type);
                                return quote! {
                                    crate::expression::type_::TypeExpr::Optional(
                                        crate::expression::type_::TypeExprBox(Box::new(#inner_expr))
                                    )
                                };
                            }
                        }
                        quote! { crate::expression::type_::TypeExpr::Optional(
                            crate::expression::type_::TypeExprBox(Box::new(crate::expression::type_::TypeExpr::Any))
                        ) }
                    }
                    "BTreeMap" | "HashMap" => {
                        if let PathArguments::AngleBracketed(args) = &segment.arguments {
                            if args.args.len() >= 2 {
                                if let (Some(GenericArgument::Type(key_type)), Some(GenericArgument::Type(value_type))) = 
                                    (args.args.first(), args.args.get(1)) {
                                    let key_expr = rust_type_to_type_expr(key_type);
                                    let value_expr = rust_type_to_type_expr(value_type);
                                    return quote! {
                                        crate::expression::type_::TypeExpr::Map(
                                            crate::expression::type_::TypeExprBox(Box::new(#key_expr)),
                                            crate::expression::type_::TypeExprBox(Box::new(#value_expr))
                                        )
                                    };
                                }
                            }
                        }
                        quote! { crate::expression::type_::TypeExpr::Map(
                            crate::expression::type_::TypeExprBox(Box::new(crate::expression::type_::TypeExpr::String)),
                            crate::expression::type_::TypeExprBox(Box::new(crate::expression::type_::TypeExpr::Any))
                        ) }
                    }
                    _ => {
                        // For custom types, try to call their type_expr() method if it exists
                        let type_name = &segment.ident;
                        quote! { 
                            // Try to call type_expr() on the type, fall back to Any
                            <#type_name>::type_expr().unwrap_or(crate::expression::type_::TypeExpr::Any)
                        }
                    }
                }
            } else {
                quote! { crate::expression::type_::TypeExpr::Any }
            }
        }
        _ => quote! { crate::expression::type_::TypeExpr::Any },
    }
} 