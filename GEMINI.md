# GitDeck Project Context

GitDeck is a production-minded, keyboard-driven Git cockpit for a single local repository. It combines repository awareness, commit exploration, command education, and safe interactive command execution in a polished Rust terminal UI.

## Project Overview

- **Main Technologies:** Rust, [Ratatui](https://ratatui.rs/), [Crossterm](https://github.com/crossterm-rs/crossterm), [git2-rs](https://github.com/rust-lang/git2-rs).
- **Architecture:**
    - `src/main.rs`: Entry point that initializes and runs the application.
    - `src/app/`: Contains the core application state (`App` struct), event loop, and keyboard input handling.
    - `src/git/`: High-level `GitService` that abstracts `git2` calls for repository status, history, branches, and stashes.
    - `src/commands/`: Metadata-driven command builder. Includes a `catalog` of supported git commands, an `executor` for running them via the Git CLI, and models for command specifications and builder state.
    - `src/ui/`: UI rendering logic using Ratatui. `layout.rs` defines the multi-pane TUI structure.
    - `src/models/`: Shared domain models (e.g., `CommitSummary`, `BranchInfo`) and UI-specific state types.
    - `src/config/`: Configuration for themes and keybindings.

## Building and Running

- **Build:** `cargo build`
- **Run:** `cargo run` (must be executed within or below a Git repository)
- **Test:** `cargo test`
- **Lint:** `cargo clippy`
- **Format:** `cargo fmt`

## Development Conventions

- **State Management:** The `App` struct in `src/app/app.rs` is the "source of truth". UI components should primarily be stateless and receive data from `App` or its sub-models.
- **Git Operations:** All direct interactions with the git repository should go through `GitService` in `src/git/`. Avoid using `git2` directly in UI or App logic.
- **Commands:** Adding new Git commands involves updating `src/commands/catalog.rs`. The `CommandSpec` allows defining flags, target inputs, and documentation without modifying the UI.
- **Error Handling:** Use `anyhow` for application-level error handling and `thiserror` for library-like error definitions (if applicable).
- **UI:** Layouts are managed in `src/ui/layout.rs`. Focus management is handled via the `FocusPane` enum in `src/models/ui.rs`.
- **Safety:** Dangerous commands (e.g., those with a `Dangerous` level or "force" flags) require a double confirmation in the Command Builder.

## Key Files

- `src/app/app.rs`: The heart of the application; manages the main loop and input processing.
- `src/git/service.rs`: Bridges the gap between the Rust app and the underlying Git repository.
- `src/commands/catalog.rs`: The definitive list of supported Git commands and their options.
- `src/ui/layout.rs`: Defines how the TUI looks and how panes are arranged.
