mod reflect;
mod packets;
mod from_string;
mod serializable;
use proc_macro::TokenStream;

// automatic read/write macro for the packet list
#[proc_macro_derive(PacketSerialization, attributes(packet, packet_type))]
pub fn packet_serialization(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse::<syn::DeriveInput>(input).unwrap();

    match packets::derive(&ast) {
        Ok(a) => a,
        Err(e) => e.into_compile_error()
    }.into()
}

#[proc_macro_derive(Serializable, attributes(serialize))]
pub fn serializable(input: TokenStream) -> TokenStream {
    let ast = syn::parse::<syn::DeriveInput>(input).unwrap();
    serializable::derive(ast)
}

#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let tokens = reflect::derive(&ast);
    // std::fs::write(format!("/tmp/debug/{}.rs", ast.ident), tokens.to_string()).unwrap();
    tokens.into()
}

#[proc_macro_derive(FromStr, attributes(from_str))]
pub fn derive_from_string(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    match from_string::derive(&ast) {
        Ok(a) => a,
        Err(e) => e.into_compile_error()
    }.into()
}
