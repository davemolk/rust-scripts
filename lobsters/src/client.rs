use core::fmt;
use std::collections::{self, HashMap};
use std::io::{self, Write};
use open;

use serde_derive::Deserialize;
use anyhow::{Result, anyhow};

const URL_HOTTEST: &str = "https://lobste.rs/hottest.json";
const URL_NEWEST: &str = "https://lobste.rs/newest.json";

type ApiResponse = Vec<Post>;

pub const USAGE: &str = r"Usage:
lobsters hot    get the hottest posts (default is newest)
";

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
            "Title:         {}\n\
            URL:           {}\n\
            Tags:          {:?}\n\
            Comment Count: {}\n\
            ID:            {}\n\
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
    client: reqwest::blocking::Client,
}

impl LobsterClient {
    pub fn new() -> Self {
        LobsterClient {
            client: reqwest::blocking::Client::new(),
        }
    }
    pub fn run(&self, args: Vec<String>) -> Result<()> {
        if !args.is_empty() && args[0].to_lowercase() == "help" {
            return Err(anyhow!("{}", USAGE))
        }
        let url = if args.is_empty() || args[0].to_lowercase() != "hot" { URL_NEWEST } else { URL_HOTTEST };
        let resp = self.get_lobsters(url)?;
        for post in &resp {
            println!("{post}");
        }
        self.prompt_user(&resp)?;
        Ok(())
    }
    fn get_lobsters(&self, url: &str) -> Result<ApiResponse> {
        let resp = self.client.get(url)
            .header(reqwest::header::ACCEPT,  "application/json")
            .header(reqwest::header::USER_AGENT, "https://github.com/davemolk/rust-scripts")
            .send()?
            .json::<ApiResponse>()?;
        Ok(resp)
    }
    fn prompt_user(&self, posts: &Vec<Post>) -> Result<()> {
        println!("type 'open <id>' to open the url in a browser, the <id> to see the post's comments, or press any key to quit");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        if input.starts_with("open") {
            self.open_browser(&input, posts)?;
        }
        if !posts.iter().any(|e| input == e.short_id) {
            return Err(anyhow!("that id is not recognized"));
        }
        self.print_comments(&mut io::stdout(), &input)?;
        Ok(())
    }
    fn open_browser(&self, input: &str, posts: &Vec<Post>) -> Result<()> {
        let parts: Vec<&str> = input.split(" ").collect();
        if parts.len() != 2 {
            return Err(anyhow!("to open a url in a browser, format as 'open s2zxwx'"));
        }
        for post in posts.iter() {
            if post.short_id == parts[1] {
                open::that(&post.url)?;
                return Ok(())
            }
        }
        return Err(anyhow!("unable to find that id, please try again"))
    }
    fn print_comments(&self, output: &mut dyn Write, input: &str) -> Result<()> {
        let mut map: HashMap<&str, usize> = collections::HashMap::new();
        let comment_url = format!("https://lobste.rs/s/{}.json", input);
        let resp = self.get_comments(&comment_url)?;
        writeln!(output, "\n\n{} comments:", resp.title)?;
        for comment in &resp.comments {
            match &comment.parent_comment {
                None => {
                    map.entry(&comment.short_id).or_default();
                },
                Some(parent) => {
                    // subsequent comments will always have a parent (or be a parent)
                    // so we can unwrap here
                    let v = map.get(parent.as_str()).unwrap();
                    // need to add extra tab for each child-level
                    map.insert(&comment.short_id, *v+1);
                },
                
            }
        }
        // go through comments, prefixing with \t in order to create nesting in the console
        for comment in &resp.comments {
            // we know elements are in the map, so can unwrap
            let value = map.get(comment.short_id.as_str()).unwrap();
            let prefix = "\t".repeat(*value);
            // close enough, keeps it in the same tab-level
            let c = comment.comment_plain.replace("\r\n\r\n", " ");
            writeln!(output, "\n{prefix}* {}", c)?;
        }
        Ok(())
    }
    fn get_comments(&self, url: &str) -> Result<CommentsResponse> {
        let resp = self.client.get(url)
            .header(reqwest::header::ACCEPT,  "application/json")
            .header(reqwest::header::USER_AGENT, "https://github.com/davemolk/rust-scripts")
            .send()?
            .json::<CommentsResponse>()?;
        Ok(resp)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    trait LobsterAPI {
        fn get_lobsters(&self, url: &str) -> Result<ApiResponse>;
        fn get_comments(&self, url: &str) -> Result<CommentsResponse>;
    }
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
        assert_eq!(resp.title, "Lila: a Lil Interpreter in Awk");
        assert_eq!(resp.comments.len(), 8);
    }
}