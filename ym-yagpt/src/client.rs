//! Клиент для взаимодействия с YandexGPT API.

use crate::errors::GPTError;
use crate::models::*;
use reqwest::Client;
use serde_json::json;
use std::error::Error;
use std::path::PathBuf;

/// Клиент для текстового общения с языковой моделью.
///
/// Документация: <https://clck.ru/3Qf3nV>
#[derive(Debug)]
pub struct GPTClient {
    pub access: AccessData,
    /// Ссылка на API Yandex Cloud для работы с YandexGPT.
    pub api_url: String,
    pub gpt_options: GPTOptions,
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
    /// Создать новый клиент с настройками по умолчанию.
    pub fn new() -> Self {
        Self::default()
    }

    /// Установить данные авторизации.
    pub fn set_auth(mut self, id_catalog: String, api_key: String) -> Self {
        self.access = AccessData::new(id_catalog, api_key);
        self
    }

    /// Загрузить данные авторизации из файла.
    pub fn load_auth(mut self, access_file: PathBuf) -> Self {
        self.access = AccessData::load_it(access_file);
        self
    }

    /// Изменить URL API.
    pub fn with_new_url(mut self, api_url: String) -> Self {
        self.api_url = api_url;
        self
    }

    /// Изменить модель.
    pub fn with_model(mut self, model: &str) -> Self {
        self.gpt_options.model = model.to_string();
        self
    }

    /// Изменить температуру.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        if !(0.0..=1.0).contains(&temperature) {
            panic!(
                "Температура должна быть между 0 и 1, получено: {}",
                temperature
            );
        }
        self.gpt_options.temperature = temperature;
        self
    }

    /// Изменить максимальное количество токенов.
    pub fn with_max_tokens(mut self, max_tokens: i64) -> Self {
        if max_tokens <= 0 {
            panic!("Количество токенов должно быть больше 0");
        }
        self.gpt_options.max_tokens = max_tokens;
        self
    }

    /// Сформировать URI модели, по шаблону: gpt://{id_catalog}/{model_name}.
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
        if !self.access.has_data() {
            return Err(Box::new(GPTError::InvalidCredential));
        }

        let request_data = self.build_ask_request(prompt);
        let response = self.send_request(&request_data).await?;
        let answer = self.extract_answer(response).await?;

        Ok(answer)
    }

    /// Собрать запрос к API.
    fn build_ask_request(&self, prompt: &str) -> serde_json::Value {
        let message = vec![ChatMessage {
            role: "user".to_string(),
            text: prompt.to_string(),
        }];

        self.build_request(message)
    }

    /// Отправить HTTP-запрос.
    async fn send_request(
        &self,
        body: &serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn Error>> {
        let client = Client::new();

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

    /// Извлечь ответ из JSON.
    async fn extract_answer(&self, response: reqwest::Response) -> Result<String, Box<dyn Error>> {
        let parsed: ApiResponse = response.json().await?;

        parsed
            .result
            .alternatives
            .into_iter()
            .next()
            .map(|alt| alt.message.text)
            .ok_or_else(|| Box::new(GPTError::EmptyResponse) as Box<dyn Error>)
    }

    /// Общение модели с историей сообщений.
    pub async fn chat_with_gpt(
        &self,
        messages: &[String],
    ) -> Result<String, Box<dyn Error>> {

        let request_data = self.build_chat_request(messages);
        let response = self.send_request(&request_data).await?;
        let answer = self.extract_answer(response).await?;

        Ok(answer)

    }

    /// Формирование тела запроса с историей сообщений.
    fn build_chat_request(&self, messages: &[String]) -> serde_json::Value {
        let role = ["assistant", "user"];

        let msg_pack: Vec<ChatMessage> = messages
            .iter()
            // .rev()
            .enumerate()
            .map(|(i, m)| ChatMessage {
                role: role[i % 2].to_string(),
                text: m.clone(),
            })
            .collect();

        self.build_request(msg_pack)
    }

    /// Единый компоновщик тела запроса к языковой модели.
    fn build_request(&self, messages: Vec<ChatMessage>) -> serde_json::Value {
        let completion_options = CompletionOptions {
            stream: false,
            temperature: self.gpt_options.temperature,
            max_tokens: self.gpt_options.max_tokens,
        };

        let api_req = ApiRequest {
            model_uri: self.model_uri(),
            completion_options,
            messages,
        };

        json!(api_req)
    }
}
