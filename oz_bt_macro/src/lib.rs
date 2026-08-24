use darling::ast::NestedMeta;
use darling::FromMeta;
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ImplItem, ImplItemType, ItemImpl};

fn has_assoc_type(input_impl: &syn::ItemImpl, name: &str) -> bool {
    input_impl
        .items
        .iter()
        .any(|item| matches!(item, ImplItem::Type(ImplItemType { ident, .. }) if ident == name))
}

#[derive(Debug, FromMeta)]
struct NodeArgs {
    #[darling(rename = "crate")]
    crate_alias: Option<String>,
}

#[proc_macro_attribute]
pub fn node(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => return darling::Error::from(e).write_errors().into(),
    };

    let args = match NodeArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

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

    let struct_path = match &*input_impl.self_ty {
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

    let oz_bt_crate: proc_macro2::TokenStream = if let Some(alias) = &args.crate_alias {
        let ident = syn::Ident::new(alias, proc_macro2::Span::call_site());
        quote! { ::#ident }
    } else {
        quote! { ::oz_bt }
    };

    let struct_ident = &struct_path.segments.last().unwrap().ident;
    let helper_name = syn::Ident::new(
        &struct_ident.to_string().to_snake_case(),
        struct_ident.span(),
    );

    let helper_name_i = format_ident!("{}_i", helper_name);
    let helper_name_o = format_ident!("{}_o", helper_name);
    let helper_name_io = format_ident!("{}_io", helper_name);

    let mut added_helpers = Vec::new();

    if has_input && has_output {
        added_helpers.push(quote! {
            fn #helper_name_io(
                input: impl Fn(&mut <#struct_path as Node>::Data) -> <#struct_path as Node>::Input + 'static,
                output: std::rc::Rc<std::cell::RefCell<<#struct_path as Node>::Output>>,
            ) -> Box<CustomNode<#struct_path>> {
                #oz_bt_crate::CustomNode::<#struct_path>::new_io(input, output)
            }
        });
    }

    if has_input {
        added_helpers.push(quote! {
            fn #helper_name_i(
                input: impl Fn(&mut <#struct_path as Node>::Data) -> <#struct_path as Node>::Input + 'static,
            ) -> Box<CustomNode<#struct_path>> {
                #oz_bt_crate::CustomNode::<#struct_path>::new_i(input)
            }
        });
    }

    if has_output {
        added_helpers.push(quote! {
            fn #helper_name_o(
                output: std::rc::Rc<std::cell::RefCell<<#struct_path as Node>::Output>>,
            ) -> Box<CustomNode<#struct_path>> {
                #oz_bt_crate::CustomNode::<#struct_path>::new_o(output)
            }
        });
    }

    let expanded = quote! {
        #input_impl

        fn #helper_name() -> Box<CustomNode<#struct_path>> {
            #oz_bt_crate::CustomNode::<#struct_path>::new()
        }

        #(#added_helpers)*

    };

    expanded.into()
}
