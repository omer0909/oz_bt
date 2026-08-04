use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, NodeTypes, States, WatchContent, WatchState,
};

struct NodeData<T> {
    node: Box<dyn ExecutableAndWatch<T>>,
    watch_state: WatchState,
}

pub struct AsyncWait<T> {
    nodes: Vec<NodeData<T>>,
    comment: Option<String>,
}

impl<T> Executable<T> for AsyncWait<T> {
    fn start(&mut self, data: &mut T) {
        for node in &mut self.nodes {
            node.node.start(data);
            node.watch_state = WatchState::Running;
        }
    }

    fn execute(&mut self, data: &mut T) -> States {
        for node in &mut self.nodes {
            if node.watch_state == WatchState::Running {
                let state = node.node.execute(data);
                if state != States::Running {
                    node.node.end(data);
                    if state == States::Fail {
                        node.watch_state = WatchState::Failed;
                        return States::Fail;
                    } else {
                        node.watch_state = WatchState::Succeeded;
                    }
                }
            }
        }

        if self
            .nodes
            .iter()
            .all(|x| x.watch_state != WatchState::Running)
        {
            return States::Success;
        }

        States::Running
    }

    fn end(&mut self, data: &mut T) {
        for node in &mut self.nodes {
            if node.watch_state == WatchState::Running {
                node.node.end(data);
                node.watch_state = WatchState::Cancelled;
            }
        }
    }
}

impl<T> ExecutableWatch for AsyncWait<T> {
    fn get_content(&self) -> WatchContent {
        let childs = self
            .nodes
            .iter()
            .map(|x| WatchContent {
                watch_state: x.watch_state,
                ..x.node.get_content()
            })
            .collect();

        WatchContent {
            node_type: NodeTypes::Flow,
            name: "async_wait".to_string(),
            watch_state: WatchState::None,
            childs: childs,
            comment: self.comment.clone(),
        }
    }
}

impl<T> AsyncWait<T> {
    pub fn new(nodes: Vec<Box<dyn ExecutableAndWatch<T>>>) -> Box<Self> {
        Box::new(AsyncWait {
            nodes: nodes
                .into_iter()
                .map(|node| NodeData {
                    node: node,
                    watch_state: WatchState::None,
                })
                .collect(),
            comment: None,
        })
    }

    pub fn comment(mut self: Box<Self>, comment: &str) -> Box<Self> {
        self.comment = Some(comment.to_string());
        self
    }
}

#[macro_export]
macro_rules! async_wait {
    ( $( $x:expr ),* $(,)? ) => {
        $crate::AsyncWait::new(vec![ $( $x ),* ])
    };
}
