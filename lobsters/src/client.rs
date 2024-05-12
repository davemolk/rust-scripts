use core::fmt;
use std::collections::{self, HashMap};

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
    tags: Vec<String>,
}

impl fmt::Display for Post {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Title: {}\n\
            URL: {}\n\
            Tags: {:?}\n\
            Comment Count: {}\n\
            ID: {}\n\
            ", self.title, self.url, self.tags, self.comment_count, self.short_id
        )
    }
}

#[derive(Debug, Deserialize)]
struct CommentsResponse {
    title: String,
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
        for post in &resp {
            println!("{post}");
        }
        // check if user wants to see comments
        println!("enter an id to see the comments, or press any key to quit");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        if !resp.iter().any(|e| input == e.short_id) {
            return Ok(())
        }
        println!();
        println!();
        let mut map: HashMap<&str, usize> = collections::HashMap::new();
        let comment_url = format!("https://lobste.rs/s/{}.json", input);
        let resp = self.get_comments(&comment_url)?;
        println!("{} comments:", resp.title);
        for comment in &resp.comments {
            match &comment.parent_comment {
                None => {
                    map.entry(&comment.short_id).or_default();
                },
                Some(parent) => {
                    // subsequent comments will always have a parent (or be a parent)
                    let v = map.get(parent.as_str()).unwrap();
                    // need to add extra tab for each child-level
                    map.insert(&comment.short_id, *v+1);
                },
                
            }
        }
        // go through resp.comments, have prefix as repeated \t
        for comment in &resp.comments {
            // we know elements are in the map, so can unwrap
            let value = map.get(comment.short_id.as_str()).unwrap();
            let prefix = "\t".repeat(*value);
            // close enough, keeps it in the same tab-level
            let c = comment.comment_plain.replace("\r\n\r\n", " ");
            println!("\n{prefix}* {}", c);
        }
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