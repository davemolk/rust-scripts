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
    #[clap(short, long)]
    #[arg(value_parser = parse_ndt)]
    before: Option<NaiveDateTime>,
    #[clap(short, long)]
    #[arg(value_parser = parse_ndt)]
    after: Option<NaiveDateTime>,
}

fn parse_ndt(arg: &str) -> Result<NaiveDateTime> {
    let date_v: Vec<&str> = validate_ndt(arg)?;
    let with_time_added = format!("{}{}{}000000", date_v[2], date_v[0], date_v[1]);
    let ndt = NaiveDateTime::parse_from_str(&with_time_added, "%Y%m%d%H%M%S")?;
    Ok(ndt)
}

fn validate_ndt(arg: &str) -> Result<Vec<&str>> {
    let date_v: Vec<&str> = arg.split("/").collect();
    // todo split each statement and return msg
    // add 0 if needed
    if date_v.len() != 3 || date_v[0].len() != 2 || date_v[1].len() != 2 || date_v[2].len() != 4 {
        return Err(anyhow!("failed to provide a valid date: {:?}", arg));
    }
    Ok(date_v)
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
                // let mut filtering: bool = false;
                // if let Some(before) = self.args.before {
                //     println!("from {}", before);
                // };

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
            if !self.filter_needed() {
                let mut from = String::new();
                if let Ok(f) =  self.parse_to_ndt(&entry[2]) {
                    from = f.to_string();
                };
                let mut to = String::new();
                if let Ok(t) =  self.parse_to_ndt(&entry[3]) {
                    to = t.to_string();
                };
                if from.len() == 0 || to.len() == 0 {
                    println!("{}", entry[0]);
                    continue
                }
                println!("{0: <20} | {1: <20} | {2: <6} | {3: <6} | {4: <25} | {5: }", from, to, entry[4], entry[5], entry[1], entry[0]);
            } else {
                let mut from = String::new();
                if let Ok(f) =  self.parse_to_ndt(&entry[2]) {
                    from = f.to_string();
                };
                let mut to = String::new();
                if let Ok(t) =  self.parse_to_ndt(&entry[3]) {
                    to = t.to_string();
                };
                if from.len() == 0 || to.len() == 0 {
                    println!("{}", entry[0]);
                    continue
                }
                if let Some(before) = self.args.before {
                    if before - self.parse_to_ndt(&entry[2]).unwrap() > chrono::TimeDelta::new(0, 0).unwrap() {
                        println!("{0: <20} | {1: <20} | {2: <6} | {3: <6} | {4: <25} | {5: }", from, to, entry[4], entry[5], entry[1], entry[0]);
                    } else {
                        continue
                    }
                }
            }
            
        }
    }
    fn filter_needed(&self) -> bool {
        match self.args.before {
            Some(_) => return true,
            None => return false,
        }
    }
    fn parse_to_ndt(&self, date: &str) -> Result<NaiveDateTime> {
        let ndt = NaiveDateTime::parse_from_str(&date, "%Y%m%d%H%M%S")?;
        Ok(ndt)
    }
}

type WaybackResponse = Vec<Vec<String>>;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
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
    fn test_parse_ndt() {
        let ndt = parse_ndt("10/31/2020").expect("want valid date");
        let expect_ndt = NaiveDate::from_ymd_opt(2020, 10, 31).unwrap().and_hms_opt(0, 0, 0).unwrap();
        assert_eq!(ndt, expect_ndt);
    }
    #[test]
    #[should_panic]
    fn test_parse_ndt_fail() {
        _ = parse_ndt("10/31/200").expect("want valid date");
    }
}
