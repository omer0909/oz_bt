use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, NodeTypes, States, WatchContent, WatchState,
};

pub struct GroupOut<T> {
    node: Box<dyn ExecutableAndWatch<T>>,
    watch_state: WatchState,
    branch_name: String,
    comment: Option<String>,
}

impl<T> Executable<T> for GroupOut<T> {
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

impl<T> ExecutableWatch for GroupOut<T> {
    fn get_content(&self) -> WatchContent {
        WatchContent {
            node_type: NodeTypes::GroupOut(self.branch_name.clone()),
            name: "group_out".to_string(),
            watch_state: WatchState::None,
            childs: vec![WatchContent {
                watch_state: self.watch_state,
                ..self.node.get_content()
            }],
            comment: self.comment.clone(),
        }
    }
}

impl<T> GroupOut<T> {
    pub fn new(branch_name: &str, node: Box<dyn ExecutableAndWatch<T>>) -> Box<Self> {
        Box::new(GroupOut {
            branch_name: branch_name.to_string(),
            node,
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
macro_rules! group_out {
    ($name:expr, $node:expr $(,)?) => {
        $crate::GroupOut::new($name, $node)
    };
}
