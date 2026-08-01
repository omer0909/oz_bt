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

    #[derive(Default)]
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
                return crate::exec::States::Success;
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
    use crate::exec::VisualizerMessage;

    use super::*;
    use bt_manager_macro::handle;
    use exec::States;
    use serial_test::serial;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::thread;

    #[test]
    #[serial]
    fn tree() {
        let context = zmq::Context::new();
        let publisher = context.socket(zmq::PUSH).unwrap();
        publisher.bind("tcp://*:5555").expect("Yayıncı bağlanamadı");

        let mut data = MyData { dt: 1.0 };
        handle!(input, 2.0, 5);
        // println!("{}", input3.borrow());
        let mut tree_manager: TreeManager<MyData> = TreeManager::new(
            sequence![sequence![
                sleep!(|_| sleep::Input { time: 1.0 }),
                async_first![
                    sleep!(|_| sleep::Input { time: 2.0 }),
                    sleep!(|_| sleep::Input { time: 1.0 })
                ],
                invert!(fail!(sleep!(move |_| sleep::Input {
                    time: *input2.borrow(),
                }))
                .comment("comment asdasdasdasd")),
                sleep!(move |_| sleep::Input { time: 2.0 }),
                group_in!(
                    "test group",
                    fallback![
                        group_in!(
                            "my group",
                            fallback![
                                event_node!("asf", |_: &mut MyData| {
                                    println!("yazmadı");
                                    false
                                }),
                                group_out!(
                                    "çıkacak",
                                    event_node!("test", |data: &mut MyData| {
                                        println!("yazdırıldı! {}", data.dt);
                                        true
                                    })
                                ),
                                group_out!(
                                    "çıkmış",
                                    event_node!("geliş", |data: &mut MyData| {
                                        println!("yazdırıldı! {}", data.dt);
                                        true
                                    })
                                ),
                                group_out!(
                                    "i",
                                    event_node!("geliş", |data: &mut MyData| {
                                        println!("yazdırıldı! {}", data.dt);
                                        true
                                    })
                                ),
                                group_out!(
                                    "test",
                                    event_node!("geliş", |data: &mut MyData| {
                                        println!("yazdırıldı! {}", data.dt);
                                        true
                                    })
                                ),
                                group_out!(
                                    "",
                                    event_node!("geliş", |data: &mut MyData| {
                                        println!("yazdırıldı! {}", data.dt);
                                        true
                                    })
                                ),
                            ]
                        ),
                        event_node!("oldu", |_: &mut MyData| {
                            println!("yazmadı");
                            false
                        }),
                        group_out!(
                            "out",
                            event_node!("fff", |data: &mut MyData| {
                                println!("yazdırıldı! {}", data.dt);
                                true
                            })
                        ),
                    ]
                ),
                event_node!("<<<<zzz", |data: &mut MyData| {
                    println!("yazdırıldı! {}", data.dt);
                    true
                }),
                sequence![
                    sleep!(move |_| sleep::Input { time: 2.0 }),
                    sleep!(move |_| sleep::Input { time: 2.0 }),
                    sleep!(move |_| sleep::Input { time: 2.0 }),
                    sleep!(move |_| sleep::Input { time: 2.0 }),
                    sleep!(move |_| sleep::Input { time: 2.0 }),
                    sleep!(move |_| sleep::Input { time: 2.0 }),
                    sleep!(move |_| sleep::Input { time: 2.0 }),
                ],
                sleep!(move |_| sleep::Input { time: 2.0 })
                    .with_output(Rc::new(RefCell::new(sleep::Output {}))),
            ]],
            10.0,
        );

        let start_time = chrono::Utc::now();

        loop {
            data.dt = tree_manager.sleep_loop();
            let state = tree_manager.execute(&mut data);

            let message = VisualizerMessage {
                start_time: start_time,
                send_time: chrono::Utc::now(),
                watch_content: tree_manager.get_content(),
            };

            let bytes =
                bincode_next::serde::encode_to_vec(&message, bincode_next::config::standard())
                    .expect("Bincode encode başarısız oldu!");
            _ = publisher.send(bytes.as_slice(), zmq::DONTWAIT);
            if state != States::Running {
                break;
            }
        }
        thread::sleep(std::time::Duration::from_millis(500));
    }
}
