use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, States, WatchContent, WatchState,
};

pub struct Invert<T> {
    node: Box<dyn ExecutableAndWatch<T>>,
    watch_state: WatchState,
}

impl<T> Executable<T> for Invert<T> {
    fn start(&mut self, data: &mut T) {
        self.node.start(data);
        self.watch_state = WatchState::Running;
    }

    fn execute(&mut self, data: &mut T) -> States {
        let state = self.node.execute(data);
        if state != States::Running {
            if state == States::Success {
                self.watch_state = WatchState::Succeeded;
            } else {
                self.watch_state = WatchState::Failed;
            }

            return if state == States::Success {
                States::Fail
            } else {
                States::Success
            };
        }

        States::Running
    }

    fn end(&mut self, data: &mut T) {
        if self.watch_state == WatchState::Running {
            self.node.end(data);
            self.watch_state = WatchState::Cancelled;
        }
    }
}

impl<T> ExecutableWatch for Invert<T> {
    fn get_content(&self) -> WatchContent {
        WatchContent {
            name: "invert".to_string(),
            watch_state: WatchState::None,
            childs: vec![WatchContent {
                watch_state: self.watch_state,
                ..self.node.get_content()
            }],
        }
    }
}

impl<T> Invert<T> {
    pub fn new(node: Box<dyn ExecutableAndWatch<T>>) -> Box<Self> {
        Box::new(Invert {
            node: node,
            watch_state: WatchState::None,
        })
    }
}

#[macro_export]
macro_rules! invert {
    ( $x:expr $(,)? ) => {
        Invert::new($x)
    };
}
