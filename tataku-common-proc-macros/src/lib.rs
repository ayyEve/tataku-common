use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::*;
use syn::*;

// automatic read/write macro for the packet list
#[proc_macro_derive(PacketSerialization, attributes(Packet))]
pub fn packet_serialization(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build the impl
    let gen = impl_packet(&ast);
    
    // Return the generated impl
    proc_macro::TokenStream::from(gen)
}
fn impl_packet(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    struct EnumData {
        id: u16,
        name: String,
        fields: Vec<String>,
        // version: u64
    }
    let mut extra_logging = false;
    let mut type_ = "u8".to_owned();
    let mut should_impl_into_from_type = false;
    // let mut rolling_id = 0;

    // find the type of the packet, and if it should gen 
    for a in ast.attrs.iter() {
        if !a.path.is_ident("Packet") { continue }
        if let Ok(Meta::List(list)) = a.parse_meta() {
            for i in list.nested {
                if let NestedMeta::Meta(Meta::NameValue(name_value)) = &i {
                    if name_value.path.is_ident("type") {
                        if let Lit::Str(i) = &name_value.lit {
                            type_ = i.value()
                        }
                    }
                }

                if let NestedMeta::Meta(Meta::Path(name)) = &i {
                    if name.is_ident("gen_to_from") {
                        should_impl_into_from_type = true;
                    }
                    if name.is_ident("extra_logging") {
                        extra_logging = true;
                    }
                }
            }
        }
    }
    if extra_logging { println!("got type {type_:?} for enum {}", ast.ident.to_string()) }

    let mut id_map: HashMap<u16, String> = HashMap::new();
    let mut variant_list:Vec<EnumData> = Vec::new();
    let mut default_variant = "Unknown".to_owned();

    if let Data::Enum(data) = &ast.data {
        for v in data.variants.iter() {
            let variant_name = &v.ident;
            let mut id:Option<u16> = None;

            // let mut version = 0;

            // find the id of the packet
            for a in v.attrs.iter() {
                if !a.path.is_ident("Packet") { continue }
                if let Ok(Meta::List(list)) = a.parse_meta() {
                    for i in list.nested {
                        if let NestedMeta::Meta(Meta::NameValue(name_value)) = &i {
                            if name_value.path.is_ident("id") {
                                if let Lit::Int(i) = &name_value.lit {
                                    id = Some(i.base10_parse::<u16>().unwrap())
                                }
                            }
                            // if name_value.path.is_ident("version") {
                            //     if let Lit::Int(i) = &name_value.lit {
                            //         version = i.base10_parse::<u64>().unwrap()
                            //     }
                            // }
                        }
                        if let NestedMeta::Meta(Meta::Path(name)) = &i {
                            if name.is_ident("default_variant") {
                                default_variant = variant_name.to_string();
                            }
                        }
                    }
                }
            }

            // ensure this packet has an id
            if id.is_none() {
                panic!("Packet has no id!! {}", variant_name.to_string())
            }
            let id = id.unwrap();
            // ensure the id is free
            if let Some(packet_name) = id_map.insert(id, variant_name.to_string()) {
                panic!("Id {} already used by packet {}", id, packet_name);
            }

            // create packet data
            variant_list.push(EnumData {
                name: variant_name.to_string(),
                id,
                // version,
                fields: v.fields.iter().map(|f|f.ident.as_ref().unwrap().to_string()).collect()
            })
        }
    }

    // make sure we're adding things to the list
    if variant_list.is_empty() {
        panic!("packet list is empty?")
    }

    let enum_name = ast.ident.to_string();

    // create the match strs
    let mut read_match = "    Ok(match packet_id {\n".to_owned();
    let mut write_match = "    match self {\n".to_owned();

    for EnumData { id, name, fields } in variant_list.iter() {
        // read match
        {
            /* ie:
                1 => PacketId::UserJoined {
                    user_id: sr.read(),
                    username: sr.read(),
                },
            */

            // if theres fields, we need to read them
            let read_fields = (!fields.is_empty()).then(|| format!("{{\n{}\n}}", fields.iter().map(|f| format!("{f}: sr.read(\"{f}\")?,")).collect::<Vec<_>>().join("\n")) ).unwrap_or_default();
            
            read_match += &format!("{id} => Self::{name} {read_fields},\n");
        }

        // write match
        {
            /* ie:
                PacketId::UserJoined {user_id, username} =>  {
                    sw.write(&id);
                    sw.write(&user_id);
                    sw.write(&username);
                },
            */

            let match_fields = if fields.len() > 0 { format!("{{{}}} ", fields.join(", ")) } else { String::new() };
            let write_fields = fields.iter().map(|f| format!("sw.write({f});")).collect::<Vec<_>>().join("\n");

            write_match += &format!("Self::{name} {match_fields} => {{
                sw.write::<{type_}>(&{id});
                {write_fields}
                }},
            ");
        }
    }
    
    read_match += &format!("        _ => Self::{default_variant}\n    }})");
    write_match += "    }";

    #[allow(unused_mut)]
    let mut debug_read_line = format!("");
    #[cfg(feature = "packet_logging")] {
        debug_read_line = format!("println!(\"[Packet] Reading packet {{packet_id:?}} from enum {enum_name}\");");
    }
    
    let mut impl_str = format!("
        impl Serializable for {enum_name} {{
            fn read(sr: &mut crate::serialization::SerializationReader) -> SerializationResult<Self> {{
                sr.push_parent(\"{enum_name}\");
                let packet_id = sr.read::<{type_}>(\"packet_id\")?;
                {debug_read_line}
                let a = {read_match};
                
                sr.pop_parent();
                a
            }}
            fn write(&self, sw: &mut crate::serialization::SerializationWriter) {{
                {write_match}
            }}
        }}
    ");

    if should_impl_into_from_type {
        let list1 = variant_list
            .iter()
            .map(|ed| format!("{} => {enum_name}::{},", ed.id, ed.name))
            .collect::<Vec<String>>()
            .join("\n");

        let list2 = variant_list
            .iter()
            .map(|ed| format!("{enum_name}::{} => {},", ed.name, ed.id))
            .collect::<Vec<String>>()
            .join("\n");

        impl_str += &format!(r#" 
        impl Into<{enum_name}> for {type_} {{
            fn into(self) -> {enum_name} {{
                match self {{
                    {list1}
                    _ => {enum_name}::{default_variant},
                }}
            }}
        }}
        impl Into<{type_}> for {enum_name} {{
            fn into(self) -> {type_} {{
                match self {{
                    {list2}
                }}
            }}
        }}
        "#);
    }

    #[cfg(feature="serialization_logging")] {
        println!("generated: {}", impl_str);
        std::fs::create_dir_all("debug/").unwrap();
        std::fs::write(format!("debug/{enum_name}.rs"), &impl_str).unwrap();
    }


    let impl_tokens = impl_str.parse::<proc_macro2::TokenStream>().unwrap();
    quote! {
        #impl_tokens
    }
}


#[proc_macro_derive(Serializable, attributes(Serialize))]
pub fn serializable(input: TokenStream)  -> TokenStream {
    let ast = syn::parse(input).unwrap();

    // Build the impl
    let gen = impl_serializable(&ast);
    
    // Return the generated impl
    proc_macro::TokenStream::from(gen)
}
fn impl_serializable(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let mut read_lines = Vec::new();
    let mut write_lines = Vec::new();

    let struct_name = ast.ident.to_string();
    let mut read_version = false;

    read_lines.push("let mut s = Self::default();".to_owned());

    if let Data::Struct(data) = &ast.data {

        // check if this struct has a version attached
        for attr in ast.attrs.iter() {
            if !attr.path.is_ident("Serialize") { continue }
            if let Ok(Meta::List(list)) = attr.parse_meta() {
                for i in list.nested {
                    if let NestedMeta::Meta(Meta::NameValue(name_value)) = &i {
                        if name_value.path.is_ident("read_version") {
                            if let Lit::Bool(i) = &name_value.lit {
                                read_version = i.value;
                            }
                        }
                    }
                }
            }
        }

        // check to see if we have a version field
        if let Some(f) = data.fields.iter().next() {
            if f.ident.as_ref().unwrap().to_string() == "version" {
                read_lines.push(format!("s.version = sr.read(\"version\")?;"));
                read_lines.push(format!("let version = s.version;"));
            } else {
                if read_version {
                    read_lines.push(format!("let version = sr.read::<u16>(\"version\")?;"));
                } else {
                    read_lines.push("let version = 0u16; //version.unwrap_or_default();".to_owned());
                }
            }
        }


        for field in data.fields.iter() {
            let name = field.ident.as_ref().unwrap().to_string();
            let mut version = 0;

            // check for version tag
            for a in field.attrs.iter() {
                if !a.path.is_ident("Serialize") { continue }
                if let Ok(Meta::List(list)) = a.parse_meta() {
                    for i in list.nested {
                        if let NestedMeta::Meta(Meta::NameValue(name_value)) = &i {
                            if name_value.path.is_ident("version") {
                                if let Lit::Int(i) = &name_value.lit {
                                    version = i.base10_parse::<u64>().unwrap()
                                }
                            }
                        }
                    }
                }
            }

            if version > 0 {
                read_lines.push(format!("if version > {version} {{s.{name} = sr.read(\"{name}\")?;}}"));
            } else {
                read_lines.push(format!("s.{name} = sr.read(\"{name}\")?;"));
            }

            write_lines.push(format!("sw.write(&self.{name});"));
        }
    }

    let read_lines = read_lines.join("\n");
    let write_lines = write_lines.join("\n");

    let impl_str = format!("
        impl Serializable for {struct_name} {{
            fn read(sr: &mut SerializationReader) -> SerializationResult<Self> where Self: Sized {{
                sr.push_parent(\"{struct_name}\");

                {read_lines}

                sr.pop_parent();
                Ok(s)
            }}
            fn write(&self, sw: &mut SerializationWriter) {{
                {write_lines}
            }}
        }}"
    );

    #[cfg(feature="serialization_logging")] {
        std::fs::create_dir_all("debug").unwrap();
        std::fs::write(format!("debug/{struct_name}.rs"), &impl_str).unwrap();
        println!("generated: {}", impl_str)
    }

    let impl_tokens = impl_str.parse::<proc_macro2::TokenStream>().unwrap();
    quote! {
        #impl_tokens
    }
}
