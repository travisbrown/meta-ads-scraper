# Meta Ads Archive API client

[![Rust build status](https://img.shields.io/github/actions/workflow/status/travisbrown/meta-ads-scraper/ci.yaml?branch=main)](https://github.com/travisbrown/meta-ads-scraper/actions)
[![Coverage status](https://img.shields.io/codecov/c/github/travisbrown/meta-ads-scraper/main.svg)](https://codecov.io/github/travisbrown/meta-ads-scraper)

A [Meta Ads Archive][meta-ads-archive] API client for Rust.

## Getting started

In order to use this project, you will need to confirm your identity with Meta, create a Meta for Developers account, and add a new app
(see Meta's [onboarding][onboarding] documentation for details).

Once you've created an app, you should confirm that you are able to make requests against the `/ads_archive` endpoint through the [Graph API Explorer][graph-api-explorer]
(with the path `ads_archive?search_terms='chess'&ad_reached_countries=['DE']`, for example). Note that the "User or Page" field should be set to "User Token".

The Graph API Explorer will give you a User Access Token that you can use here, but it will only last a few hours.
You can upgrade this short-lived token to a long-lived token by running the following commands (replacing `123`, `XXX`, and `ABC`):

```bash
> cargo build --release
> target/release/meta-ads-scraper -vvvv upgrade-token --app-id 123 --app-secret XXX --token ABC > creds.toml
```

Note that you will need to have [installed][rust-installation] [Rust][rust] on your system for this to work.
You can find your app's ID and "secret" by going to the [dashboard][facebook-apps] and selecting "App settings > Basic".
You can paste the short-lived token from the Graph API Explorer, but note that it must be active at the time the upgrade request is made.

If the command above succeeds, your new long-lived token will be saved to a `creds.toml` file in your current directory.
This file will be used by default for all future API requests.

## License

This software is licensed under the [GNU General Public License v3.0][gpl-v3] (GPL-3.0).

[facebook-apps]: https://developers.facebook.com/apps/
[gpl-v3]: https://www.gnu.org/licenses/gpl-3.0.en.html
[graph-api-explorer]: https://developers.facebook.com/tools/explorer/
[meta-ads-archive]: https://developers.facebook.com/docs/graph-api/reference/ads_archive/
[onboarding]: https://www.facebook.com/ads/library/api/?source=onboarding
[rust]: https://rust-lang.org/
[rust-installation]: https://doc.rust-lang.org/cargo/getting-started/installation.html
