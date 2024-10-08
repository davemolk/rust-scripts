use rand::{thread_rng, Rng};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::thread::sleep;
use std::{fs, io};
use anyhow::{Result, anyhow};
use rand::seq::{IteratorRandom, SliceRandom};
use colored::Colorize;

use super::{
    ascii,
    color,
};

#[derive(Debug, Deserialize)]
struct Config {
    // key is floor name
    house_map: HashMap<String, Floor>,
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

#[derive(Debug, Clone, PartialEq)]
enum Difficulty {
    Easy,
    Medium,
    MediumAdvanced,
    Advanced,
    Expert,
    Custom,
}

impl Difficulty {
    fn print_difficulties() {
        let choices = [
            Difficulty::Easy,
            Difficulty::Medium,
            Difficulty::MediumAdvanced,
            Difficulty::Advanced,
            Difficulty::Expert,
            Difficulty::Custom,
        ];
        for (i, d) in choices.iter().enumerate() {
            println!("{}: {:?}", i+1, d);
        }
    }
}

const ALL_FLOORS: [&str; 3] = ["First Floor", "Second Floor", "Basement Floor"];
const ALL_ROOMS: [&str; 7] = [
    "Bedroom", "Bathroom", "Office", "Kitchen", "Playroom", "Dining Room", "Closet",
];

fn get_room_hiding_spots() -> HashMap<&'static str, Vec<&'static str>> {
    let mut map = HashMap::new();
    map.insert("Bedroom", vec!["under the covers", "under the bed", "in the bed", "behind the pillow", "under the pillow", "in the closet", "behind the chair", "under the chair"]);
    map.insert("Bathroom", vec!["behind the tub", "on the potty", "in the tub", "in the potty", "under the sink", "behind the towels"]);
    map.insert("Office", vec!["under the desk", "under the table", "behind the lamp", "on the chair", "under the chair", "behind the chair", "in the closet"]);
    map.insert("Kitchen", vec!["in the fridge", "in the cupboard", "under the sink", "in the sink", "in the freezer", "in the dishwasher"]);
    map.insert("Playroom", vec!["in the toy chest", "behind the toy chest", "in the toy truck", "among the toy trains", "among the stuffies"]);
    map.insert("Dining Room", vec!["under the table", "on the table", "under a chair", "on a chair", "behind a chair", "behind the plants"]);
    map.insert("Closet", vec!["behind the clothes", "under the clothes", "among the shoes", "behind the shoes", "behind the towels", "behind the coats"]);
    map
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
        let current_floor = config.house_map.values().next().expect("no floor");
        let floors: Vec<String> = config.house_map.keys().cloned().collect();
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
        println!("listen for hints that you're getting close!");
        println!("press q at any time to quit\n\n");
        if let Err(e) = self.play() {
            eprintln!("{}", e);
        };
        println!("thanks for playing!!!")
    }
    fn play(&mut self) -> Result<()> {
        // todo: remove secret
        // println!("{:?}", self.hiding_spot);
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
                return Ok(());
                // return Err(anyhow!("thanks for playing!\n"));
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
        let available_rooms = self.config.house_map.get(&self.current_floor).unwrap();
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
        println!("you've entered the {}\n", room.name);
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
            let nope = Self::get_nope();
            println!("{}\n", nope);
        }
    }
    fn get_nope() -> String {
        let mut rng = rand::thread_rng();
        let choice = match ["nope", "not there", "keep looking", "not quite", "try again", "almost", "keep going"].choose(&mut rng) {
            Some(c) => c,
            None => "nope"
        };
        choice.to_string()
    }
    fn correct_floor(&mut self) {
        if !self.get_first_hint {
            return;
        }
        if !Self::should_show_hint() {
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
    fn should_show_hint() -> bool {
        let mut rng = rand::thread_rng();
        let show = rng.gen_range(0..=1);
        show %2 == 0 
    }
    fn correct_room(&mut self) {
        if !self.get_second_hint {
            return;
        }
        if !Self::should_show_hint() {
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
    if config.house_map.is_empty() {
        return Err(anyhow!("config can't be empty"));
    }
    for (key, floor) in &config.house_map {
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
    let random_floor = config.house_map.values().choose(&mut rng)?;
    let random_room = random_floor.rooms.choose(&mut rng)?;
    let random_hiding_spot = random_room.hiding_spots.choose(&mut rng)?;
    Some(ActualHidingSpot{
        name: random_hiding_spot.name.to_owned(),
        room: random_room.name.to_owned(),
        floor: random_floor.name.to_owned(),
    })
}
fn get_difficulty_level() -> Result<Difficulty> {
    println!("pick a difficulty level:");
    Difficulty::print_difficulties();
    println!();
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("failed to read input");
        println!();
        let choice = match input.trim().parse::<usize>() {
            Err(_) => {
                eprintln!("invalid input, please try again");
                continue
            },
            Ok(num) => num,
        };
        if choice > 6 {
            eprintln!("invalid input, please try again");
            continue
        }
        match choice {
            1 => return Ok(Difficulty::Easy),
            2 => return Ok(Difficulty::Medium),
            3 => return Ok(Difficulty::MediumAdvanced),
            4 => return Ok(Difficulty::Advanced),
            5 => return Ok(Difficulty::Expert),
            6 => return Ok(Difficulty::Custom),
            _ => {
                eprintln!("invalid input, please try again");
                continue
            },
        };
    }
}
fn get_config_for_level(difficulty_level: Difficulty) -> Result<Config> {
    let config = match difficulty_level {
        Difficulty::Easy => create_map(1, 3, 2),
        Difficulty::Medium => create_map(1, 5, 3),
        Difficulty::MediumAdvanced => create_map(2, 3, 3),
        Difficulty::Advanced => create_map(2, 5, 4),
        Difficulty::Expert => create_map(3, 5, 4),
        Difficulty::Custom => load_config()?,
    };
    // prob validate here
    if difficulty_level == Difficulty::Custom {
        validate_config(&config)?;
    }
    Ok(config)
}
fn create_map(num_floors: usize, num_rooms: usize, num_hiding_spots: usize) -> Config {
    let mut rng = thread_rng();
    let mut map  = HashMap::new();
    let room_hiding_spots = get_room_hiding_spots(); 

    for floor_name in ALL_FLOORS.choose_multiple(&mut rng, num_floors) {
        let floor_name = floor_name.to_string();
        let mut rooms = vec![];

        let mut all_rooms = ALL_ROOMS.to_vec();
        all_rooms.shuffle(&mut rng);
        for _ in 0..num_rooms {
            let room_name = all_rooms.pop().expect("need a room");
            let hiding_spot_options = room_hiding_spots.get(room_name).expect("can't get hiding spot options");
            let hiding_spots = (0..num_hiding_spots)
                .map(|_| HidingSpot {
                    name: hiding_spot_options.choose(&mut rng).expect("need a hiding spot").to_string(),
                })
                .collect();
            rooms.push(Room {
                name: room_name.to_string(),
                hiding_spots,
            });
        }
        map.insert(floor_name.clone(), Floor {
            name: floor_name.clone(),
            rooms,
        });
    }
    Config { house_map: map }
}
pub fn run_hide_and_seek() -> Result<()> {
    let difficulty_level = get_difficulty_level()?;
    let config = get_config_for_level(difficulty_level)?;
    validate_config(&config)?;
    let hiding_spot = match generate_hiding_spot(&config) {
        Some(h) => h,
        None => return Err(anyhow!("failed to generate a hiding spot")),
    };
    let mut game = Game::new(hiding_spot, config);
    let (r, g, b) = color();
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
        let config = Config{ house_map: map };
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
        let config = Config{ house_map: map };
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
        let config = Config{ house_map: map };
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
        let config = Config{ house_map: map };
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
        let config = Config{ house_map: map };
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
    #[test]
    fn get_config_for_level_easy() {
        let config = get_config_for_level(Difficulty::Easy).expect("need config");
        assert_eq!(config.house_map.len(), 1);
        // skip the name field
        let floor = config.house_map.values().next().expect("need rooms");
        assert_eq!(floor.rooms.len(), 3);
        for room in &floor.rooms {
            assert_eq!(room.hiding_spots.len(), 2);
        }
    }
    #[test]
    fn get_config_for_level_medium() {
        let config = get_config_for_level(Difficulty::Medium).expect("need config");
        assert_eq!(config.house_map.len(), 1);
        // skip the name field
        let floor = config.house_map.values().next().expect("need rooms");
        assert_eq!(floor.rooms.len(), 5);
        for room in &floor.rooms {
            assert_eq!(room.hiding_spots.len(), 3);
        }
    }
    #[test]
    fn get_config_for_level_medium_advanced() {
        let config = get_config_for_level(Difficulty::MediumAdvanced).expect("need config");
        assert_eq!(config.house_map.len(), 2);
        for floor in config.house_map.values() {
            assert_eq!(floor.rooms.len(), 3);
            for room in &floor.rooms {
                assert_eq!(room.hiding_spots.len(), 3);
            }
        }
    }
    #[test]
    fn get_config_for_level_advanced() {
        let config = get_config_for_level(Difficulty::Advanced).expect("need config");
        assert_eq!(config.house_map.len(), 2);
        for floor in config.house_map.values() {
            assert_eq!(floor.rooms.len(), 5);
            for room in &floor.rooms {
                assert_eq!(room.hiding_spots.len(), 4);
            }
        }
    }
    #[test]
    fn get_config_for_level_expert() {
        let config = get_config_for_level(Difficulty::Expert).expect("need config");
        assert_eq!(config.house_map.len(), 3);
        for floor in config.house_map.values() {
            assert_eq!(floor.rooms.len(), 5);
            for room in &floor.rooms {
                assert_eq!(room.hiding_spots.len(), 4);
            }
        }
    }
}
