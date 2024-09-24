use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, States, WatchContent, WatchState,
};

struct NodeData<T> {
    node: Box<dyn ExecutableAndWatch<T>>,
    watch_state: WatchState,
}

pub struct Fallback<T> {
    nodes: Vec<NodeData<T>>,
    counter: usize,
}

impl<T> Executable<T> for Fallback<T> {
    fn start(&mut self, _: &mut T) {
        self.counter = 0;
    }

    fn execute(&mut self, data: &mut T) -> States {
        loop {
            if self.counter >= self.nodes.len() {
                return States::Fail;
            }

            let node = &mut self.nodes[self.counter];

            if node.watch_state != WatchState::Running {
                node.node.start(data);
                node.watch_state = WatchState::Running;
            }

            let state = node.node.execute(data);

            if state == States::Running {
                return States::Running;
            } else {
                node.node.end(data);
                if state == States::Succes {
                    node.watch_state = WatchState::Succeeded;
                } else {
                    node.watch_state = WatchState::Failed;
                }
            }

            if state == States::Fail {
                self.counter += 1;
            }

            if state == States::Succes {
                return States::Succes;
            }
        }
    }

    fn end(&mut self, data: &mut T) {
        let node = &mut self.nodes[self.counter];

        if node.watch_state == WatchState::Running {
            node.node.end(data);
            node.watch_state = WatchState::Cancelled;
        }
    }
}

impl<T> ExecutableWatch for Fallback<T> {
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
            name: "async_wait".to_string(),
            watch_state: WatchState::None,
            childs: childs,
        }
    }
}

impl<T> Fallback<T> {
    pub fn new(nodes: Vec<Box<dyn ExecutableAndWatch<T>>>) -> Box<Self> {
        Box::new(Fallback {
            nodes: nodes
                .into_iter()
                .map(|node| NodeData {
                    node: node,
                    watch_state: WatchState::None,
                })
                .collect(),
            counter: 0,
        })
    }
}
