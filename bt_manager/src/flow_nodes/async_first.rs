use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, States, WatchContent, WatchState,
};

struct NodeData {
    node: Box<dyn ExecutableAndWatch>,
    watch_state: WatchState,
}

pub struct AsyncFirst {
    nodes: Vec<Box<dyn ExecutableAndWatch>>,
}

impl Executable for AsyncFirst {
    fn start(&mut self) {
        for node in &mut self.nodes {
            node.start();
        }
    }

    fn execute(&mut self, dt: f32) -> States {
        for node in &mut self.nodes {
            let state = node.execute(dt);
            if state != States::Running {
                return state;
            }
        }
        States::Running
    }

    fn end(&mut self) {
        for node in &mut self.nodes {
            node.end();
        }
    }
}

impl ExecutableWatch for AsyncFirst {
    fn get_content(&self) -> WatchContent {
        let childs = self.nodes.iter().map(|x| x.get_content()).collect();

        WatchContent {
            name: "async_first".to_string(),
            watch_state: WatchState::None,
            childs: childs,
        }
    }
}

impl AsyncFirst {
    pub fn new(nodes: Vec<Box<dyn ExecutableAndWatch>>) -> Box<Self> {
        Box::new(AsyncFirst { nodes: nodes })
    }
}
