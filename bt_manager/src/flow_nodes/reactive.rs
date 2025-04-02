use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, States, WatchContent, WatchState,
};

struct NodeData<T> {
    node: Box<dyn ExecutableAndWatch<T>>,
    watch_state: WatchState,
}

pub struct Reactive<T> {
    requirements: Vec<NodeData<T>>,
    node: NodeData<T>,
}

impl<T> Executable<T> for Reactive<T> {
    fn execute(&mut self, data: &mut T) -> States {
        for node in &mut self.requirements {
            node.node.start(data);
            let state = node.node.execute(data);
            node.node.end(data);
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
            self.node.node.start(data);
        }
        self.node.node.execute(data)
    }

    fn end(&mut self, data: &mut T) {
        if self.node.watch_state == WatchState::Running {
            self.node.node.end(data);
            self.node.watch_state = WatchState::Cancelled;
        }
    }
}

impl<T> ExecutableWatch for Reactive<T> {
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
            name: "reactive".to_string(),
            watch_state: WatchState::None,
            childs: childs,
        }
    }
}

impl<T> Reactive<T> {
    pub fn new(
        requirements: Vec<Box<dyn ExecutableAndWatch<T>>>,
        node: Box<dyn ExecutableAndWatch<T>>,
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
