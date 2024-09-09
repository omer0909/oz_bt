use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, States, WatchContent, WatchState,
};

struct NodeData {
    node: Box<dyn ExecutableAndWatch>,
    watch_state: WatchState,
}

pub struct AsyncWait {
    nodes: Vec<NodeData>,
}

impl Executable for AsyncWait {
    fn start(&mut self) {
        for node in &mut self.nodes {
            node.node.start();
            node.watch_state = WatchState::Running;
        }
    }

    fn execute(&mut self, dt: f32) -> States {
        for node in &mut self.nodes {
            if node.watch_state == WatchState::Running {
                let state = node.node.execute(dt);
                if state != States::Running {
                    node.node.end();
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
            return States::Succes;
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

impl ExecutableWatch for AsyncWait {
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

impl AsyncWait {
    pub fn new(nodes: Vec<Box<dyn ExecutableAndWatch>>) -> Box<Self> {
        Box::new(AsyncWait {
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
