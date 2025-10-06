use crate::version::GraphApiVersion;
use bounded_static_derive_more::ToStatic;
use chrono::{DateTime, SubsecRound, Utc};
use std::borrow::Cow;
use std::time::Duration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenStatus {
    Ok,
    Expired,
    ExpiringSoon,
}

#[derive(Clone, Debug, Eq, PartialEq, ToStatic, serde::Deserialize, serde::Serialize)]
pub struct Creds<'a> {
    pub token: Cow<'a, str>,
    pub expiration: DateTime<Utc>,
}

impl Creds<'_> {
    #[must_use]
    pub fn status(&self, now: DateTime<Utc>) -> TokenStatus {
        let remaining = self.expiration - now;

        if remaining.num_seconds() < 0 {
            TokenStatus::Expired
        } else if remaining.num_days() < 1 {
            TokenStatus::ExpiringSoon
        } else {
            TokenStatus::Ok
        }
    }
}

#[derive(Clone, Debug, Eq, ToStatic, PartialEq, serde::Deserialize)]
pub struct Response<'a> {
    pub access_token: Cow<'a, str>,
    pub token_type: TokenType,
    pub expires_in: u32,
}

impl<'a> Response<'a> {
    #[must_use]
    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.expires_in.into())
    }

    #[must_use]
    pub fn creds(&self, now: DateTime<Utc>) -> Creds<'a> {
        let expiration = (now + chrono::Duration::seconds(self.expires_in.into())).trunc_subsecs(0);

        Creds {
            token: self.access_token.clone(),
            expiration,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub enum TokenType {
    #[serde(rename = "bearer")]
    Bearer,
}

/// Exchange a short-lived user token for a long-lived one.
///
/// Long-lived tokens typically last 60 days.
///
/// See the [documentation](https://developers.facebook.com/docs/facebook-login/guides/access-tokens/get-long-lived/) for details.
pub async fn upgrade_token(
    version: GraphApiVersion,
    app_id: u64,
    app_secret: &str,
    token: &str,
) -> Result<Response<'static>, reqwest::Error> {
    let url = format!(
        "https://graph.facebook.com/v{version}/oauth/access_token?grant_type=fb_exchange_token&client_id={app_id}&client_secret={app_secret}&fb_exchange_token={token}"
    );

    let response = reqwest::get(url).await?;

    response.json().await
}
