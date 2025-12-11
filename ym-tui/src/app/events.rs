//! Обработка пользовательского ввода и событий TUI.

use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};

use super::core::App;
use super::{clear_messages, messaging};

/// Считывание событий и обновление состояния приложения.
pub async fn handle_crossterm_events(app: &mut App) -> Result<()> {
    let event = app.event_stream.next().fuse().await;

    if let Some(Ok(evt)) = event {
        match evt {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                handle_key_event(app, key).await;
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    Ok(())
}

/// Обработка нажатий клавиш.
pub async fn handle_key_event(app: &mut App, key: KeyEvent) {
    match (key.modifiers, key.code) {
        // Выход.
        (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
            app.quit()
        }

        // Очистка истории сообщений.
        (KeyModifiers::CONTROL, KeyCode::Char('r') | KeyCode::Char('R')) => {
            clear_messages(app);
        }
        
        // Отправка сообщения.
        (_, KeyCode::Enter) => messaging::send_message_to_gpt(app).await,

        // Ctrl+Left — на слово назад.
        (KeyModifiers::CONTROL, KeyCode::Left) => {
            let chars: Vec<char> = app.input_buffer.chars().collect();
            app.cursor_pos = app.cursor_pos.saturating_sub(1);
            while app.cursor_pos > 0 && chars[app.cursor_pos - 1].is_alphanumeric() {
                app.cursor_pos -= 1;
            }
        }

        // Ctrl+Right — на слово вперёд.
        (KeyModifiers::CONTROL, KeyCode::Right) => {
            let chars: Vec<char> = app.input_buffer.chars().collect();
            while app.cursor_pos < chars.len() && chars[app.cursor_pos].is_alphanumeric() {
                app.cursor_pos += 1;
            }
            if app.cursor_pos < chars.len() {
                app.cursor_pos += 1;
            }
        }

        // Движение курсора.
        (_, KeyCode::Left) => {
            if app.cursor_pos > 0 {
                app.cursor_pos -= 1;
            }
        }
        (_, KeyCode::Right) => {
            if app.cursor_pos < app.input_buffer.len() {
                app.cursor_pos += 1;
            }
        }
        (_, KeyCode::Home) => {
            app.cursor_pos = 0;
        }
        (_, KeyCode::End) => {
            app.cursor_pos = app.input_buffer.len();
        }

        // Ввод текста.
        (_, KeyCode::Char(c)) => {
            insert_char_at_cursor(app, c);
        }

        // Удаление символа (Backspace).
        (_, KeyCode::Backspace) => {
            delete_char_before_cursor(app);
        }

        // Удаление символа (Delete).
        (_, KeyCode::Delete) => {
            delete_char_at_cursor(app);
        }
        
        _ => {}
    }
}

/// Вставить символ в позицию курсора.
fn insert_char_at_cursor(app: &mut App, c: char) {
    let mut chars: Vec<char> = app.input_buffer.chars().collect();
    if app.cursor_pos <= chars.len() {
        chars.insert(app.cursor_pos, c);
        app.input_buffer = chars.iter().collect();
        app.cursor_pos += 1;
    }
}

/// Удалить символ перед курсором (Backspace).
fn delete_char_before_cursor(app: &mut App) {
    if app.cursor_pos > 0 {
        let mut chars: Vec<char> = app.input_buffer.chars().collect();
        chars.remove(app.cursor_pos - 1);
        app.input_buffer = chars.iter().collect();
        app.cursor_pos -= 1;
    }
}

/// Удалить символ на позиции курсора (Delete).
fn delete_char_at_cursor(app: &mut App) {
    let mut chars: Vec<char> = app.input_buffer.chars().collect();
    if app.cursor_pos < chars.len() {
        chars.remove(app.cursor_pos);
        app.input_buffer = chars.iter().collect();
    }
}
