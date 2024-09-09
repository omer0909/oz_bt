use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, States, WatchContent, WatchState,
};

struct NodeData {
    node: Box<dyn ExecutableAndWatch>,
    watch_state: WatchState,
}

pub struct Sequence {
    nodes: Vec<NodeData>,
    counter: usize,
}

impl Executable for Sequence {
    fn start(&mut self) {
        self.counter = 0;
    }

    fn execute(&mut self, dt: f32) -> States {
        loop {
            if self.counter >= self.nodes.len() {
                return States::Succes;
            }

            let node = &mut self.nodes[self.counter];

            if node.watch_state != WatchState::Running {
                node.node.start();
                node.watch_state = WatchState::Running;
            }

            let state = node.node.execute(dt);

            if state == States::Running {
                return States::Running;
            } else {
                node.node.end();
                if state == States::Succes {
                    node.watch_state = WatchState::Succeeded;
                } else {
                    node.watch_state = WatchState::Failed;
                }
            }

            if state == States::Succes {
                self.counter += 1;
            }

            if state == States::Fail {
                return States::Fail;
            }
        }
    }

    fn end(&mut self) {
        println!("aaaaaaaaaaaa");
        if self.counter < self.nodes.len() {
            let node = &mut self.nodes[self.counter];

            if node.watch_state == WatchState::Running {
                node.node.end();
                node.watch_state = WatchState::Cancelled;
            }
        }
    }
}

impl ExecutableWatch for Sequence {
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

impl Sequence {
    pub fn new(nodes: Vec<Box<dyn ExecutableAndWatch>>) -> Box<Self> {
        Box::new(Sequence {
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
