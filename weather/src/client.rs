use reqwest::blocking::get;
use anyhow::{Result, anyhow};
use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[clap(short, long)]
    days: u32,
    #[clap(short, long)]
    location: String,
    #[clap(long)]
    #[arg(value_parser = parse_hours)]
    hours: u32,
}

fn parse_hours(arg: &str) -> Result<u32> {
    let hours = arg.parse::<u32>()?;
    anyhow::ensure!(hours < 24, "no more than 24 hours in a day");
    Ok(hours)
}

pub struct Client {
    args: Args,
    client: reqwest::blocking::Client,
}

// const OPEN_METEO: &str = "https://api.open-meteo.com/v1/forecast?latitude=39.73915&longitude=-104.9847&daily=temperature_2m_max,temperature_2m_min&timezone=America%2FDenver";

const URL: &str = "https://api.weather.gov/points/39.73915,-104.9847";

impl Client {
    pub fn new(args: Args) -> Self {
        Client { 
            args, 
            client: reqwest::blocking::Client::new() 
        }
    }
    pub fn run(&self) -> Result<()> {
        // let client = req
        let resp: serde_json::Value = self.client.get(URL)
            .header(reqwest::header::ACCEPT,  "application/json")
            .header(reqwest::header::USER_AGENT,  "myWeatherApp, testing")
            .send()?
            .json()?;
        println!("{:?}", resp);
        Ok(())
    }
}