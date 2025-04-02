use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, States, WatchContent, WatchState,
};

struct NodeData<T> {
    node: Box<dyn ExecutableAndWatch<T>>,
    watch_state: WatchState,
}

pub struct AsyncFirst<T> {
    nodes: Vec<NodeData<T>>,
}

impl<T> Executable<T> for AsyncFirst<T> {
    fn start(&mut self, data: &mut T) {
        for node in &mut self.nodes {
            node.node.start(data);
            node.watch_state = WatchState::Running;
        }
    }

    fn execute(&mut self, data: &mut T) -> States {
        for node in &mut self.nodes {
            let state = node.node.execute(data);
            if state != States::Running {
                if state == States::Succes {
                    node.watch_state = WatchState::Succeeded;
                } else {
                    node.watch_state = WatchState::Failed;
                }
                return state;
            }
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

impl<T> ExecutableWatch for AsyncFirst<T> {
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
            name: "async_first".to_string(),
            watch_state: WatchState::None,
            childs: childs,
        }
    }
}

impl<T> AsyncFirst<T> {
    pub fn new(nodes: Vec<Box<dyn ExecutableAndWatch<T>>>) -> Box<Self> {
        Box::new(AsyncFirst {
            nodes: nodes
                .into_iter()
                .map(|node| NodeData {
                    node: node,
                    watch_state: WatchState::None,
                })
                .collect(),
        })
    }
}
