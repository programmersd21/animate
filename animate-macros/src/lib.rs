use darling::{FromField, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, LitStr, Type, parse_macro_input};
#[derive(Debug, Default, FromMeta)]
struct AnimateAttr {
    #[darling(default)]
    update: Option<LitStr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum AnimType {
    #[default]
    Tween,
    Spring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum AnimMode {
    #[default]
    Once,
    Cycle,
    Alternate,
}

impl FromMeta for AnimMode {
    fn from_string(value: &str) -> darling::Result<Self> {
        match value {
            "once" => Ok(AnimMode::Once),
            "cycle" => Ok(AnimMode::Cycle),
            "alternate" => Ok(AnimMode::Alternate),
            other => Err(darling::Error::unknown_value(other)),
        }
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(spring, tween))]
struct AnimateField {
    ident: Option<syn::Ident>,
    ty: Type,
    #[darling(default)]
    duration: Option<u64>,
    #[darling(default)]
    easing: Option<syn::Path>,
    #[darling(default)]
    interp: Option<syn::Path>,
    #[darling(default)]
    mode: Option<AnimMode>,
    #[darling(default)]
    stiffness: Option<f64>,
    #[darling(default)]
    damping: Option<f64>,
    #[darling(default)]
    mass: Option<f64>,
}


#[proc_macro_attribute]
pub fn animate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = match darling::ast::NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(darling::Error::from(e).write_errors()),
    };
    let animate_attr = match AnimateAttr::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let input = parse_macro_input!(item as DeriveInput);
    process(input, animate_attr)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}


fn process(input: DeriveInput, attr: AnimateAttr) -> syn::Result<TokenStream2> {
    let struct_name = &input.ident;
    let vis = &input.vis;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(f) => &f.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "#[animate] only supports named field structs",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "#[animate] only supports structs",
            ));
        }
    };

    let animate_fields: Vec<AnimateField> = fields
        .iter()
        .map(AnimateField::from_field)
        .collect::<darling::Result<_>>()
        .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e))?;

    let update_method = attr
        .update
        .as_ref()
        .map(|s| syn::Ident::new(&s.value(), s.span()))
        .unwrap_or_else(|| syn::Ident::new("animate", proc_macro2::Span::call_site()));

    let final_fields: Vec<TokenStream2> = fields
        .iter()
        .zip(animate_fields.iter())
        .map(|(raw, gf)| {
            let name = gf.ident.as_ref().unwrap();
            let ty = &gf.ty;
            let field_vis = &raw.vis;

            let anim_type = field_anim_type(raw)?;
            let mode = field_mode(gf, anim_type);

            let attrs: Vec<_> = raw
                .attrs
                .iter()
                .filter(|a| !["spring", "tween"].iter().any(|attr| a.path().is_ident(attr)))
                .collect();

            let field = match (mode, anim_type) {
                (None, _) => quote! { #(#attrs)* #field_vis #name: #ty },

                (Some(mode), Some(AnimType::Tween)) => {
                    let spring_mode = tween_mode_ident(&mode);
                    quote! {
                        #(#attrs)*
                        #field_vis #name: animate::Tween<
                            #ty,
                            fn(f64) -> f64,
                            fn(&#ty, &#ty, f64) -> #ty,
                            animate::#spring_mode
                        >
                    }
                }

                (Some(mode), Some(AnimType::Spring)) => {
                    let spring_mode = spring_mode_ident(&mode);
                    quote! {
                        #(#attrs)*
                        #field_vis #name: animate::Spring<
                            #ty,
                            fn(&#ty, &#ty, &<#ty as animate::SpringAnim>::Velocity, animate::SpringParams, f64) -> (#ty, <#ty as animate::SpringAnim>::Velocity),
                            animate::#spring_mode
                        >
                    }
                }

                (Some(_), None) => quote! { #(#attrs)* #field_vis #name: #ty },
            };

            Ok(field)
        })
        .collect::<syn::Result<_>>()?;

    let mut params = Vec::new();
    let mut inits = Vec::new();
    let mut update_calls = Vec::new();

    for (raw, gf) in fields.iter().zip(animate_fields.iter()) {
        let name = gf.ident.as_ref().unwrap();
        let ty = &gf.ty;
        params.push(quote! { #name: #ty });

        let anim_type = field_anim_type(raw)?;
        let mode = field_mode(gf, anim_type);

        match (mode, anim_type) {
            (None, _) => {
                inits.push(quote! { #name });
            }

            (Some(_mode), Some(AnimType::Tween)) => {
                let duration = gf.duration.unwrap_or(0) as f64;
                let easing = easing_path(gf.easing.as_ref());
                let interp = tween_interp_path(ty, gf.interp.as_ref());
                inits.push(quote! {
                    #name: animate::Tween::new(#name, #duration, #easing, #interp)
                });
                update_calls.push(quote! {
                    animate::Animate::update(&mut self.#name);
                });
            }

            (Some(_mode), Some(AnimType::Spring)) => {
                let stiffness = gf.stiffness.unwrap_or(200.0) as f32;
                let damping = gf.damping.unwrap_or(20.0) as f32;
                let mass = gf.mass.unwrap_or(1.0) as f32;
                let interp = spring_interp_path(ty);
                inits.push(quote! {
                    #name: animate::Spring::new(
                        #name,
                        animate::SpringParams {
                            stiffness: #stiffness,
                            damping: #damping,
                            mass: #mass,
                            ..animate::SpringParams::default()
                        },
                        #interp,
                    )
                });
                update_calls.push(quote! {
                    animate::Animate::update(&mut self.#name);
                });
            }

            (Some(_), None) => {
                inits.push(quote! { #name });
            }
        }
    }

    Ok(quote! {
        #vis struct #struct_name #impl_generics #where_clause {
            #(#final_fields),*
        }

        use animate::Animate as _;

        impl #impl_generics #struct_name #ty_generics #where_clause {
            pub fn new(#(#params),*) -> Self {
                Self { #(#inits),* }
            }
            pub fn #update_method(&mut self) {
                #(#update_calls)*
            }
        }
    })
}


fn field_mode(gf: &AnimateField, anim_type: Option<AnimType>) -> Option<TokenStream2> {
    if anim_type.is_none() {
        return None;
    }

    Some(mode_token(gf.mode.unwrap_or(AnimMode::Once)))
}

fn field_anim_type(raw: &syn::Field) -> syn::Result<Option<AnimType>> {
    let has_spring = raw.attrs.iter().any(|a| a.path().is_ident("spring"));
    let has_tween = raw.attrs.iter().any(|a| a.path().is_ident("tween"));

    match (has_spring, has_tween) {
        (true, true) => Err(syn::Error::new_spanned(
            raw,
            "field cannot have both #[spring] and #[tween]",
        )),
        (true, false) => Ok(Some(AnimType::Spring)),
        (false, true) => Ok(Some(AnimType::Tween)),
        (false, false) => Ok(None),
    }
}

fn mode_token(mode: AnimMode) -> TokenStream2 {
    match mode {
        AnimMode::Once => quote!(once),
        AnimMode::Cycle => quote!(cycle),
        AnimMode::Alternate => quote!(alternate),
    }
}

fn tween_mode_ident(mode: &TokenStream2) -> TokenStream2 {
    let s = mode.to_string();
    match s.as_str() {
        "once" => quote!(Once),
        "cycle" => quote!(Cycle),
        "alternate" => quote!(Alternate),
        _ => quote!(Once),
    }
}

fn spring_mode_ident(mode: &TokenStream2) -> TokenStream2 {
    let s = mode.to_string();
    match s.as_str() {
        "once" => quote!(Once),
        "cycle" => quote!(Cycle),
        "alternate" => quote!(Alternate),
        _ => quote!(Once),
    }
}

fn easing_path(path: Option<&syn::Path>) -> TokenStream2 {
    match path {
        Some(p) if p.leading_colon.is_none() && p.segments.len() == 1 => {
            quote! { animate::easing::#p as fn(f64) -> f64 }
        }
        Some(p) => quote! { #p as fn(f64) -> f64 },
        None => quote! { animate::easing::linear as fn(f64) -> f64 },
    }
}

fn tween_interp_path(ty: &syn::Type, path: Option<&syn::Path>) -> TokenStream2 {
    match path {
        Some(p) if p.leading_colon.is_none() && p.segments.len() == 1 => {
            let module = type_to_module(ty);
            quote! { animate::types::#module::#p as fn(&#ty, &#ty, f64) -> #ty }
        }
        Some(p) => quote! { #p as fn(&#ty, &#ty, f64) -> #ty },
        None => quote! { <#ty as animate::TweenAnim>::tween as fn(&#ty, &#ty, f64) -> #ty },
    }
}

fn spring_interp_path(ty: &syn::Type) -> TokenStream2 {
    quote! {
        <#ty as animate::SpringAnim>::spring
            as fn(&#ty, &#ty, &<#ty as animate::SpringAnim>::Velocity, animate::SpringParams, f64)
                -> (#ty, <#ty as animate::SpringAnim>::Velocity)
    }
}

fn type_to_module(ty: &syn::Type) -> syn::Ident {
    match ty {
        syn::Type::Path(tp) => {
            let name = tp.path.segments.last().unwrap().ident.to_string();
            let module = match name.as_str() {
                "String" => "string",
                "f64" | "f32" | "usize" | "isize" | "u64" | "i64" | "u32" | "i32" | "u16"
                | "i16" | "u8" | "i8" => "num",
                _ => Box::leak(name.to_lowercase().into_boxed_str()),
            };
            syn::Ident::new(module, proc_macro2::Span::call_site())
        }
        _ => syn::Ident::new("unknown", proc_macro2::Span::call_site()),
    }
}
