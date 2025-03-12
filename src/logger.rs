//! Logger configuration module for SolDag.
use std::io::Write;

use env_logger::{fmt::Color, Builder, Env};
use log::Level;

/// Sets up the application's logging configuration.
///
/// Initializes the logger with custom formatting and color-coded output based on
/// log level. The logger supports different colors for different severity levels:
/// - Green for info messages
/// - Yellow for warnings
/// - Red for errors
/// - Default color for other levels
pub fn setup() {
    Builder::from_env(Env::default().default_filter_or("warn,info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{}{} {}",
                match (record.level(), buf.style().set_bold(true)) {
                    (Level::Warn, style) => style.set_color(Color::Yellow).value("warning"),
                    (Level::Error, style) => style.set_color(Color::Red).value("error"),
                    (Level::Info, style) => style.set_color(Color::Green).value("info"),
                    (level, style) => style.value(level.as_str()),
                },
                buf.style().set_bold(true).value(":"),
                record.args()
            )
        })
        .init();
}
