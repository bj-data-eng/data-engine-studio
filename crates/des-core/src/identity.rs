pub const APP_DISPLAY_NAME: &str = "Data Engine Studio";
pub const APP_INTERNAL_ID: &str = "data-engine-studio";
pub const APP_PACKAGE_NAME: &str = "data-engine-studio";
pub const APP_PYTHON_MODULE: &str = "data_engine_studio";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn window_title() -> String {
    APP_DISPLAY_NAME.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_identity_is_stable() {
        assert_eq!(APP_DISPLAY_NAME, "Data Engine Studio");
        assert_eq!(APP_INTERNAL_ID, "data-engine-studio");
        assert_eq!(APP_PACKAGE_NAME, "data-engine-studio");
        assert_eq!(APP_PYTHON_MODULE, "data_engine_studio");
        assert_eq!(APP_VERSION, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn window_title_defaults_to_display_name() {
        assert_eq!(window_title(), APP_DISPLAY_NAME);
    }
}
