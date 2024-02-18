use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn make_struct_marshal_impl(
    ident: &syn::Ident,
    generics: &syn::Generics,
    fields: &syn::Fields,
) -> TokenStream {
    let (impl_gen, typ_gen, clause_gen) = generics.split_for_impl();
    let marshal = struct_field_marshal(fields);

    quote! {
        impl #impl_gen ::rustbus::Marshal for #ident #typ_gen #clause_gen {
            #[inline]
            fn marshal(&self, ctx: &mut ::rustbus::wire::marshal::MarshalContext<'_,'_>) -> Result<(), ::rustbus::wire::errors::MarshalError> {
                #marshal
            }
        }
    }
}
pub fn make_struct_unmarshal_impl(
    ident: &syn::Ident,
    generics: &syn::Generics,
    fields: &syn::Fields,
) -> TokenStream {
    let marshal = struct_field_unmarshal(fields);

    let mut bufdef = syn::LifetimeParam {
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
            fn unmarshal(ctx: &mut ::rustbus::wire::unmarshal::UnmarshalContext<'_,'__internal_buf>) -> Result<(usize,Self), ::rustbus::wire::errors::UnmarshalError> {
                #marshal
            }
        }
    }
}
pub fn make_struct_signature_impl(
    ident: &syn::Ident,
    generics: &syn::Generics,
    fields: &syn::Fields,
) -> TokenStream {
    let (impl_gen, typ_gen, clause_gen) = generics.split_for_impl();
    let signature = struct_field_sigs(fields);
    let has_sig = struct_field_has_sigs(fields);

    quote! {
        impl #impl_gen ::rustbus::Signature for #ident #typ_gen #clause_gen {
            #[inline]
            fn signature() -> ::rustbus::signature::Type {
                #signature
            }
            fn alignment() -> usize {
                8
            }
            fn has_sig(sig: &str) -> bool {
                #has_sig
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
fn struct_field_has_sigs(fields: &syn::Fields) -> TokenStream {
    let field_types = fields
        .iter()
        .map(|field| field.ty.to_token_stream())
        .collect::<Vec<_>>();
    if field_types.is_empty() {
        panic!("Signature can not be derived for empty structs!")
    }

    quote! {
        if sig.starts_with('(') {
            let mut iter = ::rustbus::signature::SignatureIter::new(&sig[1..sig.len() - 1]);
            let mut accu = true;

            #(
                accu &= <#field_types as rustbus::Signature>::has_sig(iter.next().unwrap());
            )*

            accu
        } else {
            false
        }
    }
}
