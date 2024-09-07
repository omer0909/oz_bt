use crate::exec;
use std::time::{Duration, Instant};

pub struct TreeManager {
    node: Box<dyn exec::ExecutableAndWatch>,
    last: Instant,
    loop_wait: f32,
}

impl TreeManager {
    pub fn new(node: Box<dyn exec::ExecutableAndWatch>, loop_rate: f32) -> Self {
        TreeManager {
            node: node,
            last: Instant::now(),
            loop_wait: 1.0 / loop_rate,
        }
    }

    pub fn work(&mut self) -> exec::States {
        self.node.start();
        loop {
            let start = Instant::now();

            let dt = start.duration_since(self.last).as_secs_f32();

            let status = self.node.execute(dt);
            if status != exec::States::Running {
                self.node.end();
                return status;
            }
            let working_time = Instant::now().duration_since(start).as_secs_f32();
            if working_time < self.loop_wait {
                std::thread::sleep(Duration::from_secs_f32(self.loop_wait - working_time));
            }
            self.last = start;
        }
    }

    pub fn get_content(&self) -> exec::WatchContent {
        self.node.get_content()
    }
}
