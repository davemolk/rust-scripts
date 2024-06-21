use std::fs;
use std::collections::HashMap;
use regex::Regex;
use anyhow::{anyhow, Result, Context};

pub fn find_frequencies(args: Vec<String>) -> Result<()> {
    if args.len() != 2 {
        return Err(anyhow!("not enough arguments provided, need a file name"));
    }
    // get rid of punctuation, numbers, etc
    let re = Regex::new(r"[^a-z\s]")?;
    let mut counts: HashMap<&str, u64> = HashMap::new();
    let mut file_contents = fs::read_to_string(&args[1]).context("failed to read file")?;
    file_contents = file_contents.to_lowercase();
    file_contents = re.replace_all(&file_contents, "").to_string();
    for line in file_contents.lines() {
        for word in line.split_whitespace() {
            counts.entry(word).and_modify(|w| *w += 1).or_insert(1);
        }
    }
    let mut count_vec: Vec<_> = counts.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (k, v) in count_vec {
        println!("{}: {}", k, v);
    }
    Ok(())
}

