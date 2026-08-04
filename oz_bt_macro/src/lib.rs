use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, ExprLit, ImplItem, ImplItemType, ItemImpl, ItemMod, Lit, Meta, Token,
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

#[proc_macro_attribute]
pub fn node_(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemMod);
    let mod_name = &input.ident;
    let mod_name_string = mod_name.to_string();

    let content = if let Some((_, ref items)) = input.content {
        let content_tokens = items
            .iter()
            .map(|item| quote! { #item })
            .collect::<TokenStream2>();
        content_tokens
    } else {
        TokenStream2::new()
    };

    let expanded: TokenStream2 = quote! {
        pub mod #mod_name {

            #content

            pub use lib::NodeManager;

            trait CustomNode {
                fn start(&mut self, _: &mut CustomData) {}
                fn execute(&mut self, _: &mut CustomData) -> crate::exec::States;
                fn end(&mut self, _: &mut CustomData) {}
            }

            struct CustomData<'a> {
                data: &'a mut Data,
                input: &'a Input,
                output: &'a mut Output,
            }

            pub fn new(
                input: impl Fn(&mut Data) -> Input + 'static,
            ) -> Box<NodeManager> {
                NodeManager::new(input)
            }

            pub mod lib {
                impl NodeManager {
                    pub fn new(
                        input: impl Fn(&mut super::Data) -> super::Input + 'static,
                    ) -> Box<Self> {
                        Box::new(NodeManager {
                            output_handle: std::rc::Rc::new(std::cell::RefCell::new(super::Output::default())),
                            input_handle: Box::new(input),
                            node: None,
                            comment: None,
                        })
                    }

                    pub fn comment(mut self: Box<Self>, comment: &str) -> Box<Self> {
                        self.comment = Some(comment.to_string());
                        self
                    }

                    pub fn with_output(mut self: Box<Self>, output: std::rc::Rc<std::cell::RefCell<super::Output>>) -> Box<Self>{
                        self.output_handle = output;
                        self
                    }
                }

                pub struct NodeManager {
                    pub input_handle: Box<dyn Fn(&mut super::Data) -> super::Input>,
                    pub output_handle: std::rc::Rc<std::cell::RefCell<super::Output>>,
                    pub node: Option<Box<dyn super::CustomNode>>,
                    pub comment: Option<String>,
                }

                impl ::oz_bt::executable::exec::Executable<super::Data> for NodeManager {
                    fn start(&mut self, data: &mut super::Data) {
                        self.node = Some(Box::new(super::Node::default()));
                        let input_data = self.input_handle.as_ref()(data);
                        let mut output_data = self.output_handle.borrow_mut();
                        let mut custom_data = super::CustomData {
                            input: &input_data,
                            output: &mut output_data,
                            data: data,
                        };
                        self.node.as_mut().unwrap().start(&mut custom_data);
                    }

                    fn execute(&mut self, data: &mut super::Data) -> ::oz_bt::executable::exec::States {
                        let input_data = self.input_handle.as_ref()(data);
                        let mut output_data = self.output_handle.borrow_mut();
                        let mut custom_data = super::CustomData {
                            input: &input_data,
                            output: &mut output_data,
                            data: data,
                        };
                        self.node.as_mut().unwrap().execute(&mut custom_data)
                    }

                    fn end(&mut self, data: &mut super::Data) {
                        let input_data = self.input_handle.as_ref()(data);
                        let mut output_data = self.output_handle.borrow_mut();
                        let mut custom_data = super::CustomData {
                            input: &input_data,
                            output: &mut output_data,
                            data: data,
                        };
                        self.node.as_mut().unwrap().end(&mut custom_data);
                        self.node = None;
                    }
                }

                impl ::oz_bt::executable::exec::ExecutableWatch for NodeManager {
                    fn get_content(&self) -> ::oz_bt::executable::exec::WatchContent {
                        ::oz_bt::executable::exec::WatchContent {
                            node_type: ::oz_bt::executable::exec::NodeTypes::Leaf,
                            name: #mod_name_string.to_string(),
                            watch_state: ::oz_bt::executable::exec::WatchState::None,
                            childs: Vec::new(),
                            comment: self.comment.clone(),
                        }
                    }
                }
            }
        }

        #[macro_export]
        macro_rules! #mod_name {
            ( $x:expr $(,)? ) => {
                #mod_name::new($x)
            };
        }

    };

    TokenStream::from(expanded)
}
