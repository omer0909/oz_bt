use crate::exec;
use std::time::{Duration, Instant};

pub struct TreeManager {
    node: Box<dyn exec::ExecutableAndWatch>,
    last: Instant,
    loop_wait: f32,
    before_events: Vec<Box<dyn Fn()>>,
    after_events: Vec<Box<dyn Fn()>>,
}

impl TreeManager {
    pub fn new(node: Box<dyn exec::ExecutableAndWatch>, loop_rate: f32) -> Self {
        TreeManager {
            node: node,
            last: Instant::now(),
            loop_wait: 1.0 / loop_rate,
            before_events: Vec::new(),
            after_events: Vec::new(),
        }
    }

    pub fn work(&mut self) -> exec::States {
        self.node.start();
        loop {
            let start = Instant::now();

            let dt = start.duration_since(self.last).as_secs_f32();

            for event in &mut self.before_events {
                event();
            }

            let status = self.node.execute(dt);

            for event in &mut self.after_events {
                event();
            }

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

    pub fn define_before_event(&mut self, event: impl Fn() + 'static) {
        self.before_events.push(Box::new(event));
    }

    pub fn define_after_event(&mut self, event: impl Fn() + 'static) {
        self.after_events.push(Box::new(event));
    }
}
