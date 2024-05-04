use std::fs::read_to_string;

use anyhow::{Result, anyhow};
use chrono::{Datelike, Local, Weekday};
use serde_derive::Deserialize;

const URL_BASE: &str = "https://webapis.schoolcafe.com/api/CalendarView/GetDailyMenuitemsByGrade?SchoolId=";

#[derive(Debug, Deserialize)]
pub struct Config {
    school_id: String,
    grade: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct LunchResponse {
    pub entree: Vec<FoodResponse>
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FoodResponse {
    pub menu_item_description: String, 
}

pub trait FoodApi {
    fn get_lunch(&self, url: String) -> Result<LunchResponse>;
}

pub struct LunchClient {}

impl LunchClient {
    pub fn new() -> Self {
        LunchClient{}
    }
    pub fn get_url(&self) -> Result<String> {
        // get school id and grade
        let config_string = read_to_string("config.json").expect("failed to read config");
        let config: Config = serde_json::from_str(&config_string)?;
        // avoid unnecessary api call
        match Local::now().weekday() {
            Weekday::Sat | Weekday::Sun => { return Err(anyhow!("no school on the weekend")); },
            _ => {},
        }
        let local = Local::now().date_naive();
        let month = if local.month() < 10 { format!("0{}", local.month()) } else { local.month().to_string() };
        let date = format!{"{}%2F{}%2F{}", month, local.day(), local.year()};
        
        Ok(format!("{URL_BASE}{}&ServingDate={}&ServingLine=Traditional%20Lunch&MealType=Lunch&Grade={}&PersonId=null", config.school_id, date, config.grade))
    }
}

impl FoodApi for LunchClient {
    fn get_lunch(&self, url: String) -> Result<LunchResponse> {
        let client = reqwest::blocking::Client::new();
        let resp = client.get(url)
            .header(reqwest::header::ACCEPT,  "application/json")
            .send()?
            .json::<LunchResponse>()?;
        Ok(resp)
    }
}