use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, States, WatchContent, WatchState,
};

struct NodeData {
    node: Box<dyn ExecutableAndWatch>,
    watch_state: WatchState,
}

pub struct Reactive {
    requirements: Vec<NodeData>,
    node: NodeData,
}

impl Executable for Reactive {
    fn execute(&mut self, dt: f32) -> States {
        for node in &mut self.requirements {
            node.node.start();
            let state = node.node.execute(dt);
            node.node.end();
            if state != States::Succes {
                if state == States::Fail {
                    node.watch_state = WatchState::Failed;
                } else {
                    node.watch_state = WatchState::Cancelled;
                }
                return States::Fail;
            }
            node.watch_state = WatchState::Succeeded;
        }

        if self.node.watch_state != WatchState::Running {
            self.node.watch_state = WatchState::Running;
            self.node.node.start();
        }
        self.node.node.execute(dt)
    }

    fn end(&mut self) {
        if self.node.watch_state == WatchState::Running {
            self.node.node.end();
            self.node.watch_state = WatchState::Cancelled;
        }
    }
}

impl ExecutableWatch for Reactive {
    fn get_content(&self) -> WatchContent {
        let mut childs: Vec<WatchContent> = self
            .requirements
            .iter()
            .map(|x| WatchContent {
                watch_state: x.watch_state,
                ..x.node.get_content()
            })
            .collect();

        childs.push(WatchContent {
            watch_state: self.node.watch_state,
            ..self.node.node.get_content()
        });

        WatchContent {
            name: "async_wait".to_string(),
            watch_state: WatchState::None,
            childs: childs,
        }
    }
}

impl Reactive {
    pub fn new(
        requirements: Vec<Box<dyn ExecutableAndWatch>>,
        node: Box<dyn ExecutableAndWatch>,
    ) -> Box<Self> {
        Box::new(Reactive {
            requirements: requirements
                .into_iter()
                .map(|node| NodeData {
                    node: node,
                    watch_state: WatchState::None,
                })
                .collect(),
            node: NodeData {
                node: node,
                watch_state: WatchState::None,
            },
        })
    }
}
