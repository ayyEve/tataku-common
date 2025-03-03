
use syn::*;
use quote::*;
use syn::spanned::Spanned;
use proc_macro::TokenStream;
use std::collections::HashMap;

const PACKET_ATTRIBUTE: &str = "packet";

const TYPE_ATTRIBUTE: &str = "type";
const EXTRA_LOGGING_ATTRIBUTE: &str = "extra_logging";
const GEN_TO_FROM_ATTRIBUTE: &str = "gen_to_from";

const ID_ATTRIBUTE: &str = "id";
const DEFAULT_VARIANT_ATTRIBUTE: &str = "default_variant";

// automatic read/write macro for the packet list
pub fn derive(ast: syn::DeriveInput) -> TokenStream {
    let mut extra_logging = false;
    let mut type_ = format_ident!("u8");
    let mut should_impl_into_from_type = false;
    // let mut rolling_id = 0;

    // find the type of the packet, and if it should gen
    for a in ast.attrs.iter() {
        if !a.path().is_ident(PACKET_ATTRIBUTE) { continue }

        if let Err(e) = a.parse_nested_meta(|meta| {
            if meta.path.is_ident(TYPE_ATTRIBUTE) {
                let _ = meta.value()?;
                let name: LitStr = meta.input.parse()?;
                type_ = format_ident!("{}", name.value());
            } else if meta.path.is_ident(GEN_TO_FROM_ATTRIBUTE) {
                should_impl_into_from_type = true;
            } else if meta.path.is_ident(EXTRA_LOGGING_ATTRIBUTE) {
                extra_logging = true;
            } else {
                return Err(meta.error("invalid attribute"))
            }

            Ok(())
        }) {
            return e.into_compile_error().into()
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

                if let Err(e) = a.parse_nested_meta(|meta| {
                    if meta.path.is_ident(ID_ATTRIBUTE) {
                        let _ = meta.value()?;
                        let id_in: LitInt = meta.input.parse()?;
                        let value = id_in.base10_parse::<u16>()?;

                        id = Some(value);
                    } else if meta.path.is_ident(DEFAULT_VARIANT_ATTRIBUTE) {
                        default_variant = DefaultVariant::Variant(variant_name);
                    } else {
                        return Err(meta.error("invalid attribute"))
                    }

                    Ok(())
                }) {
                    return e.into_compile_error().into()
                }
            }

            // ensure this packet has an id
            let Some(id) = id else {
                return Error::new(
                    v.span(), 
                    format!("Variant has no id!! {variant_name}")
                ).into_compile_error().into();
            };

            if extra_logging { println!("got id {id} for variant {variant_name}") }

            
            // ensure the id isnt already used
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
                        sw.write(&(#id as #type_));
                        #( sw.write(#fields); )*
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
