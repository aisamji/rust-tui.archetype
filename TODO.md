## Phase 1: Template the CICD
- [ ] Set up xtasks
  - [ ] Configure `clap` to run when `cargo xtask` is called.
  - [ ] Create an xtask to get the versions of all packages in the workspace
  - [ ] Add an argument to the above command to get the version of a single package or multiple packages
  - [ ] Add an argument to the above command to perform a version bump of a given level on the specified targets.
  - [ ] Create an xtask to the `git tag` the HEAD with the current version.
  - [ ] Add `git push` to the `git tag` xtask.
- [ ] Create Github Actions workflows
  - [ ] Create a workflow that runs on tag push — as long as the tag matches semver, including any optional alpha/beta information. The workflow should run `cargo check` and `cargo test`. Assuming those succeed, it should run `cargo build --release`. This sequence of steps should be run on multiple platforms. Only if they all succeed, the binaries should all be pushed to a new Github release.

## Phase 2: Simplify Background Task Creation
- [ ] Add basic `TuiEvent`s for updates from background tasks.
  - [ ] `ShowProgress(String)`. This variant is used to tell the TUI to show a message to the user. For the basic rendering of the archetype, we will just print this to the screen between the keymap and "Hello World" line.
  - [ ] `ShowError(String)`. This wariant is used to tell the TUI to show an error message to the user. For the basic rendering of the archetype, we will just print this to the screen between the keyamp and "Hello World" line, but in red font.
- [ ] Create `{{ project-name }}-macros` crate. This will be a proc-macro crate that will be dependency of the main crate.
  - [ ] Add a `background::task` attribute that is applied to a function to turn it into an callback for a background taskspec. Essentially, the function will be written to accept an additional parameter before the existing ones — `tx: Sender<TuiEvent>` — marked as `async` and simply execute the body of the function it is applied to.
      - [ ] Add a `macro_rules!` to the generated `background::task`: `send_event!`. This will accept a `TuiEvent` and simply send it using `tx`, it will then discard the result.
      - [ ] Add a `macro_rules!` to the generated `background::task`: `send_error_message!`. This accepts the same arguments as `format!` and send a formatted message on `tx` as a `TuiEvent::ShowError(String)`. It does nothing on success, but on error it simply passes the same message to `eprintln!`.
      - [ ] Make `TuiEvent` user-defined. In other words, `tx` will be of type `Sender<UserDefinedEnum>`.
  - [ ] Add a `background::taskspec` derive that is applied to an enum. The derive will generate an `impl` block with a single function: `spawn_task(self, tx: Sender<TuiEvent>)`. This function will match on `self` and simply `tokio::task:spawn` the appropriate function. The function to spawn (i.e. the callback) will be determined by snake_casing the variant names. The arguments should be passed to the callback in the same order that they appear in the variant.
      - [ ] Add an attribute to `background::taskspec` called `callback`. `callback` should be the name of a function in scope and it will override the default snake_cased callback.
