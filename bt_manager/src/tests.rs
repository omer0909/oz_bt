use crate::*;

#[node]
mod sleep {
    use crate::exec::{Executable, States};

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
    use exec::States;
    use serde_json;
    use serial_test::serial;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    #[serial]
    fn tree() {
        let input = Rc::new(RefCell::new(2.0));
        let mut tree_manager: TreeManager = TreeManager::new(
            Sequence::new(vec![
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

        loop {
            let dt = tree_manager.sleep_loop();
            if tree_manager.execute(dt) != States::Running {
                break;
            }
            println!(
                "tree: {}",
                serde_json::to_string_pretty(&tree_manager.get_content()).unwrap()
            )
        }
        println!(
            "tree: {}",
            serde_json::to_string_pretty(&tree_manager.get_content()).unwrap()
        )
    }

    // #[test]
    // #[serial]
    // fn reactive_node() {
    //     let mut tree_manager = TreeManager::new(
    //         Reactive::new(
    //             vec![EventNode::new("print".to_string(), || {
    //                 println!("kontrol edildi!");
    //                 true
    //             })],
    //             sleep::lib::NodeManager::new(
    //                 sleep::lib::InputsHandles {
    //                     time: Box::new(|| 1.0),
    //                 },
    //                 sleep::lib::OutputsHandles {},
    //             ),
    //         ),
    //         10.0,
    //     );

    //     tree_manager.define_after_event(|_: &mut TreeManager| println!("after event"));

    //     println!("Result: {:?}", tree_manager.work());
    // }
}
