use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Expr, ItemMod, Lit};

#[proc_macro_attribute]
pub fn node(_attr: TokenStream, item: TokenStream) -> TokenStream {
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

            pub mod lib {
                impl NodeManager {
                    pub fn new(
                        input: impl Fn(&mut super::Data) -> super::Input + 'static,
                        output: std::rc::Rc<std::cell::RefCell<super::Output>>,
                    ) -> Box<Self> {
                        Box::new(NodeManager {
                            output_handle: output,
                            input_handle: Box::new(input),
                            node: None,
                        })
                    }
                }

                pub struct NodeManager {
                    pub input_handle: Box<dyn Fn(&mut super::Data) -> super::Input>,
                    pub output_handle: std::rc::Rc<std::cell::RefCell<super::Output>>,
                    pub node: Option<Box<dyn super::CustomNode>>,
                }

                impl crate::executable::exec::Executable<super::Data> for NodeManager {
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

                    fn execute(&mut self, data: &mut super::Data) -> crate::executable::exec::States {
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

                impl crate::executable::exec::ExecutableWatch for NodeManager {
                    fn get_content(&self) -> crate::executable::exec::WatchContent {
                        crate::executable::exec::WatchContent {
                            name: #mod_name_string.to_string(),
                            watch_state: crate::executable::exec::WatchState::None,
                            childs: Vec::new(),
                        }
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn add_wrap(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let wrapped_str = format!("({})", input_str);
    wrapped_str.parse().unwrap()
}

#[proc_macro]
pub fn handle(input: TokenStream) -> TokenStream {
    let wrapped_input = add_wrap(input);
    let input = parse_macro_input!(wrapped_input as syn::ExprTuple);

    let var_name = match &input.elems[0] {
        Expr::Path(path) => path.path.segments.last().unwrap().ident.clone(),
        _ => panic!("Input must be a valid identifier"),
    };
    let data = &input.elems[1];

    let number = match &input.elems[2] {
        Expr::Lit(lit) => {
            if let Lit::Int(lit_int) = &lit.lit {
                let value: usize = lit_int.base10_parse().unwrap();
                value
            } else {
                panic!("3. param must be an integer");
            }
        }
        _ => panic!("3. param must be an integer"),
    };

    let extra = if number > 1 {
        (1..number)
            .map(|ref i| {
                let extra_name = format_ident!("{}{}", var_name, i.to_string());

                quote! {
                    let #extra_name = std::rc::Rc::clone(&#var_name);
                }
            })
            .collect()
    } else {
        quote! {}
    };

    let expanded = quote! {
        let #var_name = std::rc::Rc::new(std::cell::RefCell::new(#data));
        #extra
    };

    TokenStream::from(expanded)
}
