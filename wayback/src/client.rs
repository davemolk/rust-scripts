use clap::Parser;
use reqwest;
use anyhow::{Result, anyhow};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::NaiveDateTime;

#[derive(Parser)]
pub struct Args {
    #[clap(short, long)]
    domain: String,
    #[clap(short, long, action)]
    verbose: bool,
}

pub struct WaybackClient {
    args: Args,
    client: reqwest::blocking::Client,
}

// hit one of the backend apis to get URLs captured for the given URL prefix
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
            // first vec is always the key, [ "original", "mimetype", "timestamp", "endtimestamp", "groupcount", "uniqcount" ]
            return Err(anyhow!("failed to get any results"));
        }
        if self.args.verbose {
            self.print_verbose(resp);
        } else {
            for result in resp {
                // url is the first result
                println!("{}", result[0]);
            }
        }
        Ok(())
    }
    fn get_all_sitemap(&self, url: String) -> Result<WaybackResponse> {
        let resp = self.client.get(url)
            .send()?
            .json::<WaybackResponse>()?;
        Ok(resp)
    }
    fn print_verbose(&self, resp: Vec<Vec<String>>) {
        println!(
                "{0: <20} | {1: <20} | {2: <6} | {3: <6} | {4: <25} | {5: }",
                "From:", "To:", "Duplicates:", "Uniques:", "MIME Type:", "URL:"
            );
        for entry in resp {
            if entry.len() != 6 {
                // best effort to print url
                if entry.len() > 1 {
                    println!("{}", entry[0]);
                } 
                continue
            }
            let mut from = String::new();
            if let Ok(f) =  self.parse_date(&entry[2]) {
                from = f.to_string();
            };
            let mut to = String::new();
            if let Ok(t) =  self.parse_date(&entry[3]) {
                to = t.to_string();
            };
            if from.len() == 0 || to.len() == 0 {
                println!("{}", entry[0]);
                continue
            }
            println!("{0: <20} | {1: <20} | {2: <6} | {3: <6} | {4: <25} | {5: }", from, to, entry[4], entry[5], entry[1], entry[0]);
        }
    }
    fn parse_date(&self, date: &str) -> Result<NaiveDateTime> {
        let ndt = NaiveDateTime::parse_from_str(&date, "%Y%m%d%H%M%S")?;
        Ok(ndt)
    }
}

type WaybackResponse = Vec<Vec<String>>;

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
    fn test_get_all_sitemap() {
        let client = ClientMock{};
        let resp = client.get_all_sitemap("blah".to_string()).expect("get comments failed");
        assert_eq!(10, resp.len());
        assert_eq!(String::from("http://rust-lang.org/"), resp[1][0]);
    }
    #[test]
    fn test_foo() {
        // get as YYYYMMDDhhmmss
        let date_str = "20240201100502";
        let ndt = NaiveDateTime::parse_from_str(date_str, "%Y%m%d%H%M%S").unwrap();
        println!("{}", ndt);
    }
}