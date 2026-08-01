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

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
    pub enum NodeTypes {
        Flow,
        Leaf,
        Decorator,
        Event(String),
        GroupIn(String),
        GroupOut(String),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct WatchContent {
        pub node_type: NodeTypes,
        pub name: String,
        pub watch_state: WatchState,
        pub childs: Vec<WatchContent>,
        pub comment: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct VisualizerMessage {
        pub start_time: chrono::DateTime<chrono::Utc>,
        pub send_time: chrono::DateTime<chrono::Utc>,
        pub watch_content: WatchContent,
    }

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
    pub enum States {
        Running,
        Success,
        Fail,
    }

    impl States {
        pub fn from_bool(success: bool) -> Self {
            if success {
                States::Success
            } else {
                States::Fail
            }
        }
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
