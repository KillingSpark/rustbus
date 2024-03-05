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
            fn marshal(&self, ctx: &mut ::rustbus::wire::marshal::MarshalContext<'_,'_>) -> Result<(), ::rustbus::wire::errors::MarshalError> {
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
            // Unnamed fields
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

pub fn make_variant_unmarshal_impl(
    ident: &syn::Ident,
    generics: &syn::Generics,
    variant: &Punctuated<Variant, Comma>,
) -> TokenStream {
    let marshal = variant
        .iter()
        .fold(Default::default(), |mut tokens: TokenStream, variant| {
            tokens.extend(variant_unmarshal(ident.clone(), variant));
            tokens
        });

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
            fn unmarshal(ctx: &mut ::rustbus::wire::unmarshal_context::UnmarshalContext<'_,'__internal_buf>) -> Result<Self, ::rustbus::wire::errors::UnmarshalError> {
                let sig = ctx.read_signature()?;

                #marshal
                Err(::rustbus::wire::errors::UnmarshalError::NoMatchingVariantFound)
            }
        }
    }
}

fn variant_unmarshal(enum_name: syn::Ident, variant: &syn::Variant) -> TokenStream {
    let name = variant.ident.clone();
    let field_types1 = variant
        .fields
        .iter()
        .map(|field| field.ty.to_token_stream());

    let field_types2 = field_types1.clone();

    if !variant.fields.is_empty() {
        if variant.fields.iter().next().unwrap().ident.is_some() {
            // Named fields
            let field_names = variant
                .fields
                .iter()
                .map(|field| field.ident.as_ref().unwrap().to_token_stream());

            quote! {
                let mut expected_sig = "(".to_owned();
                let mut sig_str = ::rustbus::wire::marshal::traits::SignatureBuffer::new();
                #(
                    sig_str.clear();
                    <#field_types1 as ::rustbus::Signature>::sig_str(&mut sig_str);
                    expected_sig.push_str(sig_str.as_ref());
                )*
                expected_sig.push(')');
                if sig.eq(&expected_sig) {
                    ctx.align_to(8)?;
                    let this = #enum_name::#name{
                        #(
                            #field_names: <#field_types2 as ::rustbus::Unmarshal>::unmarshal(ctx)?,
                        )*
                    };
                    return Ok(this);
                }
            }
        } else if variant.fields.iter().next().unwrap().ident.is_none() && variant.fields.len() > 1
        {
            quote! {
                let mut expected_sig = "(".to_owned();
                let mut sig_str = ::rustbus::wire::marshal::traits::SignatureBuffer::new();
                #(
                    sig_str.clear();
                    <#field_types1 as ::rustbus::Signature>::sig_str(&mut sig_str);
                    expected_sig.push_str(sig_str.as_ref());
                )*
                expected_sig.push(')');
                if sig.eq(&expected_sig) {
                    ctx.align_to(8)?;
                    let this = #enum_name::#name(
                        #(
                            <#field_types2 as ::rustbus::Unmarshal>::unmarshal(ctx)?,
                        )*
                    );
                    return Ok(this);
                }
            }
        } else {
            // One unnamed field
            let mut field_types = field_types1;
            let ty = field_types.next().unwrap();
            quote! {
                let mut sig_str = ::rustbus::wire::marshal::traits::SignatureBuffer::new();
                <#ty as ::rustbus::Signature>::sig_str(&mut sig_str);

                if sig.eq(sig_str.as_ref()) {
                    let this = #enum_name::#name(
                        <#ty as ::rustbus::Unmarshal>::unmarshal(ctx)?,
                    );
                    return Ok(this);
                }
            }
        }
    } else {
        panic!("Variants with no fields are not supported yet")
    }
}
