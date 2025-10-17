use chrono::Utc;
use cli_helpers::prelude::*;
use meta_ads_scraper::{
    exchange::Exchange,
    model::{Ad, Response, ResponseSuccess},
    token::Creds,
    version::GraphApiVersion,
};
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("CLI argument reading error")]
    Args(#[from] cli_helpers::Error),
    #[error("API client error")]
    Api(#[from] meta_ads_scraper::client::Error),
    #[error("Library client error")]
    LibraryClient(#[from] meta_ads_scraper::library::Error),
    #[error("Library model error")]
    LibraryModel(PathBuf, meta_ads_scraper::model::library::Error),
    #[error("HTTP client error")]
    Http(#[from] reqwest::Error),
    #[error("CSV error")]
    Csv(#[from] csv::Error),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("File JSON error")]
    JsonFile(PathBuf, serde_json::Error),
    #[error("TOML deserialization error")]
    TomlDe(#[from] toml::de::Error),
    #[error("TOML serialization error")]
    TomlSer(#[from] toml::ser::Error),
    #[error("Invalid ID line")]
    InvalidIdLine(String),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();
    opts.verbose.init_logging()?;

    match opts.command {
        Command::Search {
            creds,
            version,
            terms,
            country,
            exact,
            after,
            output,
            delay,
        } => {
            let creds: Creds = toml::from_str(&std::fs::read_to_string(creds)?)?;
            log_token_status(creds.status(Utc::now()));

            let client = meta_ads_scraper::client::Client::new(creds.token, output);
            let search_type = if exact {
                meta_ads_scraper::client::SearchType::KeywordExactPhrase
            } else {
                meta_ads_scraper::client::SearchType::KeywordUnordered
            };

            let results = client
                .search(
                    version,
                    &terms,
                    &country,
                    search_type,
                    after.as_deref(),
                    std::time::Duration::from_secs(delay),
                )
                .await?;

            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(std::io::stdout());

            for result in results {
                for ad in result.result().unwrap_or_default() {
                    writer.write_record([
                        ad.id.to_string(),
                        ad.page_id.to_string(),
                        ad.page_name.to_string(),
                    ])?;
                }
            }

            writer.flush()?;
        }
        Command::LibraryAd { id, output } => {
            let client = meta_ads_scraper::library::Client::new::<_, String>(output, None)?;

            let _response = client.app(id).await?;
        }
        Command::LibraryAds { output, delay } => {
            let ids = std::io::stdin()
                .lines()
                .map(|line| {
                    let line = line?;

                    line.parse::<u64>().map_err(|_| Error::InvalidIdLine(line))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let client = meta_ads_scraper::library::Client::new::<_, String>(output, None)?;

            for id in ids {
                let _response = client.app(id).await?;
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
            }
        }
        Command::UpgradeToken {
            version,
            app_id,
            app_secret,
            token,
        } => {
            let response =
                meta_ads_scraper::token::upgrade_token(version, app_id, &app_secret, &token)
                    .await?;

            ::log::info!("Expires in {} seconds", response.expires_in);

            println!("{}", toml::to_string(&response.creds(Utc::now()))?);
        }
        Command::SearchArchive { data } => {
            let mut paths = std::fs::read_dir(data)?
                .map(|entry| {
                    let entry = entry?;
                    let path = entry.path();
                    let modified = path.metadata()?.modified()?;

                    Ok((modified, entry.path()))
                })
                .collect::<Result<Vec<_>, Error>>()?;

            // Process newest additions first, in order to speed up catching errors.
            paths.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));

            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(std::io::stdout());

            for (_, path) in paths {
                let content = std::fs::read_to_string(&path)?;

                let exchange = match serde_json::from_str::<Exchange<Response<'static, Ad<'static>>>>(
                    &content,
                ) {
                    Ok(exchange) => Ok(exchange),
                    Err(_) => {
                        // Hack to get better error message.
                        let reparsed = serde_json::from_str::<
                            Exchange<ResponseSuccess<'static, Ad<'static>>>,
                        >(&content);

                        Err(reparsed
                            .map_err(|error| Error::JsonFile(path.clone(), error))
                            .expect_err("Should have failed"))
                    }
                }?;

                match exchange.response.data.result() {
                    Ok(ads) => {
                        for ad in ads {
                            writer.write_record([
                                ad.id.to_string(),
                                ad.page_id.to_string(),
                                ad.page_name.to_string(),
                            ])?;
                        }
                    }
                    Err(error) => {
                        ::log::warn!("{}", error.message);
                    }
                }
            }

            writer.flush()?;
        }
        Command::LibraryArchive { data } => {
            let mut paths = std::fs::read_dir(data)?
                .map(|entry| {
                    let entry = entry?;
                    let path = entry.path();
                    let modified = path.metadata()?.modified()?;

                    Ok((modified, entry.path()))
                })
                .collect::<Result<Vec<_>, Error>>()?;

            // Process newest additions first, in order to speed up catching errors.
            paths.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));

            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(std::io::stdout());

            for (_, path) in paths {
                let content = std::fs::read_to_string(&path)?;
                let exchange = serde_json::from_str::<Exchange<serde_json::Value>>(&content)?;

                let ad = meta_ads_scraper::model::library::Ad::extract(&exchange.response.data)
                    .map_err(|error| Error::LibraryModel(path.clone(), error))?;

                match ad {
                    Some(ad) => {
                        writer.write_record([
                            ad.deeplink_ad_card.ad_archive_id.to_string(),
                            ad.deeplink_ad_card.snapshot.page_id.to_string(),
                            ad.deeplink_ad_card
                                .snapshot
                                .link_url
                                .map(|link_url| link_url.to_string())
                                .unwrap_or_default(),
                            ad.deeplink_ad_card
                                .snapshot
                                .page_profile_picture_url
                                .to_string(),
                            ad.deeplink_ad_card
                                .snapshot
                                .videos
                                .first()
                                .and_then(|video| video.video_preview_image_url.as_ref())
                                .map(|video_preview_image_url| video_preview_image_url.to_string())
                                .unwrap_or_default(),
                        ])?;
                    }
                    None => {
                        ::log::warn!("Empty library response: {:?}", path);
                    }
                }
            }

            writer.flush()?;
        }
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[clap(name = "meta-ads-scraper", version, author)]
struct Opts {
    #[clap(flatten)]
    verbose: Verbosity,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    /// Perform a search (possibly paginated) and print the ad IDs, page IDs, and page names as CSV
    Search {
        #[clap(long, default_value = "creds.toml")]
        creds: PathBuf,
        #[clap(long, default_value = "24.0")]
        version: GraphApiVersion,
        #[clap(long)]
        terms: String,
        #[clap(long, default_value = "DE")]
        country: Vec<String>,
        #[clap(long)]
        exact: bool,
        /// Optional pagination token
        #[clap(long)]
        after: Option<String>,
        /// Archive directory to log requests and responses to
        #[clap(long)]
        output: Option<PathBuf>,
        /// Optional duration (in seconds) between requests
        #[clap(long, default_value = "0")]
        delay: u64,
    },
    LibraryAd {
        #[clap(long)]
        id: u64,
        /// Directory to log requests and responses to
        #[clap(long)]
        output: Option<PathBuf>,
    },
    /// Download ads for a list of IDs from standard input
    LibraryAds {
        /// Directory to log requests and responses to
        #[clap(long)]
        output: Option<PathBuf>,
        /// Optional duration (in seconds) between requests
        #[clap(long, default_value = "0")]
        delay: u64,
    },
    /// Upgrade a short-lived token to a long-lived one and print as TOML
    UpgradeToken {
        #[clap(long, default_value = "24.0")]
        version: GraphApiVersion,
        #[clap(long)]
        app_id: u64,
        #[clap(long)]
        app_secret: String,
        /// Active short-lived token
        #[clap(long)]
        token: String,
    },
    /// Print ad IDs, page IDs, and page names as CSV for all archived exchanges
    SearchArchive {
        /// Archive directory
        #[clap(long)]
        data: PathBuf,
    },
    LibraryArchive {
        /// Archive directory
        #[clap(long)]
        data: PathBuf,
    },
}

fn log_token_status(status: meta_ads_scraper::token::TokenStatus) {
    match status {
        meta_ads_scraper::token::TokenStatus::Expired => {
            ::log::error!("Token is expired, request is likely to fail");
        }
        meta_ads_scraper::token::TokenStatus::ExpiringSoon => {
            ::log::error!("Token is expiring soon");
        }
        meta_ads_scraper::token::TokenStatus::Ok => {}
    }
}
