
use syn::*;
use quote::*;
use syn::spanned::Spanned;
use proc_macro2::TokenStream;
use std::collections::HashMap;

const PACKET_ATTRIBUTE: &str = "packet";

const TYPE_ATTRIBUTE: &str = "packet_type";
const REPR_ATTRIBUTE: &str = "repr";
const EXTRA_LOGGING_ATTRIBUTE: &str = "extra_logging";
const GEN_TO_FROM_ATTRIBUTE: &str = "gen_to_from";

const ID_ATTRIBUTE: &str = "id";
const DEFAULT_VARIANT_ATTRIBUTE: &str = "default";

// automatic read/write macro for the packet list
pub fn derive(ast: &syn::DeriveInput) -> Result<TokenStream> {
    // let mut type_ = format_ident!("u8");
    // let mut rolling_id = 0;

    // find the type of the packet, and if it should gen
    let packet_attrs = PacketAttrs::parse(&ast.attrs)?;
    if packet_attrs.extra_logging { 
        println!("got type {:?} for enum {}", ast.ident, packet_attrs.type_) 
    }

    let mut id_map: HashMap<u16, &Ident> = HashMap::new();
    let mut default_variant = DefaultVariant::Default;

    let enum_name = &ast.ident;
    let type_ = packet_attrs.type_;

    let mut variants = Vec::new();
    let mut ids = Vec::new();

    let mut read_fields = Vec::new();
    let mut write_fields = Vec::new();

    if let Data::Enum(data) = &ast.data {
        for v in data.variants.iter() {
            let variant_name = &v.ident;

            // find the id of the packet
            let variant_attrs = PacketAttrs::parse(&v.attrs)?;
            if variant_attrs.is_default {
                default_variant = DefaultVariant::Variant(variant_name);
            }

            // ensure this packet has an id
            let Some(id) = variant_attrs.id else {
                return Err(Error::new(
                    v.span(), 
                    format!("Variant has no id!! {variant_name}")
                ));
            };

            if packet_attrs.extra_logging { println!("got id {id} for variant {variant_name}") }

            // ensure the id isnt already used
            if let Some(variant) = id_map.insert(id, variant_name) {
                return Err(Error::new(
                    v.span(), 
                    format!("Id {id} already used by variant {variant}")
                ));
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
        return Err(Error::new(ast.span(), "Packet list is empty?"));
    }

    #[allow(unused_mut)] // must be mut when packet_logging is enabled
    let mut debug_read_line = proc_macro2::TokenStream::new();
    #[cfg(feature = "packet_logging")] {
        debug_read_line = quote! {
            println!("[Packet] Reading packet {:?} from enum {}", packet_id, stringify!(#enum_name));
        }
    }

    let name = enum_name.to_string();
    let mut tokens = quote! {
        impl Serializable for #enum_name {
            fn read(sr: &mut crate::serialization::SerializationReader) -> SerializationResult<Self> {
                sr.push_parent(#name);
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

    if packet_attrs.should_impl_into_from_type {
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

    std::fs::create_dir_all("/tmp/debug").unwrap();
    std::fs::write(format!("/tmp/debug/{enum_name}.rs"), tokens.to_string()).unwrap();

    Ok(tokens)
}



#[derive(Default)]
enum DefaultVariant<'a> {
    #[default] Default,
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


#[derive(Default)]
struct PacketAttrs {
    // container attrs
    extra_logging: bool,
    should_impl_into_from_type: bool,
    type_: TokenStream,

    // variant attrs
    id: Option<u16>,
    is_default: bool,
}
impl PacketAttrs {
    fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut this = Self {
            type_: quote! { u8 },
            ..Self::default()
        };

        for a in attrs {
            let path = a.path();
            if path.is_ident(REPR_ATTRIBUTE) || path.is_ident(TYPE_ATTRIBUTE) { 
                this.type_ = a.parse_args::<TokenStream>()?;
            }
            
            if !a.path().is_ident(PACKET_ATTRIBUTE) { continue }

            a.parse_nested_meta(|meta| {
                if meta.path.is_ident(GEN_TO_FROM_ATTRIBUTE) {
                    this.should_impl_into_from_type = true;
                } else if meta.path.is_ident(EXTRA_LOGGING_ATTRIBUTE) {
                    this.extra_logging = true;
                } 
                
                
                else if meta.path.is_ident(ID_ATTRIBUTE) {
                    let _ = meta.value()?;
                    let id_in: LitInt = meta.input.parse()?;
                    let value = id_in.base10_parse::<u16>()?;

                    this.id = Some(value);
                } 
                else if meta.path.is_ident(DEFAULT_VARIANT_ATTRIBUTE) {
                    this.is_default = true;
                } 
                
                else {
                    return Err(meta.error("invalid attribute"))
                }

                Ok(())
            })?;
        }

        Ok(this)
    }
}
