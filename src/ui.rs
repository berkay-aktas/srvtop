use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, SortColumn, SortDirection};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
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
    let header = Line::from(vec![
        Span::styled(" srvtop ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("│ "),
        Span::styled(format!("v{}", env!("CARGO_PKG_VERSION")), Style::default().fg(Color::DarkGray)),
        Span::raw(" │ "),
        Span::styled(mode, Style::default().fg(Color::Yellow)),
    ]);
    frame.render_widget(Paragraph::new(header), area);
}

fn sort_indicator(app: &App, col: SortColumn) -> &'static str {
    if app.sort_column == col {
        match app.sort_direction {
            SortDirection::Ascending => " ▲",
            SortDirection::Descending => " ▼",
        }
    } else {
        ""
    }
}

fn draw_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = [
        format!("PID{}", sort_indicator(app, SortColumn::Pid)),
        format!("NAME{}", sort_indicator(app, SortColumn::Name)),
        format!("PORT{}", sort_indicator(app, SortColumn::Port)),
        format!("PROTO{}", sort_indicator(app, SortColumn::Proto)),
        format!("CPU%{}", sort_indicator(app, SortColumn::Cpu)),
        format!("MEMORY{}", sort_indicator(app, SortColumn::Memory)),
    ];

    let header = Row::new(
        header_cells
            .iter()
            .map(|h| Cell::from(h.as_str()).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
    )
    .height(1);

    let rows: Vec<Row> = app
        .processes
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.name.clone()),
                Cell::from(p.port.to_string()).style(Style::default().fg(Color::Green)),
                Cell::from(p.protocol.clone()),
                Cell::from(format!("{:.1}", p.cpu_percent)),
                Cell::from(p.memory_display.clone()),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Min(15),
        Constraint::Length(7),
        Constraint::Length(6),
        Constraint::Length(8),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(table, area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let status = if let Some(ref msg) = app.status_message {
        Span::styled(format!(" {} ", msg), Style::default().fg(Color::Yellow))
    } else {
        Span::raw("")
    };

    let footer = Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Cyan)),
        Span::raw(" quit │ "),
        Span::styled("k", Style::default().fg(Color::Cyan)),
        Span::raw(" kill │ "),
        Span::styled("s", Style::default().fg(Color::Cyan)),
        Span::raw(" sort │ "),
        Span::styled("a", Style::default().fg(Color::Cyan)),
        Span::raw(" all │ "),
        Span::styled("r", Style::default().fg(Color::Cyan)),
        Span::raw(" refresh │ "),
        Span::raw(format!("{} processes ", app.processes.len())),
        status,
    ]);
    frame.render_widget(Paragraph::new(footer), area);
}

fn draw_kill_confirm(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 5u16;
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    let text = if let Some(p) = app.selected_process() {
        format!("Kill {} (PID {}) on :{}? [y/n]", p.name, p.pid, p.port)
    } else {
        "No process selected".to_string()
    };

    let popup = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(" Confirm Kill ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}
