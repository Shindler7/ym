//! YM — консольный чат-менеджер общения с нейронными сетями, с помощью терминала.
//! Предполагается поддержка только текстового режима генерации.

mod app;
mod cli;
mod settings;
mod utils;

use app::App;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // Первоначально обработка командной строки.
    cli::cli_action();

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}
