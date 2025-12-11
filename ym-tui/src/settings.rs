//! Модуль настроек YM.
extern crate directories;
use std::path::PathBuf;

/// Название файла для хранения конфигурации данных "по-умолчанию".
pub const ACCESS_FILE: &str = "access.json";

/// Предоставляет полный путь `PathBut` к `ACCESS_FILE` в режиме разработки.
#[cfg(debug_assertions)]
pub fn access_file_path() -> PathBuf {
    let mut access_path = PathBuf::new();
    access_path.push(env!("CARGO_MANIFEST_DIR"));
    access_path.push(ACCESS_FILE);

    access_path
}

/// Предоставляет полный путь к `ACCESS_FILE` после сборки.
///
/// В текущей реализации файл сохраняется в системный каталог ОС. Перед возвратом ссылки проверяет
/// существование пути, при необходимости создаёт недостающие элементы (каталоги). Наличие самого
/// файла конфигурации не проверяет.
///
/// Linux:
///
/// * /home/пользователь/.config/ym/{ACCESS_FILE}
///
/// Windows:
///
/// * C:\Users\Пользователь\AppData\Roaming\intelligence\ym\{ACCESS_FILE}
#[cfg(not(debug_assertions))]
pub fn access_file_path() -> PathBuf {
    let proj_dirs = directories::ProjectDirs::from("com", "intelligence", "ym")
        .expect("Не удаётся определить проектную директорию");

    if !proj_dirs.config_dir().exists() {
        std::fs::create_dir_all(proj_dirs.config_dir())
            .expect("Не удалось создать директорию для данных");
    }

    proj_dirs.config_dir().join(ACCESS_FILE)
}
