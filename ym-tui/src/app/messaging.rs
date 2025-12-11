//! Работа с сообщениями и взаимодействие с YandexGPT API.

use super::core::App;

/// Отправить сообщение нейросети и обработать полученный результат.
pub async fn send_message_to_gpt(app: &mut App) {
    if !app.input_buffer.trim().is_empty() {
        // Добавляем сообщение пользователя в историю
        app.messages.push(format!("Вы: {}", app.input_buffer));

        let gpt_answer = app.gpt_client.chat_with_gpt(&app.messages)
            .await
            .unwrap_or_else(
                |err| {format!("Ошибка ответа модели: {err}")}
            );

        // Добавляем ответ GPT в историю
        app.messages.push(gpt_answer);

        // Очищаем буфер ввода и сбрасываем курсор
        app.input_buffer.clear();
        app.cursor_pos = 0;

        // Автоматическая прокрутка к новым сообщениям.
        update_scroll_offset(app);
    }
}

/// Обновить смещение скролла для показа новых сообщений.
fn update_scroll_offset(app: &mut App) {
    const VISIBLE_LINES: usize = 20;
    if app.messages.len() > VISIBLE_LINES {
        app.scroll_offset = (app.messages.len() - VISIBLE_LINES) as u16;
    }
}

/// Добавить системное сообщение в историю.
pub fn add_system_message(app: &mut App, message: &str) {
    app.messages.push(format!("Система: {}", message));
}

/// Очистить историю сообщений.
pub fn clear_messages(app: &mut App) {
    app.messages.clear();
    app.messages.push("YandexGPT готов к диалогу.".to_string());
    app.scroll_offset = 0;
}
