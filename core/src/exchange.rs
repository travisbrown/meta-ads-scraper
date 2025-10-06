use crate::model::attributes::timestamp_millis_str;
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Header value error")]
    RequestHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Header value error")]
    ResponseHeaderValue(#[from] reqwest::header::ToStrError),
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Exchange<T> {
    pub request: Request,
    pub response: Response<T>,
}

impl<T> Exchange<T> {
    pub const fn new(
        url: String,
        timestamp: DateTime<Utc>,
        request_headers: IndexMap<String, String>,
        request_body: Option<String>,
        response_headers: IndexMap<String, String>,
        data: T,
    ) -> Self {
        Self {
            request: Request {
                url,
                timestamp,
                headers: request_headers,
                body: request_body,
            },
            response: Response {
                headers: response_headers,
                data,
            },
        }
    }
}

impl<T: serde::ser::Serialize> Exchange<T> {
    pub fn write<P: AsRef<Path>>(&self, base: P) -> Result<PathBuf, std::io::Error> {
        std::fs::create_dir_all(&base)?;

        let output_path = base.as_ref().join(format!(
            "{}.json",
            self.request.timestamp.timestamp_millis()
        ));

        std::fs::write(&output_path, serde_json::json!(self).to_string())?;

        Ok(output_path)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Request {
    pub url: String,
    #[serde(rename = "timestamp_ms", with = "timestamp_millis_str")]
    pub timestamp: DateTime<Utc>,
    pub headers: IndexMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Response<T> {
    pub headers: IndexMap<String, String>,
    pub data: T,
}

pub fn response_headers(
    response_headers: &reqwest::header::HeaderMap,
) -> Result<IndexMap<String, String>, Error> {
    response_headers
        .iter()
        .map(|(name, value)| {
            value
                .to_str()
                .map_err(Error::from)
                .map(|value| (name.as_str().to_string(), value.to_string()))
        })
        .collect::<Result<indexmap::IndexMap<_, _>, Error>>()
}
