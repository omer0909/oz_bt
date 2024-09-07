use once_cell::sync::OnceCell;
use std::sync::RwLock;

use crate::*;

#[derive(Default)]
struct App {
    pub test: i32,
}

static APP_INSTANCE: OnceCell<RwLock<App>> = OnceCell::new();

pub fn initialize_app() {
    let _ = APP_INSTANCE.set(RwLock::new(App { test: 0 }));
}

pub fn AppW() -> std::sync::RwLockWriteGuard<'static, App> {
    APP_INSTANCE
        .get()
        .expect("App not initialized")
        .write()
        .unwrap()
}

pub fn AppR() -> std::sync::RwLockReadGuard<'static, App> {
    APP_INSTANCE
        .get()
        .expect("App not initialized")
        .read()
        .unwrap()
}

#[node]
mod sleep {
    use crate::exec::{Executable, States, WatchContent};

    struct Input {
        time: f32,
    }

    #[derive(Default)]
    pub struct Node {
        elapsed: f32,
    }

    impl Executable for Node {
        fn start(&mut self) {
            println!("started");
        }

        fn execute(&mut self, dt: f32) -> crate::exec::States {
            self.elapsed += dt;
            println!("{}", dt);

            if self.elapsed > self.get_time() {
                return States::Succes;
            }
            States::Running
        }

        fn end(&mut self) {
            println!("ended");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flow_nodes::async_wait::AsyncWait;
    use serde_json;
    use serial_test::serial;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    #[serial]
    fn global() {
        initialize_app();

        {
            let mut app = AppW();
            app.test = 42;
        }

        {
            let app = AppR();
            println!("{}", app.test);
        }
    }

    #[test]
    #[serial]
    fn tree() {
        let input = Rc::new(RefCell::new(2.0));
        let mut tree_manager = TreeManager::new(
            AsyncWait::new(vec![
                sleep::lib::NodeManager::new(
                    sleep::lib::InputsHandles {
                        time: Box::new(move || *input.borrow_mut()),
                    },
                    sleep::lib::OutputsHandles {},
                ),
                sleep::lib::NodeManager::new(
                    sleep::lib::InputsHandles {
                        time: Box::new(|| 1.0),
                    },
                    sleep::lib::OutputsHandles {},
                ),
                EventNode::new("printer".to_string(), || {
                    println!("yazdırıldı!");
                    true
                }),
            ]),
            10.0,
        );

        let debug = serde_json::to_string_pretty(&tree_manager.get_content()).unwrap();
        println!("{:}", debug);

        println!("Result: {:?}", tree_manager.work());
    }

    #[test]
    #[serial]
    fn reactive_node() {
        let mut tree_manager = TreeManager::new(
            Reactive::new(
                vec![EventNode::new("print".to_string(), || {
                    println!("kontrol edildi!");
                    true
                })],
                sleep::lib::NodeManager::new(
                    sleep::lib::InputsHandles {
                        time: Box::new(|| 1.0),
                    },
                    sleep::lib::OutputsHandles {},
                ),
            ),
            10.0,
        );

        println!("Result: {:?}", tree_manager.work());
    }
}
