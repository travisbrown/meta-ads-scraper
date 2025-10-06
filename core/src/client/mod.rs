use chrono::Utc;
use indexmap::IndexMap;
use std::{fmt::Display, path::PathBuf};

const BASE_URL: &str = "https://graph.facebook.com";
const FIELDS: &[&str] = &[
    "id",
    "page_id",
    "page_name",
    "ad_snapshot_url",
    "ad_creation_time",
    "ad_delivery_start_time",
    "ad_delivery_stop_time",
    "ad_creative_bodies",
    "ad_creative_link_titles",
    "ad_creative_link_captions",
    "ad_creative_link_descriptions",
    "age_country_gender_reach_breakdown",
    "beneficiary_payers",
    "eu_total_reach",
    "languages",
    "publisher_platforms",
    "target_ages",
    "target_gender",
    "target_locations",
    "total_reach_by_location",
    "br_total_reach",
    "bylines",
    "currency",
    "delivery_by_region",
    "demographic_distribution",
    "estimated_audience_size",
    "impressions",
    "spend",
];

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

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub enum SearchType {
    #[default]
    KeywordUnordered,
    KeywordExactPhrase,
}

impl SearchType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::KeywordUnordered => "KEYWORD_UNORDERED",
            Self::KeywordExactPhrase => "KEYWORD_EXACT_PHRASE",
        }
    }
}

impl Display for SearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
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
        search_type: SearchType,
        after: Option<&str>,
        delay: std::time::Duration,
    ) -> Result<Vec<crate::model::Response<'static, crate::model::Ad<'static>>>, Error> {
        let ad_reached_countries = countries
            .iter()
            .map(|country| format!("'{country}'"))
            .collect::<Vec<_>>();
        let ad_reached_countries = format!("[{}]", ad_reached_countries.join(","));
        let fields = FIELDS.to_vec();
        let fields = fields.join(",");

        let url = format!(
            "{BASE_URL}/v{}/ads_archive?search_terms='{}'&ad_reached_countries={}&fields={}&access_token={}&unmask_removed_content={}{}{}",
            version,
            urlencoding::encode(terms),
            urlencoding::encode(&ad_reached_countries),
            urlencoding::encode(&fields),
            self.access_token,
            self.unmask_removed_content,
            if search_type == SearchType::default() {
                String::new()
            } else {
                format!("&search_type={search_type}")
            },
            after
                .map(|after| format!("&after={after}"))
                .unwrap_or_default()
        );

        let timestamp = Utc::now();

        let response = self.underlying.get(&url).send().await?;
        let response_headers = crate::exchange::response_headers(response.headers())?;
        let data = response.json::<serde_json::Value>().await?;

        let exchange = crate::exchange::Exchange::new(
            url,
            timestamp,
            IndexMap::default(),
            None,
            response_headers,
            data,
        );

        if let Some(base) = &self.output {
            exchange.write(base)?;
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

            let url = next.to_string();
            let timestamp = Utc::now();

            let response = self.underlying.get(&url).send().await?;
            let response_headers = crate::exchange::response_headers(response.headers())?;
            let data = response.json::<serde_json::Value>().await?;

            let exchange = crate::exchange::Exchange::new(
                url,
                timestamp,
                IndexMap::default(),
                None,
                response_headers,
                data,
            );

            if let Some(base) = &self.output {
                exchange.write(base)?;
            }

            let response: crate::model::Response<'static, crate::model::Ad<'static>> =
                serde_json::from_value(exchange.response.data)?;

            responses.push(response);
        }

        Ok(responses)
    }
}
