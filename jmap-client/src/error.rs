use thiserror::Error;

#[derive(Debug, Error)]
pub enum JmapError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error: {0}")]
    Api(String),

    #[error("Authentication failed")]
    Auth,

    #[error("Server does not support JMAP Mail capability")]
    NoMailCapability,

    #[error("No mail account found")]
    NoAccount,

    #[error("Method error: {type_}{}", description.as_ref().map(|d| format!(": {d}")).unwrap_or_default())]
    MethodError {
        type_: String,
        description: Option<String>,
    },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
