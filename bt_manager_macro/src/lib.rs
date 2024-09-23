use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Field, Fields, Item, ItemMod, ItemStruct};

#[proc_macro_attribute]
pub fn node(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemMod);
    let mod_name = &input.ident;
    let mod_name_string = mod_name.to_string();

    let mut input_struct: Option<ItemStruct> = None;
    let mut output_struct: Option<ItemStruct> = None;

    let add_data_node_struct = |item_struct: &mut ItemStruct| {
        if let Fields::Named(ref mut fields_named) = item_struct.fields {
            let new_field: Field = syn::parse_quote! {
                pub node_data: lib::NodeDataPtr
            };
            fields_named.named.push(new_field);
        }
    };

    if let Some((_, ref mut items)) = input.content {
        for item in items {
            if let Item::Struct(item_struct) = item {
                if item_struct.ident == "Input" {
                    input_struct = Some(item_struct.clone());
                }
                if item_struct.ident == "Output" {
                    output_struct = Some(item_struct.clone());
                }
                if item_struct.ident == "Node" {
                    add_data_node_struct(item_struct);
                }
            }
        }
    }

    let mut get_handles: TokenStream2 = quote! {}.into();
    let mut set_handles: TokenStream2 = quote! {}.into();

    if let Some(ref item_struct) = input_struct {
        get_handles = item_struct
            .fields
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();

                let field_type = &field.ty;

                let fn_name = format_ident!("get_{}", field_name);

                quote! {
                    fn #fn_name(&self) -> #field_type {
                        unsafe{(*self.node_data.0).inputs.as_ref().unwrap().#field_name.clone()}
                    }
                }
            })
            .collect();
    }

    if let Some(ref item_struct) = output_struct {
        set_handles = item_struct
            .fields
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();

                let field_type = &field.ty;

                let fn_name = format_ident!("set_{}", field_name);

                quote! {
                    fn #fn_name(& mut self, data : #field_type) {
                        *unsafe{(*self.node_data.0).outputs.#field_name.borrow_mut() = data;}
                    }
                }
            })
            .collect();
    }

    let mut input_data: TokenStream2 = quote! {}.into();
    if let Some(ref item_struct) = input_struct {
        input_data = item_struct
            .fields
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;

                quote! {
                    pub #field_name: Box<dyn Fn() -> #field_type> ,
                }
            })
            .collect();
    }

    let mut input_data_get: TokenStream2 = quote! {}.into();
    if let Some(ref item_struct) = input_struct {
        input_data_get = item_struct
            .fields
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();

                quote! {
                    #field_name: (*self.input_handles.#field_name)() ,
                }
            })
            .collect();
    }

    let mut output_data: TokenStream2 = quote! {}.into();
    if let Some(ref item_struct) = output_struct {
        output_data = item_struct
            .fields
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;

                quote! {
                    pub #field_name: std::rc::Rc<std::cell::RefCell<#field_type>> ,
                }
            })
            .collect();
    }

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

            pub mod lib{
                impl NodeManager{
                    pub fn get_all_inputs(&mut self) {
                        self.node_data.inputs = Some(super::Input {
                            #input_data_get
                        });
                    }

                    pub fn new(inputs : InputsHandles, outputs : OutputsHandles) -> Box<Self>{
                        Box::new(NodeManager {
                            node_data: NodeData {
                                inputs: None,
                                outputs: outputs,
                            },
                            input_handles: inputs,
                            node: None,
                        })
                    }
                }

                pub struct OutputsHandles {
                    #output_data
                }

                pub struct InputsHandles {
                    #input_data
                }

                pub struct NodeData {
                    pub inputs : Option<super::Input>,
                    pub outputs : OutputsHandles,
                }

                pub struct NodeDataPtr(pub *mut NodeData);

                impl Default for NodeDataPtr {
                    fn default() -> Self {
                        NodeDataPtr(std::ptr::null_mut())
                    }
                }

                pub struct NodeManager {
                    pub input_handles : InputsHandles,
                    pub node_data : NodeData,
                    pub node : Option<Box<dyn crate::executable::exec::Executable>>
                }

                impl crate::executable::exec::Executable for NodeManager {
                    fn start(&mut self) {
                        self.node = Some(Box::new(super::Node {
                            node_data: NodeDataPtr(&mut self.node_data),
                            ..super::Node::default()
                        }));
                        self.get_all_inputs();
                        self.node.as_mut().unwrap().start();
                    }

                    fn execute(&mut self, dt: f32) -> crate::executable::exec::States{
                        self.node.as_mut().unwrap().execute(dt)
                    }

                    fn end(&mut self) {
                        self.node.as_mut().unwrap().end();
                        self.node = None;
                    }
                }

                impl crate::executable::exec::ExecutableWatch for NodeManager {
                    fn get_content(&self) -> crate::executable::exec::WatchContent {
                        crate::executable::exec::WatchContent{
                            name: #mod_name_string.to_string(),
                            watch_state: crate::executable::exec::WatchState::None,
                            childs: Vec::new(),
                        }
                    }
                }
            }

            impl Node{
                #get_handles
                #set_handles
            }
        }
    };

    TokenStream::from(expanded)
}
