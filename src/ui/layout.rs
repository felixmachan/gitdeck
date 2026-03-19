use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::App,
    commands::{BuilderFocus, DangerLevel, TargetType},
    config::keybindings::FOOTER_KEYS,
    models::{domain::RepoOperation, ui::FocusPane},
};

pub fn render(frame: &mut Frame<'_>, app: &App) {
    // Dinamikus magasság számítása: ha üres az output, kicsi a footer, ha van benne valami, nagyobb.
    let footer_height = if app.command_output.trim().is_empty() {
        3
    } else {
        // Maximum 10 sor, de legalább 6, ha van kimenet
        let lines_count = app.command_output.lines().count() as u16;
        (lines_count + 4).min(10).max(6)
    };

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Fill(1), // Ez a rész (a body) kap meg minden maradék helyet
            Constraint::Length(footer_height),
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
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Bal oszlop
            Constraint::Percentage(40), // Jobb oszlop
        ])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70), // Felül: History
            Constraint::Percentage(30), // Alul: Branchek
        ])
        .split(main_chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Felül: Commands
            Constraint::Percentage(40), // Alul: Details
        ])
        .split(main_chunks[1]);

    render_history(frame, left_chunks[0], app);
    render_branches(frame, left_chunks[1], app);
    render_commands(frame, right_chunks[0], app);
    render_details(frame, right_chunks[1], app);
}

fn render_history(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let active = app.focus == FocusPane::History;
    let title = if app.query.is_empty() {
        " History "
    } else {
        " History (filtered) "
    };

    let items = app
        .filtered_commits
        .iter()
        .enumerate()
        .map(|(i, idx)| {
            let c = &app.commits[*idx];
            let prefix = if i == app.selected_commit && active { ">" } else { " " };
            ListItem::new(format!(
                "{} {:<8} {:<12} {:<10} {}",
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
            let selected = if i == app.selected_branch && active { ">" } else { " " };
            ListItem::new(format!("{} {} {:<20} {}", selected, mark, b.name, remote))
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        List::new(items).block(pane_block(" Branches ", active, app)),
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
            let selected = if idx == app.builder.selected_command && active {
                ">"
            } else {
                " "
            };
            ListItem::new(format!("{} [{:<10}] git {}", selected, cmd.category, cmd.base))
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        List::new(items).block(pane_block(" Command Center ", active, app)),
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
        Line::from(vec![
            Span::styled(" • What: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(selected.docs.description.clone()),
        ]),
        Line::from(vec![
            Span::styled(" • When: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(selected.docs.when_to_use.clone()),
        ]),
        Line::from(vec![
            Span::styled(" • Danger: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:?}", selected.docs.danger_level)),
        ]),
        Line::raw(""),
    ];

    if let Some(details) = &app.commit_details {
        lines.push(Line::from(vec![
            Span::styled("Selected Commit: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&details.summary.short_id),
        ]));
        lines.push(Line::raw(format!(" Subject: {}", details.summary.subject)));
        lines.push(Line::raw(format!(
            " Changes: {} files, +{} -{}",
            details.files_changed, details.insertions, details.deletions
        )));
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
    let active = app.focus == FocusPane::Output;
    let mut lines = vec![Line::from(vec![
        Span::styled("Keys: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(FOOTER_KEYS),
        Span::raw(" • "),
        Span::styled(
            app.status_message.as_str(),
            Style::default().fg(app.theme.subtle),
        ),
    ])];

    if !app.command_output.trim().is_empty() {
        lines.push(Line::raw(""));
        for l in app.command_output.lines() {
            lines.push(Line::from(vec![
                Span::styled("> ", Style::default().fg(app.theme.accent)),
                Span::raw(l),
            ]));
        }
    }

    // Ha a terminál fókuszban van, a felhasználó görgetését használjuk.
    // Ha nincs, automatikusan az aljára görgetünk.
    let content_height = lines.len() as u16;
    let available_height = area.height.saturating_sub(2);
    
    let scroll = if active {
        app.terminal_scroll
    } else if content_height > available_height {
        content_height - available_height
    } else {
        0
    };

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(pane_block(" Mini Terminal / Output ", active, app))
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
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
            "Tab/S-Tab command • j/k option • space toggle • type target • Enter execute • q close",
        ),
        Line::raw(""),
    ];

    for (i, opt) in selected.toggles.iter().enumerate() {
        let marker = if app.builder.option_enabled(opt.key) {
            "[x]"
        } else {
            "[ ]"
        };
        let is_focused = app.builder.focus == BuilderFocus::Options && i == app.builder.selected_option;
        let cursor = if is_focused {
            ">"
        } else {
            " "
        };
        let style = if is_focused {
            Style::default().fg(app.theme.accent)
        } else {
            Style::default()
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{} {} ", cursor, marker), style),
            Span::raw(opt.label),
            Span::raw(format!(" ({})", opt.cli_flag)),
        ]));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(opt.help, Style::default().fg(app.theme.subtle)),
        ]));
    }

    lines.push(Line::raw(""));
    if let Some(label) = selected.target_label {
        let is_focused = app.builder.focus == BuilderFocus::Target;
        let style = if is_focused {
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let cursor = if is_focused { "> " } else { "  " };
        lines.push(Line::from(vec![
            Span::styled(cursor, style),
            Span::styled(format!("{}: ", label), style),
            Span::raw(app.builder.target_input.clone()),
            if is_focused {
                Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK))
            } else {
                Span::raw("")
            },
        ]));

        if is_focused && selected.target_type == TargetType::Branch {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    "↑/↓ to cycle branches",
                    Style::default().fg(app.theme.subtle),
                ),
            ]));
        }
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
            "Dangerous command path detected. Press Enter twice to execute.",
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

fn pane_block<'a>(title: &'a str, active: bool, app: &'a App) -> Block<'a> {
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
