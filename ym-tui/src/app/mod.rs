//! Модуль приложения TUI для общения с YandexGPT.
//!
//! Разделен на логические компоненты:
//! - `core` — основная структура и жизненный цикл;
//! - `ui` — отрисовка интерфейса;
//! - `events` — обработка пользовательского ввода;
//! - `messaging` — работа с сообщениями и GPT.

mod core;
mod events;
mod messaging;
mod ui;

// Реэкспорт для удобства использования
pub use core::App;
pub use messaging::clear_messages;
