use serde;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::{any, fs};
use serde_json;
use anyhow::{Result, anyhow};
use rand::thread_rng;
use rand::seq::SliceRandom;

#[derive(Debug, Deserialize)]
struct Config {
    // key is floor name
    map: HashMap<String, Floor>,
}

#[derive(Debug, Deserialize)]
struct Floor {
    rooms: Vec<Room>,
}

#[derive(Debug, Deserialize)]
struct Room {
    name: String,
    hiding_spots: Vec<HidingSpot>,
}

#[derive(Debug, Deserialize, Clone)]
struct HidingSpot {
    name: String,
}

struct Game {
    config: Config,
    hiding_spot: HidingSpot,
    current_floor: Floor,
    current_room: Option<Room>,
}

impl Game {
    fn new(hiding_spot: HidingSpot, config: Config) -> Self {
        // we've validated that there is at least one floor
        let floor = config.map.values().next().unwrap();
        let room = &floor.rooms[0];
        let new_floor = Floor{
            rooms: vec![Room{
                name: room.name.to_owned(),
                hiding_spots: vec![],
            }]
        };
        Game {
            hiding_spot: hiding_spot,
            current_floor: new_floor,
            current_room: None,
            config: config,
        }
    }
}

// pub fn populate_game_board() -> Result() {
//     let mut map = HashMap::new();
    
// }

pub fn run_hide_and_seek() -> Result<()> {
    let config = load_config()?;
    validate_config(&config)?;
    for (floor_name, floor) in config.map {
        println!("floor: {}", floor_name);
        for room in floor.rooms {
            println!("room: {}", room.name);
            for spot in room.hiding_spots {
                println!("hiding spots: {}", spot.name);
            }
        }
    }
    Ok(())
}

fn load_config() -> Result<Config> {
    let data = fs::read_to_string("./src/hs_config.json")?;
    let config: Config = serde_json::from_str(&data)?;
    Ok(config)
}

fn validate_config(config: &Config) -> Result<()> {
    if config.map.is_empty() {
        return Err(anyhow!("config can't be empty"));
    }
    for (key, value) in &config.map {
        if value.rooms.is_empty() {
            return Err(anyhow!("{} needs some rooms", key));
        }
    }
    Ok(())
}

fn generate_hiding_spot(config: &Config) -> Option<HidingSpot> {
    let mut rng = thread_rng();
    let floor_keys: Vec<&String> = config.map.keys().collect();
    let random_floor_key = floor_keys.choose(&mut rng)?;
    let floor_key_str: &str = &random_floor_key.as_ref();
    let floor = &config.map[floor_key_str];
    let random_room = floor.rooms.choose(&mut rng)?;
    let random_hiding_spot = random_room.hiding_spots.choose(&mut rng)?;
    Some(random_hiding_spot.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_config_no_floors() {
        let map: HashMap<String, Floor> = HashMap::new();
        let config = Config{ map };
        assert!(validate_config(&config).is_err());
    }
    #[test]
    fn validate_config_no_rooms() {
        let mut map: HashMap<String, Floor> = HashMap::new();
        _ = map.insert("First Floor".to_owned(), Floor{
            rooms: vec![],
        });
        let config = Config{ map };
        assert!(validate_config(&config).is_err());
    }
    #[test]
    fn validate_config_success() {
        let mut map: HashMap<String, Floor> = HashMap::new();
        _ = map.insert("First Floor".to_owned(), Floor{
            rooms: vec![Room{name: "bathroom".to_owned(), hiding_spots: vec![]}],
        });
        let config = Config{ map };
        assert!(validate_config(&config).is_ok());
    }
}
