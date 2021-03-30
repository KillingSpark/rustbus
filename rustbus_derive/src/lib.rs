use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[proc_macro_derive(Marshal)]
pub fn derive_marshal(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    match ast.data {
        syn::Data::Struct(data) => {
            make_struct_marshal_impl(&ast.ident, &ast.generics, &data.fields).into()
        }
        _ => unimplemented!("Nothing but structs can be derived on right now"),
    }
}
#[proc_macro_derive(Unmarshal)]
pub fn derive_unmarshal(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    match ast.data {
        syn::Data::Struct(data) => {
            make_struct_unmarshal_impl(&ast.ident, &ast.generics, &data.fields).into()
        }
        _ => unimplemented!("Nothing but structs can be derived on right now"),
    }
}
#[proc_macro_derive(Signature)]
pub fn derive_signature(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    match ast.data {
        syn::Data::Struct(data) => {
            make_struct_signature_impl(&ast.ident, &ast.generics, &data.fields).into()
        }
        _ => unimplemented!("Nothing but structs can be derived on right now"),
    }
}

fn make_struct_marshal_impl(
    ident: &syn::Ident,
    generics: &syn::Generics,
    fields: &syn::Fields,
) -> TokenStream {
    let (impl_gen, typ_gen, clause_gen) = generics.split_for_impl();
    let marshal = struct_field_marshal(fields);

    quote! {
        impl #impl_gen ::rustbus::Marshal for #ident #typ_gen #clause_gen {
            #[inline]
            fn marshal(&self, ctx: &mut ::rustbus::wire::marshal::MarshalContext<'_,'_>) -> Result<(), ::rustbus::Error> {
                #marshal
            }
        }
    }
}
fn make_struct_unmarshal_impl(
    ident: &syn::Ident,
    generics: &syn::Generics,
    fields: &syn::Fields,
) -> TokenStream {
    let marshal = struct_field_unmarshal(fields);

    let mut bufdef = syn::LifetimeDef {
        attrs: Vec::new(),
        lifetime: syn::Lifetime::new("'__internal_buf", proc_macro2::Span::call_site()),
        colon_token: None,
        bounds: syn::punctuated::Punctuated::new(),
    };

    let mut new_generics = generics.clone();
    for lt in new_generics.lifetimes_mut() {
        bufdef.bounds.push(lt.lifetime.clone());
        lt.bounds.push(bufdef.lifetime.clone());
    }

    let typ_generics = new_generics.clone();
    let (_, typ_gen, _) = typ_generics.split_for_impl();

    new_generics
        .params
        .insert(0, syn::GenericParam::Lifetime(bufdef));

    let (impl_gen, _, clause_gen) = new_generics.split_for_impl();

    quote! {
        impl #impl_gen ::rustbus::Unmarshal<'__internal_buf, '_> for #ident #typ_gen #clause_gen {
            #[inline]
            fn unmarshal(ctx: &mut ::rustbus::wire::unmarshal::UnmarshalContext<'_,'__internal_buf>) -> Result<(usize,Self), ::rustbus::wire::unmarshal::Error> {
                #marshal
            }
        }
    }
}
fn make_struct_signature_impl(
    ident: &syn::Ident,
    generics: &syn::Generics,
    fields: &syn::Fields,
) -> TokenStream {
    let (impl_gen, typ_gen, clause_gen) = generics.split_for_impl();
    let signature = struct_field_sigs(fields);

    quote! {
        impl #impl_gen ::rustbus::Signature for #ident #typ_gen #clause_gen {
            #[inline]
            fn signature() -> ::rustbus::signature::Type {
                #signature
            }
            fn alignment() -> usize {
                8
            }
        }
    }
}

fn struct_field_marshal(fields: &syn::Fields) -> TokenStream {
    let field_names = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_token_stream());

    quote! {
            ctx.align_to(8);
            #(
                self.#field_names.marshal(ctx)?;
            )*
            Ok(())
    }
}
fn struct_field_unmarshal(fields: &syn::Fields) -> TokenStream {
    let field_names = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_token_stream());

    let field_types = fields.iter().map(|field| field.ty.to_token_stream());

    quote! {
            let start_offset = ctx.offset;
            ctx.align_to(8)?;

            let this = Self{
                #(
                    #field_names: <#field_types as ::rustbus::Unmarshal>::unmarshal(ctx)?.1,
                )*
            };
            let total_bytes = ctx.offset - start_offset;
            Ok((total_bytes, this))
    }
}
fn struct_field_sigs(fields: &syn::Fields) -> TokenStream {
    let field_types = fields
        .iter()
        .map(|field| field.ty.to_token_stream())
        .collect::<Vec<_>>();
    if field_types.is_empty() {
        panic!("Signature can not be derived for empty structs!")
    }

    quote! {
            let mut sigs = vec![];

            #(
                sigs.push(<#field_types as rustbus::Signature>::signature());
            )*

            ::rustbus::signature::Type::Container(::rustbus::signature::Container::Struct(
                ::rustbus::signature::StructTypes::new(sigs).unwrap()
            ))
    }
}
