//! Базовые модели для организации взаимодействия с YaGPT.

//! Модели данных для работы с YandexGPT API.

use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use serde_json::json;

pub const URL_API: &str = "https://llm.api.cloud.yandex.net/foundationModels/v1/completion";

/// Структура для опций по обработке запросов.
#[derive(Debug, Clone)]
pub struct GPTOptions {
    /// Название модели. Например, 'yandexgpt/latest'.
    pub model: String,
    /// "Температура" генерации ответа (условная креативность).
    pub temperature: f32,
    /// Максимальное количество токенов (символов) в ответе.
    pub max_tokens: i64,
}

impl Default for GPTOptions {
    fn default() -> Self {
        GPTOptions {
            model: "yandexgpt/latest".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
        }
    }
}

/// Структура для хранения данных авторизации.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct AccessData {
    pub id_catalog: String,
    pub api_key: String,
}

impl Display for AccessData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            f,
            "AccessData | id-catalog: {}, api-key: {}",
            self.mask_key(&self.id_catalog),
            self.mask_key(&self.api_key)
        )
    }
}

impl AccessData {
    pub fn new(id_catalog: String, api_key: String) -> Self {
        Self { id_catalog, api_key }
    }

    pub fn has_data(&self) -> bool {
        !self.id_catalog.trim().is_empty() && !self.api_key.trim().is_empty()
    }

    /// Сохранить информацию из созданного экземпляра в файл с параметрами.
    pub fn save_me(&self, access_file: PathBuf) -> bool {
        let json = json!({
            "id_catalog": self.id_catalog,
            "api_key": self.api_key,
        });

        fs::write(&access_file, json.to_string()).is_ok()
    }

    /// Загрузить информацию из файла параметров (при наличии) и создать на их основе экземпляр.
    pub fn load_it(access_file: PathBuf) -> Self {
        let contents = fs::read_to_string(&access_file)
            .unwrap_or_else(|_| panic!("Файл {} недоступен", access_file.display()));

        serde_json::from_str(&contents)
            .unwrap_or_else(|_| panic!("Ошибка парсинга файла {}", access_file.display()))
    }

    /// Проверить корректность предоставленного id_catalog.
    pub fn validator_id_catalog(input: &str) -> bool {
        !input.is_empty()
    }

    /// Проверить корректность предоставленного api_key.
    pub fn validator_api_key(input: &str) -> bool {
        !input.is_empty()
    }

    fn mask_key(&self, key: &str) -> String {
        const VISIBLE_CHARS: usize = 5;
        if key.len() <= VISIBLE_CHARS {
            return key.to_string();
        }
        format!("{}*****", &key[..VISIBLE_CHARS])
    }
}

// Структуры для ответов API.
#[derive(Deserialize)]
pub struct ApiResponse {
    pub result: ResultField,
}

#[derive(Deserialize)]
pub struct ResultField {
    pub alternatives: Vec<Alternative>,
}

#[derive(Deserialize)]
pub struct Alternative {
    pub message: Message,
}

#[derive(Deserialize)]
pub struct Message {
    pub text: String,
}

// Структура для запросов
#[derive(Serialize)]
pub struct CompletionOptions {
    pub stream: bool,
    pub temperature: f32,
    pub max_tokens: i64,
}

#[derive(Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub text: String,
}

#[derive(Serialize)]
pub struct ApiRequest {
    pub model_uri: String,
    pub completion_options: CompletionOptions,
    pub messages: Vec<ChatMessage>,
}
