use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, States, WatchContent, WatchState,
};

pub struct EventNode {
    event: Box<dyn Fn() -> bool>,
    name: String,
}

impl Executable for EventNode {
    fn execute(&mut self, _: f32) -> States {
        if (*self.event)() {
            States::Succes
        } else {
            States::Fail
        }
    }
}

impl ExecutableWatch for EventNode {
    fn get_content(&self) -> WatchContent {
        WatchContent {
            name: format!("event<{}>", self.name),
            watch_state: WatchState::None,
            childs: Vec::new(),
        }
    }
}

impl EventNode {
    pub fn new(name: String, event: impl Fn() -> bool + 'static) -> Box<Self> {
        Box::new(EventNode {
            event: Box::new(event),
            name: name,
        })
    }
}
