use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

use crate::{
    commands::{command_catalog, execute_preview, BuilderState, DangerLevel},
    config::theme::Theme,
    git::GitService,
    models::{
        domain::{BranchInfo, CommitDetails, CommitSummary, RepoSummary},
        ui::FocusPane,
    },
    ui,
};

pub struct App {
    git: GitService,
    pub theme: Theme,
    pub focus: FocusPane,
    pub show_help: bool,
    pub query: String,
    pub command_mode: bool,
    pub should_quit: bool,

    pub repo: RepoSummary,
    pub commits: Vec<CommitSummary>,
    pub filtered_commits: Vec<usize>,
    pub selected_commit: usize,
    pub commit_details: Option<CommitDetails>,

    pub branches: Vec<BranchInfo>,
    pub selected_branch: usize,

    pub commands: Vec<crate::commands::CommandSpec>,
    pub builder: BuilderState,
    pub command_output: String,
    pub confirm_required: bool,
    pub status_message: String,
}

impl App {
    pub fn new() -> Result<Self> {
        let git = GitService::discover(".")?;
        let commands = command_catalog();

        let mut app = Self {
            repo: git.repo_summary()?,
            commits: git.commit_history(400)?,
            filtered_commits: Vec::new(),
            commit_details: None,
            selected_commit: 0,
            branches: git.branches()?,
            selected_branch: 0,
            commands,
            builder: BuilderState::new(),
            command_output: String::new(),
            command_mode: false,
            show_help: false,
            query: String::new(),
            should_quit: false,
            confirm_required: false,
            status_message: "Ready".to_string(),
            focus: FocusPane::History,
            theme: Theme::default(),
            git,
        };

        app.refresh_derived_state()?;
        Ok(app)
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        result
    }

    fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|f| ui::render(f, self))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code, key.modifiers)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        if self.show_help {
            if matches!(code, KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q')) {
                self.show_help = false;
            }
            return Ok(());
        }

        if self.command_mode {
            return self.handle_command_builder_input(code, modifiers);
        }

        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => self.focus = self.focus.next(),
            KeyCode::BackTab => self.focus = self.focus.prev(),
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('/') => {
                self.query.clear();
                self.status_message = "Filter history by typing; Enter applies".to_string();
            }
            KeyCode::Char('c') => {
                self.focus = FocusPane::Commands;
                self.command_mode = true;
            }
            KeyCode::Char('h') => self.focus = FocusPane::History,
            KeyCode::Char('g') => self.focus = FocusPane::Branches,
            KeyCode::Char('d') => self.focus = FocusPane::Details,
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Enter => self.activate_selection()?,
            KeyCode::Char(ch) if modifiers.is_empty() => {
                self.query.push(ch);
                self.apply_history_filter();
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.apply_history_filter();
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_command_builder_input(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> Result<()> {
        let command_len = self.commands.len();
        let selected = &self.commands[self.builder.selected_command];

        match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.command_mode = false;
                self.confirm_required = false;
            }
            KeyCode::Tab => {
                self.builder.selected_command = (self.builder.selected_command + 1) % command_len;
                self.builder.reset_for_command();
            }
            KeyCode::BackTab => {
                self.builder.selected_command = if self.builder.selected_command == 0 {
                    command_len.saturating_sub(1)
                } else {
                    self.builder.selected_command - 1
                };
                self.builder.reset_for_command();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = selected.toggles.len().saturating_sub(1);
                self.builder.selected_option = (self.builder.selected_option + 1).min(max);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.builder.selected_option = self.builder.selected_option.saturating_sub(1);
            }
            KeyCode::Char(' ') => {
                if let Some(opt) = selected.toggles.get(self.builder.selected_option) {
                    self.builder.toggle_option(opt.key);
                }
            }
            KeyCode::Char('x') => {
                let preview = self.builder.preview_command(selected);
                if matches!(selected.docs.danger_level, DangerLevel::Dangerous)
                    || self.builder.option_enabled("force")
                    || self.builder.option_enabled("force_delete")
                {
                    if !self.confirm_required {
                        self.confirm_required = true;
                        self.status_message =
                            "Dangerous action: press x again to confirm execution".to_string();
                        return Ok(());
                    }
                }

                self.command_output = execute_preview(&preview).unwrap_or_else(|e| e.to_string());
                self.status_message = "Command executed".to_string();
                self.confirm_required = false;
                self.refresh_repo_data()?;
            }
            KeyCode::Enter => {
                self.command_mode = false;
                self.confirm_required = false;
            }
            KeyCode::Backspace => {
                self.builder.target_input.pop();
            }
            KeyCode::Char(c) if modifiers.is_empty() => {
                if selected.target_label.is_some() {
                    self.builder.target_input.push(c);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn move_selection(&mut self, delta: isize) {
        match self.focus {
            FocusPane::History => {
                let len = self.filtered_commits.len();
                if len == 0 {
                    return;
                }
                self.selected_commit =
                    ((self.selected_commit as isize + delta).rem_euclid(len as isize)) as usize;
            }
            FocusPane::Branches => {
                let len = self.branches.len();
                if len == 0 {
                    return;
                }
                self.selected_branch =
                    ((self.selected_branch as isize + delta).rem_euclid(len as isize)) as usize;
            }
            FocusPane::Commands => {
                let len = self.commands.len();
                if len == 0 {
                    return;
                }
                self.builder.selected_command = ((self.builder.selected_command as isize + delta)
                    .rem_euclid(len as isize))
                    as usize;
                self.builder.reset_for_command();
            }
            FocusPane::Details => {}
        }
    }

    fn activate_selection(&mut self) -> Result<()> {
        match self.focus {
            FocusPane::History => {
                if let Some(commit) = self.current_commit() {
                    self.commit_details = Some(self.git.commit_details(&commit.id)?);
                }
            }
            FocusPane::Commands => {
                self.command_mode = true;
            }
            _ => {}
        }
        Ok(())
    }

    fn current_commit(&self) -> Option<&CommitSummary> {
        self.filtered_commits
            .get(self.selected_commit)
            .and_then(|&idx| self.commits.get(idx))
    }

    fn apply_history_filter(&mut self) {
        if self.query.trim().is_empty() {
            self.filtered_commits = (0..self.commits.len()).collect();
        } else {
            let q = self.query.to_lowercase();
            self.filtered_commits = self
                .commits
                .iter()
                .enumerate()
                .filter(|(_, c)| {
                    c.subject.to_lowercase().contains(&q)
                        || c.author.to_lowercase().contains(&q)
                        || c.short_id.contains(&q)
                })
                .map(|(idx, _)| idx)
                .collect();
        }
        self.selected_commit = 0;
    }

    fn refresh_repo_data(&mut self) -> Result<()> {
        self.repo = self.git.repo_summary()?;
        self.branches = self.git.branches()?;
        self.commits = self.git.commit_history(400)?;
        self.apply_history_filter();
        Ok(())
    }

    fn refresh_derived_state(&mut self) -> Result<()> {
        self.apply_history_filter();
        if let Some(commit) = self.current_commit() {
            self.commit_details = Some(self.git.commit_details(&commit.id)?);
        }
        let stash_count = self.git.stash_count()?;
        self.status_message = format!("Ready • stashes: {stash_count}");
        Ok(())
    }
}
