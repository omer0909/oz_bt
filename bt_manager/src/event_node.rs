use crate::exec::{Executable, ExecutableWatch, NodeTypes, States, WatchContent, WatchState};

pub struct EventNode<T> {
    event: Box<dyn Fn(&mut T) -> bool>,
    event_name: String,
    comment: Option<String>,
}

impl<T> Executable<T> for EventNode<T> {
    fn execute(&mut self, data: &mut T) -> States {
        States::from_bool((*self.event)(data))
    }
}

impl<T> ExecutableWatch for EventNode<T> {
    fn get_content(&self) -> WatchContent {
        WatchContent {
            node_type: NodeTypes::Event(self.event_name.clone()),
            name: "event".to_string(),
            watch_state: WatchState::None,
            childs: Vec::new(),
            comment: self.comment.clone(),
        }
    }
}

impl<T> EventNode<T> {
    pub fn new(event_name: &str, event: impl Fn(&mut T) -> bool + 'static) -> Box<Self> {
        Box::new(EventNode {
            event: Box::new(event),
            event_name: event_name.to_string(),
            comment: None,
        })
    }

    pub fn comment(mut self: Box<Self>, comment: &str) -> Box<Self> {
        self.comment = Some(comment.to_string());
        self
    }
}

#[macro_export]
macro_rules! event_node {
    ($name:expr, $event:expr $(,)?) => {
        EventNode::new($name, $event)
    };
}
