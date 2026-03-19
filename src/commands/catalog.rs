use super::{CommandDoc, CommandOption, CommandSpec, DangerLevel};

pub fn command_catalog() -> Vec<CommandSpec> {
    vec![
        cmd_status(),
        cmd_add(),
        cmd_restore(),
        cmd_commit(),
        cmd_switch(),
        cmd_branch(),
        cmd_fetch(),
        cmd_pull(),
        cmd_push(),
        cmd_stash(),
        cmd_diff(),
        cmd_log(),
    ]
}

fn cmd_status() -> CommandSpec {
    CommandSpec {
        id: "status",
        category: "Working Tree",
        base: "status",
        target_label: Some("pathspec"),
        docs: CommandDoc {
            description: "Show working tree and index state.",
            when_to_use: "Before committing or when orienting inside a repo.",
            examples: vec!["git status", "git status -sb"],
            related: vec!["add", "restore", "diff"],
            danger_level: DangerLevel::Safe,
            danger_note: None,
        },
        toggles: vec![
            opt("short", "Short", "-s", "Use short output format"),
            opt("branch", "Branch", "-b", "Show branch and tracking info"),
            opt("ignored", "Ignored", "--ignored", "Also show ignored files"),
        ],
    }
}

fn cmd_add() -> CommandSpec {
    mk(
        "add",
        "Working Tree",
        "add",
        Some("pathspec"),
        "Stage content for next commit.",
        "When you are preparing files to commit.",
        vec!["git add .", "git add -p src/main.rs"],
        vec!["status", "commit", "restore"],
        DangerLevel::Safe,
        None,
        vec![
            opt(
                "all",
                "All",
                "-A",
                "Stage all tracked and untracked changes",
            ),
            opt("patch", "Patch", "-p", "Interactively stage hunks"),
            opt(
                "update",
                "Update",
                "-u",
                "Stage only modified/deleted tracked files",
            ),
        ],
    )
}
fn cmd_restore() -> CommandSpec {
    mk(
        "restore",
        "Working Tree",
        "restore",
        Some("pathspec"),
        "Restore working tree files.",
        "Discard unstaged edits or unstage files.",
        vec!["git restore src/lib.rs", "git restore --staged Cargo.toml"],
        vec!["status", "add", "checkout"],
        DangerLevel::Caution,
        Some("Can discard local changes."),
        vec![
            opt("staged", "Staged", "--staged", "Restore index (unstage)"),
            opt(
                "worktree",
                "Worktree",
                "--worktree",
                "Restore working tree paths",
            ),
            opt(
                "source_head",
                "From HEAD",
                "--source=HEAD",
                "Restore from HEAD revision",
            ),
        ],
    )
}
fn cmd_commit() -> CommandSpec {
    mk(
        "commit",
        "History",
        "commit",
        None,
        "Record staged changes as a new commit.",
        "After staging a coherent change.",
        vec![
            "git commit -m \"feat: add pane\"",
            "git commit --amend --no-edit",
        ],
        vec!["add", "status", "log"],
        DangerLevel::Caution,
        Some("--amend rewrites commit history."),
        vec![
            opt(
                "message",
                "Message placeholder",
                "-m \"<message>\"",
                "Add inline message (edit preview manually)",
            ),
            opt(
                "all",
                "All tracked",
                "-a",
                "Stage tracked files before commit",
            ),
            opt("amend", "Amend", "--amend", "Rewrite last commit"),
        ],
    )
}
fn cmd_switch() -> CommandSpec {
    mk(
        "switch",
        "Branches",
        "switch",
        Some("branch/ref"),
        "Switch branches.",
        "Move between branches or detach HEAD.",
        vec!["git switch main", "git switch -c feature/x"],
        vec!["branch", "checkout", "merge"],
        DangerLevel::Caution,
        Some("Uncommitted changes may block switching."),
        vec![
            opt("create", "Create", "-c", "Create and switch to new branch"),
            opt(
                "detach",
                "Detach",
                "--detach",
                "Switch to detached HEAD at given commit",
            ),
        ],
    )
}
fn cmd_branch() -> CommandSpec {
    mk(
        "branch",
        "Branches",
        "branch",
        Some("branch"),
        "Manage branches.",
        "Create/list/delete local branches.",
        vec!["git branch", "git branch feature/a", "git branch -d old"],
        vec!["switch", "merge", "push"],
        DangerLevel::Caution,
        Some("Deleting branches can drop easy references to commits."),
        vec![
            opt("all", "All", "-a", "Show local and remote branches"),
            opt("delete", "Delete merged", "-d", "Delete branch (merged)"),
            opt(
                "force_delete",
                "Delete force",
                "-D",
                "Delete branch regardless of merge status",
            ),
        ],
    )
}
fn cmd_fetch() -> CommandSpec {
    mk(
        "fetch",
        "Remotes",
        "fetch",
        Some("remote"),
        "Download refs/objects from remote.",
        "Refresh remote-tracking branches safely.",
        vec!["git fetch", "git fetch origin --prune"],
        vec!["pull", "push", "branch"],
        DangerLevel::Safe,
        None,
        vec![
            opt("all", "All remotes", "--all", "Fetch every remote"),
            opt(
                "prune",
                "Prune",
                "--prune",
                "Delete stale remote-tracking refs",
            ),
        ],
    )
}
fn cmd_pull() -> CommandSpec {
    mk(
        "pull",
        "Remotes",
        "pull",
        Some("remote [branch]"),
        "Fetch then integrate remote updates.",
        "Update local branch from tracked remote.",
        vec!["git pull", "git pull --rebase origin main"],
        vec!["fetch", "merge", "rebase"],
        DangerLevel::Caution,
        Some("Creates merge commits or rebases depending options."),
        vec![
            opt(
                "rebase",
                "Rebase",
                "--rebase",
                "Rebase local commits after fetch",
            ),
            opt(
                "ff_only",
                "Fast-forward only",
                "--ff-only",
                "Reject non-fast-forward pulls",
            ),
        ],
    )
}
fn cmd_push() -> CommandSpec {
    mk(
        "push",
        "Remotes",
        "push",
        Some("remote branch"),
        "Upload commits to remote.",
        "Share your local commits.",
        vec!["git push", "git push --set-upstream origin feature/x"],
        vec!["fetch", "pull", "branch"],
        DangerLevel::Caution,
        Some("Force pushes can rewrite remote history."),
        vec![
            opt(
                "set_upstream",
                "Set upstream",
                "--set-upstream",
                "Set upstream tracking",
            ),
            opt("force", "Force", "--force", "Overwrite remote ref"),
            opt(
                "force_with_lease",
                "Force with lease",
                "--force-with-lease",
                "Safer force push with lease check",
            ),
        ],
    )
}
fn cmd_stash() -> CommandSpec {
    mk(
        "stash",
        "Working Tree",
        "stash",
        None,
        "Temporarily store local modifications.",
        "Quickly shelve changes to switch context.",
        vec!["git stash", "git stash -u", "git stash pop"],
        vec!["switch", "status", "restore"],
        DangerLevel::Caution,
        Some("Popping stash can produce conflicts."),
        vec![
            opt(
                "include_untracked",
                "Include untracked",
                "-u",
                "Stash untracked files",
            ),
            opt(
                "keep_index",
                "Keep index",
                "--keep-index",
                "Leave staged entries untouched",
            ),
        ],
    )
}
fn cmd_diff() -> CommandSpec {
    mk(
        "diff",
        "Inspection",
        "diff",
        Some("pathspec/ref"),
        "Show line-level changes.",
        "Inspect what changed before staging/committing.",
        vec!["git diff", "git diff --staged", "git diff main..feature"],
        vec!["status", "add", "log"],
        DangerLevel::Safe,
        None,
        vec![
            opt("staged", "Staged", "--staged", "Diff staged changes"),
            opt("stat", "Stat", "--stat", "Show summary statistics"),
            opt("name_only", "Name-only", "--name-only", "Only file names"),
        ],
    )
}
fn cmd_log() -> CommandSpec {
    mk(
        "log",
        "Inspection",
        "log",
        Some("range/ref"),
        "Show commit history.",
        "Explore project history and references.",
        vec![
            "git log --oneline --graph --decorate",
            "git log main..feature",
        ],
        vec!["show", "reflog", "diff"],
        DangerLevel::Safe,
        None,
        vec![
            opt("oneline", "Oneline", "--oneline", "Compact one-line format"),
            opt("graph", "Graph", "--graph", "Draw ASCII commit graph"),
            opt("decorate", "Decorate", "--decorate", "Show branch/tag refs"),
        ],
    )
}

fn opt(
    key: &'static str,
    label: &'static str,
    cli_flag: &'static str,
    help: &'static str,
) -> CommandOption {
    CommandOption {
        key,
        label,
        cli_flag,
        help,
    }
}

#[allow(clippy::too_many_arguments)]
fn mk(
    id: &'static str,
    category: &'static str,
    base: &'static str,
    target_label: Option<&'static str>,
    description: &'static str,
    when_to_use: &'static str,
    examples: Vec<&'static str>,
    related: Vec<&'static str>,
    danger_level: DangerLevel,
    danger_note: Option<&'static str>,
    toggles: Vec<CommandOption>,
) -> CommandSpec {
    CommandSpec {
        id,
        category,
        base,
        target_label,
        docs: CommandDoc {
            description,
            when_to_use,
            examples,
            related,
            danger_level,
            danger_note,
        },
        toggles,
    }
}
