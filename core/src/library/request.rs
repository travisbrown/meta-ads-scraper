use chrono::{DateTime, Utc};
use scraper_trail::request::{Request, params::ParseError};

const DOMAIN: &str = "www.facebook.com";
const EXPECTED_URL_MESSAGE: &str = "Facebook Ads Library URL";

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Params {
    pub ad_id: u64,
}

impl Params {
    #[must_use]
    pub const fn new(ad_id: u64) -> Self {
        Self { ad_id }
    }

    const fn error() -> ParseError {
        ParseError::InvalidUrl {
            expected: EXPECTED_URL_MESSAGE,
        }
    }

    #[must_use]
    pub fn parse_url(url: &url::Url) -> Option<Self> {
        if url.scheme() == "https" && url.domain() == Some(DOMAIN) && url.path() == "/ads/library/"
        {
            url.query()?
                .strip_prefix("id=")
                .and_then(|id| id.parse().ok().map(Self::new))
        } else {
            None
        }
    }
}

impl<'a> scraper_trail::request::params::Params<'a> for Params {
    fn parse_request(request: &Request<'a>) -> Result<Self, ParseError> {
        Self::parse_url(&request.url).ok_or(Self::error())
    }

    fn build_request(&'a self, timestamp: Option<DateTime<Utc>>) -> Request<'a> {
        let url = format!("https://www.facebook.com/ads/library/?id={}", self.ad_id);

        Request::new::<_, String, String, Vec<(String, String)>, String>(
            url, timestamp, None, None, None,
        )
        .unwrap()
    }
}
