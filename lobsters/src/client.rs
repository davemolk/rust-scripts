use core::fmt;
use std::collections::{self, HashMap, HashSet};
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
        let posts = self.get_lobsters(url)?;
        for post in &posts {
            println!("{post}");
        }
        let input = self.prompt_user()?;
        let post_url = self.get_url_from_user_input(input, &posts)?;
        // comments
        if post_url.starts_with("https://lobste") {
            self.print_comments(&mut io::stdout(), &post_url)?;
        } else {
            open::that(post_url)?;
        }
        Ok(())
    }
    fn prompt_user(&self) -> Result<String> {
        println!("type 'open <id>' to open the url in a browser, the <id> to see the post's comments, or press any key to quit");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string().to_lowercase())
    }
    fn get_lobsters(&self, url: &str) -> Result<ApiResponse> {
        let resp = self.client.get(url)
            .header(reqwest::header::ACCEPT,  "application/json")
            .header(reqwest::header::USER_AGENT, "https://github.com/davemolk/rust-scripts")
            .send()?;
        match resp.status() {
            reqwest::StatusCode::OK => {
                let decoded = resp.json::<ApiResponse>()?;
                return Ok(decoded);
            }
            _ => {
                return Err(anyhow!("got unexpected status code: {}", resp.status()));
            }
        }
    }
    fn get_url_from_user_input(&self, input: String, posts: &Vec<Post>) -> Result<String> {
        let parts: Vec<&str> = input.split(" ").collect();
        // open browser
        if parts.len() > 1 {
            return self.get_browser_url(&input, posts);
        } 
        self.get_comments_url(&input, posts)
    }
    fn get_browser_url(&self, input: &str, posts: &Vec<Post>) -> Result<String> {
        if input.len() < 6 {
            return Err(anyhow!("to open a url in a browser, format as 'open s2zxwx' or 'open <fragment of the title>'"));
        }
        // skip "open" plus the space
        let cropped = match input.char_indices().skip(5).next() {
            Some((pos, _)) => &input[pos..],
            None => "",
        };
        if cropped.is_empty() {
            return Err(anyhow!("please supply part of a post title or its id"));
        }
        self.check_for_matches(cropped, posts, false)
    }
    fn check_for_matches(&self, input: &str, posts: &Vec<Post>, for_comments: bool) -> Result<String> {
        // check for multiple matches for the input
        let mut matches: Vec<&str> = vec![];
        let mut out = String::new();
        for post in posts.iter() {
            if post.title.to_lowercase().starts_with(input) {
                matches.push(&post.title);
                if for_comments {
                    out = post.short_id.to_owned();
                } else {
                    out = post.url.to_owned();
                }
            }
            if post.short_id.to_lowercase().starts_with(input) {
                matches.push(&post.short_id);
                if for_comments {
                    out = post.short_id.to_owned();
                } else {
                    out = post.url.to_owned();
                }
            }
        }
        if matches.len() > 1 {
            return Err(anyhow!("multiple matches for that input, please try again: {:?}", matches));
        }  
        if matches.is_empty() {
            return Err(anyhow!("unable to find that title or id, please try again"));
        }
        Ok(out)
    }
    fn get_comments_url(&self, input: &str, posts: &Vec<Post>) -> Result<String>  {
        match self.check_for_matches(input, posts, true) {
            Ok(short_id) => Ok(format!("https://lobste.rs/s/{}.json", short_id)),
            Err(e) => Err(e),
        }
    }
    fn print_comments(&self, output: &mut dyn Write, comment_url: &str) -> Result<()> {
        let mut map: HashMap<&str, usize> = collections::HashMap::new();
        let resp = self.get_comments(comment_url)?;
        writeln!(output, "\n\ncomments for {}:", resp.title)?;
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
        assert_eq!(resp[0].title, "The await event horizon in JavaScript");
    }
    #[test]
    fn get_comments() {
        let client = ClientMock{};
        let resp = client.get_comments("blah").expect("get comments failed");
        assert_eq!(resp.title, "Lila: a Lil Interpreter in Awk");
        assert_eq!(resp.comments.len(), 8);
    }
    #[test]
    fn get_browser_url_input_too_short() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").expect("failed to read newest test data");
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        let input = "foo";
        assert!(l.get_browser_url(input, &posts).is_err());
    }
    #[test]
    fn get_browser_url_input_missing_identifier() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").expect("failed to read newest test data");
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        let input = "open ";
        assert!(l.get_browser_url(input, &posts).is_err());
    }
    #[test]
    fn get_browser_url_input_ambiguous_identifier() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").expect("failed to read newest test data");
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        // matches short_id and title
        let input = "open a";
        assert!(l.get_browser_url(input, &posts).is_err());
    }
    #[test]
    fn get_browser_url_no_matches() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").expect("failed to read newest test data");
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        let input = "open b";
        assert!(l.get_browser_url(input, &posts).is_err());
    }
    #[test]
    fn get_browser_url_input_success_partial_title_match() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").expect("failed to read newest test data");
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        let input = "open the";
        assert!(l.get_browser_url(input, &posts).is_ok());
        let got = l.get_browser_url(input, &posts).unwrap();
        assert_eq!(got, posts[0].url);
    }
    #[test]
    fn get_browser_url_input_success_full_id() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").expect("failed to read newest test data");
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        let input = "open aedhvm";
        assert!(l.get_browser_url(input, &posts).is_ok());
        let got = l.get_browser_url(input, &posts).unwrap();
        assert_eq!(got, posts[0].url);
    }
    #[test]
    fn get_comments_url_success() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").expect("failed to read newest test data");
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        let input = "aedhvm";
        assert!(l.get_comments_url(input, &posts).is_ok());
        let got = l.get_comments_url(input, &posts).unwrap();
        assert_eq!(got, format!("https://lobste.rs/s/{}.json", input));
    }
    #[test]
    fn get_comments_url_failure() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").expect("failed to read newest test data");
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        let input = "b";
        assert!(l.get_comments_url(input, &posts).is_err());
    }
    #[test]
    fn check_for_matches_no_matches() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").expect("failed to read newest test data");
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        let input = "b";
        // for comments
        assert!(l.check_for_matches(input, &posts, true).is_err());        
        // for titles
        assert!(l.check_for_matches(input, &posts, false).is_err());
    }
    #[test]
    fn check_for_matches_multiple_matches() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").unwrap();
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        let input = "a";
        // comments
        assert!(l.check_for_matches(input, &posts, true).is_err());
        // titles
        assert!(l.check_for_matches(input, &posts, false).is_err());
    }
    #[test]
    fn check_for_matches_success_partial_id() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").unwrap();
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        let input = "q";
        // comments
        assert!(l.check_for_matches(input, &posts, true).is_ok());
        let got = l.check_for_matches(input, &posts, true).unwrap();
        assert_eq!(got, posts[2].short_id);
        // titles
        assert!(l.check_for_matches("b", &posts, false).is_err());
    }
    #[test]
    fn check_for_matches_success_full_id() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").unwrap();
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        let input = "qyaupk";
        // comments
        assert!(l.check_for_matches(input, &posts, true).is_ok());
        let got = l.check_for_matches(input, &posts, true).unwrap();
        assert_eq!(got, posts[2].short_id);
        // titles
        assert!(l.check_for_matches("b", &posts, false).is_err());
    }
    #[test]
    fn check_for_matches_success_partial_title() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").unwrap();
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        let input = "the";
        // comments
        assert!(l.check_for_matches("b", &posts, true).is_err());
        // title
        assert!(l.check_for_matches(input, &posts, false).is_ok());
        let got = l.check_for_matches(input, &posts, false).unwrap();
        assert_eq!(got, posts[0].url);
    }
    #[test]
    fn check_for_matches_success_full_title() {
        let l = LobsterClient::new();
        let data = std::fs::read_to_string("newest_response.json").unwrap();
        let posts: ApiResponse = serde_json::from_str(&data).unwrap();
        let input = "the await event horizon in javascript";
        // comments
        assert!(l.check_for_matches("b", &posts, true).is_err());
        // titles
        assert!(l.check_for_matches(input, &posts, false).is_ok());
        let got = l.check_for_matches(input, &posts, false).unwrap();
        assert_eq!(got, posts[0].url);
    }
}
