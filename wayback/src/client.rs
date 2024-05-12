use clap::Parser;
use reqwest;
use anyhow::{Result, anyhow};
use serde_derive::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
pub struct Args {
    #[clap(short, long)]
    domain: String,
    #[clap(short, long, action)]
    subs: bool,
    #[clap(short, long, action)]
    verbose: bool,
}

pub struct WaybackClient {
    args: Args,
    client: reqwest::blocking::Client,
}

// hit one of the backend apis
const WAYBACK_BASE: &str = "https://web.archive.org/web/timemap/json?matchType=prefix&collapse=urlkey&output=json&fl=original%2Cmimetype%2Ctimestamp%2Cendtimestamp%2Cgroupcount%2Cuniqcount&filter=!statuscode%3A[45]..&limit=10000&_=";

trait WaybackAPI {
    fn get_all_sitemap(&self, url: String) -> Result<WaybackResponse>;
}

impl WaybackClient {
    pub fn new(args: Args) -> Self {
        WaybackClient{
            args,
            client: reqwest::blocking::Client::new(),
        }
    }
    fn get_url(&self) -> String {
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).expect("problem with epoch").as_millis();
        format!("{}{}&url={}", WAYBACK_BASE, since_epoch, self.args.domain)
    }
    pub fn run(&self) -> Result<()> {
        let url = self.get_url();
        let resp = self.get_all_sitemap(url)?;
        if resp.len() < 2 {
            // first vec is always the key
            return Err(anyhow!("failed to get any results"));
        }
        for result in resp {
            // url is the first result
            println!("{}", result[0]);
        }
        Ok(())
    }
    fn get_all_sitemap(&self, url: String) -> Result<WaybackResponse> {
        let resp = self.client.get(url)
            .send()?
            .json::<WaybackResponse>()?;
        Ok(resp)
    }
}

type WaybackResponse = Vec<Vec<String>>;

#[derive(Debug, Deserialize)]
struct Snapshot {
    original: String,
    mimetype: String,
    timestamp: String,
    endtimestamp: String,
    groupcount: String,
    uniqcount: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    struct ClientMock{}
        impl WaybackAPI for ClientMock {
            fn get_all_sitemap(&self, _url: String) -> Result<WaybackResponse> {
                let data = std::fs::read_to_string("wayback_response.json").expect("failed to read test data");
                let resp: WaybackResponse = serde_json::from_str(&data)?;
                Ok(resp)
            }
        }
    #[test]
    fn get_all_sitemap() {
        let client = ClientMock{};
        let resp = client.get_all_sitemap("blah".to_string()).expect("get comments failed");
        assert_eq!(10, resp.len());
        assert_eq!(String::from("http://rust-lang.org/"), resp[1][0]);
    }
}