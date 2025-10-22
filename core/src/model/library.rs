use chrono::{DateTime, Utc};
use scraper_trail::archive::Archiveable;
use serde_field_attributes::integer_or_integer_str;
use serde_json::Value;
use std::borrow::Cow;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("Missing markup element")]
    MissingMarkupElement,
    #[error("Multiple markup elements")]
    MultipleMarkupElements,
    #[error("Missing snapshot element")]
    MissingSnapshotElement,
    #[error("Multiple snapshot elements")]
    MultipleSnapshotElements,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct PartialAd<'a> {
    markup_values: Vec<Markup<'a>>,
    deeplink_ad_card_values: Vec<Option<DeeplinkAdCard<'a>>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ad<'a> {
    pub markup: Markup<'a>,
    pub deeplink_ad_card: DeeplinkAdCard<'a>,
}

impl<'a> Ad<'a> {
    #[allow(clippy::missing_panics_doc)]
    pub fn extract(value: &Value) -> Result<Option<Self>, Error> {
        if value.as_array().is_some_and(std::vec::Vec::is_empty) {
            Ok(None)
        } else {
            let mut partial_ad = PartialAd::default();

            Self::extract_rec(value, &mut partial_ad)?;

            if partial_ad.markup_values.is_empty() {
                if partial_ad.deeplink_ad_card_values.len() == 1
                    && partial_ad.deeplink_ad_card_values[0].is_none()
                {
                    Ok(None)
                } else {
                    Err(Error::MissingMarkupElement)
                }
            } else if partial_ad.markup_values.len() > 1 {
                Err(Error::MultipleMarkupElements)
            } else if partial_ad.deeplink_ad_card_values.is_empty() {
                Err(Error::MissingSnapshotElement)
            } else if partial_ad.deeplink_ad_card_values.len() > 1 {
                Err(Error::MultipleSnapshotElements)
            } else {
                // Safe because we've just checked the lengths.
                let markup = partial_ad.markup_values.pop().unwrap();
                let deeplink_ad_card = partial_ad.deeplink_ad_card_values.pop().unwrap();

                deeplink_ad_card.map_or(Err(Error::MissingSnapshotElement), |deeplink_ad_card| {
                    Ok(Some(Self {
                        markup,
                        deeplink_ad_card,
                    }))
                })
            }
        }
    }

    fn extract_rec(value: &Value, current: &mut PartialAd<'a>) -> Result<(), Error> {
        if let Some(as_array) = value.as_array() {
            for value in as_array {
                Self::extract_rec(value, current)?;
            }
        } else if let Some(as_object) = value.as_object() {
            for (key, value) in as_object {
                if key == "markup" {
                    let markup = serde_json::from_value::<MarkupElement<'a>>(value.clone())?;

                    current.markup_values.push(Markup {
                        id: markup.0.0,
                        html: markup.0.1.html,
                    });
                } else if key == "deeplinkAdCard" {
                    let snapshot = serde_json::from_value(value.clone())?;

                    current.deeplink_ad_card_values.push(snapshot);
                } else {
                    Self::extract_rec(value, current)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdResponse<'a> {
    Value(Ad<'a>),
    Empty,
}

impl<'a> From<Option<Ad<'a>>> for AdResponse<'a> {
    fn from(value: Option<Ad<'a>>) -> Self {
        value.map_or(Self::Empty, Self::Value)
    }
}

impl<'a> From<AdResponse<'a>> for Option<Ad<'a>> {
    fn from(value: AdResponse<'a>) -> Self {
        match value {
            AdResponse::Value(ad) => Some(ad),
            AdResponse::Empty => None,
        }
    }
}

impl Archiveable for AdResponse<'_> {
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
        let next = map.next_entry::<scraper_trail::archive::entry::Field, scraper_trail::exchange::Response<'_, serde_json::Value>>()?;

        next.map(|(field, response)| {
            response
                .and_then(|data| Ad::extract(&data).map(std::convert::Into::into))
                .map(|response| (field, response))
        })
        .map_or(Ok(None), |value| {
            value.map_err(serde::de::Error::custom).map(Some)
        })
    }
}

type MarkupElement<'a> = ((Cow<'a, str>, MarkupHtml<'a>, u8, MarkupType),);

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
struct MarkupHtml<'a> {
    #[serde(rename = "__html")]
    html: Cow<'a, str>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
enum MarkupType {
    #[serde(rename = "HTML")]
    Html,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Markup<'a> {
    pub id: Cow<'a, str>,
    pub html: Cow<'a, str>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DeeplinkAdCard<'a> {
    #[serde(rename = "adArchiveID", with = "integer_or_integer_str")]
    pub ad_archive_id: u64,
    pub snapshot: Snapshot<'a>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Snapshot<'a> {
    pub title: Option<Cow<'a, str>>,
    pub link_url: Option<Cow<'a, str>>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub creation_time: DateTime<Utc>,
    #[serde(with = "integer_or_integer_str")]
    pub page_id: u64,
    pub page_name: Cow<'a, str>,
    pub current_page_name: Option<Cow<'a, str>>,
    pub page_profile_picture_url: Cow<'a, str>,
    pub page_entity_type: PageEntityType,
    pub page_is_profile_page: bool,
    pub page_like_count: usize,
    pub instagram_url: Cow<'a, str>,
    pub instagram_handle: Cow<'a, str>,
    pub instagram_actor_name: Cow<'a, str>,
    pub instagram_profile_pic_url: Cow<'a, str>,
    pub videos: Vec<Video<'a>>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PageEntityType {
    #[serde(rename = "person_profile")]
    PersonProfile,
    #[serde(rename = "regular_page")]
    RegularPage,
    #[serde(rename = "ig_ads_identity")]
    IgAdsIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Video<'a> {
    pub video_hd_url: Option<Cow<'a, str>>,
    pub video_sd_url: Cow<'a, str>,
    pub watermarked_video_hd_url: Option<Cow<'a, str>>,
    pub watermarked_video_sd_url: Option<Cow<'a, str>>,
    pub video_preview_image_url: Option<Cow<'a, str>>,
}
