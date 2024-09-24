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

    pub trait Executable<T> {
        fn start(&mut self, _: &mut T) {}
        fn execute(&mut self, _: &mut T) -> States;
        fn end(&mut self, _: &mut T) {}
    }

    pub trait ExecutableWatch {
        fn get_content(&self) -> WatchContent;
    }

    pub trait ExecutableAndWatch<T>: Executable<T> + ExecutableWatch {}
    impl<T, C> ExecutableAndWatch<C> for T where T: Executable<C> + ExecutableWatch {}
}
