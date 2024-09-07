use crate::exec::{
    Executable, ExecutableAndWatch, ExecutableWatch, States, WatchContent, WatchState,
};

pub struct Reactive {
    requirements: Vec<Box<dyn ExecutableAndWatch>>,
    node: Box<dyn ExecutableAndWatch>,
    working: bool,
}

impl Executable for Reactive {
    fn execute(&mut self, dt: f32) -> States {
        for node in &mut self.requirements {
            node.start();
            let state = node.execute(dt);
            node.end();
            if state != States::Succes {
                return States::Fail;
            }
        }

        if !self.working {
            self.working = true;
            self.node.start();
        }
        self.node.execute(dt)
    }

    fn end(&mut self) {
        if self.working {
            self.node.end();
        }
    }
}

impl ExecutableWatch for Reactive {
    fn get_content(&self) -> WatchContent {
        let mut childs: Vec<WatchContent> =
            self.requirements.iter().map(|x| x.get_content()).collect();
        childs.push(self.node.get_content());

        WatchContent {
            name: "reactive".to_string(),
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
            requirements: requirements,
            node: node,
            working: false,
        })
    }
}
