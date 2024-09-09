pub mod exec {
    use serde::{Deserialize, Serialize};

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
    pub enum WatchState {
        Running,
        Succeeded,
        Failed,
        Cancelled,
        None,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct WatchContent {
        pub name: String,
        pub watch_state: WatchState,
        pub childs: Vec<WatchContent>,
    }

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
    pub enum States {
        Running,
        Succes,
        Fail,
    }

    pub trait Executable {
        fn start(&mut self) {}
        fn execute(&mut self, dt: f32) -> States;
        fn end(&mut self) {}
    }

    pub trait ExecutableWatch {
        fn get_content(&self) -> WatchContent;
    }

    pub trait ExecutableAndWatch: Executable + ExecutableWatch {}
    impl<T: Executable + ExecutableWatch> ExecutableAndWatch for T {}
}
