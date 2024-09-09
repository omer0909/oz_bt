use crate::exec::{self, States, WatchState};
use std::time::{Duration, Instant};

pub struct TreeManager {
    node: Box<dyn exec::ExecutableAndWatch>,
    last: Instant,
    loop_wait: f32,
    watch_state: WatchState,
}

impl TreeManager {
    pub fn new(node: Box<dyn exec::ExecutableAndWatch>, loop_rate: f32) -> Self {
        TreeManager {
            node: node,
            last: Instant::now(),
            loop_wait: 1.0 / loop_rate,
            watch_state: WatchState::None,
        }
    }

    pub fn sleep_loop(&mut self) -> f32 {
        {
            let now = Instant::now();
            let dt = Instant::now().duration_since(now).as_secs_f32();
            if dt < self.loop_wait {
                std::thread::sleep(Duration::from_secs_f32(self.loop_wait - dt));
            }
        }
        let now = Instant::now();
        let dt = now.duration_since(self.last).as_secs_f32();
        self.last = now;
        dt
    }

    pub fn execute(&mut self, dt: f32) -> States {
        if self.watch_state != WatchState::Running {
            self.node.start();
            self.watch_state = WatchState::Running;
        }

        let status = self.node.execute(dt);

        if status != States::Running {
            self.node.end();
            if status == States::Succes {
                self.watch_state = WatchState::Succeeded;
            } else {
                self.watch_state = WatchState::Failed;
            }
        }
        status
    }

    pub fn cancel(&mut self) {
        if self.watch_state == WatchState::Running {
            self.node.end();
            self.watch_state = WatchState::Cancelled;
        }
    }

    pub fn get_content(&self) -> exec::WatchContent {
        self.node.get_content()
    }
}
