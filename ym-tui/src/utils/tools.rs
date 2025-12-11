//! Модуль общих универсальных методов обработки.
use std::io::{Error, Write, stdin, stdout};

/// Получить ответ пользователя (yes/no) и вернуть соответствующий логический тип (`true`/`false`).
///
/// Предустановлены значения: "y, n, yes, no, д, да, н, нет".
///
/// **Args**:
///
/// * `answer` – полученный ответ от пользователя.
///
/// * `default` – какой из предустановленных вариантов считать ответом, если пользователь передал
///   пустую строку.
fn yes_or_no(answer: &str, default: &str) -> Result<bool, String> {
    match answer.trim().to_lowercase().as_str() {
        "y" | "yes" | "д" | "да" => Ok(true),
        "n" | "no" | "н" | "нет" => Ok(false),
        "" => Ok(yes_or_no(default, default)?), // Пустой ввод считаем за ответ "по-умолчанию".
        _ => {
            let error = format!(
                "Некорректный ответ. Ожидается 'да', 'нет', получено: {}",
                answer.trim()
            );
            Err(error)
        }
    }
}

/// Задать вопрос пользователю в консоли и получить его ответ. Диспетчер по обработке вопросов
/// с односложными ответами с функцией `yes_or_no`.
///
/// Результатом должно стать возвращение логического типа `true`/`false`, соответствующего ответу
/// пользователя. Действует в цикле до получения успешного результата, ошибки подавляются.
///
/// **Args**:
///
/// * `question` – вопрос для пользователя.
///
/// * `default` – ответ по-умолчанию, если пользователь ничего не ввёл.
pub fn ask_user(question: &str, default: &str) -> bool {
    loop {
        if let Ok(answer) = user_input_with_question(question, false) {
            match yes_or_no(answer.as_str(), default) {
                Ok(result) => return result,
                Err(e) => {
                    println!("{e}");
                    continue;
                }
            }
        } else {
            eprintln!("Некорректный ввод данных. Попробуем снова...");
            continue;
        }
    }
}

/// Получить текстовую строку, введённую пользователем в консоли.
pub fn user_input() -> Result<String, Error> {
    let mut answer = String::new();
    stdin().read_line(&mut answer)?;
    Ok(answer.to_string())
}

/// Получить текстовую строку от пользователя в консоли, но перед этим опубликовать для него
/// информационное сообщение (вопрос, запрос).
///
/// **Args**:
/// * `ask` - текст вопроса к пользователю.
/// * `new_line` - если `True`, для ответа каретка будет переведена на новую строку.
pub fn user_input_with_question(question: &str, new_line: bool) -> Result<String, Error> {
    print!("{}", question);
    if new_line {
        println!();
    } else {
        stdout().flush()?;
    }

    user_input()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yes_answers() {
        let yes_inputs = ["y", "Y", "yes", "YES", "д", "Д", "да", "Да", "ДА"];
        for input in yes_inputs {
            let res = yes_or_no(input, "no");
            assert_eq!(res.unwrap(), true, "Не распознано как 'да': {}", input);
        }
    }

    #[test]
    fn test_no_answers() {
        let no_inputs = ["n", "N", "no", "NO", "н", "Н", "нет", "Нет", "НЕТ"];
        for input in no_inputs {
            let res = yes_or_no(input, "yes");
            assert_eq!(res.unwrap(), false, "Не распознано как 'нет': {}", input);
        }
    }

    #[test]
    fn test_empty_uses_default_yes() {
        let res = yes_or_no("", "yes").unwrap();
        assert_eq!(res, true);
    }

    #[test]
    fn test_empty_uses_default_no() {
        let res = yes_or_no("", "no").unwrap();
        assert_eq!(res, false);
    }

    #[test]
    fn test_invalid_input_returns_error() {
        let res = yes_or_no("maybe", "no");
        assert!(res.is_err());
        let msg = res.err().unwrap();
        assert!(
            msg.contains("Некорректный ответ"),
            "Не то сообщение об ошибке: {}",
            msg
        );
    }

    #[test]
    fn test_default_invalid_recursion_propagates_error() {
        // Проверим, что ошибка из default значения влияет на результат по пустому вводу
        let res = yes_or_no("", "invalid");
        assert!(res.is_err());
    }
}
