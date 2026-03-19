# GitDeck

GitDeck is a production-minded, keyboard-driven Git cockpit for a **single local repository**. It combines repository awareness, commit exploration, command education, and safe interactive command execution in a polished Rust terminal UI.

## Features (MVP)

- Top repository status bar with branch/head/remotes/working-tree summary and in-progress operation indicators.
- Multi-pane TUI:
  - Commit history (scroll + filter)
  - Branches pane (local + remote, graph extension point)
  - Command center
  - Details/help pane (context-sensitive)
- Interactive command builder with:
  - Flag toggles
  - Target input
  - Live command preview
  - Validation warnings for incompatible options
  - Confirmation gates for dangerous paths
- Built-in command documentation for:
  - `status`, `add`, `restore`, `commit`, `switch`, `branch`, `fetch`, `pull`, `push`, `stash`, `diff`, `log`
- Real repository inspection via `git2` and command execution through the Git CLI.

## Architecture

```text
src/
  app/       App state, event loop, keyboard handling
  ui/        Layout and pane rendering
  git/       Repository discovery and domain queries
  commands/  Command metadata, builder model, execution
  models/    Shared domain and UI model types
  config/    Theme and keybinding text
  main.rs    Entrypoint
```

### Design notes

- `git::GitService` owns repository inspection logic.
- `commands::CommandSpec` is reusable metadata, so future commands (merge/rebase/reset/revert/cherry-pick/tag) can be added without UI rewrites.
- UI rendering is separated from input/state updates for maintainability.

## Keybindings

- `Tab` / `Shift-Tab`: cycle focused pane
- `j` / `k` (or arrows): move selection
- `Enter`: inspect commit / activate current selection
- `?`: help overlay
- `/`: type to filter commit history
- `q`: quit (or close overlay/builder)
- `c`: open command builder
- `g`: focus branches pane
- `h`: focus history pane
- `d`: focus details pane
- Builder mode:
  - `space`: toggle selected flag
  - `Tab`/`Shift-Tab`: cycle command
  - type text: fill target input
  - `x`: execute preview command (double press for dangerous paths)

## Running

```bash
cargo run
```

Run GitDeck from inside (or below) a Git repository directory.

## Roadmap

- True commit graph rendering in branch pane
- Search/filter for branches and commands
- Mini command prompt (`:`)
- Richer argument pickers (refs/files/remotes/stashes)
- Command coverage expansion: merge/rebase/reset/revert/cherry-pick/tag/reflog
- Configurable themes + user keymaps via TOML
- Async/background command execution and progress states

