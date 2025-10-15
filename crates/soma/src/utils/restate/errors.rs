// TAKEN FROM https://github.com/restatedev/restate/blob/main/cli/src/clients/errors.rs

use serde::Deserialize;
use url::Url;

// use restate_cli_util::ui::stylesheet::Style;

// use crate::console::Styled;

#[derive(Deserialize, Debug, Clone)]
pub struct ApiErrorBody {
    pub restate_code: Option<String>,
    pub message: String,
}

impl From<String> for ApiErrorBody {
    fn from(message: String) -> Self {
        Self {
            message,
            restate_code: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub http_status_code: reqwest::StatusCode,
    pub url: Url,
    pub body: ApiErrorBody,
}

impl std::fmt::Display for ApiErrorBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = self.restate_code.as_deref().unwrap_or("<UNKNOWN>");
        write!(f, "{} {}", code, self.message)?;
        Ok(())
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.body)?;
        write!(
            f,
            "  -> Http status code {} at '{}'",
            &self.http_status_code, &self.url,
        )?;
        Ok(())
    }
}

impl std::error::Error for ApiError {}
