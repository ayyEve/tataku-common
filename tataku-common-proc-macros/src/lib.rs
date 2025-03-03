mod reflect;
mod packets;
mod serializable;
use proc_macro::TokenStream;

// automatic read/write macro for the packet list
#[proc_macro_derive(PacketSerialization, attributes(packet))]
pub fn packet_serialization(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse::<syn::DeriveInput>(input).unwrap();
    packets::derive(ast)
}

#[proc_macro_derive(Serializable, attributes(Serialize))]
pub fn serializable(input: TokenStream) -> TokenStream {
    let ast = syn::parse::<syn::DeriveInput>(input).unwrap();
    serializable::derive(ast)
}

#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    reflect::derive(&ast).into()
}
