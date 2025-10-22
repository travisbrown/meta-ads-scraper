use bounded_static_derive_more::ToStatic;
use chrono::NaiveDate;
use scraper_trail::archive::Archiveable;
use serde_field_attributes::{integer_str, optional_integer_str, optional_integer_str_array};
use std::borrow::Cow;

pub mod library;

#[derive(Clone, Debug, Eq, PartialEq, ToStatic, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields, untagged)]
pub enum Response<'a, D> {
    Success(ResponseSuccess<'a, D>),
    Failure { error: ResponseError<'a> },
}

impl<'a, D> Response<'a, D> {
    #[must_use]
    pub const fn paging(&self) -> Option<&Paging<'a>> {
        match self {
            Self::Success(response) => response.paging.as_ref(),
            Self::Failure { .. } => None,
        }
    }

    pub fn result(&self) -> Result<&[D], ResponseError<'a>> {
        match self {
            Self::Success(response) => Ok(response.data.as_slice()),
            Self::Failure { error } => Err(error.clone()),
        }
    }
}

impl<'r> Archiveable for Response<'r, Ad<'r>> {
    type RequestParams = crate::client::request::Params<'r>;

    fn deserialize_response_field<'de, A: serde::de::MapAccess<'de>>(
        _request_params: &Self::RequestParams,
        map: &mut A,
    ) -> Result<
        Option<(
            scraper_trail::archive::entry::Field,
            scraper_trail::exchange::Response<'de, Self>,
        )>,
        A::Error,
    > {
        map
            .next_entry::<scraper_trail::archive::entry::Field, scraper_trail::exchange::Response<'_, Self>>()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, ToStatic, serde::Deserialize, serde::Serialize)]
pub struct ResponseSuccess<'a, D> {
    pub data: Vec<D>,
    pub paging: Option<Paging<'a>>,
}

#[derive(Clone, Debug, Eq, PartialEq, ToStatic, serde::Deserialize, serde::Serialize)]
pub struct ResponseError<'a> {
    pub message: Cow<'a, str>,
    #[serde(rename = "type")]
    pub error_type: ErrorType,
    pub code: u32,
    pub fbtrace_id: Cow<'a, str>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ErrorType {
    OAuthException,
}

#[derive(Clone, Debug, Eq, PartialEq, ToStatic, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Paging<'a> {
    pub cursors: Cursors<'a>,
    pub next: Cow<'a, str>,
}

#[derive(Clone, Debug, Eq, PartialEq, ToStatic, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Cursors<'a> {
    pub after: Cow<'a, str>,
}

#[derive(Clone, Debug, Eq, PartialEq, ToStatic, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ad<'a> {
    #[serde(with = "integer_str")]
    pub id: u64,
    #[serde(with = "integer_str")]
    pub page_id: u64,
    pub page_name: Cow<'a, str>,
    pub ad_snapshot_url: Cow<'a, str>,
    pub ad_creation_time: NaiveDate,
    pub ad_delivery_start_time: NaiveDate,
    pub ad_delivery_stop_time: Option<NaiveDate>,
    pub ad_creative_bodies: Option<Vec<Cow<'a, str>>>,
    pub ad_creative_link_titles: Option<Vec<Cow<'a, str>>>,
    pub ad_creative_link_captions: Option<Vec<Cow<'a, str>>>,
    pub ad_creative_link_descriptions: Option<Vec<Cow<'a, str>>>,
    pub age_country_gender_reach_breakdown: Option<Vec<CountryAgeGenderBreakdowns<'a>>>,
    pub beneficiary_payers: Option<Vec<BeneficiaryPayer<'a>>>,
    pub eu_total_reach: Option<usize>,
    pub languages: Option<Vec<Cow<'a, str>>>,
    pub publisher_platforms: Option<Vec<PublisherPlatforms>>,
    #[serde(with = "optional_integer_str_array", default)]
    pub target_ages: Option<Vec<usize>>,
    pub target_gender: Option<TargetGender>,
    pub target_locations: Option<Vec<TargetLocation<'a>>>,
    pub total_reach_by_location: Option<Vec<KeyValue<Cow<'a, str>, usize>>>,
    pub impressions: Option<Bounds>,
    pub spend: Option<Bounds>,
    pub br_total_reach: Option<()>,
    pub bylines: Option<Cow<'a, str>>,
    pub currency: Option<Cow<'a, str>>,
    pub delivery_by_region: Option<serde_json::Value>,
    pub demographic_distribution: Option<serde_json::Value>,
    pub estimated_audience_size: Option<Bounds>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    #[serde(with = "integer_str")]
    pub lower_bound: usize,
    #[serde(with = "optional_integer_str", default)]
    pub upper_bound: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, ToStatic, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CountryAgeGenderBreakdowns<'a> {
    pub country: Cow<'a, str>,
    pub age_gender_breakdowns: Vec<AgeGenderBreakdown>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgeGenderBreakdown {
    pub age_range: AgeRange,
    pub female: Option<usize>,
    pub male: Option<usize>,
    pub unknown: Option<usize>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum AgeRange {
    #[serde(rename = "13-17")]
    Range13_17,
    #[serde(rename = "18-24")]
    Range18_24,
    #[serde(rename = "25-34")]
    Range25_34,
    #[serde(rename = "35-44")]
    Range35_44,
    #[serde(rename = "45-54")]
    Range45_54,
    #[serde(rename = "55-64")]
    Range55_64,
    #[serde(rename = "65+")]
    Range65,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, ToStatic, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct BeneficiaryPayer<'a> {
    pub beneficiary: Cow<'a, str>,
    pub payer: Cow<'a, str>,
    pub current: bool,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum PublisherPlatforms {
    #[serde(rename = "audience_network")]
    AudienceNetwork,
    #[serde(rename = "facebook")]
    Facebook,
    #[serde(rename = "instagram")]
    Instagram,
    #[serde(rename = "messenger")]
    Messenger,
    #[serde(rename = "threads")]
    Threads,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum TargetGender {
    Women,
    Men,
    All,
}

#[derive(Clone, Debug, Eq, PartialEq, ToStatic, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct TargetLocation<'a> {
    pub name: Cow<'a, str>,
    pub num_obfuscated: usize,
    #[serde(rename = "type")]
    pub location_type: LocationType,
    pub excluded: bool,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
pub enum LocationType {
    #[serde(rename = "countries")]
    Countries,
    #[serde(rename = "country_groups")]
    CountryGroups,
    #[serde(rename = "regions")]
    Regions,
    #[serde(rename = "multi_city")]
    MultiCity,
    #[serde(rename = "COUNTY")]
    County,
    #[serde(rename = "CITY")]
    City,
    #[serde(rename = "NEIGHBORHOOD")]
    Neighborhood,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, ToStatic, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct KeyValue<K, V> {
    pub key: K,
    pub value: Option<V>,
}
