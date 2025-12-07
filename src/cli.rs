//! Модуль взаимодействия с командной строкой. Основано на clap.
//!
//! Предполагается, что методы являются стартовыми для приложения. Модуль проводит первичные
//! проверки "здоровья", а также настройку минимально требуемых данных (например, авторизация),
//! до вызова терминала.
use crate::libs::tools::{ask_user, user_input_with_question};
use crate::libs::yagpt::AccessData;
use crate::settings::access_file_path;
use clap::Parser;
use std::process::exit;

/// Структура аргументов командной строки при запуске приложения.
#[derive(Parser)]
#[command(about = "Консольный коммуникатор с YandexGPT")]
#[command(author, version, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Установка данных для работы с нейросетью.
    #[arg(short, long)]
    pub init: bool,
}

/// Обработка аргументов командной строки.
///
/// Подробнее в документации к clap.
pub fn cli_action() {
    let cli = Cli::parse();

    if !cli.init && !is_app_ready() {
        no_access_data()
    }

    if cli.init {
        init_user_data();
        // Обязательная проверка, что файл был создан инициализацией.
        is_app_ready();
    }
}

/// Вывод типового сообщения об отсутствии необходимых данных и рекомендации по действиям.
pub fn no_access_data() -> ! {
    eprintln!(
        "Отсутствует или повреждён файл конфигурации доступа к YandexGPT. \
        Используйте ключ --init для настройки."
    );
    exit(1);
}

/// Проверить готовность приложения к работе.
pub fn is_app_ready() -> bool {
    if !access_file_path().exists() {
        return false;
    }
    true
}

/// Получить от пользователя данные для работы с YandexGPT.
///
/// **Возможны состояния:**
///
/// * первичная регистрация. Создаётся новый файл с данными;
///
/// * перерегистрация. Данные в файле должны быть заменены новыми (перезаписаны).
fn init_user_data() {
    println!("Добро пожаловать! Давайте настроим ваш доступ к YandexGPT.");
    println!("Подробности: https://yandex.cloud/ru/docs/ai-studio/quickstart/yandexgpt");
    println!(
        "{:>10}",
        "и о моделях: https://yandex.cloud/ru/docs/ai-studio/concepts/generation/models"
    );
    println!();

    if access_file_path().exists()
        && !ask_user(
            format!(
                "Данные доступа к YandexGPT предоставлены {}. Перезаписать? (д/Н)",
                access_file_path().display()
            )
            .as_str(),
            "no",
        )
    {
        println!("Данные не изменились");
        exit(0)
    }

    // Настройка параметров.
    // API-key и id_catalog.
    let id_catalog = loop_input_user("ID-Catalog: ", AccessData::validator_id_catalog);
    let api_key = loop_input_user("API-Key: ", AccessData::validator_api_key);

    // Создание конфигурационного файла с данными.
    AccessData::new(id_catalog, api_key).save_it();
}

/// Получить от пользователя данные в командной строке.
///
/// Гарантированно возвращает текстовую строку, а при любых ошибках остаётся в цикле.
///
/// **Args**:
///
/// * `ask` — текст вопроса пользователю. Вызывает метод из модуля `tools` с опцией вопроса
///   и ответа в одной строке.
///
/// * `func_validator` — указатель на функцию, проверяющую полученные данные. Если валидация
///   провалена, пользователю возвращается вопрос.
fn loop_input_user(ask: &str, func_validator: fn(&str) -> bool) -> String {
    loop {
        if let Ok(input) = user_input_with_question(ask, false) {
            let clean_input = input.trim().to_string(); // Очистка от кареток в "хвосте".
            if !func_validator(&clean_input) {
                println!("Некорректная информация. Проверьте формат ввода.");
                continue;
            }
            println!("OK");
            return clean_input;
        } else {
            println!("Неверный формат ввода.");
            continue;
        }
    }
}
