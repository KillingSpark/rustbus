use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, token::Comma, Variant};

pub fn make_variant_signature_imp(ident: &syn::Ident, generics: &syn::Generics) -> TokenStream {
    let (impl_gen, typ_gen, clause_gen) = generics.split_for_impl();

    quote! {
        impl #impl_gen ::rustbus::Signature for #ident #typ_gen #clause_gen {
            #[inline]
            fn signature() -> ::rustbus::signature::Type {
                ::rustbus::signature::Type::Container(::rustbus::signature::Container::Variant)
            }
            fn alignment() -> usize {
                1
            }
            fn has_sig(sig: &str) -> bool {
                sig.starts_with('v')
            }
        }
    }
}

pub fn make_variant_marshal_impl(
    ident: &syn::Ident,
    generics: &syn::Generics,
    variant: &Punctuated<Variant, Comma>,
) -> TokenStream {
    let (impl_gen, typ_gen, clause_gen) = generics.split_for_impl();
    let marshal = variant
        .iter()
        .fold(Default::default(), |mut tokens: TokenStream, variant| {
            tokens.extend(variant_marshal(ident.clone(), variant));
            tokens
        });

    quote! {
        impl #impl_gen ::rustbus::Marshal for #ident #typ_gen #clause_gen {
            #[inline]
            fn marshal(&self, ctx: &mut ::rustbus::wire::marshal::MarshalContext<'_,'_>) -> Result<(), ::rustbus::Error> {
                match self {
                    #marshal
                }
            }
        }
    }
}

fn variant_marshal(enum_name: syn::Ident, variant: &syn::Variant) -> TokenStream {
    let name = variant.ident.clone();
    let field_types = variant
        .fields
        .iter()
        .map(|field| field.ty.to_token_stream());

    if !variant.fields.is_empty() {
        if variant.fields.iter().next().unwrap().ident.is_some() {
            // Named fields
            let field_names1 = variant
                .fields
                .iter()
                .map(|field| field.ident.as_ref().unwrap().to_token_stream());
            let field_names2 = field_names1.clone();

            quote! {
                #enum_name::#name{ #( #field_names1, )* } => {
                    // marshal signature
                    let pos = ctx.buf.len();
                    ctx.buf.push(0);

                    ctx.buf.push(b'(');
                    let mut sig_str = ::rustbus::wire::marshal::traits::SignatureBuffer::new();
                    #(
                        sig_str.clear();
                        <#field_types as ::rustbus::Signature>::sig_str(&mut sig_str);
                        ctx.buf.extend_from_slice(sig_str.as_ref().as_bytes());
                    )*
                    ctx.buf.push(b')');
                    ctx.buf.push(0);


                    // -2 for pos and nullbyte
                    ctx.buf[pos] = (ctx.buf.len() - pos - 2) as u8;

                    // actual marshal code
                    // align to 8 because we treat this as a struct
                    ctx.align_to(8);
                    #(
                        #field_names2.marshal(ctx)?;
                    )*
                    Ok(())
                },
            }
        } else if variant.fields.iter().next().unwrap().ident.is_none() && variant.fields.len() > 1
        {
            // Named fields
            let field_names1 = variant.fields.iter().enumerate().map(|(idx, _field)| {
                syn::Ident::new(&format!("v{}", idx), enum_name.span()).to_token_stream()
            });

            let field_names2 = field_names1.clone();

            quote! {
                #enum_name::#name( #( #field_names1, )* ) => {
                    // marshal signature
                    let pos = ctx.buf.len();
                    ctx.buf.push(0);

                    ctx.buf.push(b'(');
                    let mut sig_str = ::rustbus::wire::marshal::traits::SignatureBuffer::new();
                    #(
                        sig_str.clear();
                        <#field_types as ::rustbus::Signature>::sig_str(&mut sig_str);
                        ctx.buf.extend_from_slice(sig_str.as_ref().as_bytes());
                    )*
                    ctx.buf.push(b')');
                    ctx.buf.push(0);

                    // -2 for pos and nullbyte
                    ctx.buf[pos] = (ctx.buf.len() - pos - 2) as u8;
                    
                    // align to 8 because we treat this as a struct
                    ctx.align_to(8);
                    //actual marshal code
                    #(
                        #field_names2.marshal(ctx)?;
                    )*
                    Ok(())
                },
            }
        } else {
            // One unnamed field
            let mut field_types = field_types;
            let ty = field_types.next().unwrap();
            quote! {
                #enum_name::#name( val ) => {
                    let mut sig_str = ::rustbus::wire::marshal::traits::SignatureBuffer::new();
                    <#ty as ::rustbus::Signature>::sig_str(&mut sig_str);
                    ::rustbus::wire::util::write_signature(sig_str.as_ref(), &mut ctx.buf);

                    val.marshal(ctx)?;
                    Ok(())
                },
            }
        }
    } else {
        panic!("Variants with no fields are not supported yet")
    }
}
