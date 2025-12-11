pub mod client;
pub mod errors;
pub mod models;

// Реэкспорт наиболее важных типов для удобства.
pub use client::GPTClient;
pub use models::{AccessData, ApiRequest, ChatMessage, CompletionOptions, GPTOptions, URL_API};

// Константы для часто используемых моделей
pub const MODEL_YANDEXGPT_LATEST: &str = "yandexgpt/latest";
pub const MODEL_YANDEXGPT_PRO: &str = "yandexgpt-pro";
