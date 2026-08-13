# oz_bt

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

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
- **Flow Control Nodes**: Sequence, Fallback, AsyncFirst, AsyncWait, Invert, Reactive, Success, Fail
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
│   └── event_node  # Event-driven reactive nodes
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
use oz_bt::{node, Ctx, States, Node};

#[node]
struct MoveToGoal {
    Input: (f32, f32),
    Output: bool,
}

impl Node for MoveToGoal {
    fn execute(&mut self, ctx: &mut Ctx<Self>) -> States {
        let (x, y) = ctx.input;
        // Robot navigation logic here
        ctx.output = true;
        States::Success
    }
}
```

### Build and Run a Tree

All `new()` constructors return `Box<dyn ExecutableAndWatch<T>>` directly, so no manual boxing is needed:

```rust
use oz_bt::{TreeManager, Sequence, Fallback, CustomNode, handle};

fn main() {
    // Create a behavior tree using convenience macros
    let root = sequence![
        move_to_goal_i!(|data| (data.target_x, data.target_y)),
        fallback![
            check_obstacle::new(),
            invert!(emergency_stop::new()),
        ],
        async_first![
            wait_for_signal::new(),
            timeout_after_5s::new(),
        ],
    ];

    // Manage execution at 30 Hz
    let mut manager = TreeManager::new(root, 30.0);

    // In your robot control loop:
    loop {
        let status = manager.execute(&mut robot_data);
        let dt = manager.sleep_loop();
        // dt contains the actual elapsed time since last tick
    }
}
```

### Shared State with `with!` Macro

Easily share handles between nodes using the `with!` macro:

```rust
use oz_bt::with;

with! {
    shared_flag = false,
    shared_counter = 0,
    {
        let root = sequence![
            set_flag::new_o(shared_flag),
            read_flag::new_i(|_| *shared_flag.get()),
        ];
    }
}
```

## Node Types

| Category | Nodes | Description |
|----------|-------|-------------|
| **Composites** | `Sequence`, `Fallback`, `AsyncFirst`, `AsyncWait` | Control flow branching |
| **Decorators** | `Invert`, `Reactive` | Modify child behavior |
| **Leaf** | `Success`, `Fail` | Terminal states |
| **Custom** | `CustomNode<T>` | User-defined via `#[node]` macro |
| **Grouping** | `GroupIn`, `GroupOut` | Sub-tree encapsulation |
| **Events** | `EventNode` | Reactive event handling |

## Visualization Protocol

The library emits `VisualizerMessage` structures containing the full tree state, including:

- `start_time`: Execution timestamp (UTC)
- `send_time`: Transmission timestamp (UTC)  
- `watch_content`: Hierarchical node states (`Running`, `Succeeded`, `Failed`, `Cancelled`)

Connect the `TreeManager` to [oz_bt_visualizer](https://github.com/omer0909/oz_bt_visualizer) via ZeroMQ for real-time graph inspection.

## Requirements

- Rust 2021 edition or newer
- ZeroMQ (for visualizer integration)

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
