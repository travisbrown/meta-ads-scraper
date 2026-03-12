use chrono::{DateTime, Utc};
use scraper_trail::archive::Archiveable;
use serde_field_attributes::integer_str;
use serde_json::Value;
use std::borrow::Cow;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("Missing ad library field")]
    MissingAdLibraryMain,
}

#[derive(serde::Deserialize)]
pub struct AdLibraryResponse<'a> {
    search_results_connection: RawSearchResultsConnection<'a>,
    deeplink_ad_archive_result: RawDeeplinkAdArchiveResult<'a>,
}

#[derive(serde::Deserialize)]
struct RawDeeplinkAdArchiveResult<'a> {
    deeplink_ad_archive: Option<Ad<'a>>,
}

#[derive(serde::Deserialize)]
struct RawSearchResultsConnection<'a> {
    count: u64,
    page_info: PageInfo<'a>,
    edges: Vec<RawEdge<'a>>,
}

#[derive(serde::Deserialize)]
struct RawEdge<'a> {
    node: RawNode<'a>,
}

#[derive(serde::Deserialize)]
struct RawNode<'a> {
    collated_results: Vec<Ad<'a>>,
}

/// The search results extracted from the library page's embedded JSON scripts.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct SearchResults<'a> {
    /// Total number of matching ads across all pages.
    pub count: u64,
    /// Pagination information for fetching the next page.
    pub page_info: PageInfo<'a>,
    /// The ads returned on this page, flattened from all edges and collated results.
    pub ads: Vec<Ad<'a>>,
}

impl<'a> AdLibraryResponse<'a> {
    /// The ad may be missing (in which case `no_result_reason` will be `NOT_IN_LIBRARY`).
    #[must_use]
    pub const fn ad(&self) -> Option<&Ad<'a>> {
        self.deeplink_ad_archive_result.deeplink_ad_archive.as_ref()
    }

    #[must_use]
    pub fn search_results(&self) -> SearchResults<'a> {
        SearchResults {
            count: self.search_results_connection.count,
            page_info: self.search_results_connection.page_info.clone(),
            ads: self
                .search_results_connection
                .edges
                .iter()
                .flat_map(|edge| edge.node.collated_results.iter().cloned())
                .collect(),
        }
    }

    pub fn extract(value: &Value) -> Result<Option<Self>, Error> {
        let mut response: Option<AdLibraryResponse<'a>> = None;

        Self::extract_rec(value, &mut response)?;

        Ok(response)
    }

    fn extract_rec(value: &Value, out: &mut Option<Self>) -> Result<(), Error> {
        if out.is_none() {
            if let Some(array) = value.as_array() {
                for item in array {
                    Self::extract_rec(item, out)?;

                    if out.is_some() {
                        break;
                    }
                }
            } else if let Some(object) = value.as_object() {
                for (key, value) in object {
                    if key == "ad_library_main"
                        && let Ok(response) = serde_json::from_value::<Self>(value.clone())
                    {
                        *out = Some(response);

                        break;
                    }

                    Self::extract_rec(value, out)?;

                    if out.is_some() {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Archiveable for AdLibraryResponse<'_> {
    type RequestParams = crate::library::request::Params;

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
        let next = map.next_entry::<
            scraper_trail::archive::entry::Field,
            scraper_trail::exchange::Response<'_, Value>,
        >()?;

        next.map(|(field, response)| {
            response
                .and_then(|data| {
                    Self::extract(&data)
                        .and_then(|response| response.ok_or(Error::MissingAdLibraryMain))
                })
                .map(|response| (field, response))
        })
        .map_or(Ok(None), |value| {
            value.map_err(serde::de::Error::custom).map(Some)
        })
    }
}

/// Pagination cursor returned alongside a page of search results.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct PageInfo<'a> {
    pub end_cursor: Cow<'a, str>,
    pub has_next_page: bool,
}

/// A single ad entry from the library's `collated_results` array.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct Ad<'a> {
    #[serde(with = "integer_str")]
    pub ad_archive_id: u64,
    /// Non-null when multiple creatives share the same ad.
    pub collation_count: Option<u64>,
    pub collation_id: Option<Cow<'a, str>>,
    #[serde(with = "integer_str")]
    pub page_id: u64,
    pub page_name: Cow<'a, str>,
    pub page_is_deleted: bool,
    pub snapshot: Snapshot<'a>,
    pub is_active: bool,
    pub has_user_reported: bool,
    pub report_count: Option<u64>,
    /// Always an empty array in current responses.
    pub menu_items: Vec<Value>,
    pub state_media_run_label: Option<Cow<'a, str>>,
    pub impressions_with_index: ImpressionsWithIndex<'a>,
    pub gated_type: Cow<'a, str>,
    pub categories: Vec<Cow<'a, str>>,
    pub is_aaa_eligible: bool,
    pub contains_digital_created_media: bool,
    pub reach_estimate: Option<Value>,
    pub currency: Cow<'a, str>,
    pub spend: Option<Value>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub end_date: DateTime<Utc>,
    pub publisher_platform: Vec<PublisherPlatform>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub start_date: DateTime<Utc>,
    pub contains_sensitive_content: bool,
    pub total_active_time: Option<Value>,
    pub regional_regulation_data: RegionalRegulationData,
    pub hide_data_status: Cow<'a, str>,
    pub fev_info: Option<Cow<'a, str>>,
    pub ad_id: Option<Cow<'a, str>>,
}

/// Impression count and relative popularity index for an ad.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImpressionsWithIndex<'a> {
    pub impressions_text: Option<Cow<'a, str>>,
    /// Relative popularity index; sometimes `-1` when no impression data is available.
    pub impressions_index: Option<i64>,
}

/// Country-level regulatory metadata attached to each ad.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RegionalRegulationData {
    pub finserv: FinservData,
    pub tw_anti_scam: TwAntiScamData,
}

/// Financial-services regulatory flags.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct FinservData {
    pub is_deemed_finserv: bool,
    pub is_limited_delivery: bool,
}

/// Taiwan anti-scam regulatory flags.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct TwAntiScamData {
    pub is_limited_delivery: bool,
}

/// The creative content of an ad, including its media, copy, and page metadata.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Snapshot<'a> {
    pub branded_content: Option<Value>,
    #[serde(with = "integer_str")]
    pub page_id: u64,
    pub page_is_deleted: bool,
    pub page_profile_uri: Cow<'a, str>,
    pub root_reshared_post: Option<Value>,
    pub byline: Option<Cow<'a, str>>,
    pub disclaimer_label: Option<Value>,
    pub page_name: Cow<'a, str>,
    pub page_profile_picture_url: Cow<'a, str>,
    pub event: Option<Value>,
    pub caption: Option<Cow<'a, str>>,
    pub cta_text: Option<Cow<'a, str>>,
    pub cards: Vec<Card<'a>>,
    pub body: Option<Body<'a>>,
    pub cta_type: Option<CtaType>,
    pub display_format: Option<DisplayFormat>,
    pub link_description: Option<Cow<'a, str>>,
    pub link_url: Option<Cow<'a, str>>,
    pub images: Vec<Image<'a>>,
    pub page_categories: Vec<Cow<'a, str>>,
    pub page_like_count: Option<usize>,
    pub title: Option<Cow<'a, str>>,
    pub videos: Vec<super::Video<'a>>,
    pub is_reshared: Option<bool>,
    pub extra_links: Vec<Value>,
    pub extra_texts: Vec<Value>,
    pub extra_images: Vec<Value>,
    pub extra_videos: Vec<Value>,
    pub country_iso_code: Option<Cow<'a, str>>,
    pub brazil_tax_id: Option<Cow<'a, str>>,
    pub additional_info: Option<Value>,
    pub ec_certificates: Vec<Value>,
}

/// The primary text body of a snapshot.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Body<'a> {
    pub text: Cow<'a, str>,
}

/// A single card in a carousel or dynamic product ad.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Card<'a> {
    pub body: Option<Cow<'a, str>>,
    pub cta_type: Option<CtaType>,
    pub caption: Option<Cow<'a, str>>,
    pub link_description: Option<Cow<'a, str>>,
    pub link_url: Option<Cow<'a, str>>,
    pub title: Option<Cow<'a, str>>,
    pub cta_text: Option<Cow<'a, str>>,
    pub video_hd_url: Option<Cow<'a, str>>,
    pub video_preview_image_url: Option<Cow<'a, str>>,
    pub video_sd_url: Option<Cow<'a, str>>,
    pub watermarked_video_hd_url: Option<Cow<'a, str>>,
    pub watermarked_video_sd_url: Option<Cow<'a, str>>,
    pub image_crops: Vec<Value>,
    pub original_image_url: Option<Cow<'a, str>>,
    pub resized_image_url: Option<Cow<'a, str>>,
    pub watermarked_resized_image_url: Option<Cow<'a, str>>,
}

/// A standalone image asset attached to a snapshot.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Image<'a> {
    pub image_crops: Vec<Value>,
    pub original_image_url: Cow<'a, str>,
    pub resized_image_url: Cow<'a, str>,
    pub watermarked_resized_image_url: Cow<'a, str>,
}

/// The platforms on which an ad ran.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
pub enum PublisherPlatform {
    #[serde(rename = "FACEBOOK")]
    Facebook,
    #[serde(rename = "INSTAGRAM")]
    Instagram,
    #[serde(rename = "AUDIENCE_NETWORK")]
    AudienceNetwork,
    #[serde(rename = "MESSENGER")]
    Messenger,
    #[serde(other)]
    Unknown,
}

/// The creative format of the ad.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
pub enum DisplayFormat {
    #[serde(rename = "IMAGE")]
    Image,
    #[serde(rename = "VIDEO")]
    Video,
    /// Dynamic product ad.
    #[serde(rename = "DPA")]
    Dpa,
    /// Dynamic creative optimisation.
    #[serde(rename = "DCO")]
    Dco,
    #[serde(other)]
    Unknown,
}

/// The call-to-action type for a snapshot or card.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
pub enum CtaType {
    #[serde(rename = "DOWNLOAD")]
    Download,
    #[serde(rename = "SHOP_NOW")]
    ShopNow,
    #[serde(rename = "LEARN_MORE")]
    LearnMore,
    #[serde(rename = "SIGN_UP")]
    SignUp,
    #[serde(rename = "BUY_NOW")]
    BuyNow,
    #[serde(rename = "BOOK_TRAVEL")]
    BookTravel,
    #[serde(rename = "LIKE_PAGE")]
    LikePage,
    #[serde(rename = "VIEW_INSTAGRAM_PROFILE")]
    ViewInstagramProfile,
    #[serde(rename = "PLAY_GAME")]
    PlayGame,
    #[serde(other)]
    Unknown,
}
