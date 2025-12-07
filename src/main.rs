//! YM — консольный чат-менеджер общения с нейронными сетями, с помощью терминала.
//! Предполагается поддержка только текстового режима генерации.

mod cli;
mod errors;
mod libs;
mod settings;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};
use ratatui::{
    style::Stylize, text::Line,
    widgets::{Block, Paragraph},
    DefaultTerminal,
    Frame,
};

use cli::cli_action;
use libs::yagpt::GPTClient;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // Первоначально обработка командной строки.
    cli_action();

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

/// Структура, содержащая данные для рендеринга окна терминала.
#[derive(Debug, Default)]
pub struct App {
    /// Флаг, что приложение активно.
    running: bool,
    // Event stream.
    event_stream: EventStream,
    // История сообщений с нейросетью.
    messages: Vec<String>,
    // Буфер ввода от пользователя.
    input_buffer: String,
    // Позиция курсора.
    cursor_pos: usize,
    // Контроллер скроллинга.
    scroll_offset: u16,
    gpt_client: GPTClient,
}

impl App {
    /// Создание нового экземпляра [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            event_stream: EventStream::new(),
            messages: vec!["YandexGPT подключился к диалогу.".to_string()],
            input_buffer: String::new(),
            cursor_pos: 0,
            scroll_offset: 0,
            gpt_client: GPTClient::new(),
        }
    }

    /// Запуск приложения `App` в асинхронном процессе.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events().await?;
        }
        Ok(())
    }

    /// Отрисовка интерфейса.
    ///
    /// Подробная информация:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
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

        let title = Line::from("Консольный коммуникатор с YandexGPT")
            .bold()
            .green()
            .centered();

        frame.render_widget(Paragraph::new(title).centered(), chunks[0]);

        // Блок сообщений.
        let messages_text: Vec<Line> = self
            .messages
            .iter()
            .map(|msg| Line::from(msg.as_str()))
            .collect();

        let messages_block = Block::default()
            .title(" История диалога ")
            .borders(ratatui::widgets::Borders::ALL);

        // Создаем текстовый блок с возможностью прокрутки.
        let messages_widget = Paragraph::new(messages_text.clone())
            .block(messages_block)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .scroll((self.scroll_offset, 0)); // Используем scroll_offset из состояния

        frame.render_widget(messages_widget, chunks[1]);

        // 4. Рисуем (render) поле ввода.
        let input_block = Block::default()
            .title(" Ввод сообщения ")
            .borders(ratatui::widgets::Borders::ALL);

        // Подсветка курсора.
        let input_display = {
            let mut result = String::new();
            let chars: Vec<char> = self.input_buffer.chars().collect();

            for (i, ch) in chars.iter().enumerate() {
                if i == self.cursor_pos {
                    result.push('█');
                }
                result.push(*ch);
            }

            if self.cursor_pos == chars.len() {
                result.push('█');
            }

            result
        };

        frame.render_widget(
            Paragraph::new(input_display)
                .block(input_block)
                .wrap(ratatui::widgets::Wrap { trim: true })
                .fg(ratatui::style::Color::Yellow),
            chunks[2],
        );

        // 5. Рисуем виджет со статус-баром.
        let status = format!(
            " Сообщений: {} | Длина ввода: {} | Выйти Ctrl+C, Esc",
            self.messages.len(),
            self.input_buffer.len()
        );

        frame.render_widget(
            Paragraph::new(status).block(Block::default().borders(ratatui::widgets::Borders::TOP)),
            chunks[3],
        );
    }

    /// Считывание событий и обновление состояния [`App`].
    async fn handle_crossterm_events(&mut self) -> color_eyre::Result<()> {
        let event = self.event_stream.next().fuse().await;

        if let Some(Ok(evt)) = event {
            match evt {
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key).await,
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }

        Ok(())
    }

    /// Отслеживание нажатия на клавиши пользователем и реагирование в [`App`].
    async fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            // Выход.
            (_, KeyCode::Esc)
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),

            // Отправка сообщения.
            (_, KeyCode::Enter) => self.send_message().await,

            // Ctrl+Left — на слово назад.
            (KeyModifiers::CONTROL, KeyCode::Left) => {
                let chars: Vec<char> = self.input_buffer.chars().collect();
                self.cursor_pos = self.cursor_pos.saturating_sub(1);
                while self.cursor_pos > 0 && chars[self.cursor_pos - 1].is_alphanumeric() {
                    self.cursor_pos -= 1;
                }
            }

            // Ctrl+Right — на слово вперёд.
            (KeyModifiers::CONTROL, KeyCode::Right) => {
                let chars: Vec<char> = self.input_buffer.chars().collect();
                while self.cursor_pos < chars.len() && chars[self.cursor_pos].is_alphanumeric() {
                    self.cursor_pos += 1;
                }
                if self.cursor_pos < chars.len() {
                    self.cursor_pos += 1;
                }
            }

            // Движение курсора.
            (_, KeyCode::Left) => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            (_, KeyCode::Right) => {
                if self.cursor_pos < self.input_buffer.len() {
                    self.cursor_pos += 1;
                }
            }
            (_, KeyCode::Home) => {
                self.cursor_pos = 0;
            }
            (_, KeyCode::End) => {
                self.cursor_pos = self.input_buffer.len();
            }

            // Ввод текста.
            (_, KeyCode::Char(c)) => {
                // Вставляем символ в правильную позицию
                let mut chars: Vec<char> = self.input_buffer.chars().collect();
                if self.cursor_pos <= chars.len() {
                    chars.insert(self.cursor_pos, c);
                    self.input_buffer = chars.iter().collect();
                    self.cursor_pos += 1;
                }
            }

            // Удаление символа (Backspace).
            (_, KeyCode::Backspace) => {
                if self.cursor_pos > 0 {
                    let mut chars: Vec<char> = self.input_buffer.chars().collect();
                    chars.remove(self.cursor_pos - 1);
                    self.input_buffer = chars.iter().collect();
                    self.cursor_pos -= 1;
                }
            }

            // Удаление символа (Delete).
            (_, KeyCode::Delete) => {
                let mut chars: Vec<char> = self.input_buffer.chars().collect();
                if self.cursor_pos < chars.len() {
                    chars.remove(self.cursor_pos);
                    self.input_buffer = chars.iter().collect();
                }
            }

            _ => {}
        }
    }

    // Отправить сообщение нейросети и обработать полученный результат.
    async fn send_message(&mut self) {
        if !self.input_buffer.trim().is_empty() {
            self.messages.push(format!("Вы: {}", self.input_buffer));

            // Здесь происходит обмен сообщениями с YandexGPT. Первое место для доработки кода!
            let user_said = self.messages.last().unwrap();
            let gpt_answer = self.gpt_client.ask_gpt(user_said).await;

            let gpt_answer_text =
                gpt_answer.unwrap_or_else(|e| format!("Ошибка работы модели: {}", e));

            self.messages.push(format!("GPT: {}", gpt_answer_text));
            self.input_buffer.clear();
            self.cursor_pos = 0;

            // Автоматическая прокрутка к новым сообщениям.
            let visible_lines = 20;
            if self.messages.len() > visible_lines {
                self.scroll_offset = (self.messages.len() - visible_lines) as u16;
            }
        }
    }

    /// Сбросить флаг запущенного приложения (`running`) и остановить приложение.
    fn quit(&mut self) {
        self.running = false;
    }
}
