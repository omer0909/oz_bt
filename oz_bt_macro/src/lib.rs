use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, ExprLit, ImplItem, ImplItemType, ItemImpl, Lit, Meta, Token,
};

fn has_assoc_type(input_impl: &syn::ItemImpl, name: &str) -> bool {
    input_impl
        .items
        .iter()
        .any(|item| matches!(item, ImplItem::Type(ImplItemType { ident, .. }) if ident == name))
}

struct NodeArgs {
    crate_alias: Option<String>,
    node_type: Option<String>,
}

impl Parse for NodeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut crate_alias = None;
        let mut node_type = None;

        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        for meta in metas {
            let nv = match meta {
                Meta::NameValue(nv) => nv,
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "beklenen format: crate = \"...\" veya node_type = \"...\"",
                    ))
                }
            };
            let value = match &nv.value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) => s.value(),
                _ => {
                    return Err(syn::Error::new_spanned(
                        &nv.value,
                        "değer string literal olmalı",
                    ))
                }
            };

            if nv.path.is_ident("crate") {
                crate_alias = Some(value);
            } else if nv.path.is_ident("node_type") {
                node_type = Some(value);
            } else {
                return Err(syn::Error::new_spanned(&nv.path, "bilinmeyen parametre"));
            }
        }

        Ok(NodeArgs {
            crate_alias,
            node_type,
        })
    }
}

#[proc_macro_attribute]
pub fn node(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as NodeArgs);
    let mut input_impl = parse_macro_input!(item as ItemImpl);

    let add_type = |input_impl: &mut ItemImpl, type_name| {
        let ident = syn::Ident::new(type_name, proc_macro2::Span::call_site());
        let assoc_type: syn::ImplItemType = syn::parse_quote! {
            type #ident = ();
        };
        input_impl.items.insert(0, ImplItem::Type(assoc_type));
    };

    let has_input = has_assoc_type(&input_impl, "Input");

    if !has_input {
        add_type(&mut input_impl, "Input");
    }

    let has_output = has_assoc_type(&input_impl, "Output");

    if !has_output {
        add_type(&mut input_impl, "Output");
    }

    let struct_name = match &*input_impl.self_ty {
        syn::Type::Path(tp) => tp.path.clone(),
        other => {
            return syn::Error::new_spanned(
                other,
                "#[node] sadece isimlendirilmiş tipler için kullanılabilir",
            )
            .to_compile_error()
            .into();
        }
    };

    let struct_path: proc_macro2::TokenStream = if let Some(nt) = &args.node_type {
        syn::parse_str(nt)
            .unwrap_or_else(|e| panic!("node_type geçerli bir path değil: {nt:?} ({e})"))
    } else {
        quote! { #struct_name }
    };

    let oz_bt_crate: proc_macro2::TokenStream = if let Some(alias) = &args.crate_alias {
        let ident = syn::Ident::new(alias, proc_macro2::Span::call_site());
        quote! { ::#ident }
    } else {
        quote! { ::oz_bt }
    };

    let struct_ident = &struct_name.segments.last().unwrap().ident;
    let macro_name = syn::Ident::new(
        &struct_ident.to_string().to_snake_case(),
        struct_ident.span(),
    );

    let macro_name_i = format_ident!("{}_i", macro_name);
    let macro_name_o = format_ident!("{}_o", macro_name);
    let macro_name_io = format_ident!("{}_io", macro_name);

    let mut added_macros = Vec::new();

    if has_input && has_output {
        added_macros.push(quote! {
            #[macro_export]
            macro_rules! #macro_name_io {
                ($input:expr, $output:expr $(,)?) => {
                    #oz_bt_crate::CustomNode::<#struct_path>::new_io($input, $output)
                };
            }
        });
    }

    if has_input {
        added_macros.push(quote! {
            #[macro_export]
            macro_rules! #macro_name_i {
                ( $input:expr $(,)? ) => {
                    #oz_bt_crate::CustomNode::<#struct_path>::new_i($input)
                };
            }
        });
    }

    if has_output {
        added_macros.push(quote! {
            #[macro_export]
            macro_rules! #macro_name_o {
                ( $output:expr $(,)? ) => {
                    #oz_bt_crate::CustomNode::<#struct_path>::new_o($output)
                };
            }
        });
    }

    let expanded = quote! {
        #input_impl

        #[macro_export]
        macro_rules! #macro_name {
            () => {
                #oz_bt_crate::CustomNode::<#struct_path>::new()
            };
        }

        #(#added_macros)*

    };

    expanded.into()
}
