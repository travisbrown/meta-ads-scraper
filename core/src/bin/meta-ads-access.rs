use chrono::Utc;
use cli_helpers::prelude::*;
use meta_ads_access::{
    client::request::SearchType,
    model::{Ad, Response},
    token::Creds,
    version::GraphApiVersion,
};
use scraper_trail::archive::entry::Entry;
use std::io::Write;
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("CLI argument reading error")]
    Args(#[from] cli_helpers::Error),
    #[error("API client error")]
    Api(#[from] meta_ads_access::client::Error),
    #[error("Library client error")]
    LibraryClient(#[from] meta_ads_access::library::Error),
    #[error("Library model error")]
    LibraryModel(PathBuf, meta_ads_access::model::library::v1::Error),
    #[error("Scraper store error")]
    ScraperStore(#[from] scraper_trail::archive::store::Error),
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
            limit,
            full,
            full_output,
            delay,
        } => {
            let creds: Creds = toml::from_str(&std::fs::read_to_string(creds)?)?;
            log_token_status(creds.status(Utc::now()));

            let client = meta_ads_access::client::Client::new(creds.token, output.as_deref());
            let library_client =
                meta_ads_access::library::Client::new::<_, String>(full_output.as_deref(), None)?;

            let search_type = if exact {
                SearchType::KeywordExactPhrase
            } else {
                SearchType::KeywordUnordered
            };

            let results = client
                .search(&meta_ads_access::client::SearchOptions {
                    version,
                    terms: &terms,
                    countries: &country,
                    search_type,
                    after: after.as_deref(),
                    limit,
                    delay: std::time::Duration::from_secs(delay),
                })
                .await?;

            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(std::io::stdout());

            for result in results {
                match result.result() {
                    Ok(ads) => {
                        for ad in ads {
                            writer.write_record([
                                ad.id.to_string(),
                                ad.page_id.to_string(),
                                ad.page_name.to_string(),
                            ])?;

                            if full {
                                library_client.app(ad.id).await?;
                            }
                        }
                    }
                    Err(error) => {
                        ::log::warn!("{}", error.message);
                    }
                }
            }

            writer.flush()?;
        }
        Command::SearchAll {
            creds,
            version,
            query_file,
            country,
            output,
            limit,
            full,
            full_output,
            delay,
        } => {
            let creds: Creds = toml::from_str(&std::fs::read_to_string(creds)?)?;
            log_token_status(creds.status(Utc::now()));

            let client = meta_ads_access::client::Client::new(creds.token, output.as_deref());
            let library_client =
                meta_ads_access::library::Client::new::<_, String>(full_output.as_deref(), None)?;

            let queries = std::fs::read_to_string(&query_file)?;

            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(std::io::stdout());

            for line in queries
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
            {
                let (terms, search_type) = if line.starts_with('"') && line.ends_with('"') {
                    (&line[1..line.len() - 1], SearchType::KeywordExactPhrase)
                } else {
                    (line, SearchType::KeywordUnordered)
                };

                let results = client
                    .search(&meta_ads_access::client::SearchOptions {
                        version,
                        terms,
                        countries: &country,
                        search_type,
                        after: None,
                        limit,
                        delay: std::time::Duration::from_secs(delay),
                    })
                    .await?;

                for result in results {
                    match result.result() {
                        Ok(ads) => {
                            for ad in ads {
                                writer.write_record([
                                    ad.id.to_string(),
                                    ad.page_id.to_string(),
                                    ad.page_name.to_string(),
                                ])?;

                                if full {
                                    library_client.app(ad.id).await?;
                                }
                            }
                        }
                        Err(error) => {
                            ::log::warn!("{}", error.message);
                        }
                    }
                }
            }

            writer.flush()?;
        }
        Command::LibraryAd { id, output } => {
            let client = meta_ads_access::library::Client::new::<_, String>(output, None)?;

            client.app(id).await?;
        }
        Command::LibraryAds { output, delay } => {
            let ids = std::io::stdin()
                .lines()
                .map(|line| {
                    let line = line?;

                    line.parse::<u64>().map_err(|_| Error::InvalidIdLine(line))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let client = meta_ads_access::library::Client::new::<_, String>(output, None)?;

            for id in ids {
                client.app(id).await?;
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
            }
        }
        Command::UpgradeToken {
            version,
            app_id,
            app_secret,
            token,
            output,
        } => {
            let response =
                meta_ads_access::token::upgrade_token(version, app_id, &app_secret, &token).await?;

            ::log::info!("Expires in {} seconds", response.expires_in);

            let contents = toml::to_string(&response.creds(Utc::now()))?;

            if let Some(output) = output {
                std::fs::write(output, format!("{contents}\n"))?;
            } else {
                writeln!(std::io::stdout(), "{contents}",)?;
            }
        }
        Command::SearchArchive {
            data,
            most_recent_first,
        } => {
            let store = scraper_trail::archive::store::Store::new(data);

            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(std::io::stdout());

            for (path, contents) in store.contents(most_recent_first)? {
                let contents = contents?;

                let archive = serde_json::from_str::<Entry<Response<Ad>>>(&contents)
                    .map_err(|error| Error::JsonFile(path, error))?;

                match archive.exchange.response.data.result() {
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
        Command::LibraryArchive {
            data,
            most_recent_first,
        } => {
            let store = scraper_trail::archive::store::Store::new(data);

            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(std::io::stdout());

            for (path, contents) in store.contents(most_recent_first)? {
                let contents = contents?;

                let archive = serde_json::from_str::<
                    Entry<meta_ads_access::model::library::v2::AdLibraryResponse>,
                >(&contents)
                .map_err(|error| Error::JsonFile(path, error))?;

                let response = archive.exchange.response.data;
                if let Some(ad) = response.ad() {
                    writer.write_record([
                        ad.ad_archive_id.to_string(),
                        ad.snapshot.page_id.to_string(),
                        ad.snapshot
                            .link_url
                            .as_ref()
                            .map(|link_url| link_url.to_string())
                            .unwrap_or_default(),
                    ])?;
                }

                for ad in response.search_results().ads {
                    writer.write_record([
                        ad.ad_archive_id.to_string(),
                        ad.snapshot.page_id.to_string(),
                        ad.snapshot
                            .link_url
                            .as_ref()
                            .map(|link_url| link_url.to_string())
                            .unwrap_or_default(),
                    ])?;
                }
            }

            writer.flush()?;
        }
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[clap(name = "meta-ads-access", version, author)]
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
        #[clap(long, default_value = "data/search")]
        output: Option<PathBuf>,
        /// Limit to a specified number of pages
        #[clap(long)]
        limit: Option<usize>,
        /// Download full ad information
        #[clap(long)]
        full: bool,
        /// Archive directory to log full requests and responses to
        #[clap(long, default_value = "data/library")]
        full_output: Option<PathBuf>,
        /// Optional duration (in seconds) between requests
        #[clap(long, default_value = "0")]
        delay: u64,
    },
    /// Perform searches for a list of queries provided as lines in the indicated text file
    SearchAll {
        #[clap(long, default_value = "creds.toml")]
        creds: PathBuf,
        #[clap(long, default_value = "24.0")]
        version: GraphApiVersion,
        /// Path to a file with one search query per line
        #[clap(long)]
        query_file: PathBuf,
        #[clap(long, default_value = "DE")]
        country: Vec<String>,
        /// Archive directory to log requests and responses to
        #[clap(long, default_value = "data/search")]
        output: Option<PathBuf>,
        /// Limit to a specified number of pages per query
        #[clap(long)]
        limit: Option<usize>,
        /// Download full ad information
        #[clap(long)]
        full: bool,
        /// Archive directory to log full requests and responses to
        #[clap(long, default_value = "data/library")]
        full_output: Option<PathBuf>,
        /// Optional duration (in seconds) between requests
        #[clap(long, default_value = "0")]
        delay: u64,
    },
    /// Download ad for the specified ID
    LibraryAd {
        #[clap(long)]
        id: u64,
        /// Directory to log requests and responses to
        #[clap(long, default_value = "data/library")]
        output: Option<PathBuf>,
    },
    /// Download ads for a list of IDs from standard input
    LibraryAds {
        /// Directory to log requests and responses to
        #[clap(long, default_value = "data/library")]
        output: Option<PathBuf>,
        /// Optional duration (in seconds) between requests
        #[clap(long, default_value = "0")]
        delay: u64,
    },
    /// Upgrade a short-lived token to a long-lived one and save as TOML
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
        /// File to save credentials to (optional; if absent will print to standard out)
        #[clap(long)]
        output: Option<PathBuf>,
    },
    /// Print ad IDs, page IDs, and page names as CSV for all archived exchanges
    SearchArchive {
        /// Archive directory
        #[clap(long, default_value = "data/search")]
        data: PathBuf,
        #[clap(long)]
        most_recent_first: bool,
    },
    LibraryArchive {
        /// Archive directory
        #[clap(long, default_value = "data/library")]
        data: PathBuf,
        #[clap(long)]
        most_recent_first: bool,
    },
}

fn log_token_status(status: meta_ads_access::token::TokenStatus) {
    match status {
        meta_ads_access::token::TokenStatus::Expired => {
            ::log::error!("Token is expired, request is likely to fail");
        }
        meta_ads_access::token::TokenStatus::ExpiringSoon => {
            ::log::error!("Token is expiring soon");
        }
        meta_ads_access::token::TokenStatus::Ok => {}
    }
}
