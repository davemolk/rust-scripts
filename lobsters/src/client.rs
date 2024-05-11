use core::fmt;
use std::fmt::format;

use serde_derive::Deserialize;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long, action)]
    hottest: bool,
}

const URL_HOTTEST: &str = "https://lobste.rs/hottest.json";
const URL_NEWEST: &str = "https://lobste.rs/newest.json";

type ApiResponse = Vec<Post>;

#[derive(Debug, Deserialize)]
struct Post {
    short_id: String,
    title: String,
    url: String,
    comment_count: u32,
    comments_url: String,
    tags: Vec<String>,
}

impl fmt::Display for Post {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Title: {}\n\
            URL: {}\n\
            Tags: {:?}\n\
            ID: {}\n\
            Num Comments: {}\n\
            Comments URL: {}\n\
            ", self.title, self.url, self.tags, self.short_id, self.comment_count, self.comments_url
        )
    }
}

#[derive(Debug, Deserialize)]
struct CommentsResponse {
    comments: Vec<Comment>,
}

#[derive(Debug, Deserialize)]
struct Comment {
    short_id: String,
    parent_comment: Option<String>,
    comment_plain: String,
}

pub struct LobsterClient {
    args: Args,
    client: reqwest::blocking::Client,
}

trait LobsterAPI {
    fn get_lobsters(&self, url: &str) -> Result<ApiResponse>;
    fn get_comments(&self, url: &str) -> Result<CommentsResponse>;
}

impl LobsterClient {
    pub fn new(args: Args) -> Self {
        LobsterClient { 
            args,
            client: reqwest::blocking::Client::new(),
        }
    }
    pub fn run(&self) -> Result<()> {
        let mut url: &str = URL_NEWEST;
        if self.args.hottest {
            url = URL_HOTTEST;
        }
        let resp = self.get_lobsters(url)?;
        let mut short_ids = Vec::new();
        for post in resp {
            short_ids.push(post.short_id.clone());
            println!("{post}");
        }
        println!("enter an id to see the comments, or press any key to quit");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !short_ids.iter().any(|e| input.contains(e)) {
            return Ok(())
        }
        let comment_url = format!("https://lobste.rs/s/{}.json", input);
        let comments = self.get_comments(&comment_url)?;
        dbg!(comments);
        Ok(())
    }
    fn get_lobsters(&self, url: &str) -> Result<ApiResponse> {
        let resp = self.client.get(url)
            .header(reqwest::header::ACCEPT,  "application/json")
            .send()?
            .json::<ApiResponse>()?;
        Ok(resp)
    }
    fn get_comments(&self, url: &str) -> Result<CommentsResponse> {
        let resp = self.client.get(url)
            .header(reqwest::header::ACCEPT,  "application/json")
            .send()?
            .json::<CommentsResponse>()?;
        Ok(resp)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    struct ClientMock{}
        impl LobsterAPI for ClientMock {
            fn get_lobsters(&self, _url: &str) -> Result<ApiResponse>  {
                let data = std::fs::read_to_string("newest_response.json").expect("failed to read newest test data");
                let resp: ApiResponse = serde_json::from_str(&data)?;
                Ok(resp)
            }
            fn get_comments(&self, _url: &str) -> Result<CommentsResponse> {
                let data = std::fs::read_to_string("comments_response.json").expect("failed to read comments test data");
                let resp: CommentsResponse = serde_json::from_str(&data)?;
                Ok(resp)
            }
        }
    #[test]
    fn get_lobsters() {
        let client = ClientMock{};
        let resp = client.get_lobsters("foo").expect("get lobsters failed");
        assert_eq!(resp[0].title, "The await event horizon in JavaScript")  
    }
    #[test]
    fn get_comments() {
        let client = ClientMock{};
        let resp = client.get_comments("blah").expect("get comments failed");
        dbg!(resp);
    }
}