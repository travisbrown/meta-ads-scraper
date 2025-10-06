use chrono::Utc;
use indexmap::IndexMap;
use scraper::Selector;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

const DEFAULT_USER_AGENT: &str = "curl/8.16.0";

static JSON_SCRIPT_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(r#"script[type="application/json"]"#).unwrap());

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("HTTP client error")]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("Exchange parsing error")]
    Exchange(#[from] crate::exchange::Error),
}

#[derive(Clone)]
pub struct Client {
    underlying: reqwest::Client,
    output: Option<PathBuf>,
}

impl Client {
    pub fn new<P: AsRef<Path>, S: Into<String>>(
        output: Option<P>,
        user_agent: Option<S>,
    ) -> Result<Self, Error> {
        let user_agent: Option<String> = user_agent.map(std::convert::Into::into);

        let client = reqwest::ClientBuilder::new()
            .user_agent(user_agent.as_deref().unwrap_or(DEFAULT_USER_AGENT));

        Ok(Self {
            underlying: client.build()?,
            output: output.map(|path| path.as_ref().to_path_buf()),
        })
    }

    pub async fn app(&self, id: u64) -> Result<String, Error> {
        let url = format!("https://www.facebook.com/ads/library/?id={id}");

        let timestamp = Utc::now();
        let response = self.underlying.get(&url).send().await?;
        let response_headers = crate::exchange::response_headers(response.headers())?;
        let body = response.text().await?;
        let html = scraper::html::Html::parse_document(&body);
        let json_scripts = html
            .select(&JSON_SCRIPT_SEL)
            .map(|element| serde_json::from_str::<serde_json::Value>(&element.inner_html()))
            .collect::<Result<Vec<_>, _>>()?;

        let exchange = crate::exchange::Exchange::new(
            url,
            timestamp,
            IndexMap::default(),
            None,
            response_headers,
            serde_json::json!(json_scripts),
        );

        if let Some(base) = &self.output {
            exchange.write(base)?;
        }

        Ok(body)
    }
}
