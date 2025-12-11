//! Модуль приложения TUI для общения с YandexGPT.
//!
//! Разделен на логические компоненты:
//! - `core` — основная структура и жизненный цикл;
//! - `ui` — отрисовка интерфейса;
//! - `events` — обработка пользовательского ввода;
//! - `messaging` — работа с сообщениями и GPT.

mod core;
mod ui;
mod events;
mod messaging;

// Реэкспорт для удобства использования
pub use core::App;
pub use events::{handle_crossterm_events, handle_key_event};
pub use messaging::{send_message_to_gpt, add_system_message, clear_messages};
pub use ui::draw_interface;

