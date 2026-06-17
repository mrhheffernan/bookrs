use serde::Deserialize;
use std::fs;
use toml::de::from_str;

#[derive(Deserialize, Debug)]
struct Config {
    hotel: Hotel,
}
#[derive(Deserialize, Debug)]
struct Hotel {
    name: String,
    rooms: Vec<Rooms>,
}

#[derive(Deserialize, Debug)]
struct Rooms {
    room_type: String,
    count: u32,
}

fn load_hotel() -> Config {
    // Load config from file
    let file = fs::read_to_string("src/config/hotel.toml").expect("Could not open file");
    let hotel_config: Config = from_str(&file).unwrap();
    hotel_config
}

fn main() {
    let config = load_hotel();
    println!("{:?}", config);
}
