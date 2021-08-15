mod variants;
mod structs;

#[proc_macro_derive(Marshal)]
pub fn derive_marshal(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    match ast.data {
        syn::Data::Struct(data) => {
            structs::make_struct_marshal_impl(&ast.ident, &ast.generics, &data.fields).into()
        }
        syn::Data::Enum(data) => {
            variants::make_variant_marshal_impl(&ast.ident, &ast.generics, &data.variants).into()
        }
        _ => unimplemented!("Nothing but structs can be derived on right now"),
    }
}
#[proc_macro_derive(Unmarshal)]
pub fn derive_unmarshal(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    match ast.data {
        syn::Data::Struct(data) => {
            structs::make_struct_unmarshal_impl(&ast.ident, &ast.generics, &data.fields).into()
        }
        _ => unimplemented!("Nothing but structs can be derived on right now"),
    }
}
#[proc_macro_derive(Signature)]
pub fn derive_signature(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    match ast.data {
        syn::Data::Struct(data) => {
            structs::make_struct_signature_impl(&ast.ident, &ast.generics, &data.fields).into()
        }
        syn::Data::Enum(_data) => {
            variants::make_variant_signature_imp(&ast.ident, &ast.generics).into()
        }
        _ => unimplemented!("Nothing but structs can be derived on right now"),
    }
}
