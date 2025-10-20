use scraper_trail::request::params::Params;
use std::path::PathBuf;

pub mod request;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("HTTP client error")]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("Unexpected pagination URL")]
    UnexpectedPaginationUrl(String),
    #[error("Scraper client error")]
    ScraperClient(#[from] scraper_trail::client::Error),
}

#[derive(Clone)]
pub struct Client {
    underlying: reqwest::Client,
    access_token: String,
    output: Option<PathBuf>,
    unmask_removed_content: bool,
}

impl Client {
    pub fn new<S: Into<String>>(access_token: S, output: Option<PathBuf>) -> Self {
        Self {
            underlying: reqwest::Client::default(),
            access_token: access_token.into(),
            output,
            unmask_removed_content: true,
        }
    }

    pub async fn search(
        &self,
        version: crate::version::GraphApiVersion,
        terms: &str,
        countries: &[String],
        search_type: request::SearchType,
        after: Option<&str>,
        delay: std::time::Duration,
    ) -> Result<Vec<crate::model::Response<'static, crate::model::Ad<'static>>>, Error> {
        let params = request::Params::new(
            &self.access_token,
            self.unmask_removed_content,
            version,
            terms,
            countries,
            search_type,
            after,
        );

        let request = params.build_request(None);
        let exchange = scraper_trail::client::json_send(&self.underlying, request).await?;

        if let Some(base) = &self.output {
            exchange.save_file(base)?;
        }

        let response: crate::model::Response<'static, crate::model::Ad<'static>> =
            serde_json::from_value(exchange.response.data)?;

        let mut responses = vec![response];

        while let Some((after, next)) = responses.last().and_then(|response| {
            response
                .paging()
                .as_ref()
                .map(|paging| (paging.cursors.after.clone(), paging.next.clone()))
        }) {
            tokio::time::sleep(delay).await;
            ::log::info!("Pagination request: {after}");

            let params = next
                .parse()
                .ok()
                .and_then(|url| request::Params::parse_url(&url))
                .ok_or_else(|| Error::UnexpectedPaginationUrl(next.to_string()))?;
            let request = params.build_request(None);
            let exchange = scraper_trail::client::json_send(&self.underlying, request).await?;

            if let Some(base) = &self.output {
                exchange.save_file(base)?;
            }

            let response: crate::model::Response<'static, crate::model::Ad<'static>> =
                serde_json::from_value(exchange.response.data)?;

            responses.push(response);
        }

        Ok(responses)
    }
}
