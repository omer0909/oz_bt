use crate::*;

struct MyData {
    dt: f32,
}

#[node]
mod sleep {
    type Data = super::MyData;

    pub struct Input {
        pub time: f32,
    }

    pub struct Output {}

    #[derive(Default)]
    pub struct Node {
        elapsed: f32,
    }

    impl CustomNode for Node {
        fn start(&mut self, _: &mut CustomData) {
            println!("started");
        }

        fn execute(&mut self, data: &mut CustomData) -> crate::exec::States {
            self.elapsed += data.data.dt;
            println!("{}", data.data.dt);

            if self.elapsed > data.input.time {
                return crate::exec::States::Succes;
            }
            crate::exec::States::Running
        }

        fn end(&mut self, _: &mut CustomData) {
            println!("ended");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_manager_macro::handle;
    use exec::States;
    use serde_json;
    use serial_test::serial;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    #[serial]
    fn tree() {
        let mut data = MyData { dt: 1.0 };
        handle!(input, 2.0, 5);
        println!("{}", input3.borrow());
        let mut tree_manager: TreeManager<MyData> = TreeManager::new(
            Sequence::new(vec![
                sleep::NodeManager::new(
                    |_| sleep::Input { time: 1.0 },
                    Rc::new(RefCell::new(sleep::Output {})),
                ),
                sleep::NodeManager::new(
                    move |_| sleep::Input {
                        time: *input2.borrow(),
                    },
                    Rc::new(RefCell::new(sleep::Output {})),
                ),
                EventNode::new("printer".to_string(), |data: &mut MyData| {
                    println!("yazdırıldı! {}", data.dt);
                    true
                }),
            ]),
            10.0,
        );

        loop {
            data.dt = tree_manager.sleep_loop();
            if tree_manager.execute(&mut data) != States::Running {
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
}
