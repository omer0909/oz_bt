use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, ItemMod};

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
