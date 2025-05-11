# Rust TUI Archetype

![Latest Release](https://img.shields.io/github/v/release/aisamji/rust-tui.archetype?style=flat-square&label=Latest%20Release&color=blue)

An [Archetect](https://archetect.github.io/) archetype for building a TUI written in Rust. Generates a TUI that leverages [tokio](https://docs.rs/tokio/latest/tokio/) for asynchronous programming, [crossterm](https://docs.rs/crossterm/latest/crossterm/) as the terminal backend, and [ratatui](https://docs.rs/ratatui/latest/ratatui/) for high-level rendering of widgets to the terminal.

## Rendering

To generate content from this Archetype, copy and execute the following command:

```sh
  archetect render git@github.com:aisamji/rust-tui.archetype.git#v1
```

Then follow the instructions outputted by Archetect.

Upon initial rendering of the archetype, you will be given some simple "Hello World" components. The following components can be safely removed or modified as needed.

- `sleep_task` - A dummy `async` function that sleeps for 5 seconds before completing. Used to demonstrate background tasks.
- `TaskSpec::SleepTask` - A dummy taskspec used to notify the [tokio::Runtime](https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html) to launch `sleep_task`. Used to demonstrate background tasks.
- `TuiEvent::ModifyCount` - A dummy event that notifies the main thread to increase or decrease the number of active background tasks.
- `App.render` - A synchronous function that renders widgets on the given [ratatui::Frame](https://docs.rs/ratatui/latest/ratatui/struct.Frame.html). Can be tweaked as needed.
- `App.process_terminal_event` - A synchronous function that processes a given [crossterm::Event](https://docs.rs/crossterm/latest/crossterm/event/enum.Event.html). Can be tweaked as needed.
