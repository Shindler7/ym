//! Отрисовка пользовательского интерфейса TUI.
//!
//! Подробная информация:
//! - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
//! - <https://github.com/ratatui/ratatui/tree/master/examples>

use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
    Frame,
};

use super::core::App;

/// Отрисовка интерфейса приложения.
pub fn draw_interface(app: &mut App, frame: &mut Frame) {
    use ratatui::layout::{Constraint, Direction, Layout};

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Min(3),
                Constraint::Percentage(70),
                Constraint::Percentage(20),
                Constraint::Min(3),
            ]
            .as_ref(),
        )
        .split(frame.area());

    draw_title(frame, chunks[0]);
    draw_messages(app, frame, chunks[1]);
    draw_input(app, frame, chunks[2]);
    draw_status_bar(app, frame, chunks[3]);
}

/// Отрисовка заголовка приложения.
fn draw_title(frame: &mut Frame, area: ratatui::layout::Rect) {
    let title = Line::from("Консольный коммуникатор с YandexGPT")
        .bold()
        .green()
        .centered();

    frame.render_widget(Paragraph::new(title).centered(), area);
}

/// Отрисовка блока с историей сообщений.
fn draw_messages(app: &mut App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let messages_text: Vec<Line> = app
        .messages
        .iter()
        .map(|msg| Line::from(msg.as_str()))
        .collect();

    let messages_block = Block::default()
        .title(" История диалога ")
        .borders(ratatui::widgets::Borders::ALL);

    let messages_widget = Paragraph::new(messages_text.clone())
        .block(messages_block)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .scroll((app.scroll_offset, 0));

    frame.render_widget(messages_widget, area);
}

/// Отрисовка поля ввода сообщения.
fn draw_input(app: &mut App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let input_block = Block::default()
        .title(" Ввод сообщения ")
        .borders(ratatui::widgets::Borders::ALL);

    // Подсветка курсора.
    let input_display = {
        let mut result = String::new();
        let chars: Vec<char> = app.input_buffer.chars().collect();

        for (i, ch) in chars.iter().enumerate() {
            if i == app.cursor_pos {
                result.push('█');
            }
            result.push(*ch);
        }

        if app.cursor_pos == chars.len() {
            result.push('█');
        }

        result
    };

    frame.render_widget(
        Paragraph::new(input_display)
            .block(input_block)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .fg(ratatui::style::Color::Yellow),
        area,
    );
}

/// Отрисовка статус-бара.
fn draw_status_bar(app: &mut App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let status = format!(
        " Сообщений: {} | Длина ввода: {} | Очистить историю: Ctrl+R | Выйти: Ctrl+C, Esc",
        app.messages.len(),
        app.input_buffer.len()
    );

    frame.render_widget(
        Paragraph::new(status).block(Block::default().borders(ratatui::widgets::Borders::TOP)),
        area,
    );
}
