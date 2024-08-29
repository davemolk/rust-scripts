use std::fs::read_to_string;
use std::path::Path;

use anyhow::{Result, anyhow};
use chrono::{Datelike, Local, Weekday};
use serde_derive::Deserialize;

const URL_BASE: &str = "https://webapis.schoolcafe.com/api/CalendarView/GetDailyMenuitemsByGrade?SchoolId=";

#[derive(Debug, Deserialize)]
struct Config {
    school_id: String,
    grade: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct LunchResponse {
    entree: Vec<FoodResponse>
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FoodResponse {
    menu_item_description: String, 
}

trait FoodApi {
    fn get_lunch(&self, url: String) -> Result<LunchResponse>;
}

pub struct LunchClient {}

impl LunchClient {
    pub fn new() -> Self {
        LunchClient{}
    }
    fn get_url(&self) -> Result<String> {
        // get school id and grade
        let config: Config = match self.get_config_path() {
            Some(config_path) => {
                let config_string = read_to_string(config_path).expect("failed to read config");
                serde_json::from_str(&config_string)?
            },
            None => return Err(anyhow!("config file not found")),
        };
        
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
    pub fn run(&self) -> Result<()> {
        let url = self.get_url()?;
        let menu_options = self.get_lunch(url)?;
        for option in menu_options.entree {
            println!("{}", option.menu_item_description);
        } 
        Ok(())
    }
    fn get_config_path(&self ) -> Option<String> {
        let home_dir = std::env::var("HOME").ok();
        let standard_path = Path::new("/etc/lunch/config.json");
        let home_path = home_dir.map(|d| Path::new(&d).join(".lunch/config.json"));

        if standard_path.exists() {
            Some(standard_path.to_str().unwrap().to_string())
        } else if let Some(path) = home_path {
            if path.exists() {
                Some(path.to_str().unwrap().to_string())
            } else {
                None
            }
        } else {
            None
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    struct ClientMock{}
    impl FoodApi for ClientMock {
        fn get_lunch(&self, _url: String) -> Result<LunchResponse> {
            let data = std::fs::read_to_string("menu_response.json").expect("failed to read test data");
            let resp: LunchResponse = serde_json::from_str(&data)?;
            Ok(resp)
        }
    }
    #[test]
    fn get_lunch() {
        let client = ClientMock {};
        let resp = client.get_lunch("blah".to_string()).expect("get lunch failed");
        assert_eq!(resp.entree[0].menu_item_description, "Yogurt Basket with Fresh Baked Blueberry Muffin");
    }
}
