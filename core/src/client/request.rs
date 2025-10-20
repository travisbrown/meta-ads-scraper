use crate::version::GraphApiVersion;
use chrono::{DateTime, Utc};
use regex::Regex;
use scraper_trail::request::{Request, params::ParseError};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::LazyLock;

const DOMAIN: &str = "graph.facebook.com";
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

const EXPECTED_URL_MESSAGE: &str = "Meta Ads Archive URL";

static PATH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^/v(\d+)\.(\d+)/ads_archive$").unwrap());
static QUOTED_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^'([^']*)'$").unwrap());
static BRACKETED_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\[([^\]]*)\]$").unwrap());

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

impl FromStr for SearchType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "KEYWORD_UNORDERED" => Ok(Self::KeywordUnordered),
            "KEYWORD_EXACT_PHRASE" => Ok(Self::KeywordExactPhrase),
            _ => Err(()),
        }
    }
}

impl Display for SearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub struct Params<'a> {
    pub access_token: Cow<'a, str>,
    pub unmask_removed_content: bool,
    pub version: crate::version::GraphApiVersion,
    pub terms: Cow<'a, str>,
    pub countries: Vec<Cow<'a, str>>,
    pub search_type: SearchType,
    pub after: Option<Cow<'a, str>>,
}

impl<'a> Params<'a> {
    #[must_use]
    pub fn new(
        access_token: &'a str,
        unmask_removed_content: bool,
        version: crate::version::GraphApiVersion,
        terms: &'a str,
        countries: &'a [String],
        search_type: SearchType,
        after: Option<&'a str>,
    ) -> Self {
        Self {
            access_token: access_token.into(),
            unmask_removed_content,
            version,
            terms: terms.into(),
            countries: countries.iter().map(std::convert::Into::into).collect(),
            search_type,
            after: after.map(std::convert::Into::into),
        }
    }

    const fn error() -> ParseError {
        ParseError::InvalidUrl {
            expected: EXPECTED_URL_MESSAGE,
        }
    }

    pub fn parse_url(url: &url::Url) -> Option<Self> {
        let query_params = url.query_pairs().collect::<HashMap<_, _>>();

        if url.scheme() == "https" && url.domain() == Some(DOMAIN) {
            let path_captures = PATH_RE.captures(url.path())?;
            let version_major = path_captures
                .get(1)
                .and_then(|major| major.as_str().parse().ok())?;
            let version_minor = path_captures
                .get(2)
                .and_then(|minor| minor.as_str().parse().ok())?;

            let access_token = query_params.get("access_token")?.to_string().into();

            let unmask_removed_content = query_params
                .get("unmask_removed_content")
                .and_then(|unmask_removed_content| unmask_removed_content.parse().ok())?;

            let terms = query_params
                .get("search_terms")
                .and_then(|terms| QUOTED_RE.captures(terms))
                .and_then(|terms| terms.get(1))
                .map(|terms| terms.as_str().to_string().into())?;

            let countries = query_params
                .get("ad_reached_countries")
                .and_then(|countries| BRACKETED_RE.captures(countries))
                .and_then(|countries| countries.get(1))
                .and_then(|countries| {
                    countries
                        .as_str()
                        .split(',')
                        .map(|country| {
                            QUOTED_RE
                                .captures(country)
                                .and_then(|country| country.get(1))
                                .map(|country| country.as_str().to_string().into())
                        })
                        .collect()
                })?;

            let search_type = query_params.get("search_type").map_or_else(
                || Some(SearchType::default()),
                |search_type| search_type.parse().ok(),
            )?;

            let after = query_params
                .get("after")
                .map(|after| after.to_string().into());

            Some(Self {
                access_token,
                unmask_removed_content,
                version: GraphApiVersion::new(version_major, version_minor),
                terms,
                countries,
                search_type,
                after,
            })
        } else {
            None
        }
    }
}

impl<'a> scraper_trail::request::params::Params<'a> for Params<'a> {
    fn parse_request(request: &Request<'a>) -> Result<Self, ParseError> {
        Self::parse_url(&request.url).ok_or(Self::error())
    }

    fn build_request(&'a self, timestamp: Option<DateTime<Utc>>) -> Request<'a> {
        let ad_reached_countries = self
            .countries
            .iter()
            .map(|country| format!("'{country}'"))
            .collect::<Vec<_>>();
        let ad_reached_countries = format!("[{}]", ad_reached_countries.join(","));
        let fields = FIELDS.to_vec();
        let fields = fields.join(",");

        let url = format!(
            "{BASE_URL}/v{}/ads_archive?search_terms='{}'&ad_reached_countries={}&fields={}&access_token={}&unmask_removed_content={}{}{}",
            self.version,
            urlencoding::encode(&self.terms),
            urlencoding::encode(&ad_reached_countries),
            urlencoding::encode(&fields),
            self.access_token,
            self.unmask_removed_content,
            if self.search_type == SearchType::default() {
                String::new()
            } else {
                format!("&search_type={}", self.search_type)
            },
            self.after
                .as_ref()
                .map(|after| format!("&after={after}"))
                .unwrap_or_default()
        );

        Request::new::<_, String, String, Vec<(String, String)>, String>(
            url, timestamp, None, None, None,
        )
        .unwrap()
    }
}
