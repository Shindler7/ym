//! Модуль взаимодействия с языковой моделью.
//!
//! Инициализация и организация обмена сообщениями.

#![allow(dead_code)]
#![allow(unused_variables)]

use crate::errors::GPTError;
use crate::settings::{access_file_path, URL_API};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use std::fmt::Display;
use std::fs;

/// Структура для опций по обработке запросов (температура, токены, название модели).
#[derive(Debug)]
pub struct GPTOptions {
    /// Название модели. Например, 'yandexgpt/latest'.
    model: String,
    /// "Температура" генерации ответа (условная креативность).
    temperature: f32,
    /// Максимальное количество токенов (символов) в ответе.
    max_tokens: i64,
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

/// Структура для хранения данных авторизации при обращении к языковой модели.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AccessData {
    id_catalog: String,
    api_key: String,
}

// Деактивировано по предложению Clippy, т.к. поведение действительно дефолтное.
// impl Default for AccessData {
//     fn default() -> Self {
//         // Self::load_it()  // До обновления подхода с паникой.
//         Self {
//             id_catalog: String::new(),
//             api_key: String::new(),
//         }
//     }
// }

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

/// Набор методов для обслуживания AccessData.
impl AccessData {
    /// Создать экземпляр AccessData на основе полученных данных.
    pub fn new(id_catalog: String, api_key: String) -> AccessData {
        Self {
            id_catalog,
            api_key,
        }
    }

    /// Сохранить информацию из созданного экземпляра в файл с параметрами (ACCESS_FILE).
    pub fn save_it(&self) -> bool {
        let access_file = access_file_path();

        let json = json!({
            "id_catalog": self.id_catalog,
            "api_key": self.api_key,
        });

        let save_it = fs::write(&access_file, json.to_string());
        match save_it {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Ошибка сохранения {}: {}", access_file.display(), e);
                false
            }
        }
    }

    /// Загрузить информацию из файла параметров (при наличии) и создать на их основе экземпляр
    /// `AccessData`.
    pub fn load_it() -> Self {
        let access_file = access_file_path();

        let json = fs::read_to_string(&access_file);
        match json {
            Ok(contents) => {
                let parsed: Result<AccessData, serde_json::Error> = serde_json::from_str(&contents);
                match parsed {
                    Ok(data) => data,
                    Err(_) => {
                        panic!("Ошибка при считывании файла {}", &access_file.display());
                    }
                }
            }
            Err(_) => {
                panic!(
                    "Файл {} недоступен или не существует",
                    &access_file.display()
                );
            }
        }
    }

    /// Проверить корректность предоставленного id_catalog.
    pub fn validator_id_catalog(input: &str) -> bool {
        !input.is_empty()
    }

    /// Проверить корректность предоставленного api_key.
    pub fn validator_api_key(input: &str) -> bool {
        !input.is_empty()
    }

    /// Локальная вспомогательная функция: заменяет часть символов в строке на "звёздочки" (`*`).
    fn mask_key(&self, key: &str) -> String {
        const VISIBLE_CHARS: usize = 5;
        if key.len() <= VISIBLE_CHARS {
            return key.to_string();
        }
        let visible = &key[..VISIBLE_CHARS];
        format!("{}*****", visible)
    }
}

/// Структура содержащая набор необходимых параметров для текстового общения с языковой моделью.
///
/// Документация: <https://clck.ru/3Qf3nV>
#[derive(Debug)]
pub struct GPTClient {
    access: AccessData,
    /// Ссылка на API YandexGPT.
    api_url: String,
    gpt_options: GPTOptions,
}

impl Default for GPTClient {
    fn default() -> Self {
        Self {
            access: AccessData::default(),
            api_url: URL_API.to_string(),
            gpt_options: GPTOptions::default(),
        }
    }
}

/// Набор методов `GPTClient` позволяющих собрать индивидуальную схему для запроса и произвести
/// сам запрос.
///
/// При сборке можно изменить значения "по-умолчанию": имя модели, температуру, токены, а также
/// предоставить новые данные авторизации, заменив подгруженные из файла.
///
/// **Пример**
///
/// ```Rust
/// let model = GPTClient::new()
///     .with_temperature((0.5);
/// let result = model.ask_gpt("Каким будет сегодня наш день?");
/// ```
impl GPTClient {
    /// Создать экземпляр диспетчера на основе предустановленных данных.
    pub fn new() -> Self {
        Self {
            access: AccessData::load_it(),
            ..Self::default()
        }
    }

    /// Изменить данные авторизации при обращении к языковой модели.
    pub fn with_new_auth(mut self, access_data: AccessData) -> Self {
        self.access = access_data;
        self
    }

    /// Изменить ссылку доступа к языковым моделям YandexGPT.
    ///
    /// Обычно ссылка не изменяется, и замена может нарушить взаимодействие с нейросетью.
    pub fn with_new_url(mut self, api_url: String) -> Self {
        self.api_url = api_url;
        self
    }

    /// Изменить используемую модель.
    pub fn with_model(mut self, model: &str) -> Self {
        self.gpt_options.model = model.to_string();
        self
    }

    /// Изменить значение температуры генерации.
    ///
    /// Согласно документации, температура YandexGPT должна быть в пределах 0..=1. Значение
    /// по-умолчанию 0.3. В следующих сборках паника может быть заменена на обрабатываемое
    /// поведение.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        if !(0.0..=1.0).contains(&temperature) {
            panic!(
                "Недопустимая температура: {}. Значение должно быть между 0 и 1",
                temperature
            );
        }
        self.gpt_options.temperature = temperature;
        self
    }

    /// Изменить значение выдаваемых токенов.
    ///
    /// Значение должно быть больше 0. Максимальное число определяется параметрами используемой
    /// модели. Например, для YandexGPT Pro 5 это 32768. Паника в следующих сборках может быть
    /// заменена на обрабатываемое поведение.
    pub fn with_max_tokens(mut self, max_tokens: i64) -> Self {
        if max_tokens <= 0 {
            panic!("Количество токенов должно быть больше 0")
        }

        self.gpt_options.max_tokens = max_tokens;
        self
    }

    /// Сформировать model URI, по шаблону: gpt://{id_catalog}/{model_name}.
    fn model_uri(&self) -> String {
        format!(
            "gpt://{}/{}",
            self.access.id_catalog, self.gpt_options.model
        )
    }

    /// Сделать запрос к языковой модели.
    ///
    /// Сборка запроса происходит в отдельных методах. Возвращает ответ модели, либо ошибку,
    /// связанную с сетью или авторизацией.
    ///
    /// **Пример**
    ///
    /// ```rust,no_run
    /// let result = client.ask_gpt("Привет, как ты?").await?;
    /// ```
    pub async fn ask_gpt(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        let request_data = self.build_request(prompt);
        let response = self.send_request(&request_data).await?;
        let answer = self.extract_answer(response).await?;

        Ok(answer)
    }

    /// Сделать запрос с моделью в рамках чата с историей сообщений.
    pub fn chat_with_gpt(
        &self,
        prompt: &str,
        messages: &[String],
    ) -> Result<String, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let model = "";

        Ok(format!(
            "Запрос: {}, модель: {}, температура: {}",
            prompt, self.gpt_options.model, self.gpt_options.temperature
        ))
    }

    /// Сформировать проект запроса к API.
    fn build_request(&self, prompt: &str) -> serde_json::Value {
        json!({
            "model_uri": self.model_uri(),
            "completion_options": {
                "stream": false,
                "temperature": self.gpt_options.temperature,
                "max_tokens": self.gpt_options.max_tokens
            },
            "messages": [
                {
                    "role": "user",
                    "text": prompt
                }
            ]
        })
    }

    /// Организовать запрос к API.
    ///
    /// При запросе формируется заголовок (headers). Остальные данные передаются методу.
    async fn send_request(
        &self,
        body: &serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn Error>> {
        let client = reqwest::Client::new();

        let response = client
            .post(&self.api_url)
            .header("Authorization", format!("Api-Key {}", self.access.api_key))
            .header("Content-Type", "application/json")
            .header("User-Agent", "YM001")
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            let code = status.as_u16() as i32;

            let err = if status == reqwest::StatusCode::UNAUTHORIZED {
                GPTError::InvalidCredential
            } else {
                GPTError::APIError {
                    code,
                    description: error_text,
                }
            };

            return Err(Box::new(err));
        }

        Ok(response)
    }

    /// Распаковка ответа языковой модели.
    async fn extract_answer(&self, response: reqwest::Response) -> Result<String, Box<dyn Error>> {
        #[derive(Deserialize)]
        struct Response {
            result: ResultField,
        }

        #[derive(Deserialize)]
        struct ResultField {
            alternatives: Vec<Alternative>,
        }

        #[derive(Deserialize)]
        struct Alternative {
            message: Message,
        }

        #[derive(Deserialize)]
        struct Message {
            text: String,
        }

        let parsed: Response = response.json().await?;
        let result = parsed
            .result
            .alternatives
            .into_iter()
            .next()
            .map(|alt| alt.message.text)
            .ok_or(GPTError::EmptyResponse)?;

        Ok(result)
    }
}
