use std::fs;
use std::collections::HashMap;
use std::io::{self, Write};
use regex::Regex;
use anyhow::{anyhow, Result, Context};

pub fn run(args: Vec<String>) -> Result<()> {
    if args.len() != 2 {
        return Err(anyhow!("not enough arguments provided, need a file name"));
    }
    let content = fs::read_to_string(&args[1]).context("failed to read file")?;
    count_words(&mut io::stdout(), content)?;
    Ok(())
}

fn count_words(output: &mut dyn Write, mut content: String) -> Result<()> {
    content = content.to_lowercase();
    // get rid of punctuation, numbers, etc
    let re = Regex::new(r"[^a-z\s]")?;
    content = re.replace_all(&content, "").to_string();

    let mut counts: HashMap<&str, u64> = HashMap::new();
    for line in content.lines() {
        for word in line.split_whitespace() {
            counts.entry(word).and_modify(|w| *w += 1).or_insert(1);
        }
    }
    let mut count_vec: Vec<_> = counts.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (k, v) in count_vec {
        writeln!(output, "{}: {}", k, v)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn count_words_is_case_insensitive() {
        let content = "foo Foo fOO".to_owned();
        let mut output = Vec::<u8>::new();
        assert!(count_words(&mut output, content).is_ok());
        let got = String::from_utf8_lossy(&output);
        assert_eq!(got, "foo: 3\n".to_owned());
    }
    #[test]
    fn count_empty_string() {
        let content = "".to_owned();
        let mut output = Vec::<u8>::new();
        assert!(count_words(&mut output, content).is_ok());
        let got = String::from_utf8(output).unwrap();
        assert_eq!(got, "".to_string());
    }
    #[test]
    fn strips_punctuation_and_numbers() {
        let content = "fo1o !foo foo.".to_owned();
        let mut output = Vec::<u8>::new();
        assert!(count_words(&mut output, content).is_ok());
        let got = String::from_utf8(output).unwrap();
        assert_eq!(got, "foo: 3\n".to_owned());
    }
    #[test]
    fn strips_whitespace_stuff() {
        let content = "     foo\t\nfoo\n foo\t".to_owned();
        let mut output = Vec::<u8>::new();
        assert!(count_words(&mut output, content).is_ok());
        let got = String::from_utf8(output).unwrap();
        assert_eq!(got, "foo: 3\n".to_owned());
    }
    #[test]
    fn counts_multiple_words() {
        let content = "foo bar bar".to_owned();
        let mut output = Vec::<u8>::new();
        assert!(count_words(&mut output, content).is_ok());
        let got = String::from_utf8(output).unwrap();
        assert_eq!(got, "bar: 2\nfoo: 1\n".to_owned());
    }
}