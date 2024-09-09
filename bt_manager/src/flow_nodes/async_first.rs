use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, States, WatchContent, WatchState,
};

struct NodeData {
    node: Box<dyn ExecutableAndWatch>,
    watch_state: WatchState,
}

pub struct AsyncFirst {
    nodes: Vec<NodeData>,
}

impl Executable for AsyncFirst {
    fn start(&mut self) {
        for node in &mut self.nodes {
            node.node.start();
            node.watch_state = WatchState::Running;
        }
    }

    fn execute(&mut self, dt: f32) -> States {
        for node in &mut self.nodes {
            let state = node.node.execute(dt);
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

    fn end(&mut self) {
        for node in &mut self.nodes {
            if node.watch_state == WatchState::Running {
                node.node.end();
                node.watch_state = WatchState::Cancelled;
            }
        }
    }
}

impl ExecutableWatch for AsyncFirst {
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

impl AsyncFirst {
    pub fn new(nodes: Vec<Box<dyn ExecutableAndWatch>>) -> Box<Self> {
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
