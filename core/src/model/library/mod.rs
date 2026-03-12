pub mod v1;
pub mod v2;

use std::borrow::Cow;

/// A video asset, shared between the library models.
///
/// All URL fields are optional because availability varies by ad format and version.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Video<'a> {
    pub video_hd_url: Option<Cow<'a, str>>,
    pub video_sd_url: Option<Cow<'a, str>>,
    pub watermarked_video_hd_url: Option<Cow<'a, str>>,
    pub watermarked_video_sd_url: Option<Cow<'a, str>>,
    pub video_preview_image_url: Option<Cow<'a, str>>,
}
