use crate::exec::{Executable, ExecutableWatch, States, WatchContent, WatchState};

pub struct EventNode<T> {
    event: Box<dyn Fn(&mut T) -> bool>,
    name: String,
}

impl<T> Executable<T> for EventNode<T> {
    fn execute(&mut self, data: &mut T) -> States {
        if (*self.event)(data) {
            States::Succes
        } else {
            States::Fail
        }
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
    pub fn new(name: String, event: impl Fn(&mut T) -> bool + 'static) -> Box<Self> {
        Box::new(EventNode {
            event: Box::new(event),
            name: name,
        })
    }
}
