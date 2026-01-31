use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table,
    },
    Frame,
};

use crate::app::{App, SortColumn, SortDirection};

const SPLASH_ART: &str = r#"
        ╭──────────────────╮
        │  ┌──┐  ┌──┐  ◉  │
        │  └──┘  └──┘  ◉  │
        │  ┌──┐  ┌──┐  ◉  │
        │  └──┘  └──┘     │
        ╰───────┬──────────╯
            ┌───┴───┐
            │ ~~~~~ │
            │ ~~~~~ │
            └───────┘
          ┌───┐   ┌───┐
          │   │   │   │
          └───┘   └───┘
"#;

const BORDER_SET: border::Set = border::Set {
    top_left: "╭",
    top_right: "╮",
    bottom_left: "╰",
    bottom_right: "╯",
    vertical_left: "│",
    vertical_right: "│",
    horizontal_top: "─",
    horizontal_bottom: "─",
};

const NAME_MAX_WIDTH: usize = 20;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(frame.area());

    draw_header(frame, app, chunks[0]);
    draw_table(frame, app, chunks[1]);
    draw_footer(frame, app, chunks[2]);

    if app.show_kill_confirm {
        draw_kill_confirm(frame, app);
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let mode = if app.show_all { "ALL" } else { "DEV" };

    let elapsed = app.last_refresh.elapsed().as_secs();
    let remaining = app.tick_rate_secs.saturating_sub(elapsed);
    let countdown = format!(" \u{27f3} {}s ", remaining);

    let header = Line::from(vec![
        Span::styled(
            " srvtop ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" v{} ", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::styled(
            format!(" {} ", mode),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(countdown, Style::default().fg(Color::DarkGray)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(BORDER_SET)
        .border_style(Style::default().fg(Color::DarkGray));

    frame.render_widget(Paragraph::new(header).block(block), area);
}

fn sort_indicator(app: &App, col: SortColumn) -> &'static str {
    if app.sort_column == col {
        match app.sort_direction {
            SortDirection::Ascending => " \u{25b2}",
            SortDirection::Descending => " \u{25bc}",
        }
    } else {
        ""
    }
}

fn port_badge(port: u16) -> Cell<'static> {
    let color = match port {
        3000..=3999 => Color::Green,
        4000..=4999 => Color::Cyan,
        5000..=5999 => Color::Blue,
        6000..=6999 => Color::Magenta,
        8000..=8999 => Color::Yellow,
        9000..=9999 => Color::Red,
        27017 => Color::Green,
        _ => Color::DarkGray,
    };
    Cell::from(format!(" :{} ", port)).style(
        Style::default()
            .fg(Color::Black)
            .bg(color)
            .add_modifier(Modifier::BOLD),
    )
}

fn cpu_bar(percent: f32) -> String {
    let blocks = [' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}'];
    let idx = ((percent / 100.0) * 8.0).round().clamp(0.0, 8.0) as usize;
    format!("{} {:.1}%", blocks[idx], percent)
}

fn memory_bar(bytes: u64, max_bytes: u64, display: &str) -> String {
    let bar_width = 5;
    let ratio = if max_bytes > 0 {
        (bytes as f64 / max_bytes as f64).min(1.0)
    } else {
        0.0
    };
    let filled = (ratio * bar_width as f64).round() as usize;
    let empty = bar_width - filled;
    format!(
        "{}{} {}",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
        display
    )
}

fn truncate_name(name: &str, max_width: usize) -> String {
    if name.chars().count() > max_width {
        let truncated: String = name.chars().take(max_width - 1).collect();
        format!("{}\u{2026}", truncated)
    } else {
        name.to_string()
    }
}

fn draw_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(BORDER_SET)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Processes ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));

    if app.processes.is_empty() {
        let msg = if app.show_all {
            "No listening processes found"
        } else {
            "No dev servers found (press 'a' to show all)"
        };
        let empty = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let max_memory = app
        .processes
        .iter()
        .map(|p| p.memory_bytes)
        .max()
        .unwrap_or(1);

    let header_cells = [
        format!("PID{}", sort_indicator(app, SortColumn::Pid)),
        format!("NAME{}", sort_indicator(app, SortColumn::Name)),
        format!("PORT{}", sort_indicator(app, SortColumn::Port)),
        format!("PROTO{}", sort_indicator(app, SortColumn::Proto)),
        format!("CPU%{}", sort_indicator(app, SortColumn::Cpu)),
        format!("MEMORY{}", sort_indicator(app, SortColumn::Memory)),
        format!("UPTIME{}", sort_indicator(app, SortColumn::Uptime)),
    ];

    let header = Row::new(header_cells.iter().map(|h| {
        Cell::from(h.as_str()).style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
    }))
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .processes
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let selected = i == app.selected;
            let style = if selected {
                Style::default().bg(Color::Indexed(236)).fg(Color::White)
            } else if i % 2 == 1 {
                Style::default().bg(Color::Indexed(235))
            } else {
                Style::default()
            };

            let name_display = format!(
                "\u{25cf} {}",
                truncate_name(&p.name, NAME_MAX_WIDTH)
            );
            let name_style = if selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let cpu_style = if p.cpu_percent > 50.0 {
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD)
            } else if p.cpu_percent > 20.0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            Row::new(vec![
                Cell::from(format!(" {} ", p.pid)).style(Style::default().fg(Color::DarkGray)),
                Cell::from(name_display).style(name_style),
                port_badge(p.port),
                Cell::from(p.protocol.clone()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(cpu_bar(p.cpu_percent)).style(cpu_style),
                Cell::from(memory_bar(p.memory_bytes, max_memory, &p.memory_display))
                    .style(Style::default().fg(Color::Blue)),
                Cell::from(p.uptime_display.clone())
                    .style(Style::default().fg(Color::DarkGray)),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Min(15),
        Constraint::Length(9),
        Constraint::Length(6),
        Constraint::Length(12),
        Constraint::Length(18),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths).header(header).block(block);

    frame.render_widget(table, area);

    // Scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None);
    let scrollbar_area = Rect {
        x: area.x + area.width.saturating_sub(1),
        y: area.y + 1,
        width: 1,
        height: area.height.saturating_sub(2),
    };
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut app.scrollbar_state);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let status = if let Some(ref msg) = app.status_message {
        Span::styled(
            format!(" {} ", msg),
            Style::default().fg(Color::Black).bg(Color::Yellow),
        )
    } else {
        Span::raw("")
    };

    let count = app.processes.len();
    let badge_color = if count > 15 {
        Color::Red
    } else if count >= 5 {
        Color::Yellow
    } else {
        Color::Green
    };

    let footer = Line::from(vec![
        Span::styled(
            " q",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit ", Style::default().fg(Color::DarkGray)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " k",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" kill ", Style::default().fg(Color::DarkGray)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " s",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" sort ", Style::default().fg(Color::DarkGray)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " S",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" reverse ", Style::default().fg(Color::DarkGray)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " a",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" all ", Style::default().fg(Color::DarkGray)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " r",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" refresh ", Style::default().fg(Color::DarkGray)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::styled(
            format!(" {} ", count),
            Style::default()
                .fg(Color::Black)
                .bg(badge_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" processes ", Style::default().fg(Color::DarkGray)),
        status,
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(BORDER_SET)
        .border_style(Style::default().fg(Color::DarkGray));

    frame.render_widget(Paragraph::new(footer).block(block), area);
}

fn draw_kill_confirm(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 5u16;
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    let text = if let Some(p) = app.selected_process() {
        Line::from(vec![
            Span::raw("Kill "),
            Span::styled(
                &p.name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" (PID {}) on ", p.pid)),
            Span::styled(
                format!(":{}", p.port),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("? "),
            Span::styled(
                "[y/n]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from("No process selected")
    };

    let popup = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(Span::styled(
                    " Confirm Kill ",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_set(BORDER_SET)
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

pub fn draw_splash(frame: &mut Frame) {
    let area = frame.area();

    let lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(""),
    ]
    .into_iter()
    .chain(SPLASH_ART.lines().map(|l| {
        Line::from(Span::styled(l, Style::default().fg(Color::Magenta)))
    }))
    .chain(vec![
        Line::from(""),
        Line::from(Span::styled(
            "srvtop",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "like htop, but for your dev servers",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .collect();

    let splash = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(splash, area);
}
