use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, NodeTypes, States, WatchContent, WatchState,
};

pub struct GroupIn<T> {
    node: Box<dyn ExecutableAndWatch<T>>,
    watch_state: WatchState,
    group_name: String,
    comment: Option<String>,
}

impl<T> Executable<T> for GroupIn<T> {
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

impl<T> ExecutableWatch for GroupIn<T> {
    fn get_content(&self) -> WatchContent {
        WatchContent {
            node_type: NodeTypes::GroupIn(self.group_name.clone()),
            name: "group_in".to_string(),
            watch_state: WatchState::None,
            childs: vec![WatchContent {
                watch_state: self.watch_state,
                ..self.node.get_content()
            }],
            comment: self.comment.clone(),
        }
    }
}

impl<T> GroupIn<T> {
    pub fn new(group_name: &str, node: Box<dyn ExecutableAndWatch<T>>) -> Box<Self> {
        Box::new(GroupIn {
            group_name: group_name.to_string(),
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

pub fn group_in<T>(group_name: &str, node: Box<dyn ExecutableAndWatch<T>>) -> Box<GroupIn<T>> {
    GroupIn::new(group_name, node)
}
