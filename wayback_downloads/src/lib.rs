use anyhow::{anyhow, Result};
use clap::Parser;
use chrono::NaiveDateTime;

mod cdx;

#[derive(Debug, Parser)]
#[command(version)]
pub struct Args {
    /// URL to search for.
    url: String,
    #[arg(short, long)]
    /// print the list of found snapshots (will not download).
    list: Option<bool>,
    /// only consider 'unique' versions of duplicate files.
    /// (this is done by filtering out adjacent results that are duplicates...
    /// the showDupeCount=true doesn't appear to work anymore...)
    #[arg(short, long, default_value_t = true)]
    unique: bool,
    /// get original file (without wayback machine processing).
    #[arg(short, long, default_value_t = false)]
    raw: bool,
    /// timestamp of earliest snapshot to consider.
    /// format is YYYYMMDDhhmmss. you can omit trailing digits.
    #[arg(short, long, value_parser = validate_user_timestamp)]
    from_date: Option<String>,
    /// timestamp of most recent snapshot to consider.
    /// format is YYYYMMDDhhmmss. you can omit trailing digits.
    #[arg(short, long, value_parser = validate_user_timestamp)]
    to_date: Option<String>,
    /// limit the number of results
    #[arg(long)]
    limit: Option<usize>,
}

fn validate_user_timestamp(arg: &str) -> Result<String> {
    // not going to go nuts on validations yet, plus wayback is fine with 20201
    anyhow::ensure!(arg.chars().all(|c| c.is_ascii_digit()) == true, "");
    anyhow::ensure!(arg.len() <= 14, "format is YYYYMMDDhhmmss");
    Ok(arg.to_owned())
}

pub struct WaybackClient {
    client: reqwest::blocking::Client,
}

impl WaybackClient {
    fn new() -> Self {
        WaybackClient{ client: reqwest::blocking::Client::new()}
    }
}

pub fn run(args: Args) -> Result<()> {
    if args.url.is_empty() {
        return Err(anyhow!("need a url"));
    }

    let results = cdx::CdxClient::get_cdx(&args.url, args.unique, args.from_date, args.to_date, args.limit)?;
    // if let Some(list) = args.list {
    //     if list {
    //         for info in results {
    //             println!("{:?}", info);
    //         }
    //     }
    // }
    println!("{:?}", results);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validate_user_timestamp_error_incorrect_format_dashes() {
        assert!(validate_user_timestamp("2020-10-31").is_err());
    }
    #[test]
    fn test_validate_user_timestamp_error_too_long() {
        assert!(validate_user_timestamp("2020202020202020").is_err());
    }
    #[test]
    fn test_validate_user_timestamp_error_not_digit() {
        assert!(validate_user_timestamp("archive").is_err());
    }
    #[test]
    // wayback machine allows it so we will too
    fn test_validate_user_timestamp_success_one_digit() {
        assert!(validate_user_timestamp("2").is_ok());
    }
    #[test]
    fn test_validate_user_timestamp_success_omit_end() {
        assert!(validate_user_timestamp("20201031").is_ok());
    }
    #[test]
    fn test_validate_user_timestamp_success_full_length() {
        assert!(validate_user_timestamp("19961102145216").is_ok());
    }
    #[test]
    #[should_panic]
    fn test_validate_user_timestamp_fail() {
        _ = validate_user_timestamp("10/31/200").expect("want valid date");
    }
}