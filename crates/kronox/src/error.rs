#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http request failed")]
    Http(#[from] reqwest::Error),

    #[error("upstream returned status {status}")]
    Upstream { status: u16 },

    #[error("invalid url")]
    Url(#[from] url::ParseError),

    #[error("failed to parse schedule xml: {0}")]
    Xml(#[from] quick_xml::DeError),

    #[error("failed to parse timestamp: {0}")]
    Time(#[from] chrono::ParseError),

    #[error("invalid schools configuration: {0}")]
    Config(String),
}
