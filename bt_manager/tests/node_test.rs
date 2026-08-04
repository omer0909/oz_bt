use bt_manager::*;

struct MyData {
    dt: f32,
}

#[derive(Default)]
struct Sleep {
    elapsed: f32,
}

#[node(node_type = "$crate::Sleep")]
impl Node for Sleep {
    type Data = MyData;
    type Input = f32;
    type Output = f32;

    fn execute(&mut self, ctx: &mut Ctx<Self>) -> crate::exec::States {
        self.elapsed += ctx.data.dt;

        println!("elapsed: {}", self.elapsed);

        *ctx.output = self.elapsed;

        if self.elapsed < *ctx.input {
            return crate::exec::States::Running;
        }
        crate::exec::States::Success
    }
}

#[cfg(test)]
mod tests {
    extern crate self as bt_manager;
    use bt_manager::exec::VisualizerMessage;

    use super::*;
    use exec::States;
    use serial_test::serial;
    use std::thread;

    #[test]
    #[serial]
    fn tree() {
        let context = zmq::Context::new();
        let publisher = context.socket(zmq::PUSH).unwrap();
        publisher.bind("tcp://*:5555").expect("Yayıncı bağlanamadı");

        let mut data = MyData { dt: 1.0 };
        let my_input = handle(2.0);
        println!("{}", with!([my_input], my_input.get()));
        let mut tree_manager: TreeManager<MyData> = TreeManager::new(
            sequence![sequence![
                ::bt_manager::CustomNode::<Sleep>::new_i(|_| 5.0),
                async_first![sleep_i!(|_| 2.0), sleep_i!(|_| 1.0)],
                with!([my_input], sleep_io!(move |_| 2.0, my_input)),
                invert!(fail!(with!([my_input], sleep_i!(move |_| my_input.get())))
                    .comment("comment asdasdasdasd")),
                sleep_i!(move |_| 2.0),
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
                    sleep_i!(move |_| 2.0),
                    sleep_i!(move |_| 2.0),
                    sleep_i!(move |_| 2.0),
                    sleep_i!(move |_| 2.0),
                    sleep_i!(move |_| 2.0),
                    sleep_i!(move |_| 2.0),
                    sleep_i!(move |_| 2.0),
                ],
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
