//! Модуль собственных ошибок приложения.
use std::fmt::{Display, Formatter};

/// Перечисление ошибок, персонализированных для взаимодействия с нейросетью.
#[derive(Debug)]
#[allow(dead_code)]
pub enum GPTError {
    /// Пустой ответ от языковой модели. Кроме случаев, если есть ошибочный запрос или ошибка сети.
    EmptyResponse,
    /// Некорректные данные авторизации.
    InvalidCredential,
    /// Прочитанная ошибка от API (чаще всего при HTTP = 400 — 499).
    APIError { code: i32, description: String },
    /// Неправильная конфигурация для запроса к API.
    ConfigError { description: String },
}

impl std::error::Error for GPTError {}

impl Display for GPTError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GPTError::EmptyResponse => {
                write!(f, "Получен пустой ответ от API")
            }
            GPTError::InvalidCredential => {
                write!(f, "Данные для авторизации неверные или устарели")
            }
            GPTError::APIError { code, description } => {
                write!(f, "Некорректный запрос к API: {}, {}", code, description)
            }
            GPTError::ConfigError { description } => {
                write!(f, "Некорректная конфигурация запроса GPT: {}", description)
            }
        }
    }
}
