use crate::exec::{Executable, ExecutableWatch, NodeTypes, States, WatchContent, WatchState};
use heck::ToSnakeCase;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Node: Default + 'static {
    type Data;
    type Input;
    type Output;

    fn name() -> String {
        get_node_name::<Self>()
    }

    fn start(&mut self, _: &mut Ctx<Self>) {}
    fn execute(&mut self, ctx: &mut Ctx<Self>) -> States;
    fn end(&mut self, _: &mut Ctx<Self>) {}
}

pub struct Ctx<'a, N: Node> {
    pub data: &'a mut N::Data,
    pub input: &'a N::Input,
    pub output: &'a mut N::Output,
}

pub struct CustomNode<N: Node> {
    input_handle: Box<dyn Fn(&mut N::Data) -> N::Input>,
    output_handle: Rc<RefCell<N::Output>>,
    node: Option<N>,
    comment: Option<String>,
}

impl<N: Node> CustomNode<N>
where
    N::Output: Default,
{
    pub fn new_i(input: impl Fn(&mut N::Data) -> N::Input + 'static) -> Box<Self> {
        Self::new_io(input, Rc::new(RefCell::new(N::Output::default())))
    }
}

impl<N: Node> CustomNode<N>
where
    N::Output: Default,
    N::Input: Default,
{
    pub fn new() -> Box<Self> {
        Self::new_io(
            |_| N::Input::default(),
            Rc::new(RefCell::new(N::Output::default())),
        )
    }
}

impl<N: Node> CustomNode<N>
where
    N::Input: Default,
{
    pub fn new_o(output: Rc<RefCell<N::Output>>) -> Box<Self> {
        Self::new_io(|_| N::Input::default(), output)
    }
}

impl<N: Node> CustomNode<N> {
    pub fn new_io(
        input: impl Fn(&mut N::Data) -> N::Input + 'static,
        output: Rc<RefCell<N::Output>>,
    ) -> Box<Self> {
        Box::new(Self {
            input_handle: Box::new(input),
            output_handle: output,
            node: None,
            comment: None,
        })
    }

    pub fn comment(mut self: Box<Self>, comment: &str) -> Box<Self> {
        self.comment = Some(comment.to_string());
        self
    }
}

impl<N: Node> Executable<N::Data> for CustomNode<N> {
    fn start(&mut self, data: &mut N::Data) {
        self.node = Some(N::default());
        let input_data = self.input_handle.as_ref()(data);
        let mut output_data = self.output_handle.borrow_mut();
        let mut custom_data = Ctx {
            input: &input_data,
            output: &mut *output_data,
            data,
        };
        self.node.as_mut().unwrap().start(&mut custom_data);
    }

    fn execute(&mut self, data: &mut N::Data) -> States {
        let input_data = self.input_handle.as_ref()(data);
        let mut output_data = self.output_handle.borrow_mut();
        let mut custom_data = Ctx {
            input: &input_data,
            output: &mut *output_data,
            data,
        };
        self.node.as_mut().unwrap().execute(&mut custom_data)
    }

    fn end(&mut self, data: &mut N::Data) {
        let input_data = self.input_handle.as_ref()(data);
        let mut output_data = self.output_handle.borrow_mut();
        let mut custom_data = Ctx {
            input: &input_data,
            output: &mut *output_data,
            data,
        };
        self.node.as_mut().unwrap().end(&mut custom_data);
        self.node = None;
    }
}

impl<N: Node> ExecutableWatch for CustomNode<N> {
    fn get_content(&self) -> WatchContent {
        WatchContent {
            node_type: NodeTypes::Leaf,
            name: N::name().to_string(),
            watch_state: WatchState::None,
            childs: Vec::new(),
            comment: self.comment.clone(),
        }
    }
}

pub trait HandleExt<T> {
    fn get(&self) -> T
    where
        T: Copy;
    fn set(&self, value: T);
    fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R;
}

impl<T> HandleExt<T> for Rc<RefCell<T>> {
    fn get(&self) -> T
    where
        T: Copy,
    {
        *self.borrow()
    }

    fn set(&self, value: T) {
        *self.borrow_mut() = value;
    }

    fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.borrow())
    }
}

pub fn handle<T>(value: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(value))
}

fn get_node_name<T>() -> String {
    let full = std::any::type_name::<T>();
    let name = full.rsplit("::").next().unwrap_or(full);
    name.to_snake_case()
}

#[macro_export]
macro_rules! clone {
    ([$($name:ident),+ $(,)?], $body:expr) => {{
        $(let $name = $name.clone();)+
        $body
    }};
}

#[macro_export]
macro_rules! handle {
    ([$($name:ident = $init:expr),+ $(,)?], $body:expr) => {{
        $(let $name = std::rc::Rc::new(std::cell::RefCell::new($init));)+
        $body
    }};
}
