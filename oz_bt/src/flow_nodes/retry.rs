use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, NodeTypes, States, WatchContent, WatchState,
};

pub struct Retry<T> {
    node: Box<dyn ExecutableAndWatch<T>>,
    failed: bool,
    watch_state: WatchState,
    comment: Option<String>,
}

impl<T> Executable<T> for Retry<T> {
    fn start(&mut self, data: &mut T) {
        self.failed = false;
        self.node.start(data);
        self.watch_state = WatchState::Running;
    }

    fn execute(&mut self, data: &mut T) -> States {
        if self.failed {
            self.node.start(data);
            self.watch_state = WatchState::Running;
            self.failed = false;
        }

        let state = self.node.execute(data);
        if state != States::Running {
            if state == States::Success {
                self.watch_state = WatchState::Succeeded;
            } else {
                self.watch_state = WatchState::Failed;
            }

            if state == States::Fail {
                self.failed = true;
                self.node.end(data);
                return States::Running;
            }

            return state;
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

impl<T> ExecutableWatch for Retry<T> {
    fn get_content(&self) -> WatchContent {
        WatchContent {
            node_type: NodeTypes::Decorator,
            name: "retry".to_string(),
            watch_state: WatchState::None,
            childs: vec![WatchContent {
                watch_state: self.watch_state,
                ..self.node.get_content()
            }],
            comment: self.comment.clone(),
        }
    }
}

impl<T> Retry<T> {
    pub fn new(node: Box<dyn ExecutableAndWatch<T>>) -> Box<Self> {
        Box::new(Retry {
            node,
            failed: false,
            watch_state: WatchState::None,
            comment: None,
        })
    }

    pub fn comment(mut self: Box<Self>, comment: &str) -> Box<Self> {
        self.comment = Some(comment.to_string());
        self
    }
}

#[macro_export]
macro_rules! retry {
    ( $x:expr $(,)? ) => {
        $crate::Retry::new($x)
    };
}
