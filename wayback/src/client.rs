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
    let date_v = validate_ndt(arg)?;
    let with_time_added = format!("{}000000", date_v);
    let ndt = NaiveDateTime::parse_from_str(&with_time_added, "%Y%m%d%H%M%S")?;
    Ok(ndt)
}

fn validate_ndt(arg: &str) -> Result<String> {
    let date_v: Vec<&str> = arg.split("-").collect();
    // not comprehensive checking, but fine for this project
    anyhow::ensure!(date_v.len() == 3, "need date as yyyy-dd-mm");
    anyhow::ensure!(date_v[0].len() == 4, "need a valid year");
    anyhow::ensure!(date_v[1].len() > 0 && date_v[1].len() < 3, "need a valid day");
    let day = if date_v[1].len() == 2 { date_v[1].to_string() } else { format!("0{}", date_v[1]) };
    anyhow::ensure!(date_v[2].len() > 0 && date_v[2].len() < 3, "need a valid month");
    let month = if date_v[2].len() == 2 { date_v[2].to_string() } else { format!("0{}", date_v[2]) };
    Ok(format!("{}{}{}", date_v[0].to_string(), day, month))
}

fn parse_result_to_ndt(date: &str) -> Result<NaiveDateTime> {
    let ndt = NaiveDateTime::parse_from_str(&date, "%Y%m%d%H%M%S")?;
    Ok(ndt)
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
        self.print_results(resp);
        Ok(())
    }
    fn get_all_sitemap(&self, url: String) -> Result<WaybackResponse> {
        let resp = self.client.get(url)
            .send()?
            .json::<WaybackResponse>()?;
        Ok(resp)
    }
    fn print_results(&self, resp: Vec<Vec<String>>) {
        let filter_before = if self.args.before.is_some() { true } else { false };
        let filter_after = if self.args.after.is_some() { true } else { false };
        if self.args.verbose {
            println!(
                "{0: <20} | {1: <20} | {2: <11} | {3: <8} | {4: <25} | {5: }",
                "From:", "To:", "Duplicates:", "Uniques:", "MIME Type:", "URL:"
            );
        }
        for (i, entry) in resp.iter().enumerate() {
            // skip the key
            if i == 0 {
                continue
            }
            if entry.len() != 6 {
                // best effort to print url
                if entry.len() > 1 {
                    println!("{}", entry[0]);
                } 
                continue
            }
            // get result dates and parse
            let mut from = String::new();
            if let Ok(f) = parse_result_to_ndt(&entry[2]) {
                from = f.to_string();
            };
            let mut to = String::new();
            if let Ok(t) = parse_result_to_ndt(&entry[3]) {
                to = t.to_string();
            };
            // if we can't get the dates, we can't filter, so just print the url
            if from.len() == 0 || to.len() == 0 {
                println!("{}", entry[0]);
                continue
            }
            // not filtering
            if !filter_before && !filter_after {
                if self.args.verbose {
                    self.print_line(from, to, &entry);
                } else {
                    println!("{}", entry[0]);
                }
            } else {
                // check filter cases
                if filter_before && filter_after && self.is_before(&entry[2]) && self.is_after(&entry[3]) {
                    self.print_line(from, to, &entry);
                } else if filter_before && self.is_before(&entry[2]) {
                    self.print_line(from, to, &entry);
                } else if filter_after && self.is_after(&entry[3]) {
                    self.print_line(from, to, &entry);
                }
            }
        }
    }
    fn print_line(&self, from: String, to: String, line: &Vec<String>) {
        if self.args.verbose {
            println!("{0: <20} | {1: <20} | {2: <11} | {3: <8} | {4: <25} | {5: }", from, to, line[4], line[5], line[1], line[0]);
        } else {
            println!("{}", line[0]);
        }
    }
    fn is_before(&self, date: &str) -> bool {
        self.args.before.unwrap() - parse_result_to_ndt(date).unwrap() > chrono::TimeDelta::new(0, 0).unwrap()
    }
    fn is_after(&self, date: &str) -> bool {
        parse_result_to_ndt(date).unwrap() - self.args.after.unwrap() > chrono::TimeDelta::new(0, 0).unwrap()
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
        let ndt = parse_ndt("2020-10-31").expect("want valid date");
        let expect_ndt = NaiveDate::from_ymd_opt(2020, 10, 31).unwrap().and_hms_opt(0, 0, 0).unwrap();
        assert_eq!(ndt, expect_ndt);
    }
    #[test]
    #[should_panic]
    fn test_parse_ndt_fail() {
        _ = parse_ndt("10/31/200").expect("want valid date");
    }
    #[test]
    fn test_parse_result_to_ndt() {
        let got = parse_result_to_ndt("20240228105514").expect("want valid");
        let expect = NaiveDate::from_ymd_opt(2024, 02, 28).unwrap().and_hms_opt(10, 55, 14).unwrap();
        assert_eq!(got, expect);
    }
}
