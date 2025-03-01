mod reflect;

use syn::*;
use quote::*;
use syn::spanned::Spanned;
use proc_macro::TokenStream;
use std::collections::HashMap;
use syn::punctuated::Punctuated;


const PACKET_ATTRIBUTE: &str = "packet";

// automatic read/write macro for the packet list
#[proc_macro_derive(PacketSerialization, attributes(packet))]
pub fn packet_serialization(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse::<syn::DeriveInput>(input).unwrap();

    let mut extra_logging = false;
    let mut type_ = format_ident!("u8");
    let mut should_impl_into_from_type = false;
    // let mut rolling_id = 0;

    // find the type of the packet, and if it should gen
    for a in ast.attrs.iter() {
        if !a.path().is_ident(PACKET_ATTRIBUTE) { continue }

        if let Ok(metas) = a.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
            assert!(!metas.is_empty(), "Packet attribute cannot be empty");

            for i in metas {
                match i {
                    Meta::NameValue(name_value) => {
                        if name_value.path.is_ident("type") {
                            if let Expr::Lit(ExprLit { lit: Lit::Str(i), .. }) = name_value.value {
                                type_ = format_ident!("{}", i.value())
                            }
                        }
                    }

                    Meta::Path(name) => {
                        if name.is_ident("gen_to_from") {
                            should_impl_into_from_type = true;
                        }
                        if name.is_ident("extra_logging") {
                            extra_logging = true;
                        }
                    }

                    _ => unimplemented!("nop")
                }
            }
        }
    }
    if extra_logging { println!("got type {type_:?} for enum {}", ast.ident) }

    let mut id_map: HashMap<u16, &Ident> = HashMap::new();
    let mut default_variant = DefaultVariant::Default; //format_ident!("default()");

    let enum_name = &ast.ident;

    let mut variants = Vec::new();
    let mut ids = Vec::new();

    let mut read_fields = Vec::new();
    let mut write_fields = Vec::new();

    if let Data::Enum(data) = &ast.data {
        for v in data.variants.iter() {
            let variant_name = &v.ident;
            let mut id: Option<u16> = None;

            // let mut version = 0;

            // find the id of the packet
            for a in v.attrs.iter() {
                if !a.path().is_ident(PACKET_ATTRIBUTE) { continue }
                if let Ok(metas) = a.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
                    for i in metas {
                        match i {
                            Meta::NameValue(name_value) => {
                                if name_value.path.is_ident("id") {
                                    if let Expr::Lit(ExprLit { lit: Lit::Int(i), .. }) = name_value.value {
                                        id = Some(i.base10_parse::<u16>().unwrap())
                                    }
                                }
                                // if name_value.path.is_ident("version") {
                                //     if let Lit::Int(i) = &name_value.lit {
                                //         version = i.base10_parse::<u64>().unwrap()
                                //     }
                                // }
                            }

                            Meta::Path(name) =>  {
                                if name.is_ident("default_variant") {
                                    default_variant = DefaultVariant::Variant(variant_name);
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }

            // ensure this packet has an id
            let Some(id) = id else {
                return Error::new(
                    v.span(), 
                    format!("Variant has no id!! {variant_name}")
                ).into_compile_error().into();
            };
            
            // ensure the id is free
            if let Some(variant) = id_map.insert(id, variant_name) {
                return Error::new(
                    v.span(), 
                    format!("Id {id} already used by variant {variant}")
                ).into_compile_error().into();
            }

            variants.push(variant_name);
            ids.push(id);

            let fields = v.fields.iter()
                .filter_map(|f| f.ident.as_ref()) // ident should always exist
                .collect::<Vec<_>>();

            if fields.is_empty() {
                read_fields.push(proc_macro2::TokenStream::new());

                write_fields.push(quote! {
                     => sw.write(&(#id as #type_)),
                });
            } else {
                read_fields.push(quote!{ {
                    #( #fields: sr.read(stringify!(#fields))?, )*
                } });

                write_fields.push(quote! {
                    { #(#fields),* } => {
                        #( sw.write(#fields); )*
                        sw.write(&(#id as #type_));
                    }
                });
            }
        }
    }

    // make sure we're adding things to the list
    if variants.is_empty() {
        return Error::new(ast.span(), "Packet list is empty?")
            .into_compile_error()
            .into();
    }

    #[allow(unused_mut)] // must be mut when packet_logging is enabled
    let mut debug_read_line = proc_macro2::TokenStream::new();
    #[cfg(feature = "packet_logging")] {
        debug_read_line = quote! {
            println!("[Packet] Reading packet {:?} from enum {}", packet_id, stringify!(#enum_name));
        }
    }

    let mut tokens = quote! {
        impl Serializable for #enum_name {
            fn read(sr: &mut crate::serialization::SerializationReader) -> SerializationResult<Self> {
                sr.push_parent(stringify!(#enum_name));
                let packet_id = sr.read::<#type_>("packet_id")? as u16;
                #debug_read_line

                let a = match packet_id {
                    #( #ids => Self::#variants #read_fields, )*
                    _ => Self::#default_variant
                };

                sr.pop_parent();
                Ok(a)
            }

            fn write(&self, sw: &mut crate::serialization::SerializationWriter) {
                match self {
                    #( Self::#variants #write_fields )*
                    _ => {}
                }
            }
        }
    };

    if should_impl_into_from_type {
        tokens.extend(quote! {
            impl From<#enum_name> for #type_ {
                fn from(value: #enum_name) -> Self {
                    match value {
                        #(#enum_name::#variants => #ids)*,
                    }
                }
            }

            impl From<#type_> for #enum_name {
                fn from(value: #type_) -> Self {
                    match value {
                        #(#ids => #enum_name::#variants)*,                      

                        _ => #enum_name::#default_variant,
                    }
                }
            }
        });
    }

    // std::fs::create_dir_all("/tmp/debug").unwrap();
    // std::fs::write(format!("/tmp/debug/{enum_name}.rs"), tokens.to_string()).unwrap();

    tokens.into()
}

#[proc_macro_derive(Serializable, attributes(Serialize))]
pub fn serializable(input: TokenStream)  -> TokenStream {
    let ast = syn::parse::<syn::DeriveInput>(input).unwrap();

    let struct_name = &ast.ident;
    let struct_name_str = struct_name.to_string();

    let mut read_version = false;
    let mut read_version_line = proc_macro2::TokenStream::new();

    let mut field_names = Vec::new();
    let mut field_name_strs = Vec::new();
    let mut field_versions = Vec::new();

    if let Data::Struct(data) = &ast.data {

        // check if this struct has a version attached
        for attr in ast.attrs.iter() {
            if !attr.path().is_ident("Serialize") { continue }
            if let Ok(metas) = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
                for i in metas {
                    if let Meta::NameValue(name_value) = i {
                        if name_value.path.is_ident("read_version") {
                            if let Expr::Lit(ExprLit { lit: Lit::Bool(i), .. }) = name_value.value {
                                read_version = i.value;
                            }
                        }
                    }
                }
            }
        }

        // check to see if we have a version field
        if let Some(f) = data.fields.iter().next() {
            if *f.ident.as_ref().unwrap() == "version" {
                read_version_line.extend(quote!{
                    s.version = sr.read("version")?;
                    let version = s.version;
                });
            } else if read_version {
                read_version_line.extend(quote! {
                    let version = sr.read::<u16>("version")?;
                });
            } else {
                read_version_line.extend(quote! {
                    let version = 0u16;
                });
            }
        }


        for field in data.fields.iter() {
            let name = field.ident.as_ref().unwrap();
            let mut version = 0;

            // check for version tag
            for a in field.attrs.iter() {
                if !a.path().is_ident("Serialize") { continue }
                if let Ok(metas) = a.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
                    for i in metas {
                        if let Meta::NameValue(name_value) = i {
                            if name_value.path.is_ident("version") {
                                if let Expr::Lit(ExprLit { lit: Lit::Int(i), .. }) = name_value.value {
                                    version = i.base10_parse::<u16>().unwrap()
                                }
                            }
                        }
                    }
                }
            }

            field_names.push(name);
            field_name_strs.push(name.to_string());
            field_versions.push(version);
        }
    }

    let tokens = quote! {
        impl Serializable for #struct_name {
            fn read(sr: &mut SerializationReader) -> SerializationResult<Self> where Self: Sized {
                sr.push_parent(#struct_name_str);
                let mut s = Self::default();
                #read_version_line

                #(
                    if version >= #field_versions { 
                        s.#field_names = sr.read(#field_name_strs)?; 
                    }
                )*

                sr.pop_parent();
                Ok(s)
            }

            fn write(&self, sw: &mut SerializationWriter) {
                #(
                    sw.write(&self.#field_names);
                )*
            }
        }
    };

    #[cfg(feature="serialization_logging")] {
        std::fs::create_dir_all("debug").unwrap();
        std::fs::write(format!("debug/{struct_name}.rs"), &tokens.to_string()).unwrap();
        // println!("generated: {}", impl_str)
    }

    tokens.into()
}



#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    reflect::derive(&ast).into()
}

#[derive(Default)]
enum DefaultVariant<'a> {
    #[default]
    Default,
    Variant(&'a Ident),
}
impl quote::ToTokens for DefaultVariant<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let t = match self {
            Self::Default => quote! { default() },
            Self::Variant(ident) => quote! { #ident },
        };
        tokens.extend(quote! {
            #t
        });
    }
}
impl From<DefaultVariant<'_>> for proc_macro2::token_stream::TokenStream {
    fn from(value: DefaultVariant<'_>) -> Self {
        match value {
            DefaultVariant::Default => quote! { default() },
            DefaultVariant::Variant(ident) => quote! { #ident },
        }
    }
}
