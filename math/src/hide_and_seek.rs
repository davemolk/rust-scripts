use serde_derive::Deserialize;
use std::collections::HashMap;
use std::thread::sleep;
use std::{fs, io};
use anyhow::{Result, anyhow};
use rand::seq::{IteratorRandom, SliceRandom};
use colored::Colorize;

use crate::ascii;
use crate::util;

#[derive(Debug, Deserialize)]
struct Config {
    // key is floor name
    map: HashMap<String, Floor>,
}

#[derive(Debug, Deserialize, Clone)]
struct Floor {
    /// should be the same as the key in the config
    name: String,
    rooms: Vec<Room>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
struct Room {
    name: String,
    hiding_spots: Vec<HidingSpot>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
struct HidingSpot {
    name: String
}

#[derive(Debug, PartialEq)]
struct ActualHidingSpot {
    name: String,
    room: String,
    floor: String
}

struct Game {
    config: Config,
    hiding_spot: ActualHidingSpot,
    current_floor: String,
    // start on a floor but not in a room
    current_room: Option<Room>,
    floors: Vec<String>,
    get_first_hint: bool,
    get_second_hint: bool,
    done_playing: bool,
}

impl Game {
    fn new(hiding_spot: ActualHidingSpot, config: Config) -> Self {
        // we've validated that there are at least two floors, so unwrap is ok
        let current_floor = config.map.values().next().unwrap();
        let floors: Vec<String> = config.map.keys().cloned().collect();
        Game {
            hiding_spot,
            current_floor: current_floor.name.to_owned(),
            current_room: None,
            config,
            floors,
            get_first_hint: true,
            get_second_hint: true,
            done_playing: false,
        }
    }
    fn run_game_loop(&mut self) {
        println!("search through the house and see what you find...");
        println!("press q at any time to quit\n\n");
        if let Err(e) = self.play() {
            eprintln!("{}", e);
        };
        println!("thanks for playing!!!")
    }
    fn play(&mut self) -> Result<()> {
        // todo: remove secret
        println!("{:?}", self.hiding_spot);
        if self.current_floor == self.hiding_spot.floor {
            self.correct_floor();
        }
        // outer loop lets you change floors
        loop {
            println!("you're on the {}\n", self.current_floor);
            println!("enter 1 to search this floor");
            println!("enter 2 to move to a different floor\n");
            loop {
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                println!();
                let choice = input.trim();
                if choice == "q" {
                    return Ok(())
                }
                if choice == "1" {
                    break
                } else if choice == "2" {
                    self.change_floors()?;
                    break
                }
                eprintln!("input not recognized, please try again");
            }
            // loop lets you pick a floor to search
            loop {
                let want_to_change_floors = self.search_floor()?;
                // go to outer "change floors" loop
                if want_to_change_floors {
                    break
                }
                // loop lets you search rooms
                loop {
                    let want_to_change_rooms = self.search_room()?;
                    if want_to_change_rooms {
                        break
                    }
                    if self.done_playing {
                        return Ok(());
                    }
                }
            }
        }
    }
    fn change_floors(&mut self) -> Result<()> {
        println!("enter the number of the floor you'd like to search");
        for (i, floor) in self.floors.iter().enumerate() {
            println!("{}: {}", i+1, floor);
        }
        println!();
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            println!();
            let floor = input.trim();
            if floor == "q" {
                self.done_playing = true;
                return Err(anyhow!("thanks for playing!\n"));
            }
            let floor_idx = floor.parse::<usize>()?;
            if floor_idx != 0 && floor_idx <= self.floors.len() {
                // update current floor
                let current_floor = &self.floors[floor_idx-1];
                current_floor.clone_into(&mut self.current_floor);
                if self.current_floor == self.hiding_spot.floor {
                    self.correct_floor();
                }
                break
            }
            eprintln!("sorry, {} is not a valid choice, please try again\n", floor);
        }
        Ok(())
    }
    fn search_floor(&mut self) -> Result<bool> {
        let available_rooms = self.config.map.get(&self.current_floor).unwrap();
        println!("\nenter the number of the room you want to search");
        println!("enter d to search a different floor\n");
        for (idx, room) in available_rooms.rooms.iter().enumerate() {
            println!("{}: {}", idx+1, room.name);
        }
        println!();
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            println!();
            let room = input.trim();
            if room == "q" {
                self.done_playing = true;
                return Err(anyhow!("thanks for playing!\n")); 
            }
            if room == "d" {
                return Ok(true)
            }
            let room_idx = room.parse::<usize>()?;
            if room_idx != 0 && room_idx <= available_rooms.rooms.len() {
                let curr_room = &available_rooms.rooms[room_idx-1];
                self.current_room = Some(Room { name: curr_room.name.to_owned(), hiding_spots: curr_room.hiding_spots.clone() });
                return Ok(false)
            }
            eprintln!("sorry, {} is not a valid choice, please try again\n", room);
        }
    }
    fn search_room(&mut self) -> Result<bool> {
        if self.current_room.as_ref().unwrap().name == self.hiding_spot.room {
            self.correct_room();
        }
        let room = self.current_room.as_ref().unwrap();
        println!("you've entered {}\n", room.name);
        println!("enter the number of the hiding spot you want to search");
        println!("enter d to search a different room\n");
        for (idx, hs) in room.hiding_spots.iter().enumerate() {
            println!("{}: {}", idx+1, hs.name);
        }
        println!();
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            println!();
            let spot = input.trim();
            if spot == "q" {
                self.done_playing = true;
                return Err(anyhow!("thanks for playing!\n")); 
            }
            if spot == "d" {
                return Ok(true);
            }
            let spot_idx = spot.parse::<usize>()?;
            if spot_idx == 0 || spot_idx > room.hiding_spots.len() {
                eprintln!("{} is not a valid choice, try again", spot);
                continue
            }
            let choice = &room.hiding_spots[spot_idx-1];
            if choice.name == self.hiding_spot.name && room.name == self.hiding_spot.room && self.current_floor == self.hiding_spot.floor {
                println!("{}\n\n", ascii::BUNNY);
                println!("you win!");
                self.done_playing = true;
                return Ok(false)
            } 
            println!("not there, try again!\n")
        }
    }
    fn correct_floor(&mut self) {
        if !self.get_first_hint {
            return;
        }
        sleep(std::time::Duration::from_millis(1500));
        println!("\nshhhh...\n\n");
        sleep(std::time::Duration::from_millis(1500));
        println!("...you hear...\n\n");
        sleep(std::time::Duration::from_millis(1500));
        println!("...some giggling...\n\n");
        sleep(std::time::Duration::from_millis(1500));
        self.get_first_hint = false;
    }
    fn correct_room(&mut self) {
        if !self.get_second_hint {
            return;
        }
        sleep(std::time::Duration::from_millis(1500));
        println!("\nshhhh...\n\n");
        sleep(std::time::Duration::from_millis(1500));
        println!("...you hear...\n\n");
        sleep(std::time::Duration::from_millis(1500));
        println!("...some LOUD...\n\n");
        sleep(std::time::Duration::from_millis(1500));
        println!("...BREATHING...\n\n");
        sleep(std::time::Duration::from_millis(1500));
        self.get_second_hint = false;
    }
}
fn load_config() -> Result<Config> {
    let data = fs::read_to_string("./src/hs_config.json")?;
    let config: Config = serde_json::from_str(&data)?;
    Ok(config)
}
fn validate_config(config: &Config) -> Result<()> {
    // need a map
    if config.map.is_empty() {
        return Err(anyhow!("config can't be empty"));
    }
    // need multiple floors
    if config.map.len() < 2 {
        return Err(anyhow!("need at least two floors"));
    }
    for (key, floor) in &config.map {
        // no empty rooms
        if floor.rooms.is_empty() {
            return Err(anyhow!("{} needs some rooms", key));
        }
        // key and floor name need to match
        if *key != floor.name {
            return Err(anyhow!("key: {}, name: {}, must match", key, floor.name));
        }
        for room in floor.rooms.iter() {
            if room.hiding_spots.is_empty() {
                return Err(anyhow!("{} needs some hiding spots", room.name));
            }
        }
    }
    Ok(())
}
fn generate_hiding_spot(config: &Config) -> Option<ActualHidingSpot> {
    let mut rng = rand::thread_rng(); 
    let random_floor = config.map.values().choose(&mut rng)?;
    let random_room = random_floor.rooms.choose(&mut rng)?;
    let random_hiding_spot = random_room.hiding_spots.choose(&mut rng)?;
    Some(ActualHidingSpot{
        name: random_hiding_spot.name.to_owned(),
        room: random_room.name.to_owned(),
        floor: random_floor.name.to_owned(),
    })
}
pub fn run_hide_and_seek() -> Result<()> {
    let config = load_config()?;
    validate_config(&config)?;
    let hiding_spot = match generate_hiding_spot(&config) {
        Some(h) => h,
        None => return Err(anyhow!("failed to generate a hiding spot")),
    };
    let mut game = Game::new(hiding_spot, config);
    let (r, g, b) = util::color();
    println!("{}\n\n", ascii::HIDE_AND_SEEK.truecolor(r, g, b));
    game.run_game_loop();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validate_config_error_no_floors() {
        let map: HashMap<String, Floor> = HashMap::new();
        let config = Config{ map };
        assert!(validate_config(&config).is_err());
    }
    #[test]
    fn validate_config_error_one_floor() {
        let mut map: HashMap<String, Floor> = HashMap::new();
        _ = map.insert("First Floor".to_owned(), Floor{
            name: "First Floor".to_owned(),
            rooms: vec![Room{name: "bathroom".to_owned(), hiding_spots: vec![
                HidingSpot{
                    name: "under sink".to_owned()
                }
            ]}],
        });
        let config = Config{ map };
        assert!(validate_config(&config).is_err());
    }
    #[test]
    fn validate_config_error_floor_with_no_rooms() {
        let mut map: HashMap<String, Floor> = HashMap::new();
        _ = map.insert("First Floor".to_owned(), Floor{
            name: "First Floor".to_owned(),
            rooms: vec![Room{name: "bathroom".to_owned(), hiding_spots: vec![
                HidingSpot{
                    name: "under sink".to_owned()
                }
            ]}],
        });
        _ = map.insert("Second Floor".to_owned(), Floor { 
            name: "Second Floor".to_owned(), 
            rooms: vec![], 
        });
        let config = Config{ map };
        assert!(validate_config(&config).is_err());
    }
    #[test]
    fn validate_config_error_room_with_no_hiding_spots() {
        let mut map: HashMap<String, Floor> = HashMap::new();
        _ = map.insert("First Floor".to_owned(), Floor{
            name: "First Floor".to_owned(),
            rooms: vec![Room{name: "bathroom".to_owned(), hiding_spots: vec![
                HidingSpot{
                    name: "under sink".to_owned()
                }
            ]}],
        });
        _ = map.insert("Second Floor".to_owned(), Floor { 
            name: "Second Floor".to_owned(), 
            rooms: vec![Room{name: "bathroom".to_owned(), 
                hiding_spots: vec![]}],
        });
        let config = Config{ map };
        assert!(validate_config(&config).is_err());
    }
    #[test]
    fn validate_config_error_key_not_match_name() {
        let mut map: HashMap<String, Floor> = HashMap::new();
        _ = map.insert("First Floor".to_owned(), Floor{
            name: "First Floor".to_owned(),
            rooms: vec![Room{name: "bathroom".to_owned(), hiding_spots: vec![
                HidingSpot{
                    name: "under sink".to_owned()
                }
            ]}],
        });
        _ = map.insert("Second Floor".to_owned(), Floor { 
            name: "second floor".to_owned(), 
            rooms: vec![Room{name: "bathroom".to_owned(), hiding_spots: vec![
                HidingSpot{
                    name: "under sink".to_owned()
                }
            ]}],
        });
        let config = Config{ map };
        assert!(validate_config(&config).is_err());
    }
    #[test]
    fn validate_config_success() {
        let mut map: HashMap<String, Floor> = HashMap::new();
        _ = map.insert("First Floor".to_owned(), Floor{
            name: "First Floor".to_owned(),
            rooms: vec![Room{name: "bathroom".to_owned(), hiding_spots: vec![
                HidingSpot{
                    name: "under sink".to_owned()
                }
            ]}],
        });
        _ = map.insert("Second Floor".to_owned(), Floor{
            name: "Second Floor".to_owned(),
            rooms: vec![Room{name: "bathroom".to_owned(), hiding_spots: vec![
                HidingSpot{
                    name: "under sink".to_owned()
                }
            ]}],
        });
        let config = Config{ map };
        assert!(validate_config(&config).is_ok());
    }
    #[test]
    fn generate_hiding_spot_always_some_with_valid_config() {
        let f = fs::read_to_string("./example/dummy_hs_config.json").unwrap();
        let config:Config = serde_json::from_str(&f).unwrap();
        for _ in [1..=100] {
            assert!(generate_hiding_spot(&config).is_some());
        }
    }
}
