//! Основная структура приложения и его жизненный цикл.

use crate::settings;
use crossterm::event::EventStream;
use ratatui::DefaultTerminal;
use ym_yagpt::client::GPTClient;

/// Структура, содержащая данные для рендеринга окна терминала.
#[derive(Debug, Default)]
pub struct App {
    /// Флаг, что приложение активно.
    pub running: bool,
    // Event stream.
    pub event_stream: EventStream,
    // История сообщений с нейросетью.
    pub messages: Vec<String>,
    // Буфер ввода от пользователя.
    pub input_buffer: String,
    // Позиция курсора.
    pub cursor_pos: usize,
    // Контроллер скроллинга.
    pub scroll_offset: u16,
    pub gpt_client: GPTClient,
}

impl App {
    /// Создание нового экземпляра [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            event_stream: EventStream::new(),
            messages: vec!["YandexGPT готов к диалогу.".to_string()],
            input_buffer: String::new(),
            cursor_pos: 0,
            scroll_offset: 0,
            gpt_client: GPTClient::new().load_auth(settings::access_file_path()),
        }
    }

    /// Запуск приложения `App` в асинхронном процессе.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        use crate::app::{events, ui};

        self.running = true;
        while self.running {
            terminal.draw(|frame| ui::draw_interface(&mut self, frame))?;

            // Обработка событий
            if let Err(e) = events::handle_crossterm_events(&mut self).await {
                eprintln!("Ошибка обработки событий: {}", e);
            }
        }
        Ok(())
    }

    /// Сбросить флаг запущенного приложения (`running`) и остановить приложение.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
