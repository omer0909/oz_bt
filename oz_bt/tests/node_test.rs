use oz_bt::*;

#[derive(Default)]
struct Sleep {
    elapsed: f32,
}

struct App {
    my_data: f32,
    dt: f32,
}

struct Asd {
    dt: f32,
}

#[node]
impl Node for Sleep {
    type Data = App;
    type Input = f32;
    type Output = f32;

    fn execute(&mut self, ctx: &mut Ctx<Self>) -> crate::exec::States {
        if self.elapsed >= *ctx.input {
            return crate::exec::States::Success;
        }

        self.elapsed += ctx.data.dt;
        *ctx.output = self.elapsed;

        crate::exec::States::Running
    }
}

#[cfg(test)]
mod tests {
    extern crate self as oz_bt;
    use oz_bt::exec::VisualizerMessage;

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

        let mut data = App {
            my_data: 0.0,
            dt: 1.0,
        };
        let my_input = handle(2.0);
        println!("{}", with!([my_input], my_input.get()));
        let mut tree_manager: TreeManager<App> = TreeManager::new(
            sequence![sequence![
                ::oz_bt::CustomNode::<Sleep>::new_i(|_| 5.0),
                async_first![sleep_i(|_| 2.0), sleep_i(|_| 1.0)],
                with!([my_input], sleep_io(move |_| 2.0, my_input)),
                invert(
                    fail(with!([my_input], sleep_i(move |_| my_input.get())))
                        .comment("comment asdasdasdasd")
                ),
                sleep_i(move |_| 2.0),
                group_in(
                    "test group",
                    fallback![
                        group_in(
                            "my group",
                            fallback![
                                event_node("asf", |_: &mut App| {
                                    println!("yazmadı");
                                    false
                                }),
                                group_out(
                                    "çıkacak",
                                    event_node("test", |data: &mut App| {
                                        println!("yazdırıldı! {}", data.dt);
                                        true
                                    })
                                ),
                                group_out(
                                    "çıkmış",
                                    event_node("geliş", |data: &mut App| {
                                        println!("yazdırıldı! {}", data.dt);
                                        true
                                    })
                                ),
                                group_out(
                                    "i",
                                    event_node("geliş", |data: &mut App| {
                                        println!("yazdırıldı! {}", data.dt);
                                        true
                                    })
                                ),
                                group_out(
                                    "test",
                                    event_node("geliş", |data: &mut App| {
                                        println!("yazdırıldı! {}", data.dt);
                                        true
                                    })
                                ),
                                group_out(
                                    "",
                                    event_node("geliş", |data: &mut App| {
                                        println!("yazdırıldı! {}", data.dt);
                                        true
                                    })
                                ),
                            ]
                        ),
                        event_node("oldu", |_: &mut App| {
                            println!("yazmadı");
                            false
                        }),
                        group_out(
                            "out",
                            event_node("fff", |data: &mut App| {
                                println!("yazdırıldı! {}", data.dt);
                                true
                            })
                        ),
                    ]
                ),
                event_node("<<<<zzz", |data: &mut App| {
                    println!("yazdırıldı! {}", data.dt);
                    true
                }),
                with!(
                    [my_input, my_test = 5.0],
                    sequence![
                        with!([my_test], sleep_i(move |_| my_test.get())),
                        with!([my_test], sleep_i(move |_| my_test.get())),
                        sleep_i(move |_| my_input.get()),
                        sleep_i(move |_| 2.0),
                        sleep_i(move |_| 2.0),
                        sleep_i(move |_| 2.0),
                        sleep_i(move |_| 2.0),
                    ]
                )
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

    #[test]
    #[serial]
    fn example() {
        use oz_bt::*;

        let context = zmq::Context::new();
        let publisher = context.socket(zmq::PUSH).unwrap();
        publisher.bind("tcp://*:5555").expect("Unable to connect");

        let mut app = App {
            my_data: 2.0,
            dt: 0.0,
        };

        // Create a behavior tree using convenience macros
        let elapsed = handle(0.0);
        let root = sequence![
            sleep_i(|app| app.my_data),
            success(fallback![
                event_node("example", |_| false),
                invert(sleep_i(|app| app.my_data)),
            ]),
            async_first![
                with!([elapsed], sleep_io(|_| 5.0, elapsed)),
                retry(with!(
                    [elapsed],
                    event_node("print", move |_| {
                        println!("elapsed: {}", elapsed.get());
                        false
                    })
                )),
                retry(fail(sleep_i(|_| 0.1)))
            ],
            with!(
                [data = 0.0],
                sequence![
                    event_node("check", |app: &mut App| { app.my_data > 1.0 }),
                    with!(
                        [data],
                        event_node("writer", move |_| {
                            data.set(5.0);
                            true
                        })
                    ),
                    group_in(
                        "exaple group",
                        with!([data], sleep_i(move |_| elapsed.get()))
                    )
                ]
            )
        ];

        // Manage execution at 10 Hz
        let mut manager: TreeManager<App> = TreeManager::new(root, 10.0);

        let start_time = chrono::Utc::now();

        // In your robot control loop:
        loop {
            let state = manager.execute(&mut app);
            if state != States::Running {
                break;
            }
            let message = VisualizerMessage {
                start_time: start_time,
                send_time: chrono::Utc::now(),
                watch_content: manager.get_content(),
            };

            let bytes =
                bincode_next::serde::encode_to_vec(&message, bincode_next::config::standard())
                    .expect("Bincode encode başarısız oldu!");
            _ = publisher.send(bytes.as_slice(), zmq::DONTWAIT);

            app.dt = manager.sleep_loop();
        }
    }
}
