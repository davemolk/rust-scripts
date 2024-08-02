use anyhow::{anyhow, Result};
use reqwest;
use chrono::NaiveDateTime;

const CDX_BASE: &str = "http://web.archive.org/cdx/search/cdx?url=";
const CDX_PARAMS: &str = "&output=json&fl=original,timestamp,statuscode,mimetype,digest,length";
const DATE_FORMAT: &str = "%Y%m%d%H%M%S";
const WAYBACK_WEB_URL_BASE: &str = "http://web.archive.org/web/";
#[derive(Clone, Debug)]
pub struct CdxInfo {
    original: String,
    archived_at: NaiveDateTime,
    status_code: Option<u16>,
    mime_type: String,
    digest: String,
    length: u64,
}

impl CdxInfo {
    fn parse_row(
        original: &str,
        timestamp: &str,
        status_code: &str,
        mime_type: &str,
        digest: &str,
        length: &str,
    ) -> Result<CdxInfo> {
        let archived_at = NaiveDateTime::parse_from_str(&timestamp, DATE_FORMAT)?;
        let length = length.parse::<u64>()?;
        // wb uses "-" for unknown/null values
        let status_code = if status_code == "-" { None } else { 
            Some(status_code.parse::<u16>()?)
        };
        Ok(CdxInfo{
            original: original.to_string(),
            archived_at,
            status_code,
            mime_type: mime_type.to_string(),
            digest: digest.to_string(),
            length,
        })
    }
    fn web_url(&self, on_site: bool) -> String {
        // https://archive.org/post/1010104/cdx-digest-not-accurately-capturing-duplicates
        // To get unaltered content from wayback machine, simply add "id_" after the timestamp in the url!
        let from_wb = if on_site { "id_" } else { "if_" };
        format!("{}{}{}/{}", WAYBACK_WEB_URL_BASE, self.archived_at, from_wb, self.original)
    }
}

pub struct CdxClient {
    client: reqwest::blocking::Client,
}

type CdxResponse = Vec<Vec<String>>;


impl CdxClient {
    fn new() -> Self {
        CdxClient{
            client: reqwest::blocking::Client::new(),
        }
    }
    fn get_query_url(&self, query: &str, unique: bool, from_date: Option<String>, to_date: Option<String>, limit: Option<usize>) -> String {
        let mut query_url = format!("{}{}{}", CDX_BASE, query, CDX_PARAMS);
        if unique {
            query_url.push_str("&collapse=digest");
        }
        if let Some(from) = from_date {
            query_url.push_str(&format!("&from={}", from));
        }
        if let Some(to) = to_date {
            query_url.push_str(&format!("&to={}", to));
        }
        if let Some(l) = limit {
            query_url.push_str(&format!("&limit={}", l));
        }
        query_url
    }
    fn search(&self, url: &str) -> Result<CdxResponse> {
        let resp = self.client
            .get(url)
            .header(reqwest::header::ACCEPT,  "application/json")
            .header(reqwest::header::USER_AGENT, "https://github.com/davemolk/rust-scripts")
            .send()?;
        match resp.status() {
            reqwest::StatusCode::OK => {
                let decoded = resp.json::<CdxResponse>()?;
                return Ok(decoded)
            },
            _ => {
                return Err(anyhow!("unexpected response status {}", resp.status()))
            },
        }
    }
    fn parse_json(json: CdxResponse) -> Result<Vec<CdxInfo>> {
        for row in &json {
            if row.len() != 6 {
                return Err(anyhow!("malformed response, should have 6 elements {:?}", row));
            }
        }
        let mut out: Vec<CdxInfo> = Vec::new();
        // first row in response is the key (unless response is empty, which we checked above)
        // order is original, timestamp, statuscode, mimetype, digest, length (from CDX_PARAMS)
        for row in &json[1..] {
            let parsed = CdxInfo::parse_row(&row[0], &row[1], &row[2], &row[3], &row[4], &row[5])?;
            out.push(parsed);
        }
        Ok(out)
    }
    pub fn get_cdx(query: &str, unique: bool, from_date: Option<String>, to_date: Option<String>, limit: Option<usize>) -> Result<Vec<CdxInfo>> {
        let client = CdxClient::new();
        let query_url: String = client.get_query_url(query, unique, from_date, to_date, limit);
        let res = client.search(&query_url)?;
        if res.is_empty() {
            return Err(anyhow!("no response for cdx"))
        }
        let parsed = Self::parse_json(res)?;
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_query_url_basic() {
        let query = "foo";
        let want = format!("{}{}{}", CDX_BASE, query, CDX_PARAMS);
        let client = CdxClient::new();
        let got = client.get_query_url(query, false, None, None, None);
        assert_eq!(got, want);
    }
    #[test]
    fn get_query_url_unique() {
        let query = "foo";
        let want = format!("{}{}{}&collapse=digest", CDX_BASE, query, CDX_PARAMS);
        let client = CdxClient::new();
        let got = client.get_query_url(query, true, None, None, None);
        assert_eq!(got, want);
    }
    #[test]
    fn get_query_url_from() {
        let query = "foo";
        let from = "2022".to_owned();
        let want = format!("{}{}{}&from={}", CDX_BASE, query, CDX_PARAMS, from);
        let client = CdxClient::new();
        let got = client.get_query_url(query, false, Some(from), None, None);
        assert_eq!(got, want);
    }
    #[test]
    fn get_query_url_to() {
        let query = "foo";
        let to = "2022".to_owned();
        let want = format!("{}{}{}&to={}", CDX_BASE, query, CDX_PARAMS, to);
        let client = CdxClient::new();
        let got = client.get_query_url(query, false, None, Some(to), None);
        assert_eq!(got, want);
    }
    #[test]
    fn get_query_url_limit() {
        let query = "foo";
        let limit = 5;
        let want = format!("{}{}{}&limit={}", CDX_BASE, query, CDX_PARAMS, limit);
        let client = CdxClient::new();
        let got = client.get_query_url(query, false, None, None, Some(limit));
        assert_eq!(got, want);
    }
    #[test]
    fn get_query_url_multiple_params() {
        let query = "foo";
        let unique = true;
        let from = "2022".to_owned();
        let to = "20221031".to_owned();
        let limit = 5;
        let want = format!("{}{}{}&collapse=digest&from={}&to={}&limit={}", CDX_BASE, query, CDX_PARAMS, from, to, limit);
        let client = CdxClient::new();
        let got = client.get_query_url(query, unique, Some(from), Some(to), Some(limit));
        assert_eq!(got, want);
    }
    #[test]
    fn parse_json() {
        let file = std::fs::read_to_string("tests/cdx.json").unwrap();
        let json = serde_json::from_str(&file).unwrap();
        let res = CdxClient::parse_json(json).unwrap();
        // confirm the key is dropped
        assert_eq!(20, res.len());
        assert_eq!("http://davemolk.com/".to_owned(), res[0].original);
    }
}