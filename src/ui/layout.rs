use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::App,
    commands::DangerLevel,
    config::keybindings::FOOTER_KEYS,
    models::{domain::RepoOperation, ui::FocusPane},
};

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(frame.area());

    render_top_status(frame, root[0], app);
    render_body(frame, root[1], app);
    render_footer(frame, root[2], app);

    if app.show_help {
        render_help_overlay(frame, centered_rect(70, 70, frame.area()), app);
    }
}

fn render_top_status(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let op = match app.repo.operation {
        Some(RepoOperation::Merge) => "MERGE",
        Some(RepoOperation::Rebase) => "REBASE",
        Some(RepoOperation::CherryPick) => "CHERRY-PICK",
        None => "",
    };

    let text = Line::from(vec![
        Span::styled(
            format!(" {} ", app.repo.repo_name),
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("branch:{}  ", app.repo.branch)),
        Span::raw(format!("HEAD:{}  ", app.repo.head_short)),
        Span::raw(format!("remotes:[{}]  ", app.repo.remotes.join(","))),
        Span::raw(format!("status:{}  ", app.repo.status.label())),
        Span::styled(
            format!(
                "{}{}",
                if app.repo.detached_head {
                    "DETACHED "
                } else {
                    ""
                },
                op
            ),
            Style::default().fg(app.theme.warning),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("GitDeck")),
        area,
    );
}

fn render_body(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(38),
            Constraint::Percentage(22),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(area);

    render_history(frame, columns[0], app);
    render_branches(frame, columns[1], app);
    render_commands(frame, columns[2], app);
    render_details(frame, columns[3], app);
}

fn render_history(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let active = app.focus == FocusPane::History;
    let title = if app.query.is_empty() {
        "History"
    } else {
        "History (filtered)"
    };

    let items = app
        .filtered_commits
        .iter()
        .enumerate()
        .map(|(i, idx)| {
            let c = &app.commits[*idx];
            let prefix = if i == app.selected_commit { ">" } else { " " };
            ListItem::new(format!(
                "{} {} {} {} {}",
                prefix,
                c.short_id,
                c.author,
                c.date.format("%Y-%m-%d"),
                c.subject
            ))
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        List::new(items)
            .block(pane_block(title, active, app))
            .highlight_symbol("> "),
        area,
    );
}

fn render_branches(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let active = app.focus == FocusPane::Branches;
    let items = app
        .branches
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let mark = if b.is_head { "*" } else { " " };
            let remote = if b.is_remote { "(remote)" } else { "(local)" };
            let selected = if i == app.selected_branch { ">" } else { " " };
            ListItem::new(format!("{} {} {} {}", selected, mark, b.name, remote))
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        List::new(items).block(pane_block("Branches / Graph", active, app)),
        area,
    );
}

fn render_commands(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let active = app.focus == FocusPane::Commands || app.command_mode;

    let items = app
        .commands
        .iter()
        .enumerate()
        .map(|(idx, cmd)| {
            let selected = if idx == app.builder.selected_command {
                ">"
            } else {
                " "
            };
            ListItem::new(format!("{} [{}] git {}", selected, cmd.category, cmd.base))
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        List::new(items).block(pane_block("Command Center", active, app)),
        area,
    );
}

fn render_details(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let active = app.focus == FocusPane::Details;
    let selected = &app.commands[app.builder.selected_command];

    let mut lines = vec![
        Line::from(Span::styled(
            format!("git {}", selected.base),
            Style::default()
                .fg(app.theme.accent_alt)
                .add_modifier(Modifier::BOLD),
        )),
        Line::raw(format!("What: {}", selected.docs.description)),
        Line::raw(format!("When: {}", selected.docs.when_to_use)),
        Line::raw(format!("Danger: {:?}", selected.docs.danger_level)),
        Line::raw(""),
    ];

    if let Some(details) = &app.commit_details {
        lines.push(Line::raw(format!("Commit: {}", details.summary.short_id)));
        lines.push(Line::raw(details.summary.subject.clone()));
        lines.push(Line::raw(format!(
            "Δ files:{} +{} -{}",
            details.files_changed, details.insertions, details.deletions
        )));
        if !details.body.is_empty() {
            lines.push(Line::raw(details.body.clone()));
        }
        lines.push(Line::raw(""));
    }

    if let Some(note) = selected.docs.danger_note {
        lines.push(Line::from(Span::styled(
            format!("Warning: {note}"),
            Style::default().fg(app.theme.warning),
        )));
    }

    let preview = app.builder.preview_command(selected);
    lines.push(Line::raw(format!("Preview: {preview}")));

    if !app.command_output.trim().is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::raw("Last output:"));
        for l in app.command_output.lines().take(8) {
            lines.push(Line::raw(l.to_string()));
        }
    }

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(pane_block("Details / Help", active, app))
            .wrap(Wrap { trim: false }),
        area,
    );

    if app.command_mode {
        render_builder_overlay(frame, centered_rect(76, 72, frame.area()), app);
    }
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let status = Line::from(vec![
        Span::styled("Keys: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(FOOTER_KEYS),
        Span::raw(" • "),
        Span::styled(
            app.status_message.as_str(),
            Style::default().fg(app.theme.subtle),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(status).block(Block::default().borders(Borders::ALL)),
        area,
    );
}

fn render_builder_overlay(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let selected = &app.commands[app.builder.selected_command];
    let problems = app.builder.validate(selected);

    let mut lines = vec![
        Line::from(Span::styled(
            format!("Builder: git {}", selected.base),
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::raw(
            "Tab/S-Tab command • j/k option • space toggle • type target • x execute • q close",
        ),
        Line::raw(""),
    ];

    for (i, opt) in selected.toggles.iter().enumerate() {
        let marker = if app.builder.option_enabled(opt.key) {
            "[x]"
        } else {
            "[ ]"
        };
        let cursor = if i == app.builder.selected_option {
            ">"
        } else {
            " "
        };
        lines.push(Line::raw(format!(
            "{} {} {} ({})",
            cursor, marker, opt.label, opt.cli_flag
        )));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(opt.help, Style::default().fg(app.theme.subtle)),
        ]));
    }

    lines.push(Line::raw(""));
    if let Some(label) = selected.target_label {
        lines.push(Line::raw(format!(
            "Target {label}: {}",
            app.builder.target_input
        )));
    }

    let preview = app.builder.preview_command(selected);
    lines.push(Line::raw(format!("Preview: {preview}")));

    if !problems.is_empty() {
        lines.push(Line::from(Span::styled(
            "Validation:",
            Style::default()
                .fg(app.theme.warning)
                .add_modifier(Modifier::BOLD),
        )));
        for p in problems {
            lines.push(Line::from(Span::styled(
                format!("- {p}"),
                Style::default().fg(app.theme.warning),
            )));
        }
    }

    if matches!(selected.docs.danger_level, DangerLevel::Dangerous) || app.confirm_required {
        lines.push(Line::from(Span::styled(
            "Dangerous command path detected. Press x twice to execute.",
            Style::default().fg(app.theme.danger),
        )));
    }

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("Interactive Command Builder")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_help_overlay(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let text = Text::from(vec![
        Line::from(Span::styled(
            "GitDeck keybindings",
            Style::default().fg(app.theme.accent),
        )),
        Line::raw("j/k or ↑/↓: move within focused pane"),
        Line::raw("Tab / Shift-Tab: cycle panes"),
        Line::raw("c: open command builder"),
        Line::raw("h, g, d: focus history/branches/details"),
        Line::raw("/: filter commit history"),
        Line::raw("Enter: inspect selected commit"),
        Line::raw("q: quit / close overlay"),
        Line::raw("?: open this help"),
    ]);

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().title("Help").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn pane_block(title: &str, active: bool, app: &App) -> Block<'_> {
    let style = if active {
        Style::default()
            .fg(app.theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(title, style))
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
