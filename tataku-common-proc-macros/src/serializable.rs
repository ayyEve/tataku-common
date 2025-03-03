use syn::*;
use quote::*;
use proc_macro::TokenStream;

const SERIALIZE_ATTRIBUTE: &str = "serialize";
const READ_VERSION_ATTRIBUTE: &str = "read_version";
const VERSION_ATTRIBUTE: &str = "version";
const VERSION_FIELD: &str = "version";


pub fn derive(ast: syn::DeriveInput) -> TokenStream {
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
            if !attr.path().is_ident(SERIALIZE_ATTRIBUTE) { continue }

            if let Err(e) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(READ_VERSION_ATTRIBUTE) {
                    let _ = meta.value()?;
                    let value: LitBool = meta.input.parse()?;
                    read_version = value.value;
                } else {
                    return Err(meta.error("invalid attribute"));
                }
 
                Ok(())
            }) {
                return e.into_compile_error().into()
            }
        }

        // check to see if we have a version field
        if let Some(f) = data.fields.iter().next() {
            if *f.ident.as_ref().unwrap() == VERSION_FIELD {
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
            let mut version:u16 = 0;

            // check for version tag
            for a in field.attrs.iter() {
                if !a.path().is_ident(SERIALIZE_ATTRIBUTE) { continue }

                if let Err(e) = a.parse_nested_meta(|meta| {
                    if meta.path.is_ident(VERSION_ATTRIBUTE) {
                        let _ = meta.value()?;
                        let value: LitInt = meta.input.parse()?;
                        version = value.base10_parse()?;
                    } else {
                        return Err(meta.error("invalid attribute"));
                    }

                    Ok(())
                }) {
                    return e.into_compile_error().into();
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