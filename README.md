# oz_bt

<p align="center">
  <a href="https://github.com/omer0909/oz_bt/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/omer0909/oz_bt" alt="License">
  </a>
  <a href="https://github.com/omer0909/oz_bt/pulls">
    <img src="https://img.shields.io/badge/PRs-Welcome-brightgreen.svg" alt="PRs Welcome">
  </a>
</p>

> **A high-performance Behavior Tree framework designed for general robot management and autonomous agent control.**  
> Built in Rust with real-time monitoring, async execution, and modular node architecture.

## Overview

`oz_bt` is a Behavior Tree (BT) implementation tailored for **robot general management** and complex autonomous systems. It provides a robust, type-safe, and extensible framework for defining hierarchical task execution logic with first-class support for real-time visualization and monitoring.

The system is built around a core executor with composable flow nodes (Sequence, Fallback, etc.), custom leaf nodes via procedural macros, and a built-in telemetry system that streams tree state to external visualizers.

## Live Monitoring

The behavior tree execution can be observed in real-time using the companion visualizer:

🔗 **[oz_bt_visualizer](https://github.com/omer0909/oz_bt_visualizer)** — Live monitoring and debugging interface for `oz_bt` execution graphs.

## Features

- **Behavior Tree Core**: Full implementation of BT primitives with `Running | Success | Fail` state model
- **Robot-Oriented Design**: Optimized for general robot management, task planning, and autonomous decision loops
- **Flow Control Nodes**: Sequence, Fallback, AsyncFirst, AsyncWait, Invert, Reactive, Success, Fail, Retry
- **Custom Nodes**: Define leaf nodes using the `#[node]` procedural macro with typed inputs/outputs
- **Convenience Macros**: `sequence![]`, `fallback![]`, and `with!{}` macros for ergonomic tree construction
- **Event System**: Event-driven nodes for reactive behaviors
- **Grouping**: Sub-tree grouping with GroupIn / GroupOut for modular tree composition
- **Real-time Visualization**: Built-in `VisualizerMessage` protocol over ZeroMQ for live debugging
- **Tree Manager**: Managed execution loop with configurable tick rates and lifecycle hooks (start, execute, end)
- **Type Safety**: Leverages Rust's type system for compile-time correct node wiring
- **Serialization**: Full serde support with bincode for efficient state transmission

## Architecture

```
oz_bt_workspace/
├── oz_bt/          # Core behavior tree library
│   ├── executable  # Traits and state definitions
│   ├── tree_manger # Execution manager with tick loop
│   ├── flow_nodes  # Built-in composite & decorator nodes
│   ├── custom_node # User-defined leaf node framework
│   └── event_node  # Basic function call node
└── oz_bt_macro/    # Procedural macros for #[node]
```

## Quick Start

### Add to Cargo.toml

```toml
[dependencies]
oz_bt = { git = "https://github.com/omer0909/oz_bt" }
```

### Define a Custom Node

```rust
use oz_bt::*;

struct App {
    my_data: f32,
    dt: f32,
}

#[derive(Default)]
struct Sleep {
    elapsed: f32,
}

#[node(node_type = "$crate::Sleep")]
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
```

### Example Tree

All `new()` constructors return `Box<dyn ExecutableAndWatch<T>>` directly, so no manual boxing is needed:

```rust
let elapsed = handle(0.0);
let root = sequence![
    sleep_i(|app| app.my_data),
    success(fallback![
        event_node("example", |_| false),
        invert(sleep_i(|app| app.my_data)),
    ]),
    async_first![
        sleep_io(|_| 5.0, elapsed.clone()),
        retry(event_node("print", move |_| {
            println!("elapsed: {}", elapsed.get());
            false
        })),
        retry(fail(sleep_i(|_| 0.1)))
    ],
    handle!(
        [data = 0.0],
        sequence![
            event_node("check", |app: &mut App| { app.my_data > 1.0 }),
            event_node(
                "writer",
                clone!([data], move |_| {
                    data.set(5.0);
                    true
                })
            ),
            group_in("exaple group", sleep_i(clone!([data], move |_| data.get())))
        ]
    )
];
```

## Node Types

| Category | Nodes | Description |
|----------|-------|-------------|
| **Composites** | `Sequence`, `Fallback`, `AsyncFirst`, `AsyncWait` | Control flow branching |
| **Decorators** | `Invert`, `Reactive`, `Success`, `Fail`, `Retry` | Modify child behavior |
| **Custom** | `CustomNode<T>` | User-defined via `#[node]` macro |
| **Grouping** | `GroupIn`, `GroupOut` | Sub-tree encapsulation |
| **Event** | `EventNode` | Basic function call |

## Visualization Protocol

The library emits `VisualizerMessage` structures containing the full tree state, including:

- `start_time`: Execution timestamp (UTC)
- `send_time`: Transmission timestamp (UTC)  
- `watch_content`: Hierarchical node states (`Running`, `Succeeded`, `Failed`, `Cancelled`)

Connect the `TreeManager` to [oz_bt_visualizer](https://github.com/omer0909/oz_bt_visualizer) via ZeroMQ for real-time graph inspection.

## Requirements

- Rust 2021 edition or newer
- ZeroMQ (for visualizer integration)
