use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, States, WatchContent, WatchState,
};

pub struct AsyncWait {
    nodes: Vec<Box<dyn ExecutableAndWatch>>,
    running: Vec<bool>,
}

impl Executable for AsyncWait {
    fn start(&mut self) {
        for node in &mut self.nodes {
            node.start();
        }
        self.running.fill(true);
    }

    fn execute(&mut self, dt: f32) -> States {
        for (node, running) in self.nodes.iter_mut().zip(self.running.iter_mut()) {
            if *running {
                let state = node.execute(dt);
                if state != States::Running {
                    *running = false;
                    node.end();
                    if state == States::Fail {
                        return States::Fail;
                    }
                }
            }
        }

        if self.running.iter().all(|&x| !x) {
            return States::Succes;
        }

        States::Running
    }

    fn end(&mut self) {
        for (node, running) in self.nodes.iter_mut().zip(self.running.iter_mut()) {
            if *running {
                node.end();
            }
        }
    }
}

impl ExecutableWatch for AsyncWait {
    fn get_content(&self) -> WatchContent {
        let childs = self.nodes.iter().map(|x| x.get_content()).collect();

        WatchContent {
            name: "async_wait".to_string(),
            watch_state: WatchState::None,
            childs: childs,
        }
    }
}

impl AsyncWait {
    pub fn new(nodes: Vec<Box<dyn ExecutableAndWatch>>) -> Box<Self> {
        let size = nodes.len();
        Box::new(AsyncWait {
            nodes: nodes,
            running: vec![false; size],
        })
    }
}
