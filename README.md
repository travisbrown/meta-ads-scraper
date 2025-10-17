# Meta Ads Archive API client

[![Rust build status](https://img.shields.io/github/actions/workflow/status/travisbrown/meta-ads-scraper/ci.yaml?branch=main)](https://github.com/travisbrown/meta-ads-scraper/actions)
[![Coverage status](https://img.shields.io/codecov/c/github/travisbrown/meta-ads-scraper/main.svg)](https://codecov.io/github/travisbrown/meta-ads-scraper)

A [Meta Ads Archive][meta-ads-archive] API client for Rust.

## Warning

This project is in active development, and it will currently fail on data that is not captured in its model
(since we want to know as soon as possible if the API is providing data that we are not modeling).
It has been tests on hundreds of searches and tens of thousands of ads, but it is likely to fail on some requests.

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

## Usage

This project provides a client for the Ads Archive search API, and also supports scraping the Ads Library HTML pages directly.

Both kinds of requests support an `--output` option that saves detailed information about requests and responses.
These archived request-response pairs can be parsed later using other commands.

### Search

If you've compiled the project and set up your credentials, you can run the following command (the `-vvv` verbosity will log details about pagination requests):

```
$ target/release/meta-ads-scraper -vvv search --output data/search/ --terms "ai chess"
```

This particular search currently takes around four minutes to run, will make around 190 pagination requests, and returns around 4,700 ads.

See the CLI documentation for more information about search options:

```
Perform a search (possibly paginated) and print the ad IDs, page IDs, and page names as CSV

Usage: meta-ads-scraper search [OPTIONS] --terms <TERMS>

Options:
      --creds <CREDS>      [default: creds.toml]
  -v, --verbose...         Level of verbosity
      --version <VERSION>  [default: 24.0]
      --terms <TERMS>
      --country <COUNTRY>  [default: DE]
      --exact
      --after <AFTER>      Optional pagination token
      --output <OUTPUT>    Archive directory to log requests and responses to
      --delay <DELAY>      Optional duration (in seconds) between requests [default: 0]
  -h, --help               Print help
```

If you've used the `--output` command while making searches, you can parse the archived data (without making new requests) using the `search-archive` command:

```
$ target/release/meta-ads-scraper -vvv search-archive --data data/search/ | head
576535441426103,157817344084965,Chessiverse
589129136845222,1834313933459789,BBC StoryWorks
551290891182597,367754219763901,writer_kalyani
1059818455891627,104986081671617,it.com Domains
3839155272998853,108227973861366,Miko
1734255410761430,108227973861366,Miko
1292315335117015,669389136462669,Social Discovery Group
1224569318778818,105486451504054,Chessnut
1068754781111526,268914469632645,Patriticpttic
484518928069113,268914469632645,Patriticpttic
```

### Library scraping

Once you have a list of ad IDs (the first column returned by the commands in the previous section), you can scrape and extract data from the Ads Library HTML pages for these ads.

```
$ target/release/meta-ads-scraper -vvv library-ad --output data/library/ --id 576535441426103
```

You can also use the `library-ads` command to run requests for a batch of ad IDs provided on standard input (one numeric ID per line). Both commands will save the requests and responses to the provided `--output` directory.

You can then run the following command to list the contents of that directory:

```
$ target/release/meta-ads-scraper -vvv library-archive --data data/library/
```

This will print CSV rows where the columns are the ad ID, the advertiser page ID, the ad link, the advertiser page profile image URL, and an ad preview URL.

## License

This software is licensed under the [GNU General Public License v3.0][gpl-v3] (GPL-3.0).

[facebook-apps]: https://developers.facebook.com/apps/
[gpl-v3]: https://www.gnu.org/licenses/gpl-3.0.en.html
[graph-api-explorer]: https://developers.facebook.com/tools/explorer/
[meta-ads-archive]: https://developers.facebook.com/docs/graph-api/reference/ads_archive/
[onboarding]: https://www.facebook.com/ads/library/api/?source=onboarding
[rust]: https://rust-lang.org/
[rust-installation]: https://doc.rust-lang.org/cargo/getting-started/installation.html
