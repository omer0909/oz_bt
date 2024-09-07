use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, States, WatchContent, WatchState,
};

pub struct Fallback {
    nodes: Vec<Box<dyn ExecutableAndWatch>>,
    counter: usize,
    working: bool,
}

impl Executable for Fallback {
    fn start(&mut self) {
        self.counter = 0;
    }

    fn execute(&mut self, dt: f32) -> States {
        loop {
            if self.counter >= self.nodes.len() {
                return States::Fail;
            }

            let node = &mut self.nodes[self.counter];

            if !self.working {
                self.working = true;
                node.start();
            }

            let state = node.execute(dt);

            if state == States::Running {
                return States::Running;
            } else {
                node.end();
                self.working = false;
            }

            if state == States::Fail {
                self.counter += 1;
            }

            if state == States::Succes {
                return States::Succes;
            }
        }
    }

    fn end(&mut self) {
        if self.working {
            self.working = false;
            self.nodes[self.counter].end();
        }
    }
}

impl ExecutableWatch for Fallback {
    fn get_content(&self) -> WatchContent {
        let childs = self.nodes.iter().map(|x| x.get_content()).collect();

        WatchContent {
            name: "fallback".to_string(),
            watch_state: WatchState::None,
            childs: childs,
        }
    }
}

impl Fallback {
    pub fn new(nodes: Vec<Box<dyn ExecutableAndWatch>>) -> Box<Self> {
        Box::new(Fallback {
            nodes: nodes,
            counter: 0,
            working: false,
        })
    }
}
