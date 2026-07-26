use crate::exec::{Executable, ExecutableWatch, States, WatchContent, WatchState};

pub struct EventNode<T> {
    event: Box<dyn Fn(&mut T) -> bool>,
    name: String,
}

impl<T> Executable<T> for EventNode<T> {
    fn execute(&mut self, data: &mut T) -> States {
        States::from_bool((*self.event)(data))
    }
}

impl<T> ExecutableWatch for EventNode<T> {
    fn get_content(&self) -> WatchContent {
        WatchContent {
            name: format!("event<{}>", self.name),
            watch_state: WatchState::None,
            childs: Vec::new(),
        }
    }
}

impl<T> EventNode<T> {
    pub fn new(name: &str, event: impl Fn(&mut T) -> bool + 'static) -> Box<Self> {
        Box::new(EventNode {
            event: Box::new(event),
            name: String::from(name),
        })
    }
}

#[macro_export]
macro_rules! event_node {
    ($name:expr, $event:expr $(,)?) => {
        EventNode::new($name, $event)
    };
}
