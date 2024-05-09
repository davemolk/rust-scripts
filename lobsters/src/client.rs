use core::fmt;

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
    short_id_url: String,
    created_at: String,
    title: String,
    url: String,
    // score: i32,
    // flags: i32,
    comment_count: u32,
    description: String,
    description_plain: String,
    comments_url: String,
    submitter_user: String,
    // user_is_author: bool,
    tags: Vec<String>,
}

impl fmt::Display for Post {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Title: {}\n\
            URL: {}\n\
            ID: {}\n\
            Comments URL: {}\n\
            Tags: {:?}\n\
            ", self.short_id, self.title, self.url, self.comments_url, self.tags
        )
    }
}

pub struct LobsterClient {
    args: Args,
}

impl LobsterClient {
    pub fn new(args: Args) -> Self {
        LobsterClient { args }
    }
    pub fn run(&self) -> Result<()> {
        let mut url: &str = URL_NEWEST;
        if self.args.hottest {
            url = URL_HOTTEST;
        }
        let client = reqwest::blocking::Client::new();
        let resp = client.get(url)
            .header(reqwest::header::ACCEPT,  "application/json")
            .send()?
            .json::<ApiResponse>()?;
        for post in resp {
            println!("{post}");
        }
        Ok(())
    }
}