use serde::{Deserialize, Serialize};
use std::fmt;

pub const APP_NAME: &str = "Data Engine Studio";
pub const APP_INTERNAL_ID: &str = "data-engine-studio";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

impl AppInfo {
    pub fn current() -> Self {
        Self {
            name: APP_NAME.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
}

impl Diagnostic {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Info,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StudioError {
    message: String,
}

impl StudioError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for StudioError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for StudioError {}

pub type StudioResult<T> = Result<T, StudioError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_app_info_uses_workspace_package_version() {
        let info = AppInfo::current();

        assert_eq!(info.name, APP_NAME);
        assert_eq!(info.version, "0.1.0");
    }

    #[test]
    fn diagnostic_info_preserves_message() {
        let diagnostic = Diagnostic::info("ready");

        assert_eq!(diagnostic.severity, DiagnosticSeverity::Info);
        assert_eq!(diagnostic.message, "ready");
    }
}
