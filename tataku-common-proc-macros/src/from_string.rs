use syn::*;
use quote::*;
use proc_macro2::TokenStream;

const FROM_STRING_ATTRIBUTE: &str = "from_str";

const SKIP_ATTRIBUTE: &str = "skip";
const DEFAULT_ATTRIBUTE: &str = "default";

// automatic read/write macro for the packet list
pub fn derive(ast: &syn::DeriveInput) -> Result<TokenStream> {
    let struct_attrs = Attributes::parse(&ast.attrs)?;

    let ident = &ast.ident;
    let mut default = struct_attrs.default;
    let mut match_tokens = TokenStream::new();

    match &ast.data {
        Data::Struct(_) => panic!("structs arent supported"),
        Data::Union(_) => panic!("unions arent supported"),

        Data::Enum(data) => {
            for v in data.variants.iter() {
                let attrs = Attributes::parse(&ast.attrs)?;
                let variant_name = &v.ident;
                
                if attrs.skip { continue }
                if attrs.is_default {
                    default = Some(quote! { Self::#variant_name });
                }

                let name_str = variant_name.to_string();
                let name_lower = to_snake(&name_str);

                match_tokens.extend(quote! {
                    #name_str => Ok(Self::#variant_name),
                    #name_lower => Ok(Self::#variant_name),
                });
            }
        }
    }

    let default = if let Some(default) = default {
        quote! { Ok(#default) }
    } else {
        quote! { Err(()) }
    };
    
    let tokens = quote! {
        impl std::str::FromStr for #ident {
            type Err = ();

            fn from_str(str: &str) -> std::result::Result<Self, Self::Err> {
                match str {
                    #match_tokens

                    _ => #default
                }
            }
        }
    };

    // std::fs::create_dir_all("/tmp/debug").unwrap();
    // std::fs::write(format!("/tmp/debug/{enum_name}.rs"), tokens.to_string()).unwrap();

    Ok(tokens)
}


fn to_snake(s: &str) -> String {
    let mut snake = String::with_capacity(s.len());
    for i in s.chars() {
        if i.is_uppercase() {
            snake.push('_');
            snake.push(i.to_ascii_lowercase());
        } else {
            snake.push(i)
        }
    }

    snake
}

#[derive(Default)]
struct Attributes {
    skip: bool,
    is_default: bool,

    // 
    default: Option<TokenStream>,
}
impl Attributes {
    fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut this = Self::default();

        for a in attrs {
            if a.path().is_ident(DEFAULT_ATTRIBUTE) { this.is_default = true; }

            if !a.path().is_ident(FROM_STRING_ATTRIBUTE) { continue }

            a.parse_nested_meta(|meta| {
                if meta.path.is_ident(DEFAULT_ATTRIBUTE) {
                    if meta.value().is_err() {
                        this.is_default = true;
                    } else {
                        this.default = Some(meta.input.parse()?);
                    }
                } else if meta.path.is_ident(SKIP_ATTRIBUTE) {
                    this.skip = true;
                } else {
                    return Err(meta.error("invalid attribute"))
                }

                Ok(())
            })?;
        }

        Ok(this)
    }
}
